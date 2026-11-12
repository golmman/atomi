#!/usr/bin/env python3
"""Plan 25 Phase A: per-node attribution on `deep-win` (parsed results only).

Sequential, one process at a time, `--guide hand --memory 192 --stats`:

  * `deep-win`, seeds 0-2, at a 20 M child-eval cap and at a 10 s cap;
  * records the tree counters, the new `null_calls`/`null_replies`
    observation counters, and the derived per-expansion costs.

The script writes parsed JSON only; raw stdout/stderr stays in scratch.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import platform
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

sys.dont_write_bytecode = True

CELL = "deep-win"
FEN = "rnb1kbnr/pppp2pp/5q2/4ppN1/3P4/8/PPP1PPPP/RNBQKB1R w KQkq - 0 1"
SEEDS = (0, 1, 2)
CAPS = (("evals", 20_000_000), ("seconds", 10), ("evals_group", 20_000_000))


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--atomi", default="target/release/atomi")
    p.add_argument("--out", default="measurements/plan25/phase_a.json")
    return p.parse_args()


def last_json(text: str) -> dict:
    for line in reversed(text.splitlines()):
        line = line.strip()
        if line.startswith("{"):
            try:
                return json.loads(line)
            except json.JSONDecodeError:
                continue
    return {}


def run(cmd: list[str], timeout: int) -> tuple[int, str, float]:
    start = time.perf_counter()
    try:
        proc = subprocess.run(cmd, capture_output=True, text=True,
                              timeout=timeout + 120, check=False)
        return proc.returncode, proc.stdout + proc.stderr, time.perf_counter() - start
    except subprocess.TimeoutExpired:
        return 124, "", time.perf_counter() - start


def main() -> int:
    args = parse_args()
    rows = []
    for kind, cap in CAPS:
        for seed in SEEDS:
            flag = "evals" if kind.startswith("evals") else "seconds"
            cmd = [args.atomi, "prove", FEN, "--guide", "hand",
                   "--memory", "192", "--seed", str(seed), "--stats",
                   f"--{flag}", str(cap)]
            if kind in ("evals", "evals_group"):
                cmd += ["--seconds", "600"]
            if kind == "evals_group":
                cmd += ["--guide-group"]
            code, text, wall = run(cmd, cap if kind == "seconds" else 600)
            row = last_json(text)
            stats = row.get("stats") or {}
            expansions = row.get("expansions") or 0
            child_evals = row.get("child_evals") or 0
            aux = row.get("aux_nodes") or 0
            plain = stats.get("plain_children") or 0
            rows.append({
                "cell": CELL,
                "cap_kind": kind,
                "cap": cap,
                "seed": seed,
                "code": code,
                "wall_s": round(wall, 3),
                "exit": row.get("exit"),
                "length": row.get("length"),
                "child_evals": child_evals,
                "expansions": expansions,
                "ms": row.get("ms"),
                "us_per_expansion": round(wall * 1e6 / expansions, 3) if expansions else None,
                "aux_nodes": aux,
                "aux_per_expansion": round(aux / expansions, 4) if expansions else None,
                "sim_nodes": row.get("sim_nodes"),
                "settle_nodes": row.get("settle_nodes"),
                "plain_children": plain,
                "plain_share": round(plain / child_evals, 4) if child_evals else None,
                "null_calls": stats.get("null_calls"),
                "null_replies": stats.get("null_replies"),
                "null_replies_per_call": (round(stats.get("null_replies", 0) / stats["null_calls"], 3)
                                          if stats.get("null_calls") else None),
            })
            print(kind, cap, seed, row.get("exit"), round(wall, 3), flush=True)
    data = {
        "date": datetime.now(timezone.utc).strftime("%Y-%m-%d"),
        "atomi": args.atomi,
        "atomi_sha256": hashlib.sha256(Path(args.atomi).read_bytes()).hexdigest(),
        "host": platform.node(),
        "platform": platform.platform(),
        "machine": platform.machine(),
        "rows": rows,
    }
    Path(args.out).write_text(json.dumps(data, indent=2) + "\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
