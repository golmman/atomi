#!/usr/bin/env python3
"""Plan 21 item 31: a ledger row at its own cap, D14 accepted.

Phases
------
env     pin host/toolchain/binary/corpus, the stack, the caps.
ref     the same-session reference: provari v0.1.0 `--guide hand`, 3 repeats
        per cell at the canonical 10 s / 192 MB.
cap     atomi `--guide hand` (AUX-V1), three cells x seeds 0-2, at
        `--seconds 600 --memory 192` (no evals cap).
d14     for every proved `cap` row: an untimed `--seconds 600 --proof-out`
        rerun that must match `child_evals` and pass `atomi validate` and
        `provari check`.
ledger  factors (median cap wall / same-session median reference wall) and
        the geomean over the proved cells.
all     env, ref, cap, d14, ledger.

One atomi process at a time. Proof files live under /tmp/atomi-plan21 and are
deleted after validation; only parsed JSON is written here. The run helpers
follow `measurements/plan20/run_phase20.py` of this repo.

Size note: ~15 KB because it is five resumable phases in one stdlib-only
file (each phase rewrites its own JSON after every run, so a long
`deep-win` phase survives an interruption); splitting it would duplicate the
run/record helpers.
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
SCRATCH = Path("/tmp/atomi-plan21")

CELLS = {
    "core": "r1b1kbnr/ppp3pp/n4q2/3pppN1/1P1PP3/8/P1P2PPP/RNBQKB1R w KQkq - 0 1",
    "easy-win": "rnbq1b1r/pp1p1kpp/2p1pp1n/8/3PP1P1/2N2N2/PPP2P1P/R1BQKB1R w KQ - 0 1",
    "deep-win": "rnb1kbnr/pppp2pp/5q2/4ppN1/3P4/8/PPP1PPPP/RNBQKB1R w KQkq - 0 1",
}

# The shipped default after plan20 (Phase D): plan19's numbers algebra plus
# the two exact replays and the board-free closures (AUX-V1).
STACK = ["--guide", "hand"]
STACK_NAME = "aux_v1"

MEMORY = 192
SEEDS = [0, 1, 2]
REF_SECONDS = 10
REF_REPEATS = 3
CAP_SECONDS = 600
D14_SECONDS = 600


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--phase", default="all",
                   choices=["env", "ref", "cap", "d14", "ledger", "all"])
    p.add_argument("--atomi", default=str(DEFAULT_ATOMI))
    p.add_argument("--provari", default=str(DEFAULT_PROVARI))
    p.add_argument("--corpus", default=str(DEFAULT_CORPUS))
    p.add_argument("--outdir", default=str(HERE))
    p.add_argument("--scratch", default=str(SCRATCH))
    p.add_argument("--cells", default="",
                   help="comma-separated cells to run")
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
    return "dirty (plan21 execution; owner commits)" if out else "clean"


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


def atomi_row(args, cell: str, seed: int, seconds: int,
              proof_out: str | None = None) -> dict:
    cmd = [args.atomi, "prove", CELLS[cell], "--memory", str(MEMORY),
           "--seed", str(seed), *STACK, "--seconds", str(seconds), "--stats"]
    if proof_out:
        cmd += ["--proof-out", proof_out]
    row = run(cmd, seconds)
    row["cell"] = cell
    row["seed"] = seed
    row["stack_name"] = STACK_NAME
    row["stack"] = " ".join(STACK)
    row["seconds"] = seconds
    return row


def env(args) -> dict:
    atomi = Path(args.atomi)
    corpus = Path(args.corpus)
    pv = subprocess.run([args.provari, "--version"], capture_output=True,
                        text=True)
    data = {
        "plan": "plan21",
        "item": 31,
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
        "stack": {"name": STACK_NAME, "spelling": " ".join(STACK)},
        "memory_mb": MEMORY,
        "seeds": SEEDS,
        "ref_seconds": REF_SECONDS,
        "ref_repeats": REF_REPEATS,
        "cap_seconds": CAP_SECONDS,
        "d14_seconds": D14_SECONDS,
        "note": ("The cap is the row's own wall guard; the reference is "
                 "provari hand at its canonical 10 s (the bench set is "
                 "selected so it proves inside 10 s). Factors are same-session "
                 "ratios; no factor is compared across caps."),
    }
    write_json(Path(args.outdir) / "env.json", data)
    print(f"env: {data['atomi_binary_sha256'][:12]} on {data['host']}",
          flush=True)
    return data


def phase_ref(args) -> dict:
    path = Path(args.outdir) / "ref.json"
    rows = []
    for cell in CELLS:
        if not want(args, cell):
            continue
        for rep in range(REF_REPEATS):
            cmd = [args.provari, "prove", CELLS[cell],
                   "--seconds", str(REF_SECONDS), "--memory", str(MEMORY),
                   "--guide", "hand"]
            row = run(cmd, REF_SECONDS)
            row["cell"] = cell
            row["repeat"] = rep
            rows.append(row)
            print(f"ref {cell:9s} rep {rep}: {row.get('wall_s')} s "
                  f"nodes={row.get('nodes')}", flush=True)
            write_json(path, {"plan": "plan21", "phase": "ref",
                              "seconds": REF_SECONDS, "rows": rows})
    write_json(path, {"plan": "plan21", "phase": "ref",
                      "seconds": REF_SECONDS, "rows": rows,
                      "median_s": {c: median([r["wall_s"] for r in rows
                                              if r.get("cell") == c])
                                   for c in CELLS}})
    return {"rows": rows}


def phase_cap(args) -> dict:
    path = Path(args.outdir) / "cap.json"
    rows = []
    for cell in CELLS:
        if not want(args, cell):
            continue
        for seed in SEEDS:
            row = atomi_row(args, cell, seed, CAP_SECONDS)
            rows.append(row)
            print(f"cap {cell:9s} seed {seed}: exit={row.get('exit')} "
                  f"wall={row.get('wall_s')} evals={row.get('child_evals')} "
                  f"exp={row.get('expansions')}", flush=True)
            write_json(path, {"plan": "plan21", "phase": "cap",
                              "seconds": CAP_SECONDS, "rows": rows})
    write_json(path, {"plan": "plan21", "phase": "cap",
                      "seconds": CAP_SECONDS, "rows": rows,
                      "summary": cap_summary(rows)})
    return {"rows": rows}


def cap_summary(rows: list[dict]) -> dict:
    per = {}
    for cell in CELLS:
        sel = [r for r in rows if r.get("cell") == cell]
        if not sel:
            continue
        per[cell] = {
            "proved": sum(1 for r in sel if r.get("exit") == "proved"),
            "seeds": len(sel),
            "median_wall_s": median([r.get("wall_s") for r in sel]),
            "median_child_evals": median([r.get("child_evals") for r in sel]),
            "median_expansions": median([r.get("expansions") for r in sel]),
            "proof_full": [r.get("proof_full") for r in sel],
            "exits": [r.get("exit") for r in sel],
            "walls_s": [r.get("wall_s") for r in sel],
        }
    return {"per": per}


def check(cmd: list[str]) -> bool:
    proc = subprocess.run(cmd, capture_output=True, text=True)
    return proc.returncode == 0 and last_json(proc.stdout).get("valid") is True


def phase_d14(args) -> dict:
    cap = Path(args.outdir) / "cap.json"
    if not cap.exists():
        print("d14: no cap.json; run the cap phase first", flush=True)
        return {}
    rows = json.loads(cap.read_text())["rows"]
    scratch = Path(args.scratch)
    scratch.mkdir(parents=True, exist_ok=True)
    path = Path(args.outdir) / "d14.json"
    done = []
    if path.exists():
        done = json.loads(path.read_text()).get("rows", [])
    for row in rows:
        if row.get("exit") != "proved":
            continue
        stem = f"{row['cell']}_{row['seed']}"
        if any(r.get("cell") == row["cell"] and r.get("seed") == row["seed"]
               for r in done):
            continue
        proof = scratch / f"{stem}.jsonl"
        rerun = atomi_row(args, row["cell"], row["seed"], D14_SECONDS,
                          proof_out=str(proof))
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
        write_json(path, {"plan": "plan21", "phase": "d14",
                          "seconds": D14_SECONDS, "rows": done})
    accepted = sum(1 for r in done if r.get("accepted"))
    write_json(path, {"plan": "plan21", "phase": "d14",
                      "seconds": D14_SECONDS, "rows": done,
                      "accepted": accepted, "rows_n": len(done)})
    print(f"d14: {accepted}/{len(done)} accepted", flush=True)
    return {"rows": done}


def phase_ledger(args) -> dict:
    cap = json.loads((Path(args.outdir) / "cap.json").read_text())
    ref = json.loads((Path(args.outdir) / "ref.json").read_text())
    d14_path = Path(args.outdir) / "d14.json"
    d14 = (json.loads(d14_path.read_text()).get("rows", [])
           if d14_path.exists() else [])
    ref_s = ref["median_s"]
    per = cap["summary"]["per"]
    factors = {}
    claims = {}
    for cell, s in per.items():
        # A cell is a claim only if every seed proved AND its D14 rerun was
        # accepted (D14: otherwise it is censored with the verdict class).
        acc = [r for r in d14 if r.get("cell") == cell]
        verdicts = [r.get("proof") for r in acc] or ["no-d14"]
        claims[cell] = {
            "found": sum(1 for r in cap["rows"]
                         if r.get("cell") == cell and r.get("exit") == "proved"),
            "seeds": s["seeds"],
            "d14_accepted": sum(1 for r in acc if r.get("accepted")),
            "d14_rows": len(acc),
            "verdicts": verdicts,
            "claimed": bool(s["proved"] == s["seeds"] and acc
                            and all(r.get("accepted") for r in acc)),
        }
        if claims[cell]["claimed"] and ref_s.get(cell):
            factors[cell] = round(s["median_wall_s"] / ref_s[cell], 2)
    geomean = None
    if factors:
        acc = 1.0
        for f in factors.values():
            acc *= f
        geomean = round(acc ** (1 / len(factors)), 1)
    out = {
        "plan": "plan21",
        "phase": "ledger",
        "cap_seconds": CAP_SECONDS,
        "reference": {c: ref_s.get(c) for c in CELLS},
        "candidate_median_s": {c: per.get(c, {}).get("median_wall_s")
                               for c in CELLS},
        "claims": claims,
        "factors": factors,
        "geomean": geomean,
        "geomean_cells": len(factors),
        "note": ("Factor = median candidate wall / same-session provari hand "
                 "median, over the cells CLAIMED at this cap (search proved "
                 "on every seed AND D14 accepted). Geomean over the covered "
                 "cells of this cap only; never compared across caps."),
    }
    write_json(Path(args.outdir) / "ledger.json", out)
    print(json.dumps(out, indent=1), flush=True)
    return out


def main() -> int:
    args = parse_args()
    if args.phase in ("env", "all"):
        env(args)
    if args.phase in ("ref", "all"):
        phase_ref(args)
    if args.phase in ("cap", "all"):
        phase_cap(args)
    if args.phase in ("d14", "all"):
        phase_d14(args)
    if args.phase in ("ledger", "all"):
        phase_ledger(args)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
