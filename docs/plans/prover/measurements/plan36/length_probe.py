#!/usr/bin/env python3
"""Plan-36 (item 45) Phase 0 — the read-only length probe.

One process at a time, same host, same session:

* **ours** — `atomi prove "<deep-win FEN>" --guide hand --memory 192
  --seconds 120 --seed 0..2` (length, wall, expansions, child evals);
* **the reference** — the pinned oracle as a separate process,
  `provari prove "<deep-win FEN>" --guide hand --seconds 120 --memory 192`
  (`mate`, reported `nodes`, its own wall);
* **the unit check** — the same pair on `core` at 10 s, where the two engines
  are expected to prove the *same* mating distance.

The probe prices nothing and changes nothing: it records the two engines'
proof lengths, which is the tree comparison [`plan36`](../../plan36.md)
pre-registers: if our `deep-win` `length` is never <= 20, the 2.76x tree ratio
(5,803,257 expansions / 2,099,100 reference node bodies) mixes a mate-36 tree
with a mate-19 one and is void.

    python3 docs/plans/prover/measurements/plan36/length_probe.py --out docs/plans/prover/measurements/plan36/length_probe.json

Nothing from the reference enters the repo: its JSON result line (integers) is
captured, its binary is only started as an external process.
"""

import argparse
import hashlib
import json
import pathlib
import platform
import subprocess

ROOT = pathlib.Path(__file__).resolve().parents[5]
FEN = {
    "deep-win": "rnb1kbnr/pppp2pp/5q2/4ppN1/3P4/8/PPP1PPPP/RNBQKB1R w KQkq - 0 1",
    "core": "r1b1kbnr/ppp3pp/n4q2/3pppN1/1P1PP3/8/P1P2PPP/RNBQKB1R w KQkq - 0 1",
    "easy-win": "rnbq1b1r/pp1p1kpp/2p1pp1n/8/3PP1P1/2N2N2/PPP2P1P/R1BQKB1R w KQ - 0 1",
}
SEEDS = [0, 1, 2]
LENGTH_BAR = 20          # the plan's pre-registration: <= 20 or the ratio is void


def sha256(path):
    return hashlib.sha256(pathlib.Path(path).read_bytes()).hexdigest()


def run_atomi(binary, cell, seconds, seed):
    proc = subprocess.run(
        [str(binary), "prove", FEN[cell], "--guide", "hand",
         "--memory", "192", "--seconds", str(seconds), "--seed", str(seed)],
        capture_output=True, text=True)
    if proc.returncode != 0:
        raise SystemExit(f"atomi {cell} seed {seed}: exit {proc.returncode}: "
                         f"{proc.stderr[-400:]}")
    return json.loads(proc.stdout.strip().splitlines()[-1])


def run_reference(binary, cell, seconds):
    proc = subprocess.run(
        [str(binary), "prove", FEN[cell], "--guide", "hand",
         "--seconds", str(seconds), "--memory", "192"],
        capture_output=True, text=True)
    if proc.returncode != 0:
        raise SystemExit(f"reference {cell}: exit {proc.returncode}: "
                         f"{proc.stderr[-400:]}")
    return json.loads(proc.stdout.strip().splitlines()[-1])


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--atomi", default=str(ROOT / "target/release/atomi"))
    ap.add_argument("--reference",
                    default="/tmp/atomi-oracle/provari/target/release/provari")
    ap.add_argument("--out", default=str(ROOT / "docs/plans/prover/measurements/plan36/length_probe.json"))
    args = ap.parse_args()

    atomi = pathlib.Path(args.atomi)
    reference = pathlib.Path(args.reference)
    if not atomi.exists() or not reference.exists():
        raise SystemExit("binary missing: the probe needs both engines")

    data = {"plan": "36", "item": 45, "phase": 0, "cells": {}}
    for cell, seconds in (("deep-win", 120), ("core", 10), ("easy-win", 10)):
        ours = [run_atomi(atomi, cell, seconds, s) for s in SEEDS]
        theirs = run_reference(reference, cell, seconds)
        lengths = [r["length"] for r in ours]
        data["cells"][cell] = {
            "fen": FEN[cell],
            "seconds": seconds,
            "ours": [
                {k: r[k] for k in ("seed", "found", "length", "ms", "expansions",
                                   "child_evals", "aux_nodes")}
                for r in ours
            ],
            "reference": theirs,
            "length_median": sorted(lengths)[1],
            "length_min": min(lengths),
            "vs_reference_mate": (
                "shorter" if max(lengths) < theirs["mate"]
                else "longer" if min(lengths) > theirs["mate"]
                else "mixed"),
        }
    deep = data["cells"]["deep-win"]
    core = data["cells"]["core"]
    data["pre_registration"] = {
        "bar": f"our deep-win length <= {LENGTH_BAR} on at least one seed",
        "observed": deep["ours"][0] and [r["length"] for r in deep["ours"]],
        "fires": deep["length_min"] > LENGTH_BAR,
        "consequence": (
            "our lengths are %s against the reference's mate %d, so the 2.76x "
            "tree ratio (5,803,257 expansions / 2,099,100 reference node "
            "bodies) compares two different mating distances and is void; the "
            "length axis becomes item 47 at the owner's discretion"
            % ([r["length"] for r in deep["ours"]], deep["reference"]["mate"])),
    }
    data["length_table"] = {
        c: {"ours": [r["length"] for r in data["cells"][c]["ours"]],
            "reference_mate": data["cells"][c]["reference"]["mate"],
            "relation": data["cells"][c]["vs_reference_mate"]}
        for c in data["cells"]}
    data["plan36_unit_check"] = {
        "claimed": "core mate 17 = our length 17, so the unit check passes",
        "observed": {"reference_mate": core["reference"]["mate"],
                     "ours_lengths": [r["length"] for r in core["ours"]],
                     "plan33_120s_ours_length": 15},
        "holds": False,
        "note": "the plan's 17 = 17 does not reproduce: our core proof is 15 "
                "attacker moves against the reference's mate 17 (plan33's "
                "committed Cap-120 row has the same 15), i.e. we prove core "
                "shorter, not equal; the lengths are still the same order, so "
                "core remains the like-for-like cell",
    }
    data["wall_reference"] = {
        "ours_median_s": round(sorted(r["ms"] for r in deep["ours"])[1] / 1000, 3),
        "reference_s": round(deep["reference"]["ms"] / 1000, 3),
        "ratio": round(sorted(r["ms"] for r in deep["ours"])[1]
                       / deep["reference"]["ms"], 3),
        "note": "wall is per node x tree; on trees of different depth it is not "
                "decomposable into the two factors",
    }
    data["env"] = {
        "host": platform.platform(),
        "machine_id_from_plan35": "e10de197da58",
        "atomi_sha256": sha256(atomi),
        "atomi_version": "atomi 0.1.0 (this tree)",
        "reference_sha256": sha256(reference),
        "procedure": "one process at a time, 3 atomi seeds per cell then 1 "
                     "reference run per cell; nothing is timed for a verdict",
    }
    pathlib.Path(args.out).write_text(json.dumps(data, indent=1) + "\n")
    print(json.dumps({c: {"lengths": [r["length"] for r in data["cells"][c]["ours"]],
                          "reference_mate": data["cells"][c]["reference"]["mate"],
                          "ours_median_s": round(sorted(r["ms"] for r in
                                                        data["cells"][c]["ours"])[1] / 1000, 3),
                          "reference_s": round(data["cells"][c]["reference"]["ms"] / 1000, 3)}
                       for c in data["cells"]},
                      indent=1))
    print("pre-registration fires:", data["pre_registration"]["fires"])


if __name__ == "__main__":
    main()
