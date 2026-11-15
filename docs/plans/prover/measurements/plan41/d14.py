#!/usr/bin/env python3
"""Plan-41 D14 — export every proved (cell, seed) at the candidate weight and
claim it with both checkers; also compare the timed run's `child_evals` with an
untimed `--seconds 600 --proof-out` run (G-41c).

    python3 docs/plans/prover/measurements/plan41/d14.py \
        --atomi target/release/atomi \
        --provari /tmp/atomi-oracle/provari/target/release/provari \
        --weight 2 --out docs/plans/prover/measurements/plan41/d14.json
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


def prove(atomi, fen, seed, weight, seconds, path):
    return jline([
        atomi, "prove", fen, "--memory", "192", "--seed", str(seed),
        "--seconds", str(seconds), "--guide", "hand",
        "--guide-ef-weight", str(weight), "--proof-out", path,
    ])


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--atomi", required=True)
    ap.add_argument("--provari", required=True)
    ap.add_argument("--weight", type=int, default=2)
    ap.add_argument("--seconds", default="120")
    ap.add_argument("--untimed", default="600")
    ap.add_argument("--seeds", default="0,1,2")
    ap.add_argument("--cells", default=",".join(CELLS))
    ap.add_argument("--out", required=True)
    ap.add_argument("--dir", default="/tmp/atomi-plan41-proofs")
    args = ap.parse_args()
    os.makedirs(args.dir, exist_ok=True)
    rows = []
    accepted = 0
    for cell in args.cells.split(","):
        for seed in args.seeds.split(","):
            path = os.path.join(args.dir, f"{cell}-s{seed}-w{args.weight}.jsonl")
            timed = prove(args.atomi, CELLS[cell], seed, args.weight, args.seconds, path)
            untimed_path = os.path.join(args.dir, f"{cell}-s{seed}-w{args.weight}-600.jsonl")
            untimed = prove(args.atomi, CELLS[cell], seed, args.weight, args.untimed, untimed_path)
            row = {
                "cell": cell, "seed": int(seed), "weight": args.weight,
                "wall_ms": timed.get("ms"), "found": timed.get("found"),
                "exit": timed.get("exit"), "length": timed.get("length"),
                "child_evals": timed.get("child_evals"),
                "untimed_child_evals": untimed.get("child_evals"),
                "untimed_found": untimed.get("found"),
                "untimed_exit": untimed.get("exit"),
                "evals_match": timed.get("child_evals") == untimed.get("child_evals"),
                "proof": timed.get("proof"),
            }
            if timed.get("found") and timed.get("proof") == "written":
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
                  f"evals_match={row['evals_match']} claim={row['claim']}", flush=True)
    out = {"weight": args.weight, "seconds": int(args.seconds),
           "untimed_seconds": int(args.untimed), "rows": rows,
           "claimed": accepted, "runs": len(rows)}
    with open(args.out, "w") as f:
        json.dump(out, f, indent=2)
        f.write("\n")
    print(f"claimed {accepted}/{len(rows)}")


if __name__ == "__main__":
    main()
