#!/usr/bin/env python3
"""Plan 24 item 29 GROUP-V1: score-free plain members, `--guide-group` default.

Phases
------
env      pin host/toolchain/binary/corpus, the stacks and the caps.
phase_a  diagnosis on the PRE-Phase-B binary: two arms x three cells x seeds
         0-2 at 120 s with `--stats` —
           `default`      `--guide hand`        (group off; changes nothing)
           `group_scored` `--guide hand --guide-group` (group on, members
                          still scored: the item-15 behaviour)
         Records the plain fraction (`plain_children`), `child_evals` and
         `aux_nodes` per expansion, and ns/expansion.
gate     the Phase C gate on the SHIPPED binary: control `--guide-off group`
         (the pre-change default) vs candidate `--guide hand` (group on,
         score-free), three cells x seeds 0-2 at 120 s. Prints the per-cell
         medians, the ratios and the verdict (deep-win <= 0.9x control, no
         cell > 1.1x control).
d14      for every proved `gate` candidate row: an untimed
         `--seconds 600 --proof-out` rerun that must match `child_evals` and
         pass `atomi validate` and `provari check`.

One atomi process at a time. Proof files live under /tmp/atomi-plan24 and are
deleted after validation; only parsed JSON is written here. The run helpers
follow `measurements/plan21/run_phase21.py` of this repo.

Size note: >10 KB because it is three resumable phases plus the gate
arithmetic in one stdlib-only file; each phase rewrites its JSON after every
run, so a long `deep-win` phase survives an interruption.
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
SCRATCH = Path("/tmp/atomi-plan24")

CELLS = {
    "core": "r1b1kbnr/ppp3pp/n4q2/3pppN1/1P1PP3/8/P1P2PPP/RNBQKB1R w KQkq - 0 1",
    "easy-win": "rnbq1b1r/pp1p1kpp/2p1pp1n/8/3PP1P1/2N2N2/PPP2P1P/R1BQKB1R w KQ - 0 1",
    "deep-win": "rnb1kbnr/pppp2pp/5q2/4ppN1/3P4/8/PPP1PPPP/RNBQKB1R w KQkq - 0 1",
}

# The hand guide is the shipped stack (D21 + AUX-V1); the only difference
# between the arms is the `group` switch.
ARMS = {
    "default": ["--guide", "hand"],
    "group_scored": ["--guide", "hand", "--guide-group"],
    "group_off": ["--guide", "hand", "--guide-off", "group"],
}

MEMORY = 192
SEEDS = [0, 1, 2]
CAP_SECONDS = 120
D14_SECONDS = 600
GATE_DEEP = 0.9
GATE_OTHER = 1.1


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--phase", default="all",
                   choices=["env", "phase_a", "gate", "gate_summary",
                            "d14", "all"])
    p.add_argument("--atomi", default=str(DEFAULT_ATOMI))
    p.add_argument("--provari", default=str(DEFAULT_PROVARI))
    p.add_argument("--corpus", default=str(DEFAULT_CORPUS))
    p.add_argument("--outdir", default=str(HERE))
    p.add_argument("--scratch", default=str(SCRATCH))
    p.add_argument("--cells", default="", help="comma-separated cells to run")
    return p.parse_args()


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def want(args: argparse.Namespace, cell: str) -> bool:
    only = [s for s in args.cells.split(",") if s]
    return not only or cell in only


def write_json(path: Path, data) -> None:
    path.write_text(json.dumps(data, indent=1) + "\n")


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


def git_head() -> str:
    return subprocess.run(["git", "rev-parse", "HEAD"], cwd=REPO,
                          capture_output=True, text=True).stdout.strip()


def worktree() -> str:
    out = subprocess.run(["git", "status", "--porcelain"], cwd=REPO,
                         capture_output=True, text=True).stdout
    return "dirty (plan24 execution; owner commits)" if out else "clean"


def cpu_model() -> str:
    try:
        for line in Path("/proc/cpuinfo").read_text().splitlines():
            if line.startswith("model name"):
                return line.split(":", 1)[1].strip()
    except OSError:
        pass
    return "unknown"


def median(values) -> float | None:
    values = sorted(v for v in values if v is not None)
    if not values:
        return None
    n = len(values)
    return values[n // 2] if n % 2 else (values[n // 2 - 1] + values[n // 2]) / 2


def atomi_row(args, arm: str, cell: str, seed: int, seconds: int,
              proof_out: str | None = None) -> dict:
    stack = ARMS[arm]
    cmd = [args.atomi, "prove", CELLS[cell], "--memory", str(MEMORY),
           "--seed", str(seed), *stack, "--seconds", str(seconds), "--stats"]
    if proof_out:
        cmd += ["--proof-out", proof_out]
    row = run(cmd, seconds)
    row["arm"] = arm
    row["cell"] = cell
    row["seed"] = seed
    row["stack"] = " ".join(stack)
    row["seconds"] = seconds
    stats = row.get("stats") or {}
    exps = row.get("expansions") or 0
    if exps:
        row["plain_share"] = round(
            stats.get("plain_children", 0) / row.get("child_evals", 1), 4)
        row["evals_per_exp"] = round(row.get("child_evals", 0) / exps, 2)
        row["aux_per_exp"] = round(row.get("aux_nodes", 0) / exps, 2)
        if row.get("wall_s"):
            row["ns_per_exp"] = round(row["wall_s"] * 1e9 / exps, 1)
    return row


def summarise(rows: list[dict]) -> dict:
    per = {}
    for cell in CELLS:
        sel = [r for r in rows if r.get("cell") == cell]
        if not sel:
            continue
        per[cell] = {
            "proved": sum(1 for r in sel if r.get("exit") == "proved"),
            "seeds": len(sel),
            "exits": [r.get("exit") for r in sel],
            "walls_s": [r.get("wall_s") for r in sel],
            "median_wall_s": median([r.get("wall_s") for r in sel]),
            "median_child_evals": median([r.get("child_evals") for r in sel]),
            "median_expansions": median([r.get("expansions") for r in sel]),
            "median_evals_per_exp": median([r.get("evals_per_exp") for r in sel]),
            "median_aux_per_exp": median([r.get("aux_per_exp") for r in sel]),
            "median_ns_per_exp": median([r.get("ns_per_exp") for r in sel]),
            "median_plain_children": median(
                [(r.get("stats") or {}).get("plain_children") for r in sel]),
            "median_group_children": median(
                [(r.get("stats") or {}).get("group_children") for r in sel]),
            "plain_share_of_evals": median([r.get("plain_share") for r in sel]),
            "proof_full": [r.get("proof_full") for r in sel],
        }
    return {"per": per}


def env(args) -> dict:
    atomi = Path(args.atomi)
    corpus = Path(args.corpus)
    pv = subprocess.run([args.provari, "--version"], capture_output=True,
                        text=True)
    data = {
        "plan": "plan24",
        "item": 29,
        "phase": "env",
        "date": "2026-11-12",
        "host": platform.node(),
        "cpu": cpu_model(),
        "kernel": f"{platform.system()} {platform.release()}",
        "machine": platform.machine(),
        "python": platform.python_version(),
        "rustc": subprocess.run(["rustc", "--version"], capture_output=True,
                                text=True).stdout.strip(),
        "git_head": git_head(),
        "worktree": worktree(),
        "atomi_binary": str(atomi),
        "atomi_binary_sha256": sha256(atomi),
        "corpus": str(corpus),
        "corpus_sha256": sha256(corpus),
        "provari": args.provari,
        "provari_version": (pv.stdout or pv.stderr).strip(),
        "cells": CELLS,
        "arms": {k: " ".join(v) for k, v in ARMS.items()},
        "memory_mb": MEMORY,
        "seeds": SEEDS,
        "cap_seconds": CAP_SECONDS,
        "d14_seconds": D14_SECONDS,
        "gate": {"deep_win_ratio_max": GATE_DEEP,
                 "other_cell_ratio_max": GATE_OTHER},
        "note": ("Phase A runs on the pre-Phase-B binary (group off is "
                 "`--guide hand`); the gate re-runs its control with the "
                 "shipped binary as `--guide-off group`. One process at a "
                 "time, same-session ratios only."),
    }
    write_json(Path(args.outdir) / "env.json", data)
    print(f"env: {data['atomi_binary_sha256'][:12]} on {data['host']}",
          flush=True)
    return data


def phase_rows(args, name: str, arms: list[str], seconds: int,
               summary_key: str) -> dict:
    path = Path(args.outdir) / f"{name}.json"
    rows = []
    for arm in arms:
        for cell in CELLS:
            if not want(args, cell):
                continue
            for seed in SEEDS:
                row = atomi_row(args, arm, cell, seed, seconds)
                rows.append(row)
                print(f"{name} {arm:12s} {cell:9s} seed {seed}: "
                      f"exit={row.get('exit')} wall={row.get('wall_s')} "
                      f"evals={row.get('child_evals')} "
                      f"plain={(row.get('stats') or {}).get('plain_children')}",
                      flush=True)
                write_json(path, {"plan": "plan24", "phase": name,
                                  "seconds": seconds, "rows": rows})
    out = {"plan": "plan24", "phase": name, "seconds": seconds, "rows": rows,
           summary_key: summarise(rows)}
    write_json(path, out)
    return out


def phase_gate(args) -> dict:
    gate = phase_rows(args, "gate", ["group_off", "default"], CAP_SECONDS,
                      "summary")
    return gate_summary(gate)


def gate_summary(gate: dict, outdir: Path = None) -> dict:
    """The gate arithmetic over an existing `gate.json` (no reruns)."""
    # The arms are summarised separately: the `gate` phase's own summary
    # lumps both, so the per-cell control would be a median over the two
    # arms (that bug cost nothing but a recomputation; see report24 sec. 5).
    ctrl_rows = [r for r in gate["rows"] if r.get("arm") == "group_off"]
    cand_rows = [r for r in gate["rows"] if r.get("arm") == "default"]
    ctrl = summarise(ctrl_rows)["per"]
    cand = summarise(cand_rows)["per"]
    per = {}
    ok = True
    for cell in CELLS:
        if cell not in ctrl or cell not in cand:
            continue
        c = ctrl[cell]["median_wall_s"]
        d = cand[cell]["median_wall_s"]
        ratio = round(d / c, 3) if c and d else None
        limit = GATE_DEEP if cell == "deep-win" else GATE_OTHER
        cell_ok = (cand[cell]["proved"] == cand[cell]["seeds"]
                   and ratio is not None and ratio <= limit)
        ok = ok and cell_ok
        per[cell] = {
            "control_median_s": c,
            "candidate_median_s": d,
            "ratio": ratio,
            "ratio_max": limit,
            "candidate_proved": f"{cand[cell]['proved']}/{cand[cell]['seeds']}",
            "control_proved": f"{ctrl[cell]['proved']}/{ctrl[cell]['seeds']}",
            "candidate_evals_per_exp": cand[cell]["median_evals_per_exp"],
            "control_evals_per_exp": ctrl[cell]["median_evals_per_exp"],
            "candidate_ns_per_exp": cand[cell]["median_ns_per_exp"],
            "control_ns_per_exp": ctrl[cell]["median_ns_per_exp"],
            "candidate_tree_factor": (
                round(cand[cell]["median_child_evals"]
                      / ctrl[cell]["median_child_evals"], 3)
                if ctrl[cell]["median_child_evals"] else None),
            "pass": cell_ok,
        }
    out = {"plan": "plan24", "phase": "gate", "seconds": CAP_SECONDS,
           "per": per,
           "verdict": "pass" if ok else "fail",
           "rule": ("deep-win candidate median <= 0.9x control; every cell "
                    "<= 1.1x control; every candidate seed proved")}
    write_json(Path(outdir or HERE) / "gate_summary.json", out)
    print(json.dumps(out, indent=1), flush=True)
    return out


def check(cmd: list[str]) -> bool:
    proc = subprocess.run(cmd, capture_output=True, text=True)
    return proc.returncode == 0 and last_json(proc.stdout).get("valid") is True


def phase_d14(args) -> dict:
    gate_path = Path(args.outdir) / "gate.json"
    if not gate_path.exists():
        print("d14: no gate.json; run the gate phase first", flush=True)
        return {}
    rows = json.loads(gate_path.read_text())["rows"]
    scratch = Path(args.scratch)
    scratch.mkdir(parents=True, exist_ok=True)
    path = Path(args.outdir) / "d14.json"
    done = []
    if path.exists():
        done = json.loads(path.read_text()).get("rows", [])
    for row in rows:
        if row.get("arm") != "default" or row.get("exit") != "proved":
            continue
        stem = f"{row['cell']}_{row['seed']}"
        if any(r.get("cell") == row["cell"] and r.get("seed") == row["seed"]
               for r in done):
            continue
        proof = scratch / f"{stem}.jsonl"
        rerun = atomi_row(args, "default", row["cell"], row["seed"],
                          D14_SECONDS, proof_out=str(proof))
        rerun["expected_child_evals"] = row.get("child_evals")
        rerun["match"] = rerun.get("child_evals") == row.get("child_evals")
        rerun["proof_positions"] = rerun.get("proof_positions")
        rerun["atomi_validate"] = (
            check([args.atomi, "validate", str(proof)])
            if proof.exists() else False)
        rerun["provari_check"] = (
            check([args.provari, "check", str(proof)])
            if proof.exists() else False)
        rerun["accepted"] = bool(rerun["match"] and rerun["atomi_validate"]
                                 and rerun["provari_check"])
        print(f"d14 {stem}: match={rerun['match']} "
              f"atomi={rerun['atomi_validate']} provari={rerun['provari_check']}"
              f" accepted={rerun['accepted']} wall={rerun.get('wall_s')}",
              flush=True)
        if proof.exists():
            proof.unlink()
        done.append(rerun)
        write_json(path, {"plan": "plan24", "phase": "d14",
                          "seconds": D14_SECONDS, "rows": done})
    accepted = sum(1 for r in done if r.get("accepted"))
    write_json(path, {"plan": "plan24", "phase": "d14",
                      "seconds": D14_SECONDS, "rows": done,
                      "accepted": accepted, "rows_n": len(done)})
    print(f"d14: {accepted}/{len(done)} accepted", flush=True)
    return {"rows": done}


def main() -> int:
    args = parse_args()
    if args.phase in ("env", "all"):
        env(args)
    if args.phase in ("phase_a", "all"):
        out = phase_rows(args, "phase_a", ["default", "group_scored"],
                         CAP_SECONDS, "summary")
        print(json.dumps(out["summary"], indent=1), flush=True)
    if args.phase == "gate_summary":
        gate_summary(json.loads((Path(args.outdir) / "gate.json").read_text()),
                     Path(args.outdir))
    if args.phase in ("gate", "all"):
        phase_gate(args)
    if args.phase in ("d14", "all"):
        phase_d14(args)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
