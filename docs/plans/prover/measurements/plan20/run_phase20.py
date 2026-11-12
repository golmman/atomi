#!/usr/bin/env python3
"""Plan 20 items 27-28: AUX-V1, the auxiliary-cost stack.

Phases
------
env    pin host/toolchain/binary/corpus, the base stacks and the budgets.
a      Phase A (G2): default vs the exact-replay stack, three cells, seed 0,
       auxiliary passes on and off, at a generous child-evals cap.
gate   Phase C (G3): default vs AUX-V1, three cells x seeds 0-2, twice: a
       proof-reaching tree budget (default 2 G child evals, wall guard
       600 s) and an auxiliary-throughput budget (default 300 M child evals,
       wall guard 240 s).
matrix Phase D (only after G3): the 120 s budget matrix, default vs AUX-V1.
tier2  the 10 s tier, default vs AUX-V1 plus the pinned provari hand.
d14    D14 for every proved matrix row: an untimed `--seconds 600
       --proof-out` rerun that must match `child_evals` and pass both
       checkers.
all    env, a, gate, matrix, tier2, d14 (matrix/tier2/d14 only after G3)

One atomi process at a time. Proof files live under /tmp/atomi-plan20 and are
never committed; only parsed JSON is written here.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import platform
import subprocess
import sys
import time
from pathlib import Path

sys.dont_write_bytecode = True

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[4]
DEFAULT_ATOMI = REPO / "target/release/atomi"
DEFAULT_PROVARI = Path("/tmp/atomi-oracle/provari/target/release/provari")
DEFAULT_CORPUS = REPO / "bench/corpus.json"
SCRATCH = Path("/tmp/atomi-plan20")

CELLS = {
    "core": "r1b1kbnr/ppp3pp/n4q2/3pppN1/1P1PP3/8/P1P2PPP/RNBQKB1R w KQkq - 0 1",
    "easy-win": "rnbq1b1r/pp1p1kpp/2p1pp1n/8/3PP1P1/2N2N2/PPP2P1P/R1BQKB1R w KQ - 0 1",
    "deep-win": "rnb1kbnr/pppp2pp/5q2/4ppN1/3P4/8/PPP1PPPP/RNBQKB1R w KQkq - 0 1",
}

# From the Phase-D default change on, `--guide hand` is AUX-V1. The explicit
# spellings below resolve to the same three arms on either binary.
# Pre-plan-20 default (report19's adopted R1+R3+R6 arm).
DEFAULT = ["--guide", "hand",
           "--guide-off", "sim-exact,null-exact,closure-bits"]
# Item 27 alone (the exact replays); the Phase-A probe arm.
AUX_V1_NOBITS = ["--guide", "hand", "--guide-off", "closure-bits"]
# The adopted AUX-V1 stack: item 27 + item 28's board-free closures.
AUX_V1 = ["--guide", "hand"]
# The plan's auxiliary-off spelling: kill the greedy playout, the one-ply
# closures and the signature prior; the exact replays stay.
AUX_OFF = ["--guide-off", "sim,closures,null-exact"]
DEFAULT_AUX_OFF = DEFAULT + AUX_OFF
AUX_V1_AUX_OFF = AUX_V1_NOBITS + AUX_OFF

STACKS_A = {
    "default": DEFAULT,
    "aux_v1": AUX_V1_NOBITS,
    "default_auxoff": DEFAULT_AUX_OFF,
    "aux_v1_auxoff": AUX_V1_NOBITS + AUX_OFF,
}

# Gate arms: default (control), item 27 alone (ablation), AUX-V1 (27 + 28).
STACKS_GATE = {
    "default": DEFAULT,
    "aux_v1_nobits": AUX_V1_NOBITS,
    "aux_v1": AUX_V1,
}

MEMORY = 192
SEEDS = [0, 1, 2]
EVALS_PROBE = 2_000_000_000
SECONDS_PROBE = 600
EVALS_GATE = 300_000_000
SECONDS_GATE = 240
SECONDS_MATRIX = 120
D14_SECONDS = 600


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--phase", default="all",
                   choices=["env", "a", "gate", "matrix", "tier2", "d14",
                            "all"])
    p.add_argument("--atomi", default=str(DEFAULT_ATOMI))
    p.add_argument("--provari", default=str(DEFAULT_PROVARI))
    p.add_argument("--corpus", default=str(DEFAULT_CORPUS))
    p.add_argument("--outdir", default=str(HERE))
    p.add_argument("--scratch", default=str(SCRATCH))
    p.add_argument("--cells", default="",
                   help="comma-separated cells a phase should run")
    p.add_argument("--only", default="",
                   help="comma-separated stack names a phase should run")
    p.add_argument("--gate-evals", type=int, default=EVALS_GATE)
    p.add_argument("--gate-seconds", type=float, default=SECONDS_GATE)
    return p.parse_args()


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def want_cell(args: argparse.Namespace, cell: str) -> bool:
    only = [s for s in args.cells.split(",") if s]
    return not only or cell in only


def wanted(args: argparse.Namespace, name: str) -> bool:
    only = [s for s in args.only.split(",") if s]
    return not only or name in only


def write_json(path: Path, data) -> None:
    path.write_text(json.dumps(data, indent=1) + "\n")


def merged(path: Path, phase: str, rows: list[dict]) -> None:
    """Write a phase file, keeping rows of other stacks from a prior run."""
    if path.exists():
        old = json.loads(path.read_text())
        keep = {k: v for k, v in old.items() if k not in ("rows", "summary")}
        seen = {(r.get("cell"), r.get("stack_name"), r.get("seed"),
                 r.get("budget")) for r in rows}
        kept = [r for r in old.get("rows", [])
                if (r.get("cell"), r.get("stack_name"), r.get("seed"),
                    r.get("budget")) not in seen]
        write_json(path, keep | {"rows": kept + rows})
    else:
        write_json(path, {"plan": "plan20", "phase": phase, "rows": rows})


def last_json(text: str) -> dict:
    for line in reversed(text.splitlines()):
        line = line.strip()
        if line.startswith("{"):
            try:
                return json.loads(line)
            except json.JSONDecodeError:
                continue
    return {}


def run(cmd: list[str], timeout: float) -> dict:
    start = time.perf_counter()
    try:
        proc = subprocess.run(cmd, capture_output=True, text=True,
                              timeout=timeout + 120, check=False)
        out = last_json(proc.stdout + proc.stderr)
        out["wall_s"] = round(time.perf_counter() - start, 3)
        out["rc"] = proc.returncode
        if out.get("child_evals") is None:
            out["stderr"] = proc.stderr.strip()[:400]
        return out
    except subprocess.TimeoutExpired:
        return {"timeout": True, "wall_s": round(time.perf_counter() - start, 3)}


def git_head(repo: Path) -> str:
    try:
        return subprocess.run(["git", "rev-parse", "HEAD"], cwd=repo,
                              capture_output=True, text=True).stdout.strip()
    except OSError:
        return "unknown"


def worktree(repo: Path) -> str:
    try:
        out = subprocess.run(["git", "status", "--porcelain"], cwd=repo,
                             capture_output=True, text=True).stdout
        return "dirty (plan20 execution; owner commits)" if out else "clean"
    except OSError:
        return "unknown"


def cpu_model() -> str:
    try:
        for line in Path("/proc/cpuinfo").read_text().splitlines():
            if line.startswith("model name"):
                return line.split(":", 1)[1].strip()
    except OSError:
        pass
    return "unknown"


def atomi_row(args: argparse.Namespace, stack: list[str], cell: str,
              seed: int, budget: list[str], stats: bool = True,
              proof_out: str | None = None) -> dict:
    cmd = [args.atomi, "prove", CELLS[cell], "--memory", str(MEMORY),
           "--seed", str(seed), *stack, *budget]
    if stats:
        cmd.append("--stats")
    if proof_out:
        cmd += ["--proof-out", proof_out]
    row = run(cmd, _timeout_of(budget))
    row["cell"] = cell
    row["seed"] = seed
    row["stack"] = " ".join(stack)
    row["budget"] = " ".join(budget)
    return row


def _timeout_of(budget: list[str]) -> float:
    if "--seconds" in budget:
        return float(budget[budget.index("--seconds") + 1])
    return 900.0


def env(args: argparse.Namespace) -> dict:
    atomi = Path(args.atomi)
    corpus = Path(args.corpus)
    pv = subprocess.run([args.provari, "--version"], capture_output=True,
                        text=True)
    data = {
        "plan": "plan20",
        "items": [27, 28],
        "date": "2026-11-12",
        "host": platform.node(),
        "cpu": cpu_model(),
        "kernel": f"{platform.system()} {platform.release()}",
        "machine": platform.machine(),
        "python": platform.python_version(),
        "rustc": subprocess.run(["rustc", "--version"], capture_output=True,
                                text=True).stdout.strip(),
        "git_head": git_head(REPO),
        "worktree": worktree(REPO),
        "atomi_binary": str(atomi),
        "atomi_binary_sha256": sha256(atomi),
        "corpus": str(corpus),
        "corpus_sha256": sha256(corpus),
        "provari": args.provari,
        "provari_version": (pv.stdout or pv.stderr).strip(),
        "cells": CELLS,
        "stacks_a": STACKS_A,
        "stacks_gate": STACKS_GATE,
        "evals_probe": EVALS_PROBE,
        "seconds_probe": SECONDS_PROBE,
        "evals_gate": args.gate_evals,
        "seconds_gate": args.gate_seconds,
        "seconds_matrix": SECONDS_MATRIX,
        "d14_seconds": D14_SECONDS,
        "memory_mb": MEMORY,
        "seeds": SEEDS,
        "default_after_plan20": ("--guide hand = the plan-19 default + "
                                  "sim-exact + null-exact + closure-bits"),
        "control_after_plan20": ("--guide hand --guide-off "
                                  "sim-exact,null-exact,closure-bits"),
        "note": ("Phase A judges sim_nodes/exp <= 1 and child evals to proof; "
                 "the 20 M instrument is retired (report19 D33). The gate, "
                 "matrix and tier2 rows were measured before the Phase-D "
                 "default change; the explicit stack spellings resolve to "
                 "the same three arms on the shipped binary."),
    }
    write_json(Path(args.outdir) / "env.json", data)
    return data


def phase_a(args: argparse.Namespace) -> dict:
    out = {"plan": "plan20", "phase": "A",
           "evals": EVALS_PROBE, "seconds": SECONDS_PROBE, "rows": [],
           "summary": {}}
    path = Path(args.outdir) / "probe.json"
    budget = ["--seconds", str(SECONDS_PROBE), "--evals", str(EVALS_PROBE)]
    for cell in CELLS:
        if not want_cell(args, cell):
            continue
        for name, stack in STACKS_A.items():
            if not wanted(args, name):
                continue
            row = atomi_row(args, stack, cell, 0, budget)
            row["stack_name"] = name
            out["rows"].append(row)
            print(f"A {cell:9s} {name:16s}: exit={row.get('exit')} "
                  f"exp={row.get('expansions')} evals={row.get('child_evals')} "
                  f"sim={row.get('sim_nodes')} settle={row.get('settle_nodes')} "
                  f"aux={row.get('aux_nodes')} wall={row.get('wall_s')}",
                  flush=True)
            merged(path, "A", out["rows"])
    out["rows"] = json.loads(path.read_text())["rows"]
    out["summary"] = aux_summary(out["rows"])
    data = json.loads(path.read_text()) | {"summary": out["summary"]}
    write_json(path, data)
    return out


def _median(values: list[float]) -> float | None:
    values = [v for v in values if v is not None]
    if not values:
        return None
    values.sort()
    n = len(values)
    return values[n // 2] if n % 2 else (values[n // 2 - 1] + values[n // 2]) / 2


def aux_summary(rows: list[dict]) -> dict:
    cells = sorted({r["cell"] for r in rows},
                   key=lambda c: list(CELLS).index(c))
    names = sorted({r["stack_name"] for r in rows})
    per: dict = {}
    for cell in cells:
        for name in names:
            sel = [r for r in rows if r["cell"] == cell and r["stack_name"] == name]
            if not sel:
                continue
            exp = [r.get("expansions") for r in sel]
            aux = [r.get("aux_nodes") for r in sel]
            sim = [r.get("sim_nodes") for r in sel]
            settle = [r.get("settle_nodes") for r in sel]
            med_exp = _median(exp)
            med_aux = _median(aux)
            med_sim = _median(sim)
            per.setdefault(cell, {})[name] = {
                "proved": sum(1 for r in sel if r.get("exit") == "proved"),
                "seeds": len(sel),
                "median_expansions": med_exp,
                "median_child_evals": _median(
                    [r.get("child_evals") for r in sel]),
                "median_aux_nodes": med_aux,
                "median_sim_nodes": med_sim,
                "median_settle_nodes": _median(settle),
                "aux_per_exp": (med_aux / med_exp) if med_aux and med_exp else None,
                "sim_per_exp": (med_sim / med_exp) if med_sim and med_exp else None,
                "median_wall_s": _median(
                    [None if r.get("ms") is None else r["ms"] / 1000
                     for r in sel]),
                "exits": [r.get("exit") for r in sel],
            }
    # G2: the exact-replay arm vs the default on child evals to proof.
    cuts = {}
    for cell in cells:
        a = per.get(cell, {}).get("aux_v1", {}).get("median_child_evals")
        b = per.get(cell, {}).get("default", {}).get("median_child_evals")
        ao = per.get(cell, {}).get("aux_v1_auxoff", {}).get("median_child_evals")
        bo = per.get(cell, {}).get("default_auxoff", {}).get("median_child_evals")
        cuts[cell] = {
            "aux_v1_vs_default_evals": (b / a) if a and b else None,
            "aux_v1_vs_default_auxoff_evals": (bo / ao) if ao and bo else None,
        }
    return {"per": per, "cuts": cuts}


def phase_gate(args: argparse.Namespace) -> dict:
    out = {"plan": "plan20", "phase": "C",
           "tree_evals": EVALS_PROBE, "tree_seconds": SECONDS_PROBE,
           "aux_evals": args.gate_evals, "aux_seconds": args.gate_seconds,
           "rows": [], "summary": {}}
    path = Path(args.outdir) / "gate.json"
    budgets = {
        "tree": ["--seconds", str(SECONDS_PROBE),
                 "--evals", str(EVALS_PROBE)],
        "aux": ["--seconds", f"{args.gate_seconds:g}",
                "--evals", str(args.gate_evals)],
    }
    for cell in CELLS:
        if not want_cell(args, cell):
            continue
        for budget_name, budget in budgets.items():
            for name, stack in STACKS_GATE.items():
                if not wanted(args, name):
                    continue
                for seed in SEEDS:
                    row = atomi_row(args, stack, cell, seed, budget)
                    row["stack_name"] = name
                    row["budget_name"] = budget_name
                    out["rows"].append(row)
                    print(f"C-{budget_name} {cell:9s} {name:8s} seed {seed}: "
                          f"exit={row.get('exit')} exp={row.get('expansions')} "
                          f"evals={row.get('child_evals')} "
                          f"aux={row.get('aux_nodes')} wall={row.get('wall_s')}",
                          flush=True)
                    merged(path, "C", out["rows"])
    out["rows"] = json.loads(path.read_text())["rows"]
    out["summary"] = gate_summary(out["rows"])
    data = json.loads(path.read_text()) | {"summary": out["summary"]}
    write_json(path, data)
    return out


def gate_summary(rows: list[dict]) -> dict:
    cells = sorted({r["cell"] for r in rows},
                   key=lambda c: list(CELLS).index(c))
    budgets = sorted({r.get("budget_name") for r in rows})
    names = sorted({r["stack_name"] for r in rows})
    per: dict = {}
    for cell in cells:
        for bname in budgets:
            for name in names:
                sel = [r for r in rows if r["cell"] == cell
                       and r["stack_name"] == name
                       and r.get("budget_name") == bname]
                if not sel:
                    continue
                med_exp = _median([r.get("expansions") for r in sel])
                med_aux = _median([r.get("aux_nodes") for r in sel])
                per.setdefault(cell, {}).setdefault(bname, {})[name] = {
                    "proved": sum(1 for r in sel if r.get("exit") == "proved"),
                    "seeds": len(sel),
                    "median_expansions": med_exp,
                    "median_child_evals": _median(
                        [r.get("child_evals") for r in sel]),
                    "median_aux_nodes": med_aux,
                    "median_sim_nodes": _median(
                        [r.get("sim_nodes") for r in sel]),
                    "median_settle_nodes": _median(
                        [r.get("settle_nodes") for r in sel]),
                    "aux_per_exp": (med_aux / med_exp)
                    if med_aux and med_exp else None,
                    "median_wall_s": _median(
                        [None if r.get("ms") is None else r["ms"] / 1000
                         for r in sel]),
                    "exits": [r.get("exit") for r in sel],
                }
    cuts = {}
    for cell in cells:
        tree = per.get(cell, {}).get("tree", {})
        a = tree.get("aux_v1", {}).get("median_child_evals")
        b = tree.get("default", {}).get("median_child_evals")
        cuts[cell] = {"aux_v1_vs_default_tree_evals": (b / a) if a and b else None}
    return {"per": per, "cuts": cuts}


def phase_matrix(args: argparse.Namespace) -> dict:
    out = {"plan": "plan20", "phase": "D", "seconds": SECONDS_MATRIX,
           "rows": [], "summary": {}}
    path = Path(args.outdir) / "matrix.json"
    budget = ["--seconds", str(SECONDS_MATRIX)]
    for cell in CELLS:
        if not want_cell(args, cell):
            continue
        for name, stack in STACKS_GATE.items():
            for seed in SEEDS:
                row = atomi_row(args, stack, cell, seed, budget)
                row["stack_name"] = name
                out["rows"].append(row)
                print(f"D {cell:9s} {name:8s} seed {seed}: "
                      f"exit={row.get('exit')} wall={row.get('wall_s')} "
                      f"exp={row.get('expansions')}", flush=True)
                merged(path, "D", out["rows"])
    out["rows"] = json.loads(path.read_text())["rows"]
    out["summary"] = matrix_summary(out["rows"])
    data = json.loads(path.read_text()) | {"summary": out["summary"]}
    write_json(path, data)
    return out


def matrix_summary(rows: list[dict]) -> dict:
    cells = sorted({r["cell"] for r in rows},
                   key=lambda c: list(CELLS).index(c))
    names = sorted({r["stack_name"] for r in rows})
    per: dict = {}
    for cell in cells:
        for name in names:
            sel = [r for r in rows if r["cell"] == cell and r["stack_name"] == name]
            if not sel:
                continue
            per.setdefault(cell, {})[name] = {
                "proved": sum(1 for r in sel if r.get("exit") == "proved"),
                "seeds": len(sel),
                "median_wall_s": _median(
                    [None if r.get("ms") is None else r["ms"] / 1000
                     for r in sel]),
                "exits": [r.get("exit") for r in sel],
            }
    return {"per": per}


def phase_tier2(args: argparse.Namespace) -> dict:
    out = {"plan": "plan20", "phase": "tier2", "seconds": 10,
           "rows": [], "provari": []}
    path = Path(args.outdir) / "tier2.json"
    for cell in CELLS:
        if not want_cell(args, cell):
            continue
        for name, stack in STACKS_GATE.items():
            for seed in SEEDS:
                row = atomi_row(args, stack, cell, seed, ["--seconds", "10"])
                row["stack_name"] = name
                out["rows"].append(row)
                print(f"t2 {cell:9s} {name:8s} seed {seed}: "
                      f"exit={row.get('exit')} wall={row.get('wall_s')}",
                      flush=True)
                merged(path, "tier2", out["rows"])
        for rep in range(3):
            cmd = [args.provari, "prove", CELLS[cell], "--seconds", "10",
                   "--memory", str(MEMORY), "--guide", "hand"]
            row = run(cmd, 10)
            row["cell"] = cell
            row["repeat"] = rep
            out["provari"].append(row)
            print(f"t2 {cell:9s} provari rep {rep}: {row.get('wall_s')} s",
                  flush=True)
    path_data = json.loads(path.read_text()) if path.exists() else {}
    path_data["provari"] = out["provari"]
    path_data["provari_median_s"] = {
        cell: _median([r["wall_s"] for r in out["provari"]
                       if r.get("cell") == cell]) for cell in CELLS}
    write_json(path, path_data)
    out["provari_median_s"] = path_data["provari_median_s"]
    return out


# The gate/matrix rows were measured before the Phase-D default change; the
# shipped `--guide hand` is now AUX-V1. Map the recorded control strings to
# the explicit spellings of the same arms.
STACK_ALIASES = {
    "default": ["--guide", "hand", "--guide-off",
                "sim-exact,null-exact,closure-bits"],
    "aux_v1_nobits": ["--guide", "hand", "--guide-off", "closure-bits"],
    "aux_v1": ["--guide", "hand"],
}


def phase_d14(args: argparse.Namespace) -> dict:
    out = {"plan": "plan20", "phase": "d14", "rows": []}
    path = Path(args.outdir) / "d14.json"
    src = Path(args.outdir) / "matrix.json"
    if not src.exists():
        print("d14: no matrix.json; run the matrix first", flush=True)
        return out
    matrix = json.loads(src.read_text())["rows"]
    scratch = Path(args.scratch)
    scratch.mkdir(parents=True, exist_ok=True)
    for row in matrix:
        if row.get("exit") != "proved":
            continue
        stem = f"{row['cell']}_{row['stack_name']}_{row['seed']}"
        proof = scratch / f"{stem}.jsonl"
        budget = ["--seconds", str(D14_SECONDS)]
        stack = STACK_ALIASES.get(row["stack_name"], row["stack"].split())
        rerun = atomi_row(args, stack, row["cell"], row["seed"], budget,
                          proof_out=str(proof))
        rerun["stack_name"] = row["stack_name"]
        rerun["expected_child_evals"] = row.get("child_evals")
        rerun["match"] = rerun.get("child_evals") == row.get("child_evals")
        rerun["atomi_validate"] = check(
            [str(args.atomi), "validate", str(proof)]) if proof.exists() else False
        rerun["provari_check"] = check(
            [args.provari, "check", str(proof)]) if proof.exists() else False
        rerun["accepted"] = bool(rerun["match"] and rerun["atomi_validate"]
                                 and rerun["provari_check"])
        print(f"d14 {stem}: match={rerun['match']} "
              f"atomi={rerun['atomi_validate']} provari={rerun['provari_check']}",
              flush=True)
        if proof.exists():
            proof.unlink()
        out["rows"].append(rerun)
        write_json(path, out)
    write_json(path, out)
    return out


def check(cmd: list[str]) -> bool:
    proc = subprocess.run(cmd, capture_output=True, text=True)
    return proc.returncode == 0 and last_json(proc.stdout).get("valid") is True


def main() -> int:
    args = parse_args()
    if args.phase in ("env", "all"):
        env(args)
    if args.phase in ("a", "all"):
        phase_a(args)
    if args.phase in ("gate", "all"):
        phase_gate(args)
    if args.phase in ("matrix", "all"):
        phase_matrix(args)
    if args.phase in ("tier2", "all"):
        phase_tier2(args)
    if args.phase in ("d14", "all"):
        phase_d14(args)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
