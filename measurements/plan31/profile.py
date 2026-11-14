#!/usr/bin/env python3
"""Plan-31 Phase 0 attribution profile — `perf` self time, one binary at a time.

`perf record --call-graph none` on a fixed 5 M-eval `--guide hand` run of one
cell, then `perf report --sort symbol` parsed into JSON. Used twice: the
control (pre-plan-31 tree) and the candidate, so the rows before/after name the
same symbols. Nothing here changes the tree.

    python3 measurements/plan31/profile.py --binary /tmp/atomi-plan31-control/target/release/atomi \
        --label control --out measurements/plan31/profile_control.json
"""

import argparse
import json
import re
import subprocess
import tempfile

FEN = "rnb1kbnr/pppp2pp/5q2/4ppN1/3P4/8/PPP1PPPP/RNBQKB1R w KQkq - 0 1"
LINE = re.compile(r"^\s+(\d+\.\d+)%\s+\[\.\] (.*?)\s+-\s+-\s*$")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--binary", required=True)
    ap.add_argument("--label", required=True)
    ap.add_argument("--fen", default=FEN)
    ap.add_argument("--evals", default="5000000")
    ap.add_argument("--limit", type=float, default=0.3)
    ap.add_argument("--out", required=True)
    args = ap.parse_args()
    with tempfile.NamedTemporaryFile(suffix=".data") as tmp:
        cmd = ["perf", "record", "-q", "-o", tmp.name, "--", args.binary, "prove",
               args.fen, "--evals", args.evals, "--memory", "192", "--guide", "hand"]
        run = subprocess.run(cmd, capture_output=True, text=True, check=True)
        result = json.loads(run.stdout)
        rep = subprocess.run(
            ["perf", "report", "-i", tmp.name, "--no-children", "--stdio",
             "--sort", "symbol", f"--percent-limit", str(args.limit)],
            capture_output=True, text=True, check=True,
        ).stdout
    rows = []
    for line in rep.splitlines():
        m = LINE.match(line)
        if m:
            rows.append({"self_percent": float(m.group(1)), "symbol": m.group(2).strip()})
    out = {
        "label": args.label,
        "binary": args.binary,
        "fen": args.fen,
        "evals_cap": int(args.evals),
        "wall_ms": result["ms"],
        "child_evals": result["child_evals"],
        "expansions": result["expansions"],
        "rows": rows,
    }
    with open(args.out, "w") as f:
        json.dump(out, f, indent=2)
        f.write("\n")
    print(f"{args.label}: {len(rows)} symbols >= {args.limit}%", flush=True)


if __name__ == "__main__":
    main()
