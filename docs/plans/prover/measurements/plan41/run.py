#!/usr/bin/env python3
"""Plan-41 driver — EF-PN weight scan on the `--guide hand` product stack.

One process at a time, same binary, 120 s wall cap (`--memory 192 --seed N`).
`--guide-ef-weight` is written *after* `--guide hand` (D33).

Modes:
  phase0+phaseA  deep-win weight scan, the 3-cell x seeds 0-2 matrix for
                 w = 2 and w = 4 with same-session w = 16 controls.
  heldout        indices 1,5,9,13,17 of `corpus.json`'s `sample20` (0-based),
                 seed 0, w = 2 vs 16.

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
HELDOUT_INDICES = (1, 5, 9, 13, 17)  # 0-based indices into corpus.json sample20


def load_corpus():
    with open(os.path.join(ROOT, "bench/corpus.json")) as f:
        return json.load(f)


def run(binary, fen, seed, weight, seconds=120):
    cmd = [
        binary, "prove", fen, "--guide", "hand", "--memory", "192",
        "--seed", str(seed), "--seconds", str(seconds),
        "--guide-ef-weight", str(weight),
    ]
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


def row(label, binary, cell, fen, seed, weight, seconds):
    d = run(binary, fen, seed, weight, seconds)
    out = {
        "label": label, "cell": cell, "fen": fen, "seed": seed,
        "weight": weight, "seconds": seconds,
    }
    for key in ("found", "exit", "ms", "child_evals", "expansions", "length",
                "proof_entries", "proof"):
        out[key] = d.get(key)
    if "error" in d:
        out["error"] = d["error"]
        out["stdout"] = d.get("stdout")
        out["stderr"] = d.get("stderr")
    print(f"{label:28s} w={weight:2d} s={seed} ms={out['ms']} "
          f"exp={out['expansions']} found={out['found']} len={out['length']} "
          f"exit={out['exit']}", flush=True)
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
    ap.add_argument("--mode", choices=("phase0+phaseA", "heldout"), required=True)
    ap.add_argument("--atomi", default=os.path.join(ROOT, "target/release/atomi"))
    ap.add_argument("--seconds", type=int, default=120)
    ap.add_argument("--out", default=os.path.join(HERE, "runs.json"))
    args = ap.parse_args()

    done = labels_done(args.out)
    jobs = []
    if args.mode == "phase0+phaseA":
        for w in (16, 8, 6, 4, 3, 2, 1, 0):
            jobs.append((f"phase0_deep_w{w}_s0", "deep-win", CELLS["deep-win"], 0, w))
        for seed in (1, 2):
            for w in (16, 2):
                jobs.append((f"phase0_deep_w{w}_s{seed}", "deep-win",
                             CELLS["deep-win"], seed, w))
        for cell in ("core", "easy-win"):
            for seed in (0, 1, 2):
                for w in (16, 2):
                    jobs.append((f"phase0_{cell}_w{w}_s{seed}", cell,
                                 CELLS[cell], seed, w))
        # Phase A adds w = 4; deep-win seed 0 is the Phase-0 scan row.
        for seed in (1, 2):
            jobs.append((f"phaseA_deep-win_w4_s{seed}", "deep-win",
                         CELLS["deep-win"], seed, 4))
        for cell in ("core", "easy-win"):
            for seed in (0, 1, 2):
                jobs.append((f"phaseA_{cell}_w4_s{seed}", cell,
                             CELLS[cell], seed, 4))
    else:
        corpus = load_corpus()
        for index in HELDOUT_INDICES:
            item = corpus["sample20"][index]
            for w in (16, 2):
                jobs.append((f"heldout_{index}_{item['id']}_w{w}", item["id"],
                             item["fen"], 0, w))

    for label, cell, fen, seed, weight in jobs:
        if label in done:
            print(f"{label}: already recorded", flush=True)
            continue
        append(args.out, row(label, args.atomi, cell, fen, seed, weight, args.seconds))


if __name__ == "__main__":
    main()
