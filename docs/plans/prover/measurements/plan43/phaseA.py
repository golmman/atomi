#!/usr/bin/env python3
"""Plan-43 Phase A — scratch quantile map vs the w = 16 control.

One process at a time, same session, 120 s wall cap. The map is the frozen
shared-parent quantile band (lo = 0, hi = 26) replacing the linear EF-PN
weight map; the control is the default `--guide hand` w = 16 stack. The
binary is the clean `/tmp/atomi-plan43a` build; the map is switched by
`ATOMI_EF_MAP=1 ATOMI_EF_MAP_LO=0 ATOMI_EF_MAP_HI=26`.
"""

import json
import os
import subprocess

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "../../../../.."))
BINARY = "/tmp/atomi-plan43a/target/release/atomi"

CELLS = {
    "core": "r1b1kbnr/ppp3pp/n4q2/3pppN1/1P1PP3/8/P1P2PPP/RNBQKB1R w KQkq - 0 1",
    "easy-win": "rnbq1b1r/pp1p1kpp/2p1pp1n/8/3PP1P1/2N2N2/PPP2P1P/R1BQKB1R w KQ - 0 1",
    "deep-win": "rnb1kbnr/pppp2pp/5q2/4ppN1/3P4/8/PPP1PPPP/RNBQKB1R w KQkq - 0 1",
}
LO, HI = 0, 26


def run(label, fen, seed, seconds, mapped):
    cmd = [BINARY, "prove", fen, "--guide", "hand", "--memory", "192",
           "--seed", str(seed), "--seconds", str(seconds)]
    env = dict(os.environ)
    if mapped:
        env.update(ATOMI_EF_MAP="1", ATOMI_EF_MAP_LO=str(LO),
                   ATOMI_EF_MAP_HI=str(HI))
    proc = subprocess.run(cmd, capture_output=True, text=True,
                          timeout=seconds + 180, env=env)
    out = {"label": label, "cell": label.split("-")[0], "seed": seed,
           "seconds": seconds, "mapped": mapped}
    try:
        data = json.loads(proc.stdout.strip().splitlines()[-1])
        for key in ("found", "exit", "ms", "child_evals", "expansions",
                    "length", "proof_entries"):
            out[key] = data.get(key)
    except Exception:
        out["error"] = f"exit {proc.returncode}"
        out["stdout"] = proc.stdout[-2000:]
        out["stderr"] = proc.stderr[-2000:]
    print(f"{label:24s} ms={out.get('ms')} evals={out.get('child_evals')} "
          f"exp={out.get('expansions')} found={out.get('found')} "
          f"len={out.get('length')}", flush=True)
    return out


def main():
    path = os.path.join(HERE, "phaseA.json")
    if os.path.exists(path):
        with open(path) as f:
            data = json.load(f)
    else:
        data = {"edges": [LO, HI], "runs": []}
    done = {r["label"] for r in data["runs"]}
    for cell, fen in CELLS.items():
        for mapped in (False, True):
            label = f"{cell}-{'map' if mapped else 'control'}"
            if label in done:
                print(f"skip {label}", flush=True)
                continue
            data["runs"].append(run(label, fen, 0, 120, mapped))
            with open(path, "w") as f:
                json.dump(data, f, indent=1)
                f.write("\n")
            done.add(label)

    rows = {r["label"]: r for r in data["runs"]}
    ratios = {}
    for cell in CELLS:
        c = rows.get(f"{cell}-control", {})
        m = rows.get(f"{cell}-map", {})
        if c.get("ms") and m.get("ms"):
            ratios[cell] = {
                "control_ms": c["ms"], "map_ms": m["ms"],
                "ratio": m["ms"] / c["ms"],
                "control_expansions": c.get("expansions"),
                "map_expansions": m.get("expansions"),
                "control_length": c.get("length"),
                "map_length": m.get("length"),
            }
    data["ratios"] = ratios
    with open(path, "w") as f:
        json.dump(data, f, indent=1)
        f.write("\n")
    print(json.dumps(ratios, indent=1))


if __name__ == "__main__":
    main()
