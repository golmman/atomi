#!/usr/bin/env python3
"""Plan-33 G-40f — one 120 s `deep-win` profile splitting pass 1's wall.

Two runs, one process at a time, same session: the shipped binary (the wall
reference) and a scratch instrumented build (`/tmp/atomi-plan33-probe`, never
committed) that accumulates rdtsc deltas for the phases of `null_win`'s pass 1:

* `key`   — the whole (A) loop: do_move + `rules::status` + `key`/`rule50` + undo
* `status`— `rules::status` inside (A)
* `keyfn` — `rules::key` + `rule50` inside (A)
* `probe` — the (B) loop: `prefetch_proved` + `proved` per chunk
* `clone` — `null_board` per call

so do_move + undo = `key - status - keyfn`. The instrumented run costs one
rdtsc per measured interval; its own overhead is the wall delta against the
shipped run divided by the reads, and it is given back to each interval (the
raw numbers are kept beside the corrected ones).

    python3 docs/plans/prover/measurements/plan33/profile.py \
        --atomi target/release/atomi \
        --probe /tmp/atomi-plan33-probe/target/release/atomi \
        --tsc-hz 3400013000 --out docs/plans/prover/measurements/plan33/profile_120s.json
"""

import argparse
import json
import subprocess

FEN = "rnb1kbnr/pppp2pp/5q2/4ppN1/3P4/8/PPP1PPPP/RNBQKB1R w KQkq - 0 1"


def prove(binary, seconds, seed):
    out = subprocess.run(
        [binary, "prove", FEN, "--guide", "hand", "--memory", "192",
         "--seconds", str(seconds), "--seed", str(seed)],
        capture_output=True, text=True, check=True,
    )
    summary = json.loads(out.stdout.strip().splitlines()[-1])
    probe = None
    for line in out.stderr.strip().splitlines():
        if '"probe"' in line:
            probe = json.loads(line)["probe"]
    return summary, probe


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--atomi", required=True)
    ap.add_argument("--probe", required=True)
    ap.add_argument("--tsc-hz", type=float, required=True)
    ap.add_argument("--seconds", type=int, default=120)
    ap.add_argument("--seed", type=int, default=0)
    ap.add_argument("--out", required=True)
    args = ap.parse_args()

    ref, _ = prove(args.atomi, args.seconds, args.seed)
    inst, probe = prove(args.probe, args.seconds, args.seed)
    assert probe, "instrumented binary printed no probe line"
    assert (ref["expansions"], ref["child_evals"]) == (inst["expansions"], inst["child_evals"]), \
        "instrumented run moved the tree"

    wall = ref["ms"] / 1000.0
    exp = ref["expansions"]
    moves = probe["moves"]
    calls = probe["calls"]
    tsc_ns = 1e9 / args.tsc_hz
    reads = int(probe.get("reads", 0)) or 4 * moves + 2 * calls
    overhead_s = (inst["ms"] - ref["ms"]) / 1000.0
    per_read = overhead_s / reads

    # (counter, rdtsc reads per unit, units per run): the per-move phases read
    # the clock once at each end of their interval, the clone once per call.
    phases = {
        "status": (probe["status"], 2, moves),
        "keyfn": (probe["keyfn"], 2, moves),
        "probe": (probe["probe"], 2, moves),
        "clone": (probe["clone"], 2, calls),
    }
    rows = {}
    total = 0.0
    for name, (tics, nreads, units) in phases.items():
        raw = tics * tsc_ns / 1e9
        corr = max(0.0, raw - nreads * per_read * units)
        total += corr
        rows[name] = {
            "raw_s": round(raw, 3),
            "corrected_s": round(corr, 3),
            "pp_of_wall": round(100 * corr / wall, 2),
        }
    key_raw = probe["key"] * tsc_ns / 1e9
    key_corr = max(0.0, key_raw - 4 * per_read * moves)
    domove = max(0.0, key_corr - rows["status"]["corrected_s"] - rows["keyfn"]["corrected_s"])
    total += domove
    rows["do_move+undo"] = {
        "raw_s": round(max(0.0, key_raw - rows["status"]["raw_s"] - rows["keyfn"]["raw_s"]), 3),
        "corrected_s": round(domove, 3),
        "pp_of_wall": round(100 * domove / wall, 2),
    }
    for name in ("status", "keyfn", "probe", "do_move+undo", "clone"):
        rows[name]["ns_per_move"] = round(1e9 * rows[name]["corrected_s"] / moves, 2)
        rows[name]["ns_per_expansion"] = round(1e9 * rows[name]["corrected_s"] / exp, 2)
    rows["pass1_total"] = {
        "corrected_s": round(total, 3),
        "pp_of_wall": round(100 * total / wall, 2),
        "ns_per_expansion": round(1e9 * total / exp, 2),
    }
    rows["rest"] = {
        "corrected_s": round(wall - total, 3),
        "pp_of_wall": round(100 * (wall - total) / wall, 2),
        "ns_per_expansion": round(1e9 * (wall - total) / exp, 2),
    }
    out = {
        "fen": FEN,
        "seed": args.seed,
        "seconds": args.seconds,
        "wall_s": wall,
        "instrumented_wall_s": inst["ms"] / 1000.0,
        "expansions": exp,
        "child_evals": ref["child_evals"],
        "null_calls": calls,
        "moves_scanned": moves,
        "moves_per_call": round(moves / calls, 2),
        "tsc_hz": args.tsc_hz,
        "rdtsc_overhead_ns": round(per_read * 1e9, 2),
        "rows": rows,
    }
    with open(args.out, "w") as f:
        json.dump(out, f, indent=1)
        f.write("\n")
    for name, r in rows.items():
        print(f"{name:14s} {r['corrected_s']:7.3f} s  {r['pp_of_wall']:6.2f} pp  "
              f"{r.get('ns_per_expansion', 0):6.2f} ns/exp")


if __name__ == "__main__":
    main()
