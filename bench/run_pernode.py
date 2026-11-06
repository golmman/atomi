#!/usr/bin/env python3
"""Per-node cost baseline for `docs/plans/prover/bench.md` (E2).

Runs `atomi prove FEN --evals N --seconds 600 --memory MB`, one process at
a time, and records per run: search ms, process wall, the child's peak RSS
(via `os.wait4(pid, 0)`'s rusage — that child only, not RUSAGE_CHILDREN),
child_evals, expansions and exit. Per (case, N) the medians cover only
runs that reached the cap (exit == "evals"); earlier exits are reported
with their numbers and flagged `early`.

Top-level `m1_reference`: the median ms per 1 M child evals on `core` at
the largest N whose runs all reached the cap.

Usage:
    python3 bench/run_pernode.py --atomi target/release/atomi \\
        --cases core,1eb80b92f92c --evals 1000000,10000000 --repeats 3 \\
        --out docs/plans/prover/measurements/plan1/pernode.json

Stdlib only. No transcripts; no `__pycache__`.
"""

from __future__ import annotations

import argparse
import json
import os
import platform
import resource
import statistics
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

sys.dont_write_bytecode = True

HERE = Path(__file__).resolve().parent
REPO = HERE.parent
DEFAULT_CORPUS = HERE / "corpus.json"
CORE_ID = "core"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--atomi", required=True)
    parser.add_argument("--corpus", default=str(DEFAULT_CORPUS))
    parser.add_argument("--cases", default=CORE_ID,
                        help="comma-separated case ids")
    parser.add_argument("--evals", required=True,
                        help="comma-separated child-eval caps")
    parser.add_argument("--repeats", type=int, default=3)
    parser.add_argument("--memory", type=int, default=192, metavar="MB")
    parser.add_argument("--out", required=True)
    return parser.parse_args()


def run_once(atomi: str, fen: str, evals: int, memory: int):
    """One measured run; returns the raw row or None on spawn failure."""
    cmd = [atomi, "prove", fen,
           "--evals", str(evals), "--seconds", "600",
           "--memory", str(memory)]
    start = time.perf_counter()
    proc = subprocess.Popen(cmd, stdout=subprocess.PIPE,
                            stderr=subprocess.STDOUT, text=True)
    out = proc.stdout.read()
    # The child exited at EOF but Popen has not reaped it: wait4 returns
    # THIS child's rusage (ru_maxrss in KiB on Linux), not the cumulative
    # RUSAGE_CHILDREN figure.
    try:
        _pid, status, rusage = os.wait4(proc.pid, 0)
        peak_kib = rusage.ru_maxrss
        proc.returncode = os.waitstatus_to_exitcode(status)
    except ChildProcessError:
        peak_kib = None
        proc.wait()
    wall = time.perf_counter() - start
    payload = {}
    for line in reversed(out.splitlines()):
        line = line.strip()
        if line.startswith("{"):
            try:
                payload = json.loads(line)
                break
            except json.JSONDecodeError:
                continue
    return wall, peak_kib, payload


def cpu_model() -> str:
    try:
        for line in Path("/proc/cpuinfo").read_text().splitlines():
            if line.startswith("model name"):
                return line.split(":", 1)[1].strip()
    except OSError:
        pass
    return "unknown"


def provenance(args: argparse.Namespace) -> dict:
    head = subprocess.run(
        ["git", "-C", str(REPO), "rev-parse", "HEAD"],
        capture_output=True, text=True, check=False).stdout.strip() or "unknown"
    dirty = bool(subprocess.run(
        ["git", "-C", str(REPO), "status", "--porcelain"],
        capture_output=True, text=True, check=False).stdout.strip())
    rustc = subprocess.run(["rustc", "-V"], capture_output=True, text=True,
                           check=False).stdout.strip()
    text = Path(REPO / "Cargo.lock").read_text()
    idx = text.find('name = "atomic-movegen"')
    movegen = "unknown"
    if idx >= 0:
        for line in text[idx:].splitlines():
            if line.startswith("version"):
                movegen = line.split('"')[1]
                break
    ver = subprocess.run([args.atomi, "--version"],
                         capture_output=True, text=True,
                         check=False).stdout.strip()
    return {
        "date": datetime.now(timezone.utc).strftime("%Y-%m-%d"),
        "host": platform.node(), "platform": platform.platform(),
        "cpu_model": cpu_model(), "cpu_count": os.cpu_count(),
        "rustc": rustc, "atomic_movegen": movegen,
        "atomi": {"version": ver, "binary": args.atomi},
        "repo_head": head, "repo_dirty": dirty,
        "budgets": {"memory_mb": args.memory, "repeats": args.repeats,
                    "seconds": 600},
        "corpus": args.corpus,
    }


def main() -> int:
    args = parse_args()
    data = json.loads(Path(args.corpus).read_text())
    all_cases = [data["core"], *data["sample20"]]
    wanted = [c.strip() for c in args.cases.split(",") if c.strip()]
    cases = [c for c in all_cases if c["id"] in wanted]
    if len(cases) != len(wanted):
        missing = set(wanted) - {c["id"] for c in cases}
        raise SystemExit(f"unknown case ids: {missing}")
    caps = [int(n) for n in args.evals.split(",")]

    runs: list[dict] = []
    for case in cases:
        for cap in caps:
            for i in range(args.repeats):
                wall, peak, payload = run_once(
                    args.atomi, case["fen"], cap, args.memory)
                child = payload.get("child_evals") or 0
                exp = payload.get("expansions") or 0
                ms = payload.get("ms") or 0
                row = {
                    "case": case["id"], "evals_cap": cap, "run": i + 1,
                    "wall_s": round(wall, 3),
                    "peak_rss_kib": peak,
                    "ms": ms, "child_evals": child, "expansions": exp,
                    "exit": payload.get("exit"),
                    "early": payload.get("exit") in ("proved", "refuted"),
                }
                if child:
                    row["ms_per_1m_child_evals"] = round(ms * 1e6 / child, 3)
                if exp:
                    row["ns_per_expansion"] = round(ms * 1e6 / exp, 1)
                    row["child_evals_per_expansion"] = round(child / exp, 2)
                runs.append(row)

    cells: dict[str, dict] = {}
    for case in cases:
        for cap in caps:
            group = [r for r in runs
                     if r["case"] == case["id"] and r["evals_cap"] == cap]
            at_cap = [r for r in group
                      if r["exit"] == "evals" and not r["early"]]
            early = [r for r in group if r["early"]]
            cell: dict = {"runs": group}
            if at_cap:
                cell["median"] = {
                    "ms_per_1m_child_evals": statistics.median(
                        r["ms_per_1m_child_evals"] for r in at_cap),
                    "ns_per_expansion": statistics.median(
                        r["ns_per_expansion"] for r in at_cap),
                    "child_evals_per_expansion": statistics.median(
                        r["child_evals_per_expansion"] for r in at_cap),
                    "peak_rss_kib": statistics.median(
                        r["peak_rss_kib"] for r in at_cap
                        if r["peak_rss_kib"] is not None),
                    "wall_s": statistics.median(
                        r["wall_s"] for r in at_cap),
                    "n_at_cap": len(at_cap),
                }
            if early:
                cell["early_runs"] = early
            cells[f"{case['id']}:{cap}"] = cell

    # M1 reference: core at the largest cap whose runs all reached the cap.
    m1 = None
    for cap in sorted(caps, reverse=True):
        group = [r for r in runs
                 if r["case"] == CORE_ID and r["evals_cap"] == cap]
        at_cap = [r for r in group if r["exit"] == "evals"]
        if group and len(at_cap) == len(group) and at_cap:
            m1 = {
                "evals_cap": cap,
                "ms_per_1m_child_evals": statistics.median(
                    r["ms_per_1m_child_evals"] for r in at_cap),
                "ns_per_expansion": statistics.median(
                    r["ns_per_expansion"] for r in at_cap),
            }
            break

    result = {
        "provenance": provenance(args),
        "caps": caps,
        "runs": runs,
        "cells": cells,
        "m1_reference": m1,
    }
    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(result, indent=1) + "\n")
    # A flat CSV next to the JSON.
    csv = out.with_suffix(".csv")
    with csv.open("w") as fh:
        fh.write("case,evals_cap,run,wall_s,peak_rss_kib,ms,child_evals,"
                 "expansions,exit,early,ms_per_1m_child_evals,"
                 "ns_per_expansion,child_evals_per_expansion\n")
        for r in runs:
            fh.write(",".join(str(r.get(k, "")) for k in (
                "case", "evals_cap", "run", "wall_s", "peak_rss_kib", "ms",
                "child_evals", "expansions", "exit", "early",
                "ms_per_1m_child_evals", "ns_per_expansion",
                "child_evals_per_expansion")) + "\n")
    print(f"wrote {out} and {csv}")
    if m1:
        print(f"m1_reference: {m1['ms_per_1m_child_evals']} ms/1M child "
              f"evals at N={m1['evals_cap']} "
              f"({m1['ns_per_expansion']} ns/expansion)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
