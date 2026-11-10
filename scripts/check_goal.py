#!/usr/bin/env python3
"""Goal-consistency gate (D30).

The goal is the top rung of the D30 staircase. It is stated verbatim in four
normative documents; this gate fails if any of them drifts. Historical records
(history.md, reportN.md, planN.md, bench.md ledger rows) are not scanned.

Run from `make lint`; exit 1 on any drift.
"""

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# The canonical goal sentence. Whitespace is normalised before matching, so
# line wrapping in the docs is allowed; the words and punctuation are not.
CANONICAL = (
    "T_5 (goal): all three cells proved at 10 s, geomean wall < 1.0 vs "
    "provari --guide hand, every claim D14-accepted."
)

FILES = [
    "AGENTS.md",
    "docs/plans/prover/initiative.md",
    "docs/plans/prover/bench.md",
    "docs/plans/prover/decisions.md",
]

# Headings that the D30 reformulation removed from the normative docs.
FORBIDDEN = [
    "Tier 1 — coverage",
    "Tier 2 — wall",
]


def norm(text: str) -> str:
    return " ".join(text.split())


def main() -> int:
    canon = norm(CANONICAL)
    failures: list[str] = []
    for rel in FILES:
        path = ROOT / rel
        if not path.is_file():
            failures.append(f"{rel}: missing")
            continue
        text = norm(path.read_text(encoding="utf-8"))
        count = text.count(canon)
        if count != 1:
            failures.append(
                f"{rel}: canonical T_5 line found {count} times (want 1)"
            )
        for bad in FORBIDDEN:
            if norm(bad) in text:
                failures.append(f"{rel}: stale tier heading {bad!r}")
    if failures:
        print("goal check FAILED (D30):", file=sys.stderr)
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        return 1
    print(f"goal check OK: T_5 stated once in {len(FILES)} normative docs")
    return 0


if __name__ == "__main__":
    sys.exit(main())
