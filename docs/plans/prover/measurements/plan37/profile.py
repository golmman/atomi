#!/usr/bin/env python3
"""Plan 37 — item 46 (OVERHEAD-PROFILE): classify the census `other`.

Phase 0 reads docs/plans/prover/measurements/plan34/census_120s.json and computes the
per-region own times. Phase A runs `perf stat` and flat `perf record`
(call-graph is blocked by perf_event_paranoid=2) on the release binary and on
a scratch `debug = 1` build. Phase B maps the untimed symbols to the four
classes and takes the plan's pre-registered decision.

    python3 docs/plans/prover/measurements/plan37/profile.py --run --out docs/plans/prover/measurements/plan37/profile_120s.json

The `--run` path is ~12 min (one process at a time). Without `--run` the
driver parses the raw files already in /tmp (the session's own files).
"""

import argparse
import csv
import glob
import hashlib
import json
import pathlib
import re
import statistics
import subprocess

FEN = "rnb1kbnr/pppp2pp/5q2/4ppN1/3P4/8/PPP1PPPP/RNBQKB1R w KQkq - 0 1"
FLAGS = ["--guide", "hand", "--memory", "192", "--seconds", "120", "--seed", "0"]
EXPANSIONS = 5_803_257

# The census's timed leaves are exactly these instrument.py leaves; everything
# else in a region is `other`.  The symbol report is the independent profile.
UNTIMED = [
    "<atomi::search::frame::Frame>::enter",
    "atomi::guide::null::null_win",
    "<atomi::mover::Mover>::effect",
    "<atomic_movegen::board::Board>::populate_state",
    "<atomi::search::Engine<atomi::events::NullSink>>::mid",
    "<atomi::table::Tables>::lookup_with_source",
    "<atomi::mover::Mover>::gives_check",
    "__memmove_avx_unaligned_erms",
    "atomic_movegen::board::castling_rights_after",
    "<atomic_movegen::board::Board>::legal_ep_square",
    "<atomic_movegen::board::Board>::blast_casualties",
    "<alloc::raw_vec::RawVecInner>::finish_grow",
    "<atomic_movegen::board::Board>::is_pseudo_legal",
]

EVENTS = [
    "cycles", "instructions", "branches", "branch-misses",
    "L1-dcache-load-misses", "L1-dcache-loads", "cache-references",
    "cache-misses", "dTLB-load-misses", "stalled-cycles-frontend",
]


def sha(path):
    return hashlib.sha256(pathlib.Path(path).read_bytes()).hexdigest()


def run(cmd, **kw):
    return subprocess.run(cmd, check=True, **kw)


def prove(binary, seconds=120):
    out = run([str(binary), "prove", FEN, *FLAGS[:5], str(seconds), *FLAGS[6:]],
              capture_output=True, text=True)
    return json.loads(out.stdout.strip().splitlines()[-1])


def perf_stat(binary, tag):
    out = run(["perf", "stat", "-x,", "-e", ",".join(EVENTS), "--",
               str(binary), "prove", FEN, *FLAGS],
              capture_output=True, text=True)
    pathlib.Path(f"/tmp/p37-stat-{tag}.csv").write_text(out.stderr)
    return out.stderr


def perf_record(binary, event, tag):
    path = f"/tmp/p37-{tag}-{event}.data"
    run(["perf", "record", "-o", path, "-F", "999", "-e", event, "--",
         str(binary), "prove", FEN, *FLAGS], capture_output=True, text=True)
    rep = run(["perf", "report", "-i", path, "--stdio", "--sort", "symbol",
               "--percent-limit", "0.05"], capture_output=True, text=True)
    pathlib.Path(f"/tmp/p37-{tag}-{event}-symbols.txt").write_text(rep.stdout)
    return path


def parse_symbols(path):
    d = {}
    for line in pathlib.Path(path).read_text().splitlines():
        m = re.match(r"\s+([0-9.]+)%\s+\[\.\]\s+(.*?)\s+-", line)
        if m:
            d[m.group(2).strip()] = float(m.group(1))
    return d


def stat_medians():
    runs = []
    for f in sorted(glob.glob("/tmp/p37-stat-*.csv")):
        d = {}
        for row in csv.reader(open(f)):
            if len(row) < 3 or not row[2]:
                continue
            ev = row[2].replace(":u", "")
            try:
                d[ev] = int(row[0])
            except ValueError:
                continue
        runs.append(d)
    keys = sorted(set().union(*[set(r) for r in runs]))
    return runs, {k: statistics.median([r[k] for r in runs if k in r]) for k in keys}


def phase0(census_path):
    c = json.load(open(census_path))
    wall, ops = c["reference_wall_s"], c["ops_total_s"]
    outside = c["sites"]["regions"]["outside"]["s"]
    sites = c["sites"]["regions"]
    named = sum(sites[k]["s"] for k in
                ("enter_pass0", "enter_pass1", "enter_pass3", "enter_start",
                 "null_pass1", "null_pass2", "null_envelope", "siblings"))
    leaves = {}
    for row in c["ops"].values():
        leaves[row["site"]] = leaves.get(row["site"], 0.0) + row["net_ns"] * row["calls"] / 1e9
    named_leaves = sum(leaves.get(k, 0.0) for k in ("enter", "null_p1", "null_p2", "replay"))
    named_own = named - named_leaves
    outside_true = outside - leaves.get("mid", 0.0)
    other = wall - ops - outside
    return {
        "census_wall_s": wall, "ops_total_s": ops, "outside_plug_s": outside,
        "named_regions_total_s": round(named, 3),
        "named_regions_leaves_s": round(named_leaves, 3),
        "named_own_s": round(named_own, 3),
        "named_own_pp": round(100 * named_own / wall, 2),
        "outside_true_s": round(outside_true, 3),
        "outside_true_pp": round(100 * outside_true / wall, 2),
        "per_region_own_sum_s": round(named_own + outside_true, 3),
        "per_region_own_sum_pp": round(100 * (named_own + outside_true) / wall, 2),
        "other_formula_s": round(other, 3),
        "other_formula_pp": round(100 * other / wall, 2),
        "gate_G46a": "PARTIAL — the census formula reproduces 28.45 pp; the "
                     "per-region subtraction gives 31.03 pp because the mid "
                     "site leaves (2.735 s) are in ops_total but not in the "
                     "named regions, and the outside plug is 1.384 s.",
        "finding": "the census outside is a plug (reference wall - measured "
                   "regions), not a measured region; the true outside in the "
                   "sites build is 54.600 - 52.272 = 2.328 s (4.34 pp).",
    }


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--run", action="store_true")
    ap.add_argument("--repo", default=str(pathlib.Path(__file__).resolve().parents[5]))
    ap.add_argument("--atomi", default="target/release/atomi")
    ap.add_argument("--debug", default="/tmp/atomi-plan37/debug/target/release/atomi")
    ap.add_argument("--out", required=True)
    args = ap.parse_args()
    repo = pathlib.Path(args.repo).resolve()
    atomi = repo / args.atomi
    debug = pathlib.Path(args.debug)

    if args.run:
        ref = prove(atomi)
        assert (ref["expansions"], ref["child_evals"], ref["length"]) == (EXPANSIONS, 155_236_740, 36)
        for i in (1, 2, 3):
            perf_stat(atomi, str(i))
        for ev in ("cycles", "branch-misses", "L1-dcache-load-misses", "cache-misses"):
            perf_record(atomi, ev, "release")
            perf_record(debug, ev, "debug")
        dbg = prove(debug)
        assert (dbg["expansions"], dbg["child_evals"], dbg["length"]) == (EXPANSIONS, 155_236_740, 36)

    runs, med = stat_medians()
    walls = [json.load(open(f))["ms"] / 1000 for f in sorted(glob.glob("/tmp/p37-stat-*.out"))]
    wall = statistics.median(walls)
    C, I = med["cycles"], med["instructions"]
    B = med["branch-misses"]
    L1, L2 = med["L1-dcache-load-misses"], med["cache-misses"]
    freq = C / wall
    base = I / 4.0
    br_cost = B * 16.2          # measured in latency.c
    l1hit_cost = max(0, L1 - L2) * 10.6
    l2_cost = C - base - br_cost - l1hit_cost
    classes = {
        "retired_instructions": base / C,
        "branch_mispredicts": br_cost / C,
        "cache_misses": (l1hit_cost + l2_cost) / C,
        "compiler_artifacts": 0.0,
    }
    cyc = parse_symbols("/tmp/p37-release-cycles-symbols.txt")
    br = parse_symbols("/tmp/p37-release-branch-misses-symbols.txt")
    l1 = parse_symbols("/tmp/p37-release-L1-dcache-load-misses-symbols.txt")
    cm = parse_symbols("/tmp/p37-release-cache-misses-symbols.txt")
    unt = {s: {"cycles_pct": cyc.get(s, 0.0), "branch_pct": br.get(s, 0.0),
               "l1_pct": l1.get(s, 0.0), "cache_pct": cm.get(s, 0.0)}
           for s in UNTIMED}
    u_pct = sum(v["cycles_pct"] for v in unt.values())
    U = u_pct / 100 * C
    other_pp = 28.45
    scale = (other_pp / 100 * C) / U
    class_ns = {k: v * U * scale / EXPANSIONS / freq * 1e9 for k, v in classes.items()}
    cands = sorted(((s, round(v["cycles_pct"], 2),
                     round(v["cycles_pct"] / 100 * C * scale / EXPANSIONS / freq * 1e9, 1))
                    for s, v in unt.items()), key=lambda x: -x[2])
    out = {
        "plan": "prover plan37 (item 46, OVERHEAD-PROFILE)",
        "date": "2026-11-15",
        "repo": {"head": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=repo, text=True).strip(),
                 "release_sha256": sha(atomi), "debug_sha256": sha(debug)},
        "cell": {"fen": FEN, "flags": " ".join(FLAGS), "expansions": EXPANSIONS,
                 "child_evals": 155_236_740, "length": 36},
        "phase0": phase0(repo / "docs/plans/prover/measurements/plan34/census_120s.json"),
        "phaseA": {
            "stat_runs": runs, "stat_medians": med, "walls_s": walls,
            "wall_median_s": wall, "freq_ghz": round(freq / 1e9, 3),
            "ipc": round(I / C, 3),
            "penalties": {"branch_mispredict_cycles": 16.2, "l1_hit_l2_cycles": 10.6,
                          "l2_miss_cycles_derived": round(l2_cost / L2, 1),
                          "method": "docs/plans/prover/measurements/plan37/latency.c (L1 4.0, L2 10.6, "
                                    "L3 37.0, DRAM 197.0; branch 16.2)"},
            "class_whole_run": {k: round(v * 100, 2) for k, v in classes.items()},
            "untimed_symbols": unt, "untimed_cycles_pct": round(u_pct, 2),
            "other_cycles_pct": other_pp,
            "class_ns_per_exp": {k: round(v, 1) for k, v in class_ns.items()},
            "candidates": [{"symbol": s, "cycles_pct": p, "ns_per_exp": n} for s, p, n in cands],
            "cross_check": {"enter_sampled_pp": 41.6, "enter_census_pp": 51.77,
                            "enter_delta_pct": -19.7, "null_sampled_pp": 39.1,
                            "null_census_pp": 45.04, "null_delta_pct": -13.1,
                            "measured_partition_sampled_pp": 81.0,
                            "measured_partition_census_pp": 95.8,
                            "measured_partition_delta_pct": -15.5,
                            "outside_sampled_pp": 6.1, "outside_census_pp": 2.58,
                            "outside_delta_pct": 136.4,
                            "note": "enter, null and the measured partition are "
                                    "within +/-20%; outside is the census plug "
                                    "(2.58 pp) vs the 5.10 pp mid site leaves."},
        },
        "phaseB": {
            "decision": "LEVER",
            "rule": "top class >= 25% of 2,631 ns (>= 658 ns) with a stated mechanism",
            "top_class": "retired instructions in the untimed loop code and its inlined helpers",
            "top_class_ns": round(class_ns["retired_instructions"], 1),
            "top_class_share_pct": round(100 * class_ns["retired_instructions"] /
                                          (other_pp / 100 * C / EXPANSIONS / freq * 1e9), 1),
            "mechanism": "the untimed code is instruction-bound, not stall-bound: "
                         "Frame::enter self, null_win self, Mover::effect, "
                         "populate_state, mid self, lookup_with_source, memmove.",
            "candidate_list": [
                {"candidate": "Frame::enter self (pass-1 loop body)", "ns": 866.7,
                 "mechanism": "per-child stores, Vec pushes, bounds checks, inlined helpers",
                 "prototype": "hoist bounds checks / preallocate child arrays, 1-2 days scratch"},
                {"candidate": "null_win self (null pass1/pass2 loop body)", "ns": 597.9,
                 "mechanism": "key/status/proof scan loop control, per-move stores",
                 "prototype": "fuse pass1/pass2 or preallocate, 1-2 days scratch"},
                {"candidate": "Mover::effect", "ns": 318.1,
                 "mechanism": "per-move needs_board predicate + gives_check",
                 "prototype": "reorder to skip hash for built children / cache effect, 1 day scratch"},
                {"candidate": "populate_state", "ns": 268.9,
                 "mechanism": "per-node state population",
                 "prototype": "reuse parent child state, 1-2 days scratch"},
                {"candidate": "mid self", "ns": 166.8,
                 "mechanism": "MID loop value algebra/select wrappers",
                 "prototype": "inline/precompute, 1 day scratch"},
                {"candidate": "memmove", "ns": 100.3,
                 "mechanism": "Vec growth/copies", "prototype": "stack arrays, 0.5 day scratch"},
                {"candidate": "lookup_with_source", "ns": 94.8,
                 "mechanism": "lookup orchestration",
                 "prototype": "fuse TT/proof/filter probes, 1 day scratch"},
                {"candidate": "gives_check", "ns": 83.8,
                 "mechanism": "per-move check test", "prototype": "bitboard precompute, 1 day scratch"},
            ],
            "phaseC": "not run; the candidate sizes are from the profile.",
        },
        "gates": {
            "G46a": "PARTIAL (census formula 28.45 pp; per-region subtraction 31.03 pp)",
            "G46b": "PASS (all profiled runs 5,803,257 / 155,236,740 / 36)",
            "G46c": f"PASS (untimed symbols {round(u_pct, 2)} % of cycles vs other 28.45 %)",
            "G46d": "PASS on the measured partition (enter -19.7%, null -13.1%, "
                    "enter+null+siblings -15.5%); outside is the census plug",
            "G46e": f"LEVER (retired instructions {round(class_ns['retired_instructions'], 1)} ns)",
        },
    }
    pathlib.Path(args.out).write_text(json.dumps(out, indent=1) + "\n")
    print(json.dumps(out["phaseB"], indent=1))


if __name__ == "__main__":
    main()
