#!/usr/bin/env python3
"""Plan-32 Phase 0 — the proof-probe matrix (item 39, PROOF-FILTER).

Two measurements, one process at a time, same-session ratios:

1. `--mode counts`: on a scratch build that carries the plan's counters (patch
   in the README; never committed) report how a `Tables` lookup resolves, per
   call site: proof hit, proof key present but unusable at this counter, proof
   key absent, TT hit, TT miss, plus whether the "TT or proof" filter was still
   answering and when it dropped.
2. `--mode time`: interleaved control vs prototype runs at a fixed eval cap or
   a wall cap, with the prototype's variants selected by env switches
   (`ATOMI_PF_MB` / `ATOMI_PF_KB` / `ATOMI_PF_FROM` / `ATOMI_PF_DROP`), so the
   second-filter sizes are measured, not assumed.

    python3 measurements/plan32/phase0.py --mode counts \
        --probe /tmp/atomi-plan32-probe/target/release/atomi \
        --out measurements/plan32/counts.json
    python3 measurements/plan32/phase0.py --mode time \
        --control /workspace/atomi/target/release/atomi \
        --proto /tmp/atomi-plan32-proto/target/release/atomi \
        --seconds 120 --repeats 1 --out measurements/plan32/time.json
"""

import argparse
import json
import os
import statistics
import subprocess

CELLS = {
    "core": "r1b1kbnr/ppp3pp/n4q2/3pppN1/1P1PP3/8/P1P2PPP/RNBQKB1R w KQkq - 0 1",
    "easy-win": "rnbq1b1r/pp1p1kpp/2p1pp1n/8/3PP1P1/2N2N2/PPP2P1P/R1BQKB1R w KQ - 0 1",
    "deep-win": "rnb1kbnr/pppp2pp/5q2/4ppN1/3P4/8/PPP1PPPP/RNBQKB1R w KQkq - 0 1",
}

# G1 = proof-only lookups at the guide sites; the filter variants add a second,
# negative-exact bit filter over proof keys at `--from`'s expense.
VARIANTS = {
    "G1": {"ATOMI_PF_MB": "0"},
    "G1+pf256k_proof": {
        "ATOMI_PF_KB": "256",
        "ATOMI_PF_FROM": "proof",
        "ATOMI_PF_DROP": "50",
    },
    "G1+pf512k_proof": {
        "ATOMI_PF_KB": "512",
        "ATOMI_PF_FROM": "proof",
        "ATOMI_PF_DROP": "50",
    },
    "G1+pf2m_proof": {
        "ATOMI_PF_MB": "2",
        "ATOMI_PF_FROM": "proof",
        "ATOMI_PF_DROP": "50",
    },
    "G1+pf4m_proof": {
        "ATOMI_PF_MB": "4",
        "ATOMI_PF_FROM": "proof",
        "ATOMI_PF_DROP": "50",
    },
    "G1+pf2m_tt": {"ATOMI_PF_MB": "2", "ATOMI_PF_FROM": "tt", "ATOMI_PF_DROP": "50"},
}

TREE_KEYS = ("child_evals", "expansions", "proof_entries")


def run(binary, fen, extra, env, evals, seconds):
    e = dict(os.environ)
    e.update(env)
    cmd = [binary, "prove", fen, "--guide", "hand"] + extra
    cmd += ["--evals", str(evals)] if evals else ["--seconds", str(seconds)]
    out = subprocess.run(cmd, capture_output=True, text=True, check=True, env=e).stdout
    return json.loads(out)


def tree(d):
    return {k: d[k] for k in TREE_KEYS}


def mode_counts(args):
    result = {}
    for cell, fen in CELLS.items():
        for evals, seconds in ((args.evals, None), (None, args.seconds)):
            d = run(args.probe, fen, ["--stats"], {}, evals, seconds)
            result[f"{cell}@{evals or str(seconds) + 's'}"] = {
                "fen": fen,
                "ms": d["ms"],
                "child_evals": d["child_evals"],
                "expansions": d["expansions"],
                "proof_entries": d["proof_entries"],
                "stats": d["stats"],
                "probe": d["probe"],
            }
            print(f"{cell} @{evals or str(seconds) + 's'}: {d['ms']} ms", flush=True)
    write(args.out, result)


def mode_time(args):
    result = {}
    for cell, fen in CELLS.items():
        rows = {"control": [], **{k: [] for k in args.variants}}
        for _ in range(args.repeats):
            rows["control"].append(run(args.control, fen, [], {}, args.evals, args.seconds))
            for name in args.variants:
                env = VARIANTS[name]
                rows[name].append(run(args.proto, fen, [], env, args.evals, args.seconds))
        base = tree(rows["control"][0])
        cell_out = {
            "control_ms": [r["ms"] for r in rows["control"]],
            "variants": {},
        }
        for name in args.variants:
            runs = rows[name]
            cell_out["variants"][name] = {
                "env": VARIANTS[name],
                "ms": [r["ms"] for r in runs],
                "ratio": [
                    round(r["ms"] / c["ms"], 4)
                    for r, c in zip(runs, rows["control"], strict=True)
                ],
                "tree_identical": [tree(r) == base for r in runs],
            }
        cell_out["control_median_ms"] = statistics.median(cell_out["control_ms"])
        result[cell] = cell_out
        print(
            f"{cell}: control {cell_out['control_median_ms']} ms, "
            + ", ".join(
                f"{n} {cell_out['variants'][n]['ratio']}" for n in args.variants
            ),
            flush=True,
        )
    write(args.out, result)


def write(path, obj):
    with open(path, "w") as f:
        json.dump(obj, f, indent=1)
        f.write("\n")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--mode", choices=("counts", "time"), required=True)
    ap.add_argument("--probe")
    ap.add_argument("--control")
    ap.add_argument("--proto")
    ap.add_argument("--cells", default="core,easy-win,deep-win")
    ap.add_argument("--evals", type=int, default=0)
    ap.add_argument("--seconds", type=int, default=120)
    ap.add_argument("--repeats", type=int, default=1)
    ap.add_argument("--variants", default=",".join(VARIANTS))
    ap.add_argument("--out", required=True)
    args = ap.parse_args()
    args.variants = [v for v in args.variants.split(",") if v]
    if args.cells != "core,easy-win,deep-win":
        keep = set(args.cells.split(","))
        for k in list(CELLS):
            if k not in keep:
                del CELLS[k]
    if args.mode == "counts":
        mode_counts(args)
    else:
        mode_time(args)


if __name__ == "__main__":
    main()
