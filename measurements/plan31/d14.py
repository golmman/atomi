#!/usr/bin/env python3
"""Plan-31 D14 — export every proved (cell, seed) and claim it with both
checkers (this repo's `atomi validate` and the external `provari check`).

A win is claimed only when both accept the exported proof (D14). The export
never changes the trajectory, so the wall recorded here is the same run's.

    python3 measurements/plan31/d14.py --atomi target/release/atomi \
        --provari /tmp/atomi-oracle/provari/target/release/provari \
        --out measurements/plan31/d14.json
"""

import argparse
import json
import os
import subprocess

CELLS = {
    "core": "r1b1kbnr/ppp3pp/n4q2/3pppN1/1P1PP3/8/P1P2PPP/RNBQKB1R w KQkq - 0 1",
    "easy-win": "rnbq1b1r/pp1p1kpp/2p1pp1n/8/3PP1P1/2N2N2/PPP2P1P/R1BQKB1R w KQ - 0 1",
    "deep-win": "rnb1kbnr/pppp2pp/5q2/4ppN1/3P4/8/PPP1PPPP/RNBQKB1R w KQkq - 0 1",
}


def jline(cmd):
    out = subprocess.run(cmd, capture_output=True, text=True)
    try:
        return json.loads(out.stdout.strip().splitlines()[-1])
    except Exception:
        return {"error": out.stdout.strip() + out.stderr.strip()}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--atomi", required=True)
    ap.add_argument("--provari", required=True)
    ap.add_argument("--seconds", default="120")
    ap.add_argument("--seeds", default="0,1,2")
    ap.add_argument("--cells", default=",".join(CELLS))
    ap.add_argument("--out", required=True)
    ap.add_argument("--dir", default="/tmp/atomi-plan31-proofs")
    args = ap.parse_args()
    os.makedirs(args.dir, exist_ok=True)
    rows = []
    accepted = 0
    for cell in args.cells.split(","):
        for seed in args.seeds.split(","):
            path = os.path.join(args.dir, f"{cell}-s{seed}.jsonl")
            prove = jline([
                args.atomi, "prove", CELLS[cell], "--memory", "192", "--seed", seed,
                "--seconds", args.seconds, "--guide", "hand", "--proof-out", path,
            ])
            row = {"cell": cell, "seed": int(seed), "wall_ms": prove.get("ms"),
                   "found": prove.get("found"), "exit": prove.get("exit"),
                   "length": prove.get("length"), "proof": prove.get("proof")}
            if prove.get("found") and prove.get("proof") == "written":
                v = jline([args.atomi, "validate", path])
                c = jline([args.provari, "check", path])
                row["ours"] = v
                row["checker"] = c
                row["claim"] = (
                    bool(v.get("valid"))
                    and bool(c.get("valid"))
                    and v.get("positions") == c.get("won_positions")
                )
                accepted += int(row["claim"])
            else:
                row["claim"] = False
            rows.append(row)
            print(f"{cell:9s} seed={seed} wall={row['wall_ms']} exit={row['exit']} "
                  f"claim={row['claim']}", flush=True)
    out = {"seconds": int(args.seconds), "rows": rows,
           "claimed": accepted, "runs": len(rows)}
    with open(args.out, "w") as f:
        json.dump(out, f, indent=2)
        f.write("\n")
    print(f"claimed {accepted}/{len(rows)}")


if __name__ == "__main__":
    main()
