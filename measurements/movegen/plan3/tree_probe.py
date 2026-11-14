#!/usr/bin/env python3
"""Plan 3 tree probe: 5 M child evals on `deep-win`, `m0` vs `m1`, alternating.

The wall bench (`bench.py`) censors `deep-win` at both its caps, so the tree
identity of the two dependency pins is checked here at a fixed eval count
instead of at a wall clock. `child_evals` and `expansions` must be identical
and equal to the plan28 control's 168,910 expansions; the wall ratio is the
per-node effect of the crate alone.

Writes parsed JSON only; no transcripts, no proofs.
"""

from __future__ import annotations

import argparse
import json
import statistics
import sys
from pathlib import Path

sys.dont_write_bytecode = True

REPO = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(REPO / "bench"))

import run_bench as rb  # noqa: E402

FEN = "rnb1kbnr/pppp2pp/5q2/4ppN1/3P4/8/PPP1PPPP/RNBQKB1R w KQkq - 0 1"
CONTROL_EXPANSIONS = 168_910


def run(binary: str, evals: int, memory: int) -> dict:
    out = rb.run_process([
        binary, "prove", FEN,
        "--seconds", "600",
        "--memory", str(memory),
        "--evals", str(evals),
        "--seed", "0",
        "--guide", "hand",
        "--stats",
    ], 600)
    payload = rb.last_json(out[1]) or {}
    return {"wall_s": round(out[2], 3), "returncode": out[0],
            "child_evals": payload.get("child_evals"),
            "expansions": payload.get("expansions"),
            "ms": payload.get("ms"), "exit": payload.get("exit")}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--m0", required=True)
    parser.add_argument("--m1", required=True)
    parser.add_argument("--evals", type=int, default=5_000_000)
    parser.add_argument("--memory", type=int, default=192)
    parser.add_argument("--repeats", type=int, default=3)
    parser.add_argument("--out", required=True)
    args = parser.parse_args()

    rows = {"m0": [], "m1": []}
    for _ in range(args.repeats):
        rows["m0"].append(run(args.m0, args.evals, args.memory))
        rows["m1"].append(run(args.m1, args.evals, args.memory))

    sides = {}
    for name, group in rows.items():
        sides[name] = {
            "runs": group,
            "median_ms": statistics.median(r["ms"] for r in group),
            "child_evals": sorted({r["child_evals"] for r in group}),
            "expansions": sorted({r["expansions"] for r in group}),
            "tree_matches_plan28_control": all(
                r["expansions"] == CONTROL_EXPANSIONS for r in group),
        }
    m0, m1 = sides["m0"], sides["m1"]
    payload = {
        "plan": "docs/plans/movegen/plan3.md",
        "cell": "deep-win",
        "fen": FEN,
        "cap": {"kind": "evals", "value": args.evals, "seed": 0,
                "memory_mb": args.memory},
        "control_expansions": CONTROL_EXPANSIONS,
        "sides": sides,
        "tree_identical": (m0["child_evals"] == m1["child_evals"]
                           and m0["expansions"] == m1["expansions"]),
        "m1_over_m0": round(m1["median_ms"] / m0["median_ms"], 4),
        "binaries": {"m0": {"path": args.m0,
                            "sha256": rb.sha256_file(args.m0)},
                     "m1": {"path": args.m1,
                            "sha256": rb.sha256_file(args.m1)}},
    }
    Path(args.out).write_text(json.dumps(payload, indent=1) + "\n")
    print(f"wrote {args.out}")
    print(f"tree_identical={payload['tree_identical']} "
          f"m1_over_m0={payload['m1_over_m0']} "
          f"m0_median_ms={m0['median_ms']} m1_median_ms={m1['median_ms']}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
