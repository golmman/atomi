#!/usr/bin/env python3
"""Plan-42 driver — item 55 SCALE-LENGTH: compose the w = 2 EF-PN weight with a
bounded-length pass on the `--guide hand` product stack.

One process at a time, same binary, 120 s wall cap (`--memory 192 --seed N`).
`--guide-ef-weight` is written *after* `--guide hand` (D33); `--max-length` is
the opt-in bound from plan38.

Modes:
  phase0   deep-win seed 0 at w = 2 over L in {inf, 24, 22, 20}, 120 s.
  phaseA   3 cells x seeds 0-2 at the selected L (or inf) and w = 2, with
           same-session w = 16 controls.
  heldout  five fixed `guidance-v1` cells (0-based indices 6-10, the wins.tsv
           dec* cells), seed 0, w = 2 vs 16, 120 s.

Rows are appended to `runs.json` after every run, so an interrupted session
resumes instead of repeating work.
"""

import argparse
import json
import os
import subprocess

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "../../../../.."))

CELLS = {
    "core": "r1b1kbnr/ppp3pp/n4q2/3pppN1/1P1PP3/8/P1P2PPP/RNBQKB1R w KQkq - 0 1",
    "easy-win": "rnbq1b1r/pp1p1kpp/2p1pp1n/8/3PP1P1/2N2N2/PPP2P1P/R1BQKB1R w KQ - 0 1",
    "deep-win": "rnb1kbnr/pppp2pp/5q2/4ppN1/3P4/8/PPP1PPPP/RNBQKB1R w KQkq - 0 1",
}

# 0-based indices into guidance-v1.tsv (header excluded). 6-10 are the first
# five wins.tsv `dec*` cells: provari-hand shared wins, not sample20, and small
# enough that atomi proves at 120 s on both weights.
HELDOUT_INDICES = (6, 7, 8, 9, 10)


def load_guidance():
    path = os.path.join(ROOT, "docs/plans/prover/measurements/plan3/guidance-v1.tsv")
    rows = []
    with open(path) as f:
        header = f.readline()
        for line in f:
            parts = line.rstrip("\n").split("\t")
            if len(parts) >= 2:
                rows.append({"name": parts[0], "fen": parts[1]})
    return rows


def run(binary, fen, seed, weight, max_length, seconds=120):
    cmd = [
        binary, "prove", fen, "--guide", "hand", "--memory", "192",
        "--seed", str(seed), "--seconds", str(seconds),
        "--guide-ef-weight", str(weight),
    ]
    if max_length is not None:
        cmd += ["--max-length", str(max_length)]
    proc = subprocess.run(cmd, capture_output=True, text=True, timeout=seconds + 90)
    if proc.returncode != 0:
        return {"error": f"exit {proc.returncode}", "stdout": proc.stdout[-2000:],
                "stderr": proc.stderr[-2000:]}
    try:
        data = json.loads(proc.stdout.strip().splitlines()[-1])
    except Exception:
        return {"error": "bad json", "stdout": proc.stdout[-2000:],
                "stderr": proc.stderr[-2000:]}
    return data


def row(label, binary, cell, fen, seed, weight, max_length, seconds):
    d = run(binary, fen, seed, weight, max_length, seconds)
    out = {
        "label": label, "cell": cell, "fen": fen, "seed": seed,
        "weight": weight, "max_length": max_length, "seconds": seconds,
    }
    for key in ("found", "exit", "ms", "child_evals", "expansions", "length",
                "proof_entries", "proof"):
        out[key] = d.get(key)
    if "error" in d:
        out["error"] = d["error"]
        out["stdout"] = d.get("stdout")
        out["stderr"] = d.get("stderr")
    print(f"{label:32s} w={weight:2d} L={str(max_length):>4s} s={seed} "
          f"ms={out['ms']} exp={out['expansions']} found={out['found']} "
          f"len={out['length']} exit={out['exit']}", flush=True)
    return out


def labels_done(path):
    if not os.path.exists(path):
        return set()
    with open(path) as f:
        return {r["label"] for r in json.load(f)["runs"]}


def append(path, r):
    if os.path.exists(path):
        with open(path) as f:
            data = json.load(f)
    else:
        data = {"runs": []}
    data["runs"].append(r)
    with open(path, "w") as f:
        json.dump(data, f, indent=1)
        f.write("\n")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--mode", choices=("phase0", "phaseA", "heldout"), required=True)
    ap.add_argument("--atomi", default=os.path.join(ROOT, "target/release/atomi"))
    ap.add_argument("--seconds", type=int, default=120)
    ap.add_argument("--best-l", type=int, default=None,
                    help="selected L for phaseA; omit for L = inf")
    ap.add_argument("--out", default=os.path.join(HERE, "runs.json"))
    args = ap.parse_args()

    done = labels_done(args.out)
    jobs = []
    if args.mode == "phase0":
        for L in (None, 24, 22, 20):
            tag = "inf" if L is None else str(L)
            jobs.append((f"phase0_deep_w2_L{tag}_s0", "deep-win",
                         CELLS["deep-win"], 0, 2, L))
    elif args.mode == "phaseA":
        L = args.best_l
        tag = "inf" if L is None else str(L)
        for cell in ("core", "easy-win", "deep-win"):
            for seed in (0, 1, 2):
                for w in (16, 2):
                    jobs.append((f"phaseA_{cell}_w{w}_L{tag}_s{seed}", cell,
                                 CELLS[cell], seed, w, L))
    else:
        guidance = load_guidance()
        for index in HELDOUT_INDICES:
            item = guidance[index]
            for w in (16, 2):
                jobs.append((f"heldout_{index}_{item['name']}_w{w}",
                             item["name"], item["fen"], 0, w, None))

    for label, cell, fen, seed, weight, L in jobs:
        if label in done:
            print(f"{label}: already recorded", flush=True)
            continue
        append(args.out, row(label, args.atomi, cell, fen, seed, weight, L,
                             args.seconds))


if __name__ == "__main__":
    main()
