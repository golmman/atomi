#!/usr/bin/env python3
"""Plan-33 execution gates — NULL-PIPE (pipelined `null_win` pass 1) against the
pre-change build.

Adapted from `docs/plans/prover/measurements/plan32/execute.py` (same driver, plan-33 gates).
One process at a time; `control` is the `git archive HEAD` build of the
pre-plan-33 source outside the repo, `candidate` is this session's binary.

* `--mode tree`: N evals, the given cells, control vs candidate
  `(child_evals, expansions, proof_entries)` must be equal (G-40b, small tier
  at 1 M and the 5 M `deep-win` probe).
* `--mode wall`: interleaved repeats per cell, seed = repeat (from
  `--seed-start`, so a longer sample extends rather than repeats it), so the
  same runs also give the (cell, seed) 120 s tree identity of G-40b.

    python3 docs/plans/prover/measurements/plan33/execute.py --mode tree \
        --control /tmp/atomi-plan33-control/target/release/atomi \
        --candidate target/release/atomi --evals 1000000 \
        --out docs/plans/prover/measurements/plan33/gate_tree_1m.json
    python3 docs/plans/prover/measurements/plan33/execute.py --mode wall \
        --control /tmp/atomi-plan33-control/target/release/atomi \
        --candidate target/release/atomi --cells deep-win --seconds 120 \
        --repeats 3 --out docs/plans/prover/measurements/plan33/gate_wall_120s.json
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

TREE_KEYS = ("child_evals", "expansions", "proof_entries")


def run(binary, fen, seed, evals, seconds):
    cmd = [binary, "prove", fen, "--guide", "hand", "--memory", "192"]
    cmd += ["--evals", str(evals)] if evals else ["--seconds", str(seconds)]
    cmd += ["--seed", str(seed)]
    return json.loads(subprocess.run(cmd, capture_output=True, text=True, check=True).stdout)


def tree(d):
    t = {k: d[k] for k in TREE_KEYS}
    t["found"] = d["found"]
    t["length"] = d["length"]
    return t


def mode_tree(args):
    out = {}
    ok = True
    for cell, fen in CELLS.items():
        c = run(args.control, fen, 0, args.evals, None)
        v = run(args.candidate, fen, 0, args.evals, None)
        same = tree(c) == tree(v)
        ok &= same
        out[cell] = {"control": tree(c), "candidate": tree(v), "identical": same}
        print(f"{cell} @{args.evals} evals: identical={same} {tree(v)}", flush=True)
    write(args.out, out)
    print("G-40b small tier:", "PASS" if ok else "FAIL")


def mode_wall(args):
    out = {}
    ok = True
    ratios_by_cell = {}
    for cell, fen in CELLS.items():
        rows = []
        for rep in range(args.seed_start, args.seed_start + args.repeats):
            seed = rep
            evals = args.wall_evals or None
            c = run(args.control, fen, seed, evals, args.seconds)
            v = run(args.candidate, fen, seed, evals, args.seconds)
            same = tree(c) == tree(v)
            ok &= same
            ratio = round(v["ms"] / c["ms"], 4)
            rows.append(
                {
                    "seed": seed,
                    "control_ms": c["ms"],
                    "candidate_ms": v["ms"],
                    "ratio": ratio,
                    "tree_identical": same,
                    "tree": tree(c),
                    "found_equal": (c["found"], c["length"]) == (v["found"], v["length"]),
                }
            )
            print(
                f"{cell} seed {seed}: {c['ms']} -> {v['ms']} ms "
                f"({ratio}x, tree identical {same})",
                flush=True,
            )
        ratios = [r["ratio"] for r in rows]
        ratios_by_cell[cell] = ratios
        out[cell] = {
            "control_median_ms": statistics.median(r["control_ms"] for r in rows),
            "candidate_median_ms": statistics.median(r["candidate_ms"] for r in rows),
            "ratios": ratios,
            "median_ratio": round(statistics.median(ratios), 4),
            "max_ratio": max(ratios),
            "runs": rows,
        }
    bars = {"deep-win": 0.95, "core": 0.95, "easy-win": 0.95}
    wall_ok = all(out[c]["median_ratio"] <= bars[c] for c in bars if c in out)
    write(args.out, out)
    print("G-40b tree identity:", "PASS" if ok else "FAIL")
    print("G-40c wall (median <= 0.95):", "PASS" if wall_ok else "FAIL")
    print("medians:", {c: out[c]["median_ratio"] for c in out})


def write(path, obj):
    with open(path, "w") as f:
        json.dump(obj, f, indent=1)
        f.write("\n")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--mode", choices=("tree", "wall"), required=True)
    ap.add_argument("--control", required=True)
    ap.add_argument("--candidate", required=True)
    ap.add_argument("--cells", default="core,easy-win,deep-win")
    ap.add_argument("--evals", type=int, default=1_000_000)
    ap.add_argument("--seconds", type=int, default=120)
    ap.add_argument("--repeats", type=int, default=3)
    ap.add_argument("--seed-start", type=int, default=0)
    ap.add_argument("--wall-evals", type=int, default=0)
    ap.add_argument("--out", required=True)
    args = ap.parse_args()
    if args.cells != "core,easy-win,deep-win":
        keep = set(args.cells.split(","))
        for k in list(CELLS):
            if k not in keep:
                del CELLS[k]
    if args.mode == "tree":
        mode_tree(args)
    else:
        mode_wall(args)


if __name__ == "__main__":
    main()
