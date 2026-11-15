#!/usr/bin/env python3
"""Plan-43 Phase 0 driver — distribution-normalised EF-PN map.

Read-only probe: the /tmp histogram build is never part of the repository.
One process at a time. Two phases:

  fit   eight fixed `corpus-v1` (bench/corpus.json sample20) cells, 0-based
        indices 0,1,2,4,5,6,7,8 (3 = easy-win and 10 = deep-win excluded),
        seed 0, `--evals 2000000`.
  diag  the three claim cells plus the fixed held-out guidance-v1 cells
        0-based 6-10, seed 0, 120 s.

Every run writes its histogram to `hist/<label>.json`; parsed counters go to
`runs.json`. Histograms are the oriented attacker eval at each fresh EF-PN
read point, split by node kind and by parent/child board.
"""

import json
import os
import statistics
import subprocess
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "../../../../.."))
BINARY = "/tmp/atomi-plan43/target/release/atomi"
CORPUS = os.path.join(ROOT, "bench/corpus.json")
GUIDANCE = os.path.join(ROOT, "docs/plans/prover/measurements/plan3/guidance-v1.tsv")

FIT_INDICES = (0, 1, 2, 4, 5, 6, 7, 8)
HELDOUT_INDICES = (6, 7, 8, 9, 10)
CLAIM_IDS = ("core", "1eb80b92f92c", "db6a741c723f")


def load_corpus():
    with open(CORPUS) as f:
        data = json.load(f)
    cells = {"core": data["core"]["fen"]}
    for i, c in enumerate(data["sample20"]):
        cells[c["id"]] = c["fen"]
    return cells


def load_guidance():
    rows = []
    with open(GUIDANCE) as f:
        f.readline()
        for line in f:
            parts = line.rstrip("\n").split("\t")
            if len(parts) >= 2:
                rows.append({"name": parts[0], "fen": parts[1]})
    return rows


def run(label, fen, seed, seconds, evals=None):
    hist = os.path.join(HERE, "hist", label + ".json")
    os.makedirs(os.path.dirname(hist), exist_ok=True)
    cmd = [
        BINARY, "prove", fen, "--guide", "hand", "--memory", "192",
        "--seed", str(seed), "--seconds", str(seconds),
    ]
    if evals is not None:
        cmd += ["--evals", str(evals)]
    env = dict(os.environ, ATOMI_EF_HIST_OUT=hist)
    proc = subprocess.run(cmd, capture_output=True, text=True,
                          timeout=seconds + 180, env=env)
    out = {"label": label, "fen": fen, "seed": seed, "seconds": seconds,
           "evals": evals}
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


def load_runs(path):
    if not os.path.exists(path):
        return []
    with open(path) as f:
        return json.load(f)["runs"]


def save_runs(path, runs):
    with open(path, "w") as f:
        json.dump({"runs": runs}, f, indent=1)
        f.write("\n")


def percentile_from_hist(hist, lo, p):
    total = sum(hist)
    if total == 0:
        return None
    target = total * p
    acc = 0
    for i, n in enumerate(hist):
        acc += n
        if acc >= target:
            return lo + i
    return lo + len(hist) - 1


def merge(hist_list):
    if not hist_list:
        return []
    return [sum(h[i] for h in hist_list) for i in range(len(hist_list[0]))]


def main():
    phase = sys.argv[1] if len(sys.argv) > 1 else "all"
    corpus = load_corpus()
    guidance = load_guidance()
    out_path = os.path.join(HERE, "runs.json")
    runs = load_runs(out_path)
    done = {r["label"] for r in runs}

    jobs = []
    if phase in ("fit", "all"):
        sample_ids = json.load(open(CORPUS))["sample20"]
        for i in FIT_INDICES:
            cid = sample_ids[i]["id"]
            jobs.append((f"fit-{i:02d}-{cid}", cid,
                         corpus[cid], 0, 300, 2_000_000))
    if phase in ("diag", "all"):
        jobs.append(("diag-core", "core", corpus["core"], 0, 120, None))
        jobs.append(("diag-easy-win", "easy-win",
                     corpus["1eb80b92f92c"], 0, 120, None))
        jobs.append(("diag-deep-win", "deep-win",
                     corpus["db6a741c723f"], 0, 120, None))
        for i in HELDOUT_INDICES:
            row = guidance[i]
            jobs.append((f"diag-heldout-{i:02d}-{row['name']}", row["name"],
                         row["fen"], 0, 120, None))

    for label, cell, fen, seed, seconds, evals in jobs:
        if label in done:
            print(f"skip {label}", flush=True)
            continue
        runs.append(run(label, fen, seed, seconds, evals))
        save_runs(out_path, runs)
        done.add(label)

    # Fit percentiles for the README freeze, parent and child, OR/AND/shared.
    fit_hist = []
    for r in runs:
        if not r["label"].startswith("fit-"):
            continue
        path = os.path.join(HERE, "hist", r["label"] + ".json")
        if os.path.exists(path):
            fit_hist.append(json.load(open(path)))
    summary = {"fit_cells": [r["label"] for r in runs
                             if r["label"].startswith("fit-")],
               "percentiles": {}}
    for key in ("or_parent", "and_parent", "or_child", "and_child"):
        h = merge([x[key] for x in fit_hist])
        summary["percentiles"][key] = {
            "n": sum(h),
            "p10": percentile_from_hist(h, -256, 0.10),
            "p20": percentile_from_hist(h, -256, 0.20),
            "p50": percentile_from_hist(h, -256, 0.50),
            "p80": percentile_from_hist(h, -256, 0.80),
            "p90": percentile_from_hist(h, -256, 0.90),
        }
    shared_parent = merge([x["or_parent"] for x in fit_hist]
                          + [x["and_parent"] for x in fit_hist])
    shared_child = merge([x["or_child"] for x in fit_hist]
                         + [x["and_child"] for x in fit_hist])
    summary["percentiles"]["shared_parent"] = {
        "n": sum(shared_parent),
        "p20": percentile_from_hist(shared_parent, -256, 0.20),
        "p80": percentile_from_hist(shared_parent, -256, 0.80),
    }
    summary["percentiles"]["shared_child"] = {
        "n": sum(shared_child),
        "p20": percentile_from_hist(shared_child, -256, 0.20),
        "p80": percentile_from_hist(shared_child, -256, 0.80),
    }
    with open(os.path.join(HERE, "summary.json"), "w") as f:
        json.dump(summary, f, indent=1)
        f.write("\n")
    print(json.dumps(summary["percentiles"], indent=1))


if __name__ == "__main__":
    main()
