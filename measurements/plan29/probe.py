#!/usr/bin/env python3
"""Plan 29 Phase A probe: per-child effect counters on the search's trees.

Runs `atomi prove <cell> --evals N --seconds 600 --memory 192 --seed 0
--guide {hand,none} --stats` once per (cell, cap, stack) on two binaries:
the control (`/tmp/atomi-plan29-control`, built from `git archive HEAD`) and
the Phase A candidate (this worktree, counters on). Counters are
deterministic for a fixed binary/FEN/flags, so a single run per config
reports them; the driver also re-runs one config to pin determinism.

`--stats` counters are observation only: the driver asserts that
`child_evals` and `expansions` are identical between the control and the
counter build, i.e. the instrumentation does not move the tree.

Writes parsed JSON only; no transcripts, no proof files.
"""

from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parent.parent
CORPUS = REPO / "bench/corpus.json"

CELLS = {
    "core": "r1b1kbnr/ppp3pp/n4q2/3pppN1/1P1PP3/8/P1P2PPP/RNBQKB1R w KQkq - 0 1",
    "deep-win": "rnb1kbnr/pppp2pp/5q2/4ppN1/3P4/8/PPP1PPPP/RNBQKB1R w KQkq - 0 1",
}
EVALS_KEYS = (
    "effect_children",
    "effect_needs_board",
    "effect_captures",
    "effect_explosions",
    "effect_blast_hits",
    "effect_decided",
    "effect_extinction",
    "effect_clock",
    "effect_fallback",
    "effect_no_qp",
    "effect_terminal",
    "effect_guide_needs_board",
    "effect_boards_saved",
)


def load_cells() -> dict[str, str]:
    cells = dict(CELLS)
    data = json.loads(CORPUS.read_text())
    for cell in data["sample20"]:
        if cell["id"] == "1eb80b92f92c":
            cells["easy-win"] = cell["fen"]
    assert len(cells) == 3, cells
    return cells


def run(binary: str, fen: str, evals: int, guide: str, stats: bool) -> dict:
    cmd = [
        binary,
        "prove",
        fen,
        "--seconds",
        "600",
        "--memory",
        "192",
        "--evals",
        str(evals),
        "--seed",
        "0",
        "--guide",
        guide,
    ]
    if stats:
        cmd.append("--stats")
    out = subprocess.run(cmd, check=True, capture_output=True, text=True).stdout
    d = json.loads(out)
    row = {
        "child_evals": d["child_evals"],
        "expansions": d["expansions"],
        "exit": d["exit"],
        "ms": d["ms"],
    }
    if stats:
        row["effects"] = {k: d["stats"][k] for k in EVALS_KEYS}
    return row


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--control", required=True)
    parser.add_argument("--candidate", required=True)
    parser.add_argument("--evals", default="1000000,5000000")
    parser.add_argument("--out", required=True)
    args = parser.parse_args()
    cells = load_cells()
    rows = []
    for cap in (int(x) for x in args.evals.split(",")):
        for name, fen in cells.items():
            # The plan's Phase A cells: 1 M on all three, 5 M on `deep-win`.
            if cap == 5_000_000 and name != "deep-win":
                continue
            for guide in ("hand", "none"):
                control = run(args.control, fen, cap, guide, stats=False)
                cand = run(args.candidate, fen, cap, guide, stats=True)
                again = run(args.candidate, fen, cap, guide, stats=True)
                rows.append(
                    {
                        "cell": name,
                        "fen": fen,
                        "evals_cap": cap,
                        "guide": guide,
                        "control": control,
                        "candidate": cand,
                        "tree_identical": (
                            control["child_evals"] == cand["child_evals"]
                            and control["expansions"] == cand["expansions"]
                        ),
                        "counters_deterministic": cand["effects"]
                        == again["effects"]
                        and cand["child_evals"] == again["child_evals"],
                    }
                )
    for row in rows:
        e = row["candidate"]["effects"]
        nb = e["effect_needs_board"] or 1
        e["decided_share_of_built"] = round(e["effect_decided"] / nb, 5)
        e["terminal_share_of_built"] = round(e["effect_terminal"] / nb, 5)
        e["saved_share_of_built"] = round(e["effect_boards_saved"] / nb, 5)
    payload = {
        "probe": (
            "atomi prove <cell> --seconds 600 --memory 192 --evals N --seed 0 "
            "--guide {hand,none} [--stats]; one run per config, one process at "
            "a time; control = git archive HEAD"
        ),
        "control_binary": args.control,
        "candidate_binary": args.candidate,
        "rows": rows,
        "stop_rule": {"decided_share_threshold": 0.10},
    }
    Path(args.out).write_text(json.dumps(payload, indent=2) + "\n")
    print(json.dumps(payload["rows"], indent=None)[:4000])


if __name__ == "__main__":
    main()
