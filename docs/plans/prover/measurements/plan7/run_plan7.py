#!/usr/bin/env python3
"""Plan 7 item-16 driver: wall attribution, auxiliary budget, wall gate.

Phases
------
env          pin host/toolchain/binary and the frozen input hashes
attribution  phase A matrix: 8 stacks x easy-win/core (seeds 0-2) and
             deep-win (seed 0), fixed --evals and canonical 10 s budgets
budget       phase B grid: --guide-aux-budget on easy-win/core, seeds 0-2
wall         phase C: chosen configuration at 10 s / 192 MB, seeds 0-2,
             deep-win one attempt, D14 proof checks for claimed wins
soundness    guidance-v1 seed 0 with the chosen configuration
all          env, attribution, budget, wall, soundness

One atomi process at a time. Only parsed JSON-line fields are stored; no raw
transcript, provari artifact or bytecode is written.
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
DEFAULT_SET = REPO / "docs/plans/prover/measurements/plan3/guidance-v1.tsv"
DEFAULT_CORPUS = REPO / "bench/corpus.json"
DEFAULT_ATOMI = REPO / "target/release/atomi"
DEFAULT_PROVARI = Path("/tmp/atomi-oracle/provari/target/release/provari")
MEMORY = 192
SEEDS = [0, 1, 2]
FIXED_EVALS = 10_000_000
CANON_SECONDS = 10.0
# The product point from plan4 (report4): scale 64, EF-PN weight 0.
CAL = ["--guide-scale", "64", "--guide-ef-weight", "0"]
HAND = ["--guide", "hand", *CAL]
STACKS = {
    "bare": ["--guide", "none"],
    "hand": HAND,
    "no-closures": HAND + ["--guide-off", "closures"],
    "no-sim": HAND + ["--guide-off", "sim"],
    "neither": HAND + ["--guide-off", "sim,closures"],
    "aux-1-1": HAND + ["--guide-max-sim", "1", "--guide-max-settle", "1"],
    "sim-only-1": HAND + ["--guide-max-sim", "1", "--guide-max-settle", "0"],
    "settle-only-1": HAND + ["--guide-max-sim", "0", "--guide-max-settle", "1"],
}
CELLS = ["easy-win", "core", "deep-win"]
BUDGETS = ["fixed", "canonical"]


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--phase", choices=["env", "attribution", "cheaper", "budget",
                                       "wall", "soundness", "all"], default="all")
    p.add_argument("--atomi", default=str(DEFAULT_ATOMI))
    p.add_argument("--provari-check", default=str(DEFAULT_PROVARI))
    p.add_argument("--set", default=str(DEFAULT_SET))
    p.add_argument("--corpus", default=str(DEFAULT_CORPUS))
    p.add_argument("--outdir", default=str(HERE))
    p.add_argument("--budgets", default="10000,100000,1000000,0",
                   help="comma-separated aux budgets for the budget phase")
    p.add_argument("--chosen-budget", default="1000000",
                   help="aux budget used by the wall/soundness phases")
    return p.parse_args()


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def last_json(text: str) -> dict:
    for line in reversed(text.splitlines()):
        line = line.strip()
        if line.startswith("{"):
            try:
                return json.loads(line)
            except json.JSONDecodeError:
                continue
    return {}


def run(cmd: list[str], timeout: float) -> tuple[dict, float, int]:
    start = time.perf_counter()
    try:
        proc = subprocess.run(cmd, capture_output=True, text=True,
                              timeout=timeout + 120, check=False)
        return last_json(proc.stdout + proc.stderr), round(
            time.perf_counter() - start, 3), proc.returncode
    except subprocess.TimeoutExpired:
        return {}, round(time.perf_counter() - start, 3), 124


def load_set(path: Path) -> list[dict]:
    rows = []
    for raw in path.read_text().splitlines():
        if not raw.strip() or raw.startswith("name\t"):
            continue
        f = raw.split("\t")
        rows.append({"name": f[0], "fen": f[1], "source": f[2],
                     "ref_nodes": int(f[3]), "ref_wall_s": float(f[4])})
    return rows


def cells_by_name(args: argparse.Namespace) -> dict[str, dict]:
    return {c["name"]: c for c in load_set(Path(args.set))}


def git_head() -> str:
    try:
        return subprocess.run(["git", "rev-parse", "HEAD"], cwd=REPO,
                              capture_output=True, text=True).stdout.strip()
    except OSError:
        return "unknown"


def cpu_model() -> str:
    try:
        for line in Path("/proc/cpuinfo").read_text().splitlines():
            if line.startswith("model name"):
                return line.partition(":")[2].strip()
    except OSError:
        pass
    return platform.processor() or "unknown"


def worktree() -> str:
    try:
        out = subprocess.run(["git", "status", "--porcelain"], cwd=REPO,
                             capture_output=True, text=True).stdout
        return "dirty (plan7 execution; owner commits)" if out else "clean"
    except OSError:
        return "unknown"


def write_json(path: Path, data: dict) -> None:
    path.write_text(json.dumps(data, indent=1) + "\n")


def phase_env(args: argparse.Namespace) -> dict:
    atomi = Path(args.atomi)
    set_path = Path(args.set)
    corpus = Path(args.corpus)
    pv = subprocess.run([args.provari_check, "--version"], capture_output=True,
                        text=True)
    env = {
        "plan": "plan7",
        "date": "2026-11-09",
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
        "guidance_set": str(set_path),
        "guidance_sha256": sha256(set_path),
        "corpus": str(corpus),
        "corpus_sha256": sha256(corpus),
        "provari_check": args.provari_check,
        "provari_version": (pv.stdout or pv.stderr).strip(),
        "stacks": STACKS,
        "fixed_evals": FIXED_EVALS,
        "canonical_seconds": CANON_SECONDS,
        "memory_mb": MEMORY,
        "seeds": SEEDS,
        "chosen_budget": args.chosen_budget,
    }
    write_json(Path(args.outdir) / "env.json", env)
    return env


def atomi_row(args: argparse.Namespace, cell: dict, seed: int, flags: list[str],
              seconds: float, evals: int | None,
              proof_out: Path | None = None) -> dict:
    cmd = [args.atomi, "prove", cell["fen"], "--seconds", str(int(seconds)),
           "--memory", str(MEMORY), "--seed", str(seed), *flags]
    if evals is not None:
        cmd += ["--evals", str(evals)]
    if proof_out is not None:
        cmd += ["--proof-out", str(proof_out)]
    payload, wall, code = run(cmd, seconds)
    ms = payload.get("ms")
    evals = payload.get("child_evals")
    exps = payload.get("expansions")
    evals_per_s = (round(evals * 1000 / ms, 1)
                   if ms and evals is not None else None)
    exps_per_s = (round(exps * 1000 / ms, 1)
                  if ms and exps is not None else None)
    return {
        "cell": cell["name"], "seed": seed, "flags": flags,
        "seconds": seconds, "evals_limit": evals,
        "proved": bool(payload.get("found")),
        "exit": payload.get("exit"),
        "length": payload.get("length"),
        "child_evals": payload.get("child_evals"),
        "expansions": payload.get("expansions"),
        "sim_nodes": payload.get("sim_nodes"),
        "settle_nodes": payload.get("settle_nodes"),
        "aux_nodes": payload.get("aux_nodes"),
        "aux_skipped": payload.get("aux_skipped"),
        "proof_full": payload.get("proof_full"),
        "proof": payload.get("proof"),
        "proof_positions": payload.get("proof_positions"),
        "proof_length": payload.get("proof_length"),
        "proof_entries": payload.get("proof_entries"),
        "ms": ms,
        "evals_per_s": evals_per_s,
        "expansions_per_s": exps_per_s,
        "wall_s": wall,
        "returncode": code,
        "proof_file_exists": bool(proof_out and proof_out.exists()),
        "proof_file_bytes": (proof_out.stat().st_size
                             if proof_out and proof_out.exists() else None),
    }


def phase_attribution(args: argparse.Namespace) -> dict:
    outdir = Path(args.outdir)
    cells = cells_by_name(args)
    rows = []
    for stack, flags in STACKS.items():
        for cell in CELLS:
            seeds = SEEDS if cell != "deep-win" else [0]
            for seed in seeds:
                for budget in BUDGETS:
                    if budget == "fixed":
                        row = atomi_row(args, cells[cell], seed, flags,
                                        600.0, FIXED_EVALS)
                    else:
                        row = atomi_row(args, cells[cell], seed, flags,
                                        CANON_SECONDS, None)
                    row["stack"] = stack
                    row["budget"] = budget
                    rows.append(row)
                    print(f"{stack:14s} {cell:9s} s{seed} {budget:9s}: "
                          f"proved={row['proved']} exit={row['exit']} "
                          f"exp={row['expansions']} evals={row['child_evals']} "
                          f"sim={row['sim_nodes']} settle={row['settle_nodes']} "
                          f"wall={row['wall_s']}", flush=True)
                    write_json(outdir / "attribution.json",
                               {"phase": "attribution", "rows": rows})
    # Decision rule on easy-win, seed 0, canonical budget.
    def wall(stack: str, budget: str) -> float | None:
        for r in rows:
            if (r["stack"] == stack and r["cell"] == "easy-win"
                    and r["seed"] == 0 and r["budget"] == budget):
                return r["wall_s"]
        return None

    hand = wall("hand", "canonical")
    neither = wall("neither", "canonical")
    share = None if not hand else (hand - neither) / hand
    data = {
        "phase": "attribution",
        "stacks": STACKS,
        "fixed_evals": FIXED_EVALS,
        "canonical_seconds": CANON_SECONDS,
        "memory_mb": MEMORY,
        "rows": rows,
        "decision": {
            "easy_win_hand_wall_s": hand,
            "easy_win_neither_wall_s": neither,
            "aux_share": share,
            "premise_survives": share is not None and share >= 0.3,
        },
    }
    write_json(outdir / "attribution.json", data)
    return data


def phase_cheaper(args: argparse.Namespace) -> dict:
    """Plan7 B2 leave-one-out: simulation plies, plus the no-sim control."""
    outdir = Path(args.outdir)
    cells = cells_by_name(args)
    rows = []
    variants = [("plies-6", HAND), ("plies-4", HAND + ["--guide-sim-plies", "4"]),
                ("plies-3", HAND + ["--guide-sim-plies", "3"]),
                ("plies-2", HAND + ["--guide-sim-plies", "2"]),
                ("no-sim", HAND + ["--guide-off", "sim"])]
    for cell in ["easy-win", "core"]:
        for seed in SEEDS:
            for tag, flags in variants:
                row = atomi_row(args, cells[cell], seed, flags,
                                CANON_SECONDS, None)
                row["tag"] = tag
                rows.append(row)
                print(f"cheaper {tag:8s} {cell:9s} s{seed}: "
                      f"proved={row['proved']} exit={row['exit']} "
                      f"exp={row['expansions']} evals={row['child_evals']} "
                      f"aux={row['aux_nodes']} wall={row['wall_s']}",
                      flush=True)
                write_json(outdir / "cheaper.json",
                           {"phase": "cheaper", "rows": rows})
    data = {
        "phase": "cheaper",
        "variants": {tag: flags for tag, flags in variants},
        "canonical_seconds": CANON_SECONDS,
        "memory_mb": MEMORY,
        "rows": rows,
    }
    write_json(outdir / "cheaper.json", data)
    return data


def phase_budget(args: argparse.Namespace) -> dict:
    outdir = Path(args.outdir)
    cells = cells_by_name(args)
    budgets = [int(x) for x in args.budgets.split(",") if x]
    rows = []
    for cell in ["easy-win", "core"]:
        for seed in SEEDS:
            for b in budgets:
                flags = HAND + ["--guide-aux-budget", str(b)]
                row = atomi_row(args, cells[cell], seed, flags,
                                CANON_SECONDS, None)
                row["stack"] = "hand"
                row["budget"] = "canonical"
                row["aux_budget"] = b
                rows.append(row)
                print(f"budget {cell:9s} s{seed} aux={b:8d}: "
                      f"proved={row['proved']} exit={row['exit']} "
                      f"exp={row['expansions']} evals={row['child_evals']} "
                      f"aux={row['aux_nodes']} skipped={row['aux_skipped']} "
                      f"wall={row['wall_s']}", flush=True)
                write_json(outdir / "budget.json",
                           {"phase": "budget", "rows": rows})
    # Determinism: repeat the chosen budget on easy-win seed 0.
    chosen = int(args.chosen_budget)
    repeats = []
    for cell in ["easy-win", "core"]:
        flags = HAND + ["--guide-aux-budget", str(chosen)]
        row = atomi_row(args, cells[cell], 0, flags, CANON_SECONDS, None)
        row["stack"] = "hand"
        row["budget"] = "canonical"
        row["aux_budget"] = chosen
        repeats.append(row)
    index = {(r["cell"], r["seed"], r["aux_budget"]): r for r in rows}
    repeat_equal = {}
    for r in repeats:
        base = index.get((r["cell"], r["seed"], r["aux_budget"]))
        repeat_equal[r["cell"]] = base is not None and (
            (base["exit"], base["length"], base["child_evals"],
             base["expansions"], base["aux_nodes"], base["aux_skipped"]) ==
            (r["exit"], r["length"], r["child_evals"], r["expansions"],
             r["aux_nodes"], r["aux_skipped"]))
    data = {
        "phase": "budget",
        "stack": HAND,
        "budgets": budgets,
        "chosen_budget": chosen,
        "canonical_seconds": CANON_SECONDS,
        "memory_mb": MEMORY,
        "rows": rows,
        "repeats": repeats,
        "repeat_equal": repeat_equal,
    }
    write_json(outdir / "budget.json", data)
    return data


def check_proof(args: argparse.Namespace, atomi: Path, path: Path) -> dict:
    v, _, _ = run([str(atomi), "validate", str(path)], 60)
    c, _, _ = run([args.provari_check, "check", str(path)], 60)
    valid = bool(v.get("valid")) and c.get("valid") is True
    return {
        "atomi_valid": v.get("valid"),
        "atomi_positions": v.get("positions"),
        "provari_valid": c.get("valid"),
        "provari_won_positions": c.get("won_positions"),
        "verdict": "accepted" if valid else "rejected",
    }


def phase_wall(args: argparse.Namespace) -> dict:
    outdir = Path(args.outdir)
    proof_dir = outdir / "proofs"
    proof_dir.mkdir(exist_ok=True)
    cells = cells_by_name(args)
    chosen = int(args.chosen_budget)
    flags = HAND + ["--guide-aux-budget", str(chosen)]
    rows = []
    for cell in CELLS:
        seeds = SEEDS if cell != "deep-win" else [0]
        for seed in seeds:
            path = proof_dir / f"wall-{cell}-s{seed}.jsonl"
            row = atomi_row(args, cells[cell], seed, flags,
                            CANON_SECONDS, None, path)
            row["stack"] = "hand"
            row["aux_budget"] = chosen
            check = None
            if row["proved"] and row["proof"] == "written":
                check = check_proof(args, Path(args.atomi), path)
            row["check"] = check
            if path.exists():
                path.unlink()
            rows.append(row)
            print(f"wall {cell:9s} s{seed}: proved={row['proved']} "
                  f"exit={row['exit']} exp={row['expansions']} "
                  f"evals={row['child_evals']} aux={row['aux_nodes']} "
                  f"wall={row['wall_s']} check={check and check['verdict']}",
                  flush=True)
            write_json(outdir / "wall.json",
                       {"phase": "wall", "rows": rows})
    if proof_dir.exists() and not any(proof_dir.iterdir()):
        proof_dir.rmdir()
    easy = [r for r in rows if r["cell"] == "easy-win"]
    core = [r for r in rows if r["cell"] == "core"]
    deep = [r for r in rows if r["cell"] == "deep-win"]
    gates = {
        "easy_win_wall_le_0_5": all(r["proved"] and r["wall_s"] <= 0.5
                                    for r in easy),
        "core_proved_10s": all(r["proved"] for r in core),
        "deep_win_classified": bool(deep),
        "no_refutations": not any(r["exit"] == "refuted" for r in rows),
        "claimed_proofs_accepted": all(
            r["check"] and r["check"]["verdict"] == "accepted"
            for r in rows if r["proved"]),
    }
    data = {
        "phase": "wall",
        "stack": flags,
        "aux_budget": chosen,
        "canonical_seconds": CANON_SECONDS,
        "memory_mb": MEMORY,
        "rows": rows,
        "gates": gates,
    }
    write_json(outdir / "wall.json", data)
    return data


def phase_soundness(args: argparse.Namespace) -> dict:
    outdir = Path(args.outdir)
    proof_dir = outdir / "proofs"
    proof_dir.mkdir(exist_ok=True)
    cells = load_set(Path(args.set))
    chosen = int(args.chosen_budget)
    flags = HAND + ["--guide-aux-budget", str(chosen)]
    rows = []
    refuted = []
    proof_rows = []
    for cell in cells:
        path = proof_dir / f"sound-{cell['name']}-s0.jsonl"
        row = atomi_row(args, cell, 0, flags, 10.0, 5_000_000, path)
        rows.append(row)
        if row["exit"] == "refuted":
            refuted.append(cell["name"])
        if row["proved"] and row["proof"] == "written":
            check = check_proof(args, Path(args.atomi), path)
            proof_rows.append({"cell": cell["name"], "check": check,
                               "positions": row["proof_positions"],
                               "entries": row["proof_entries"]})
        if path.exists():
            path.unlink()
        print(f"sound {cell['name']}: exit={row['exit']} "
              f"proved={row['proved']}", flush=True)
    if proof_dir.exists() and not any(proof_dir.iterdir()):
        proof_dir.rmdir()
    rejected = [p for p in proof_rows if p["check"]["verdict"] != "accepted"]
    data = {
        "phase": "soundness",
        "stack": flags,
        "seconds": 10.0, "evals_limit": 5_000_000, "memory_mb": MEMORY,
        "seed": 0,
        "rows": rows,
        "refuted": refuted,
        "proofs": proof_rows,
        "gates": {
            "no_guidance_refutation": not refuted,
            "all_proved_proofs_accepted": not rejected,
        },
    }
    write_json(outdir / "soundness.json", data)
    return data


def main() -> None:
    args = parse_args()
    outdir = Path(args.outdir)
    outdir.mkdir(parents=True, exist_ok=True)
    if args.phase in ("env", "all"):
        phase_env(args)
    if args.phase in ("attribution", "all"):
        phase_attribution(args)
    if args.phase in ("cheaper", "all"):
        phase_cheaper(args)
    if args.phase in ("budget", "all"):
        phase_budget(args)
    if args.phase in ("wall", "all"):
        phase_wall(args)
    if args.phase in ("soundness", "all"):
        phase_soundness(args)


if __name__ == "__main__":
    main()
