#!/usr/bin/env python3
"""Plan 27 tree-identity probe: the cleaned NULL-V2 cache at 5 M child evals.

One process at a time. The default is 2^17 entries (the measured candidate);
`0` disables the cache and `2097152` requests the tree-changing 2^21 variant.
The control reference is plan26's pre-change `deep-win` run (168,910
expansions), reproduced by `report27`.

Writes parsed JSON only; no transcripts, no proof files.
"""

from __future__ import annotations

import json
import subprocess
from pathlib import Path

FEN = "rnb1kbnr/pppp2pp/5q2/4ppN1/3P4/8/PPP1PPPP/RNBQKB1R w KQkq - 0 1"
ATOMI = Path(__file__).resolve().parents[2] / "target/release/atomi"
CONFIGS = {
    "default-2^17": [],
    "disabled-0": ["--null-cache-entries", "0"],
    "requested-2^21": ["--null-cache-entries", "2097152"],
}
CONTROL_EXPANSIONS = 168_910


def run(extra: list[str]) -> dict:
    cmd = [
        str(ATOMI),
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
        *extra,
    ]
    out = subprocess.run(cmd, check=True, capture_output=True, text=True).stdout
    d = json.loads(out)
    s = d.get("stats", {})
    return {
        "child_evals": d["child_evals"],
        "expansions": d["expansions"],
        "wall_s": round(d["ms"] / 1000, 3),
        "null_cache_hits": s.get("null_cache_hits"),
        "null_cache_misses": s.get("null_cache_misses"),
        "tree_identical_to_control": d["expansions"] == CONTROL_EXPANSIONS,
    }


def main() -> None:
    print(
        json.dumps(
            {
                "cell": "deep-win",
                "fen": FEN,
                "cap": {"kind": "evals", "value": 5_000_000, "seed": 0},
                "control_expansions": CONTROL_EXPANSIONS,
                "configs": {name: run(extra) for name, extra in CONFIGS.items()},
            },
            indent=2,
        )
    )


if __name__ == "__main__":
    main()
