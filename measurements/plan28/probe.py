#!/usr/bin/env python3
"""Plan 28 G-H1 probe: 5 M child evals on `deep-win`, seed 0.

Runs the control, the S3-only build and the S1+S2+S3 candidate back to back,
one process at a time, three repeats each. The gate is the same-session
median wall ratio; `child_evals` and `expansions` must be identical.

The candidate uses a local `atomic-movegen` checkout (MG1 `Board::null_for`,
MG2 public `for_each_pseudo_legal`) through a measurement-only path
dependency. The control and S3 builds use the published 2.4.0 crate.

Writes parsed JSON only; no transcripts, no proof files.
"""

from __future__ import annotations

import argparse
import json
import statistics
import subprocess
from pathlib import Path

FEN = "rnb1kbnr/pppp2pp/5q2/4ppN1/3P4/8/PPP1PPPP/RNBQKB1R w KQkq - 0 1"
CONTROL_EXPANSIONS = 168_910


def run(binary: str, reps: int) -> dict:
    rows = []
    for _ in range(reps):
        cmd = [
            binary,
            "prove",
            FEN,
            "--seconds",
            "600",
            "--memory",
            "192",
            "--evals",
            "5000000",
            "--seed",
            "0",
            "--guide",
            "hand",
            "--stats",
        ]
        out = subprocess.run(cmd, check=True, capture_output=True, text=True).stdout
        d = json.loads(out)
        rows.append(
            {
                "ms": d["ms"],
                "child_evals": d["child_evals"],
                "expansions": d["expansions"],
                "null_calls": d.get("stats", {}).get("null_calls"),
                "null_replies": d.get("stats", {}).get("null_replies"),
            }
        )
    return {
        "runs": rows,
        "median_ms": statistics.median(r["ms"] for r in rows),
        "child_evals": sorted({r["child_evals"] for r in rows}),
        "expansions": sorted({r["expansions"] for r in rows}),
        "tree_identical_to_control": all(
            r["expansions"] == CONTROL_EXPANSIONS for r in rows
        ),
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--control", required=True)
    parser.add_argument("--s3", required=True)
    parser.add_argument("--candidate", required=True)
    parser.add_argument("--repeats", type=int, default=3)
    parser.add_argument("--out", required=True)
    args = parser.parse_args()
    configs = {
        "control": run(args.control, args.repeats),
        "s3": run(args.s3, args.repeats),
        "candidate": run(args.candidate, args.repeats),
    }
    control = configs["control"]["median_ms"]
    for name, row in configs.items():
        row["ratio_to_control"] = round(row["median_ms"] / control, 4)
        row["speedup"] = round(control / row["median_ms"], 4)
    payload = {
        "cell": "deep-win",
        "fen": FEN,
        "cap": {"kind": "evals", "value": 5_000_000, "seed": 0},
        "control_expansions": CONTROL_EXPANSIONS,
        "gate": {"max_ratio": 0.667, "min_speedup": 1.5},
        "configs": configs,
        "verdict": "pass"
        if configs["candidate"]["ratio_to_control"] <= 0.667
        and configs["candidate"]["tree_identical_to_control"]
        else "miss",
    }
    Path(args.out).write_text(json.dumps(payload, indent=2) + "\n")


if __name__ == "__main__":
    main()
