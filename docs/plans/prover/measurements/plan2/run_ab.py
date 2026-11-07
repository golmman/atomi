#!/usr/bin/env python3
"""plan2 M2 (wall guard) and M4 (`unsafe` keep rules): interleaved A/B runs.

Runs `atomi prove FEN [FLAGS]` for every `--side NAME=PATH`, interleaved
inside each repeat (repeat 1: all sides; repeat 2: ...), one process at a
time. Per run: process wall (perf_counter around spawn + reap), the child's
peak RSS (`os.wait4` rusage of that child only), and the JSON fields exit,
length, child_evals, expansions, ms. Per side: medians of wall_s, ms,
ms per 1 M child evals and peak RSS, plus the set of child_evals seen
(a deterministic search gives one value).

The result is stored under `--label` in `--out` (a JSON object of labels;
other labels in the file are kept), so one file collects every A/B.

Usage:
    python3 docs/plans/prover/measurements/plan2/run_ab.py --label prefetch \\
        --side m1=target/release/atomi --side noprefetch=/tmp/plan2-ab-nopf/target/release/atomi \\
        --case core --flags "--evals 10000000 --memory 192 --seconds 600" \\
        --repeats 5 --out docs/plans/prover/measurements/plan2/ab.json

Stdlib only. No transcripts; no `__pycache__`.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import shlex
import statistics
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

sys.dont_write_bytecode = True

REPO = Path(__file__).resolve().parents[5]
CORPUS = REPO / "bench" / "corpus.json"


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--label", required=True)
    p.add_argument("--side", action="append", required=True,
                   metavar="NAME=PATH")
    src = p.add_mutually_exclusive_group(required=True)
    src.add_argument("--case", help="bench/corpus.json case id")
    src.add_argument("--fen")
    p.add_argument("--flags", default="", help="extra `atomi prove` flags")
    p.add_argument("--repeats", type=int, default=3)
    p.add_argument("--note", default="")
    p.add_argument("--out", required=True)
    return p.parse_args()


def fen_of(args: argparse.Namespace) -> tuple[str, str]:
    if args.fen:
        return args.fen, "fen"
    data = json.loads(CORPUS.read_text())
    for c in [data["core"], *data["sample20"]]:
        if c["id"] == args.case:
            return c["fen"], c["id"]
    raise SystemExit(f"unknown case id {args.case}")


def run_once(cmd: list[str]) -> dict:
    start = time.perf_counter()
    proc = subprocess.Popen(cmd, stdout=subprocess.PIPE,
                            stderr=subprocess.STDOUT, text=True)
    out = proc.stdout.read()
    _pid, status, rusage = os.wait4(proc.pid, 0)
    wall = time.perf_counter() - start
    payload = {}
    for line in reversed(out.splitlines()):
        if line.strip().startswith("{"):
            try:
                payload = json.loads(line)
                break
            except json.JSONDecodeError:
                continue
    row = {"wall_s": round(wall, 4), "peak_rss_kib": rusage.ru_maxrss,
           "returncode": os.waitstatus_to_exitcode(status),
           **{k: payload.get(k) for k in (
               "exit", "length", "child_evals", "expansions", "ms",
               "proof_entries", "proof_full")}}
    if row["child_evals"]:
        row["ms_per_1m_child_evals"] = round(
            row["ms"] * 1e6 / row["child_evals"], 3)
    return row


def sha256(path: str) -> str:
    try:
        return hashlib.sha256(Path(path).read_bytes()).hexdigest()
    except OSError:
        return "unknown"


def main() -> int:
    args = parse_args()
    sides = []
    for spec in args.side:
        name, sep, path = spec.partition("=")
        if not sep:
            raise SystemExit(f"--side needs NAME=PATH: {spec}")
        sides.append((name, path))
    fen, case = fen_of(args)
    flags = shlex.split(args.flags)

    runs: dict[str, list[dict]] = {name: [] for name, _ in sides}
    for i in range(args.repeats):
        for name, path in sides:
            row = run_once([path, "prove", fen, *flags])
            row["run"] = i + 1
            runs[name].append(row)
            print(json.dumps({"side": name, **row}), flush=True)

    def med(rows: list[dict], key: str):
        vals = [r[key] for r in rows if r.get(key) is not None]
        return statistics.median(vals) if vals else None

    summary = {
        name: {
            "wall_s": med(rows, "wall_s"), "ms": med(rows, "ms"),
            "ms_per_1m_child_evals": med(rows, "ms_per_1m_child_evals"),
            "peak_rss_kib": med(rows, "peak_rss_kib"),
            "child_evals": sorted({r["child_evals"] for r in rows
                                   if r["child_evals"] is not None}),
            "exits": sorted({str(r["exit"]) for r in rows}),
        }
        for name, rows in runs.items()
    }
    entry = {
        "date": datetime.now(timezone.utc).strftime("%Y-%m-%d"),
        "case": case, "fen": fen, "flags": flags, "repeats": args.repeats,
        "note": args.note,
        "sides": {name: {"binary": path, "binary_sha256": sha256(path)}
                  for name, path in sides},
        "summary": summary, "runs": runs,
    }
    out = Path(args.out)
    data = json.loads(out.read_text()) if out.exists() else {}
    data[args.label] = entry
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(data, indent=1) + "\n")
    print(json.dumps({args.label: summary}, indent=1))
    return 0


if __name__ == "__main__":
    sys.exit(main())
