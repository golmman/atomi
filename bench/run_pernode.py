#!/usr/bin/env python3
"""Per-node cost driver for `docs/plans/prover/bench.md` (E2, M1 gate).

Sides:
- `--atomi NAME=PATH` (repeatable; a bare `PATH` is the side `atomi`) runs
  `atomi prove FEN --evals N --seconds 600 --memory MB` for every `--memory`
  value (comma list). Per run: search ms, process wall, the child's peak RSS
  (via `os.wait4(pid, 0)`'s rusage — that child only, not
  RUSAGE_CHILDREN), child_evals, expansions and exit.
- `--legacy-probe PATH` (+ `--legacy-commit SHA`) runs the legacy plan3
  front-end probe `probe FEN N`, which prints
  `{"evals","nodes","proved","depth","wall_s"}`. Its scratch table is fixed
  at 32 MB, so its rows carry `memory_mb: null`; `--memory` does not apply.

ms per 1 M child evals: atomi `ms / child_evals * 1e6`; probe
`wall_s / evals * 1e9` (its in-process solve time, the analogue of `ms`).

Order: for each (case, cap), each repeat runs every side (and every memory)
once before the next repeat starts, one process at a time, so the sides
share drift. Per (side, case, cap, memory) the medians cover only runs that
reached the cap (atomi: exit == "evals"; probe: not proved); earlier exits
are reported with their numbers and flagged `early`.

Top-level blocks:
- `m1_reference`: the atomi side `m1` (else the first atomi side) at the
  first memory, `core`, the largest cap whose runs all reached the cap
  (plan1 semantics).
- `gate`: `core`, the largest cap, `--memory 192`: the medians of sides
  `m1`, `m0` and `legacy-fe` (null when absent) and `pass_m0 = m1 <= m0`,
  `pass_legacy = m1 <= legacy-fe` (null when a side is missing).

Usage:
    python3 bench/run_pernode.py --atomi m0=/tmp/atomi-m0/target/release/atomi \\
        --atomi m1=target/release/atomi --legacy-probe /tmp/plan2-legacy-probe/probe \\
        --legacy-commit 5f85133ea9d751368a682e53e354bd3fc0016d00 \\
        --cases core,1eb80b92f92c --evals 1000000,10000000 --memory 192,32 \\
        --repeats 3 --out docs/plans/prover/measurements/plan2/pernode.json

Stdlib only. No transcripts; no `__pycache__`.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
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
LEGACY_SIDE = "legacy-fe"
GATE_MEMORY = 192


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--atomi", action="append", default=[],
                        metavar="[NAME=]PATH")
    parser.add_argument("--legacy-probe", metavar="PATH")
    parser.add_argument("--legacy-commit", default="unknown")
    parser.add_argument("--corpus", default=str(DEFAULT_CORPUS))
    parser.add_argument("--cases", default=CORE_ID,
                        help="comma-separated case ids")
    parser.add_argument("--evals", required=True,
                        help="comma-separated child-eval caps")
    parser.add_argument("--repeats", type=int, default=3)
    parser.add_argument("--memory", default=str(GATE_MEMORY),
                        metavar="MB[,MB...]")
    parser.add_argument("--out", required=True)
    args = parser.parse_args()
    if not args.atomi and not args.legacy_probe:
        parser.error("need at least one --atomi or --legacy-probe")
    return args


def atomi_sides(specs: list[str]) -> list[tuple[str, str]]:
    sides = []
    for spec in specs:
        name, sep, path = spec.partition("=")
        if not sep:
            name, path = "atomi", spec
        if name == LEGACY_SIDE or name in {n for n, _ in sides}:
            raise SystemExit(f"duplicate or reserved side name: {name}")
        sides.append((name, path))
    return sides


def spawn(cmd: list[str]):
    """One measured process; returns (wall_s, peak_rss_kib, last JSON)."""
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


def atomi_row(side: str, path: str, case: dict, cap: int, memory: int,
              run: int) -> dict:
    wall, peak, payload = spawn([path, "prove", case["fen"],
                                 "--evals", str(cap), "--seconds", "600",
                                 "--memory", str(memory)])
    child = payload.get("child_evals") or 0
    exp = payload.get("expansions") or 0
    ms = payload.get("ms") or 0
    row = {
        "side": side, "case": case["id"], "evals_cap": cap,
        "memory_mb": memory, "run": run,
        "wall_s": round(wall, 3), "peak_rss_kib": peak,
        "ms": ms, "child_evals": child, "expansions": exp,
        "exit": payload.get("exit"),
        "early": payload.get("exit") in ("proved", "refuted"),
        "proof_entries": payload.get("proof_entries"),
        "proof_full": payload.get("proof_full"),
    }
    if child:
        row["ms_per_1m_child_evals"] = round(ms * 1e6 / child, 3)
    if exp:
        row["ns_per_expansion"] = round(ms * 1e6 / exp, 1)
        row["child_evals_per_expansion"] = round(child / exp, 2)
    return row


def probe_row(path: str, case: dict, cap: int, run: int) -> dict:
    wall, peak, payload = spawn([path, case["fen"], str(cap)])
    evals = payload.get("evals") or 0
    nodes = payload.get("nodes") or 0
    solve_s = payload.get("wall_s") or 0.0
    proved = payload.get("proved")
    row = {
        "side": LEGACY_SIDE, "case": case["id"], "evals_cap": cap,
        "memory_mb": None, "run": run,
        "wall_s": round(wall, 3), "peak_rss_kib": peak,
        "ms": round(solve_s * 1000, 3), "child_evals": evals,
        "expansions": nodes,
        "exit": None if proved is None else ("proved" if proved else "evals"),
        "early": bool(proved),
        "depth": payload.get("depth"),
    }
    if evals:
        row["ms_per_1m_child_evals"] = round(solve_s / evals * 1e9, 3)
    if nodes:
        row["ns_per_expansion"] = round(solve_s * 1e9 / nodes, 1)
        row["child_evals_per_expansion"] = round(evals / nodes, 2)
    return row


def median_of(rows: list[dict], key: str):
    vals = [r[key] for r in rows if r.get(key) is not None]
    return statistics.median(vals) if vals else None


def cell_key(side: str, case: str, cap: int, memory) -> str:
    return f"{side}:{case}:{cap}:{'fe32' if memory is None else memory}"


def sha256(path: str) -> str:
    try:
        return hashlib.sha256(Path(path).read_bytes()).hexdigest()
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


def provenance(args: argparse.Namespace, sides: list[tuple[str, str]],
               memories: list[int]) -> dict:
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
    atomi = {}
    for name, path in sides:
        ver = subprocess.run([path, "--version"], capture_output=True,
                             text=True, check=False).stdout.strip()
        atomi[name] = {"version": ver, "binary": path,
                       "binary_sha256": sha256(path)}
    legacy = None
    if args.legacy_probe:
        legacy = {"binary": args.legacy_probe, "commit": args.legacy_commit,
                  "binary_sha256": sha256(args.legacy_probe),
                  "table_mb": 32}
    return {
        "date": datetime.now(timezone.utc).strftime("%Y-%m-%d"),
        "host": platform.node(), "platform": platform.platform(),
        "cpu_model": cpu_model(), "cpu_count": os.cpu_count(),
        "rustc": rustc, "atomic_movegen": movegen,
        "atomi": atomi, "legacy_probe": legacy,
        "repo_head": head, "repo_dirty": dirty,
        "budgets": {"memory_mb": memories, "repeats": args.repeats,
                    "seconds": 600},
        "corpus": args.corpus,
    }


def main() -> int:
    args = parse_args()
    sides = atomi_sides(args.atomi)
    memories = [int(m) for m in args.memory.split(",") if m.strip()]
    data = json.loads(Path(args.corpus).read_text())
    all_cases = [data["core"], *data["sample20"]]
    wanted = [c.strip() for c in args.cases.split(",") if c.strip()]
    cases = [c for c in all_cases if c["id"] in wanted]
    if len(cases) != len(wanted):
        missing = set(wanted) - {c["id"] for c in cases}
        raise SystemExit(f"unknown case ids: {missing}")
    caps = [int(n) for n in args.evals.split(",")]

    # The measured configurations, in interleaving order.
    configs: list[tuple[str, int | None]] = [
        (name, mem) for name, _ in sides for mem in memories]
    if args.legacy_probe:
        configs.append((LEGACY_SIDE, None))
    paths = dict(sides)

    runs: list[dict] = []
    for case in cases:
        for cap in caps:
            for i in range(args.repeats):
                for side, mem in configs:
                    if side == LEGACY_SIDE:
                        row = probe_row(args.legacy_probe, case, cap, i + 1)
                    else:
                        row = atomi_row(side, paths[side], case, cap, mem,
                                        i + 1)
                    runs.append(row)
                    print(json.dumps({k: row.get(k) for k in (
                        "side", "case", "evals_cap", "memory_mb", "run",
                        "exit", "child_evals", "ms_per_1m_child_evals")}),
                        flush=True)

    cells: dict[str, dict] = {}
    for case in cases:
        for cap in caps:
            for side, mem in configs:
                group = [r for r in runs
                         if r["side"] == side and r["case"] == case["id"]
                         and r["evals_cap"] == cap and r["memory_mb"] == mem]
                at_cap = [r for r in group
                          if r["exit"] == "evals" and not r["early"]]
                early = [r for r in group if r["early"]]
                cell: dict = {"side": side, "case": case["id"],
                              "evals_cap": cap, "memory_mb": mem,
                              "n_runs": len(group)}
                if at_cap:
                    cell["median"] = {
                        k: median_of(at_cap, k) for k in (
                            "ms_per_1m_child_evals", "ns_per_expansion",
                            "child_evals_per_expansion", "peak_rss_kib",
                            "wall_s", "ms", "child_evals")}
                    cell["median"]["n_at_cap"] = len(at_cap)
                if early:
                    cell["early_runs"] = len(early)
                cells[cell_key(side, case["id"], cap, mem)] = cell

    # M1 reference (plan1 semantics): the side `m1` if present, else the
    # first atomi side; the first memory.
    m1_reference = None
    if sides:
        names = [n for n, _ in sides]
        ref_side = "m1" if "m1" in names else names[0]
        ref_mem = memories[0]
        for cap in sorted(caps, reverse=True):
            group = [r for r in runs
                     if r["side"] == ref_side and r["case"] == CORE_ID
                     and r["evals_cap"] == cap and r["memory_mb"] == ref_mem]
            at_cap = [r for r in group if r["exit"] == "evals"]
            if group and len(at_cap) == len(group):
                m1_reference = {
                    "side": ref_side, "memory_mb": ref_mem,
                    "evals_cap": cap,
                    "ms_per_1m_child_evals": median_of(
                        at_cap, "ms_per_1m_child_evals"),
                    "ns_per_expansion": median_of(at_cap, "ns_per_expansion"),
                }
                break

    # The G1 gate cell: core, the largest cap, 192 MB.
    gate = None
    if any(c["id"] == CORE_ID for c in cases):
        cap = max(caps)

        def gate_median(side: str, mem):
            cell = cells.get(cell_key(side, CORE_ID, cap, mem))
            if not cell or "median" not in cell:
                return None
            if cell["median"]["n_at_cap"] != cell["n_runs"]:
                return None  # an early run makes the cell non-comparable
            return cell["median"]["ms_per_1m_child_evals"]

        m1 = gate_median("m1", GATE_MEMORY)
        m0 = gate_median("m0", GATE_MEMORY)
        fe = gate_median(LEGACY_SIDE, None)
        gate = {
            "case": CORE_ID, "evals_cap": cap, "memory_mb": GATE_MEMORY,
            "m1": m1, "m0": m0, LEGACY_SIDE: fe,
            "pass_m0": None if m1 is None or m0 is None else m1 <= m0,
            "pass_legacy": None if m1 is None or fe is None else m1 <= fe,
        }

    result = {
        "provenance": provenance(args, sides, memories),
        "caps": caps,
        "sides": [name for name, _ in configs],
        "runs": runs,
        "cells": cells,
        "m1_reference": m1_reference,
        "gate": gate,
    }
    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(result, indent=1) + "\n")
    # A flat CSV next to the JSON.
    columns = ("side", "case", "evals_cap", "memory_mb", "run", "wall_s",
               "peak_rss_kib", "ms", "child_evals", "expansions", "exit",
               "early", "ms_per_1m_child_evals", "ns_per_expansion",
               "child_evals_per_expansion")
    csv = out.with_suffix(".csv")
    with csv.open("w") as fh:
        fh.write(",".join(columns) + "\n")
        for r in runs:
            fh.write(",".join(
                "" if r.get(k) is None else str(r[k]) for k in columns) + "\n")
    print(f"wrote {out} and {csv}")
    for key, cell in cells.items():
        if "median" in cell:
            print(f"{key}: {cell['median']['ms_per_1m_child_evals']} ms/1M "
                  f"({cell['median']['n_at_cap']}/{cell['n_runs']} at cap)")
    if m1_reference:
        print(f"m1_reference: {json.dumps(m1_reference)}")
    if gate:
        print(f"gate: {json.dumps(gate)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
