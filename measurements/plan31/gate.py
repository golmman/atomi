#!/usr/bin/env python3
"""Plan-31 (item 38) gate driver — tree identity and same-session wall.

Runs every side once per repeat, **interleaved**, one process at a time
(reading rule 1 / `bench.md`), on one cell and one `--evals` cap, and reports
per-side medians plus the tree-identity check against the control.

    python3 measurements/plan31/gate.py \
        --side control=/tmp/atomi-plan31-control/target/release/atomi \
        --side f1=/tmp/atomi-plan31-f1/target/release/atomi \
        --side all=target/release/atomi \
        --cells deep-win --evals 5000000 --repeats 3 \
        --out measurements/plan31/gate_v2b.json

Sides are `label=path`; `--cells` takes short names (`core`, `easy-win`,
`deep-win`) or the literal corpus ids. `--evals 0` means a wall-clock run and
needs `--seconds`.
"""

import argparse
import json
import statistics
import subprocess
import sys

CELLS = {
    "core": "r1b1kbnr/ppp3pp/n4q2/3pppN1/1P1PP3/8/P1P2PPP/RNBQKB1R w KQkq - 0 1",
    "easy-win": "rnbq1b1r/pp1p1kpp/2p1pp1n/8/3PP1P1/2N2N2/PPP2P1P/R1BQKB1R w KQ - 0 1",
    "deep-win": "rnb1kbnr/pppp2pp/5q2/4ppN1/3P4/8/PPP1PPPP/RNBQKB1R w KQkq - 0 1",
    "1eb80b92f92c": "rnbq1b1r/pp1p1kpp/2p1pp1n/8/3PP1P1/2N2N2/PPP2P1P/R1BQKB1R w KQ - 0 1",
    "db6a741c723f": "rnb1kbnr/pppp2pp/5q2/4ppN1/3P4/8/PPP1PPPP/RNBQKB1R w KQkq - 0 1",
}


def run(path, fen, evals, seconds, seed, guide, extra):
    cmd = [path, "prove", fen, "--memory", "192", "--seed", str(seed),
           "--guide", guide]
    if evals:
        cmd += ["--evals", str(evals)]
    if seconds:
        cmd += ["--seconds", str(seconds)]
    cmd += extra
    out = subprocess.run(cmd, capture_output=True, text=True, check=True)
    return json.loads(out.stdout)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--side", action="append", required=True, metavar="LABEL=PATH")
    ap.add_argument("--cells", default="deep-win")
    ap.add_argument("--evals", default="5000000")
    ap.add_argument("--seconds", default="0")
    ap.add_argument("--seeds", default="0")
    ap.add_argument("--repeats", type=int, default=3)
    ap.add_argument("--guide", default="hand")
    ap.add_argument("--extra", default="", help="extra atomi flags, space separated")
    ap.add_argument("--label", default="")
    ap.add_argument("--out", required=True)
    args = ap.parse_args()

    sides = []
    for spec in args.side:
        label, _, path = spec.partition("=")
        sides.append((label, path))
    control = sides[0][0]
    extra = args.extra.split()

    rows = []
    for cell in args.cells.split(","):
        fen = CELLS[cell]
        for seed in [int(s) for s in args.seeds.split(",")]:
            found = {label: [] for label, _ in sides}
            for _ in range(args.repeats):
                for label, path in sides:
                    found[label].append(
                        run(path, fen, int(args.evals), int(args.seconds), seed,
                            args.guide, extra))
            control_row = found[control][0]
            cells = []
            for label, _path in sides:
                runs = found[label]
                walls = [r["ms"] for r in runs]
                same = all(
                    r["child_evals"] == control_row["child_evals"]
                    and r["expansions"] == control_row["expansions"]
                    for r in runs
                )
                row = {
                    "side": label,
                    "child_evals": [r["child_evals"] for r in runs],
                    "expansions": [r["expansions"] for r in runs],
                    "exit": [r["exit"] for r in runs],
                    "ms": walls,
                    "median_ms": statistics.median(walls),
                    "min_ms": min(walls),
                    "tree_identical_to_control": same,
                }
                cells.append(row)
            base = cells[0]["median_ms"]
            for row in cells:
                row["ratio_vs_control"] = round(row["median_ms"] / base, 4)
            rows.append({"cell": cell, "seed": seed, "evals": int(args.evals),
                         "seconds": int(args.seconds), "sides": cells})
            print(
                f"{cell:9s} seed={seed} " + " ".join(
                    f"{r['side']}={r['median_ms']:.0f}ms({r['ratio_vs_control']:.4f})"
                    f"{'' if r['tree_identical_to_control'] else '!TREE'}"
                    for r in cells
                ),
                file=sys.stderr,
            )
    out = {
        "label": args.label,
        "evals_cap": int(args.evals),
        "seconds": int(args.seconds),
        "repeats": args.repeats,
        "guide": args.guide,
        "extra": extra,
        "control_side": control,
        "rows": rows,
    }
    with open(args.out, "w") as f:
        json.dump(out, f, indent=2)
        f.write("\n")


if __name__ == "__main__":
    main()
