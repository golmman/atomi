#!/usr/bin/env python3
"""Plan-33 tuning — the pass-1 chunk size `K` and the prefetch, at 5 M evals.

Sides are given as `NAME=PATH` (repeatable) plus `--control PATH`. Each repeat
runs every side once, one process at a time, interleaved (`bench.md` reading
rule 1); ratios are side wall / control wall of the same repeat.

The variants are scratch builds outside the repo (`git archive HEAD` + this
session's `src/guide/null.rs` and `src/table/mod.rs`):
`control` = pre-plan-33 (byte-serial pass 1), `k8`/`k16`/`k32` = the shipped
pipeline at that `K`, `nopf` = K = 8 without the `.prefetch_proved` line.

    python3 docs/plans/prover/measurements/plan33/tune.py \
        --control /tmp/atomi-plan33-control/target/release/atomi \
        --side k8=/tmp/atomi-plan33-k8/target/release/atomi ... \
        --evals 5000000 --repeats 4 --out docs/plans/prover/measurements/plan33/tune_5m.json
"""

import argparse
import json
import statistics
import subprocess

CELLS = {
    "core": "r1b1kbnr/ppp3pp/n4q2/3pppN1/1P1PP3/8/P1P2PPP/RNBQKB1R w KQkq - 0 1",
    "easy-win": "rnbq1b1r/pp1p1kpp/2p1pp1n/8/3PP1P1/2N2N2/PPP2P1P/R1BQKB1R w KQ - 0 1",
    "deep-win": "rnb1kbnr/pppp2pp/5q2/4ppN1/3P4/8/PPP1PPPP/RNBQKB1R w KQkq - 0 1",
}

TREE_KEYS = ("child_evals", "expansions", "proof_entries", "found", "length")


def run(binary, fen, seed, evals, seconds):
    cmd = [binary, "prove", fen, "--guide", "hand", "--memory", "192"]
    cmd += ["--evals", str(evals)] if evals else ["--seconds", str(seconds)]
    cmd += ["--seed", str(seed)]
    return json.loads(subprocess.run(cmd, capture_output=True, text=True, check=True).stdout)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--control", required=True)
    ap.add_argument("--side", action="append", default=[])
    ap.add_argument("--cell", default="deep-win")
    ap.add_argument("--evals", type=int, default=5_000_000)
    ap.add_argument("--repeats", type=int, default=4)
    ap.add_argument("--out", required=True)
    args = ap.parse_args()
    fen = CELLS[args.cell]
    sides = [("control", args.control)]
    for s in args.side:
        name, path = s.split("=", 1)
        sides.append((name, path))
    rows = {name: [] for name, _ in sides}
    tree_ok = {name: True for name, _ in sides}
    base = None
    for rep in range(args.repeats):
        for name, path in sides:
            d = run(path, fen, rep, args.evals, None)
            rows[name].append(d["ms"])
            t = {k: d[k] for k in TREE_KEYS}
            if name == "control":
                base = t
            tree_ok[name] &= t == base
            print(f"rep {rep} {name}: {d['ms']} ms", flush=True)
    out = {}
    for name, _ in sides:
        ms = rows[name]
        ratios = [m / c for m, c in zip(ms, rows["control"])]
        out[name] = {
            "ms": ms,
            "median_ms": statistics.median(ms),
            "ratios": [round(r, 4) for r in ratios],
            "median_ratio": round(statistics.median(ratios), 4),
            "tree_identical": tree_ok[name],
        }
    out["_cell"] = args.cell
    out["_evals"] = args.evals
    out["_repeats"] = args.repeats
    with open(args.out, "w") as f:
        json.dump(out, f, indent=1)
        f.write("\n")
    print("medians:", {n: out[n]["median_ratio"] for n, _ in sides})
    print("tree identical:", all(v["tree_identical"] for v in out.values() if isinstance(v, dict)))


if __name__ == "__main__":
    main()
