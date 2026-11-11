#!/usr/bin/env python3
"""Plan 19 item-26: NUMBERS-V1, the starting-number algebra (R1-R6).

Phases
------
env    pin host/toolchain/binary/corpus and the Phase A/C stacks.
a      Phase A (G2): the plan's 20 M child-evals instrument, three cells x
       seeds 0-2 x {bare, product hand, analysis flag stack}, plus the
       analysis-1 §4 reproduction row (wall 120 s, auxiliary passes off) that
       the 20 M instrument cannot see.
probe  Phase B R2: the EF-PN band-edge probe {64, 128, 192} plus the no-band
       arm on `core`, seed 0, tree only (auxiliary passes off).
gate   Phase C (G3): bare, product hand (A), NUMBERS-V1 (B), B with the
       auxiliary passes off, three cells x seeds 0-2 at a proof-reaching
       child-evals budget (default 300 M, wall guard 240 s).
tree   Phase C completion: the deep-win tree cut at a tree-only budget
       (auxiliary passes off, 2 G child evals, wall guard 600 s) for the
       product hand, the analysis flag stack, NUMBERS-V1 and the adopted
       ablation arm; censored rows are lower bounds.
matrix Phase D (only after G3): the 120 s budget matrix, three cells x
       seeds 0-2, stacks bare / B / B-aux-off.
tier2  the 10 s rung column: the adopted default (atomi) and the pinned
       provari hand reference, three cells x three repeats, same session.
d14    D14 for every proved matrix row: an untimed `--seconds 600
       --proof-out` rerun that must match `child_evals` and pass both
       checkers.
all    env, a, probe, gate, matrix, d14

One atomi process at a time. Proof files live under /tmp/atomi-plan19 and are
never committed; only parsed JSON is written here. No raw transcript, provari
artifact or __pycache__ is stored.
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
SCRATCH = Path("/tmp/atomi-plan19")
P1 = HERE / "p1.txt"

CELLS = {
    "core": "r1b1kbnr/ppp3pp/n4q2/3pppN1/1P1PP3/8/P1P2PPP/RNBQKB1R w KQkq - 0 1",
    "easy-win": "rnbq1b1r/pp1p1kpp/2p1pp1n/8/3PP1P1/2N2N2/PPP2P1P/R1BQKB1R w KQ - 0 1",
    "deep-win": "rnb1kbnr/pppp2pp/5q2/4ppN1/3P4/8/PPP1PPPP/RNBQKB1R w KQkq - 0 1",
}

NUMBERS = ["--guide-on", "numbers"]
AUX_OFF = ["--guide-off", "sim,closures"]

# Phase A: the plan's 20 M child-evals instrument.
STACKS_A = {
    "bare": ["--guide", "none"],
    "hand": ["--guide", "hand"],
    "flagstack": [
        "--guide", "hand",
        "--guide-params", str(P1),
        "--guide-on", "fresh-dn",
        "--guide-ef-weight", "0",
    ],
}

# Phase C/D: bare, product hand (A), NUMBERS-V1 (B), B tree-only.
STACKS_GATE = {
    "bare": ["--guide", "none"],
    "hand": ["--guide", "hand"],
    "numbers": ["--guide", "hand", *NUMBERS],
    "numbers_auxoff": ["--guide", "hand", *NUMBERS, *AUX_OFF],
}

EVALS_A = 20_000_000
MEMORY = 192
SEEDS = [0, 1, 2]
EVALS_GATE = 300_000_000
SECONDS_GATE = 240
SECONDS_MATRIX = 120
D14_SECONDS = 600
PROBE_EVALS = 300_000_000
PROBE_SECONDS = 240
TREE_EVALS = 2_000_000_000
TREE_SECONDS = 600
# The post-gate ablation result: R1 + R3 + R6, no band and no R4/R5. From
# plan19's default change on, `--guide hand` resolves to this; the explicit
# flags spell the same configuration on the pre-change binary.
ADOPTED = ["--guide", "hand", *NUMBERS,
           "--guide-off", "efpn-banded,threat-count,hanging-quiet"]
ADOPTED_AUXOFF = ADOPTED + AUX_OFF
# The product hand before the plan-19 default change.
PRODUCT = ["--guide", "hand",
           "--guide-off", "fresh-class,evasions-count,unlikely-exact"]


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--phase", default="all",
                   choices=["env", "a", "probe", "gate", "tree", "matrix",
                            "tier2", "d14", "all"])
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


def merged(path: Path, phase: str, rows: list[dict]) -> None:
    """Write a phase file, keeping rows of other stacks from a prior run."""
    if path.exists():
        old = json.loads(path.read_text())
        keep = {k: v for k, v in old.items() if k not in ("rows", "summary")}
        seen = {(r.get("cell"), r.get("stack_name"), r.get("seed")) for r in rows}
        kept = [r for r in old.get("rows", [])
                if (r.get("cell"), r.get("stack_name"), r.get("seed")) not in seen]
        write_json(path, keep | {"rows": kept + rows})
    else:
        write_json(path, {"plan": "plan19", "phase": phase, "rows": rows})


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
        return "dirty (plan19 execution; owner commits)" if out else "clean"
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


def write_json(path: Path, data) -> None:
    path.write_text(json.dumps(data, indent=1) + "\n")


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
        "plan": "plan19",
        "item": 26,
        "date": "2026-11-11",
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
        "stacks_phase_d": {
            "bare": STACKS_GATE["bare"],
            "product": PRODUCT,
            "adopted": ADOPTED,
            "adopted_auxoff": ADOPTED_AUXOFF,
        },
        "default_after_plan19": ("--guide hand = product + fresh-class + "
                                 "evasions-count + unlikely-exact"),
        "binary_history": {
            "3de35f7b": "Phase 0 + R1-R6, before the R6 prior fix (a.json, probe.json)",
            "520fa930": "R6 prior fix, before the default change (gate.json, tree.json)",
            "bf98c6df": "default change (matrix.json, tier2.json, d14)",
            "fdd42b07": "clippy-only edit of bf98c6df; the shipped binary",
        },
        "evals_a": EVALS_A,
        "evals_gate": args.gate_evals,
        "seconds_gate": args.gate_seconds,
        "seconds_matrix": SECONDS_MATRIX,
        "d14_seconds": D14_SECONDS,
        "memory_mb": MEMORY,
        "seeds": SEEDS,
        "note": ("Phase A repeats the plan's 20 M instrument; the gate runs at "
                 "a proof-reaching child-evals budget because expansions at a "
                 "20 M cap are ~child_evals/32 for every censored stack (see "
                 "report19 §G2)"),
    }
    write_json(Path(args.outdir) / "env.json", data)
    return data


def phase_a(args: argparse.Namespace) -> dict:
    out = {"plan": "plan19", "phase": "A", "evals": EVALS_A, "rows": []}
    budget = ["--seconds", "600", "--evals", str(EVALS_A)]
    for cell in CELLS:
        for name, stack in STACKS_A.items():
            for seed in SEEDS:
                row = atomi_row(args, stack, cell, seed, budget)
                row["stack_name"] = name
                out["rows"].append(row)
                print(f"A {cell:9s} {name:10s} seed {seed}: "
                      f"exp={row.get('expansions')} evals={row.get('child_evals')}",
                      flush=True)
        # The analysis-1 §4 reproduction: tree only, wall budget, seed 0.
        row = atomi_row(args, STACKS_A["flagstack"] + AUX_OFF, cell, 0,
                        ["--seconds", "120"])
        row["stack_name"] = "flagstack_auxoff"
        out["rows"].append(row)
        print(f"A {cell:9s} flagstack_auxoff seed 0: exp={row.get('expansions')} "
              f"exit={row.get('exit')}", flush=True)
        write_json(Path(args.outdir) / "a.json", out)
    write_json(Path(args.outdir) / "a.json", out)
    return out


def phase_probe(args: argparse.Namespace) -> dict:
    """R2's band edge on `core`, seed 0, tree only (auxiliary passes off)."""
    out = {"plan": "plan19", "phase": "B-probe", "cell": "core", "seed": 0,
           "evals": PROBE_EVALS, "rows": []}
    budget = ["--seconds", str(PROBE_SECONDS), "--evals", str(PROBE_EVALS)]
    base = ["--guide", "hand", *NUMBERS, *AUX_OFF]
    arms = {
        "numbers_all": base,
        "numbers_noband": base + ["--guide-off", "efpn-banded"],
        "numbers_nor4r5": base + ["--guide-off", "threat-count,hanging-quiet"],
        "numbers_nor1": base + ["--guide-off", "fresh-class"],
        "numbers_noband_nor4r5": base
        + ["--guide-off", "efpn-banded,threat-count,hanging-quiet"],
    }
    for edge in (64, 128, 192):
        path = Path(args.outdir) / f"p_band{edge}.txt"
        path.write_text(f"ef_band={edge}\n")
        arms[f"numbers_band{edge}"] = base + ["--guide-params", str(path)]
    for name, stack in arms.items():
        row = atomi_row(args, stack, "core", 0, budget)
        row["stack_name"] = name
        out["rows"].append(row)
        print(f"probe {name:18s}: exp={row.get('expansions')} "
              f"evals={row.get('child_evals')} exit={row.get('exit')}", flush=True)
        write_json(Path(args.outdir) / "probe.json", out)
    write_json(Path(args.outdir) / "probe.json", out)
    return out


def phase_gate(args: argparse.Namespace) -> dict:
    budget = ["--seconds", f"{args.gate_seconds:g}",
              "--evals", str(args.gate_evals)]
    out = {"plan": "plan19", "phase": "C", "evals": args.gate_evals,
           "seconds": args.gate_seconds, "rows": []}
    path = Path(args.outdir) / "gate.json"
    for cell in CELLS:
        for name, stack in STACKS_GATE.items():
            if not wanted(args, name):
                continue
            for seed in SEEDS:
                row = atomi_row(args, stack, cell, seed, budget)
                row["stack_name"] = name
                out["rows"].append(row)
                print(f"C {cell:9s} {name:15s} seed {seed}: "
                      f"exit={row.get('exit')} exp={row.get('expansions')} "
                      f"evals={row.get('child_evals')} wall={row.get('ms')}",
                      flush=True)
                merged(path, "C", out["rows"])
    out["rows"] = json.loads(path.read_text())["rows"]
    out["summary"] = gate_summary(out["rows"])
    write_json(path, out)
    return out


def _median(values: list[float]) -> float | None:
    values = [v for v in values if v is not None]
    if not values:
        return None
    values.sort()
    n = len(values)
    return values[n // 2] if n % 2 else (values[n // 2 - 1] + values[n // 2]) / 2


def gate_summary(rows: list[dict]) -> dict:
    """The G3 verdict from the rows: median expansions per (cell, stack)."""
    cells = sorted({r["cell"] for r in rows},
                   key=lambda c: list(CELLS).index(c))
    names = sorted({r["stack_name"] for r in rows})
    per: dict = {}
    for cell in cells:
        for name in names:
            sel = [r for r in rows if r["cell"] == cell and r["stack_name"] == name]
            per.setdefault(cell, {})[name] = {
                "proved": sum(1 for r in sel if r.get("exit") == "proved"),
                "seeds": len(sel),
                "median_expansions": _median([r.get("expansions") for r in sel]),
                "median_evals": _median([r.get("child_evals") for r in sel]),
                "median_wall_s": _median(
                    [None if r.get("ms") is None else r["ms"] / 1000 for r in sel]),
                "exits": [r.get("exit") for r in sel],
            }
    cuts = {}
    for cell in cells:
        a = per[cell].get("hand", {}).get("median_expansions")
        b = per[cell].get("numbers", {}).get("median_expansions")
        bare = per[cell].get("bare", {}).get("median_expansions")
        cuts[cell] = {
            "numbers_vs_hand": (a / b) if a and b else None,
            "numbers_vs_bare": (bare / b) if bare and b else None,
        }
    return {"per": per, "cuts": cuts}


def phase_tree(args: argparse.Namespace) -> dict:
    """The tree cut at a proof-reaching budget, auxiliary passes off."""
    budget = ["--seconds", str(TREE_SECONDS), "--evals", str(TREE_EVALS)]
    arms = {
        "hand_auxoff": ["--guide", "hand", *AUX_OFF],
        "flagstack_auxoff": STACKS_A["flagstack"] + AUX_OFF,
        "numbers_auxoff": ["--guide", "hand", *NUMBERS, *AUX_OFF],
        "adopted_auxoff": ADOPTED_AUXOFF,
    }
    out = {"plan": "plan19", "phase": "C-tree", "evals": TREE_EVALS,
           "seconds": TREE_SECONDS, "rows": []}
    path = Path(args.outdir) / "tree.json"
    for cell in CELLS:
        if not want_cell(args, cell):
            continue
        for name, stack in arms.items():
            if not wanted(args, name):
                continue
            for seed in SEEDS:
                row = atomi_row(args, stack, cell, seed, budget)
                row["stack_name"] = name
                out["rows"].append(row)
                print(f"tree {cell:9s} {name:17s} seed {seed}: "
                      f"exit={row.get('exit')} exp={row.get('expansions')} "
                      f"evals={row.get('child_evals')} wall={row.get('ms')}",
                      flush=True)
                merged(path, "C-tree", out["rows"])
    out["rows"] = json.loads(path.read_text())["rows"]
    out["summary"] = tree_summary(out["rows"])
    write_json(path, out)
    return out


def tree_summary(rows: list[dict]) -> dict:
    cells = sorted({r["cell"] for r in rows}, key=lambda c: list(CELLS).index(c))
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
                "median_expansions": _median([r.get("expansions") for r in sel]),
                "median_evals": _median([r.get("child_evals") for r in sel]),
                "exits": [r.get("exit") for r in sel],
            }
    cuts = {}
    for cell in cells:
        hand = per.get(cell, {}).get("hand_auxoff", {}).get("median_expansions")
        bare = None
        for name in ("numbers_auxoff", "adopted_auxoff", "flagstack_auxoff"):
            v = per.get(cell, {}).get(name, {}).get("median_expansions")
            if v:
                cuts.setdefault(cell, {})[f"{name}_vs_hand"] = (
                    hand / v if hand else None)
        if bare:
            cuts[cell]["bare_vs_adopted"] = None
    return {"per": per, "cuts": cuts}


def phase_matrix(args: argparse.Namespace) -> dict:
    budget = ["--seconds", str(SECONDS_MATRIX)]
    stacks = {
        "bare": STACKS_GATE["bare"],
        "product": PRODUCT,
        "adopted": ADOPTED,
        "adopted_auxoff": ADOPTED_AUXOFF,
    }
    out = {"plan": "plan19", "phase": "D", "seconds": SECONDS_MATRIX,
           "rows": []}
    path = Path(args.outdir) / "matrix.json"
    for cell in CELLS:
        for name, stack in stacks.items():
            for seed in SEEDS:
                row = atomi_row(args, stack, cell, seed, budget)
                row["stack_name"] = name
                out["rows"].append(row)
                print(f"D {cell:9s} {name:15s} seed {seed}: "
                      f"exit={row.get('exit')} wall={row.get('wall_s')} "
                      f"exp={row.get('expansions')}", flush=True)
                write_json(path, out)
    write_json(path, out)
    return out


def phase_tier2(args: argparse.Namespace) -> dict:
    """The 10 s column: adopted default vs the pinned provari hand."""
    out = {"plan": "plan19", "phase": "tier2", "seconds": 10,
           "rows": [], "provari": []}
    path = Path(args.outdir) / "tier2.json"
    for cell in CELLS:
        for seed in SEEDS:
            row = atomi_row(args, ADOPTED, cell, seed, ["--seconds", "10"])
            row["stack_name"] = "adopted"
            out["rows"].append(row)
            print(f"t2 {cell:9s} atomi seed {seed}: exit={row.get('exit')} "
                  f"wall={row.get('wall_s')}", flush=True)
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
            merged(path, "tier2", out["rows"])
    out["rows"] = json.loads(path.read_text())["rows"]
    out["provari_median_s"] = {
        cell: _median([r["wall_s"] for r in out["provari"]
                       if r.get("cell") == cell]) for cell in CELLS}
    write_json(path, out)
    return out


def phase_d14(args: argparse.Namespace) -> dict:
    """Every proved Phase-D row reruns untimed and must match its evals."""
    scratch = Path(args.scratch)
    scratch.mkdir(parents=True, exist_ok=True)
    matrix = json.loads((Path(args.outdir) / "matrix.json").read_text())
    out = {"plan": "plan19", "phase": "D14", "provari": args.provari,
           "rows": []}
    path = Path(args.outdir) / "matrix.json"
    for i, row in enumerate(matrix["rows"]):
        if row.get("exit") != "proved" or row["stack_name"] == "bare":
            continue
        proof = scratch / f"plan19-{i}.jsonl"
        stack = row["stack"].split(" ")
        rerun = atomi_row(args, stack, row["cell"], row["seed"],
                          ["--seconds", str(D14_SECONDS)],
                          proof_out=str(proof))
        valid = None
        check = None
        if proof.exists():
            val = subprocess.run([args.atomi, "validate", str(proof)],
                                 capture_output=True, text=True)
            valid = last_json(val.stdout).get("valid")
            chk = subprocess.run([args.provari, "check", str(proof)],
                                 capture_output=True, text=True)
            check = last_json(chk.stdout).get("valid")
            proof.unlink()
        out["rows"].append({
            "cell": row["cell"], "seed": row["seed"],
            "stack": row["stack"],
            "timed_evals": row.get("child_evals"),
            "untimed_evals": rerun.get("child_evals"),
            "matched": row.get("child_evals") == rerun.get("child_evals"),
            "exit": rerun.get("exit"),
            "validator": valid, "checker": check,
        })
        print(f"D14 {row['cell']:9s} {row['stack_name']:15s} seed {row['seed']}: "
              f"matched={out['rows'][-1]['matched']} validator={valid} "
              f"checker={check}", flush=True)
        write_json(path, json.loads(path.read_text()) | {"d14": out["rows"]})
    write_json(path, json.loads(path.read_text()) | {"d14": out["rows"]})
    return out


def main() -> int:
    args = parse_args()
    outdir = Path(args.outdir)
    outdir.mkdir(parents=True, exist_ok=True)
    phases = {"env": env, "a": phase_a, "probe": phase_probe,
              "gate": phase_gate, "tree": phase_tree,
              "matrix": phase_matrix, "tier2": phase_tier2,
              "d14": phase_d14}
    order = ["env", "a", "probe", "gate", "tree", "matrix", "tier2", "d14"] \
        if args.phase == "all" else [args.phase]
    for name in order:
        data = phases[name](args)
        print(f"phase {name}: done", flush=True)
        if name == "gate":
            summary = data.get("summary") or {}
            for cell, cut in (summary.get("cuts") or {}).items():
                print(f"  G3 {cell}: numbers vs hand "
                      f"{cut['numbers_vs_hand']} vs bare {cut['numbers_vs_bare']}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
