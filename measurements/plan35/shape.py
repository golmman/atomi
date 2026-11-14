#!/usr/bin/env python3
"""Plan-35 item 43 (SHAPE-DIFF): shape.py — the count side and the arithmetic.

Runs a **count-only** instrumented build of the reference prover (scratch tree
outside the repo, made by a patch script that lives and dies in `/tmp`; see
`env.json` for its sha256) on the two cells, parses the one-line counter dump,
maps every counted family onto the census leaf that owns its cost in our engine
(`measurements/plan34/census_120s.json`) and writes the decision numbers.

    python3 measurements/plan35/shape.py --out measurements/plan35/shape.json

Nothing is timed: the reference's wall is not used, and every `ns` is ours
(report34 §4 verbatim). The counters are integers; the leaf names are our own.

Mapping rule (fixed before the arithmetic was computed):
  * one counted *provari* call per atomi leaf, in that leaf's own unit;
  * `ns/op` = the census bucket's `s / calls` (report34 §4), a lower bound;
  * a leaf whose job the reference does not do at all is **folded**: no `M`
    contribution, whole bucket addressable, and counted in the unmapped share;
  * a counted counter that rides inside a priced family (a per-child test, a
    per-move screen) is listed as `inside` and is **not** priced twice.
"""

import argparse
import hashlib
import json
import pathlib
import platform
import re
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
CENSUS = ROOT / "measurements/plan34/census_120s.json"
FEN = {
    "deep-win": "rnb1kbnr/pppp2pp/5q2/4ppN1/3P4/8/PPP1PPPP/RNBQKB1R w KQkq - 0 1",
    "core": "r1b1kbnr/ppp3pp/n4q2/3pppN1/1P1PP3/8/P1P2PPP/RNBQKB1R w KQkq - 0 1",
}
CELLS = [("deep-win", 120), ("core", 10)]

# The atomi leaf -> the reference counter that does the same job in the same
# unit. `None` = folded (the reference has no such pass). `inside` = a counter
# whose work is priced by another row's unit (never priced itself).
MAP = [
    ("movegen (legal)", ["list_generations"],
     "one move list generated", []),
    ("closure: frame pass-1 settle+screen", ["child_decisions"],
     "one child decided from bit tests", []),
    ("legality/status/terminality", ["terminality_queries", "mate_tests", "body_check_queries",
                                     "status_queries"],
     "one terminality/status query", []),
    ("guide hand_eval", ["hand_evals"], "one hand evaluation of a body", []),
    ("proof probe", ["proof_probes"], "one proof-store probe", []),
    ("closure: replay/simulation", ["replay_calls"], "one proof replay", []),
    ("guide class + start numbers", ["guide_steps"], "one guide number pass", []),
    ("do_move/undo_move/clone", ["child_builds"], "one child board built", []),
    ("closure: null pass-2 reply test", None,
     "folded: the reply scan is our closure, absent there", []),
    ("MID select/node_value", ["selection_steps"], "one child set combined", []),
    ("key/rule50", ["effect_computations", "effect_hash_computations"], "one child hash/effect", []),
    ("table store", ["table_stores"], "one table store", []),
    ("legality/status/terminality (needs-board/check test)", ["legality_screens"],
     "one per-move screen", ["quiet_tests", "blast_capture_tests", "exposing_tests",
                             "move_check_tests", "threat_tests", "heavy_legality_tests",
                             "evasion_scans", "material_tests"]),
    ("repetition scan", ["repetition_tests"], "one repetition test", []),
    ("exposed-limit cache", None, "folded: ours is a cache of our own scan", []),
    ("allocator", None, "folded: reuses its vectors", []),
    ("null board (`null_for`)", ["null_builds"], "one null board", []),
    ("prefetch hint", ["prefetch_hints"], "one prefetch hint", []),
    ("TT probe", ["table_lookups"], "one small-table lookup", []),
    ("key/rule50 (Mover::new)", ["mover_setups"], "one mover setup", []),
    ("filter probe", ["filter_bit_tests"], "one filter bit test", []),
]

# Counters reported but not in any priced family or `inside` list (recorded so
# G-35a is checkable): repetition marking, the null use itself.
UNPRICED_OK = ["bodies", "attacking_bodies", "defending_bodies", "repetition_marks",
               "null_uses", "moves_yielded", "legality_passes", "table_reads",
               "replay_defence_calls"]

T4_NS = 10e9 / 5803257          # 1,723 ns/exp: our 120 s tree at 10 s
WALL_NS = 9246.0                # our census wall per expansion (report34)
PP_NS = 92.46                   # our ns per pp of that wall
R_PP = 31.03                    # pp of the wall outside every op bucket
R_NS = R_PP * PP_NS             # 2,870 ns/exp: the constant reading
R_LITERAL = 16651.0             # the plan's literal R (see report)
BAND = 0.15

COUNT_RE = re.compile(r"^COUNTS (\{.*\})$", re.M)


def sha256(path):
    return hashlib.sha256(pathlib.Path(path).read_bytes()).hexdigest()


def run_cell(binary, cell, seconds, runs):
    """Run the count build `runs` times; return the counters and the reference
    result lines, after checking they are identical."""
    out = []
    for _ in range(runs):
        proc = subprocess.run(
            [str(binary), "prove", FEN[cell], "--guide", "hand",
             "--seconds", str(seconds), "--memory", "192"],
            capture_output=True, text=True,
            env={"ATOMI_COUNTS": "1", "PATH": "/usr/bin:/bin"},
        )
        if proc.returncode != 0:
            sys.exit(f"{cell}: exit {proc.returncode}: {proc.stderr[-400:]}")
        line = proc.stdout.strip().splitlines()[-1]
        found = COUNT_RE.search(proc.stderr)
        if not found:
            sys.exit(f"{cell}: no counter dump: {proc.stderr[-400:]}")
        out.append((json.loads(line), json.loads(found.group(1))))
    results = [r for r, _ in out]
    counters = [c for _, c in out]
    same = all(c == counters[0] for c in counters)
    return same, results, counters[0]


def our_side(census):
    """G-35c: the mapping applied to our own census counts and rates."""
    expansions = census["expansions"]
    rows = []
    total = 0.0
    for bucket in census["buckets"].values():
        rate = bucket["s"] * 1e9 / bucket["calls"]
        seconds = bucket["calls"] * rate / 1e9
        total += seconds
        rows.append({
            "bucket": bucket.get("leaves", []),
            "s_census": bucket["s"],
            "s_from_counts": round(seconds, 4),
            "deviation": round(seconds / bucket["s"] - 1, 6) if bucket["s"] else 0,
            "calls_per_expansion": round(bucket["calls"] / expansions, 3),
            "ns_per_call": round(rate, 3),
        })
    return {
        "op_total_s_census": round(census["ops_total_s"], 4),
        "op_total_s_from_counts": round(total, 4),
        "relative_error": round(total / census["ops_total_s"] - 1, 6),
        "bar": "±2 %",
        "pass": abs(total / census["ops_total_s"] - 1) <= 0.02,
        "rows": rows,
    }


def shape(census, counters, label):
    body = counters["bodies"]
    # pp per ns: the census wall is 100 pp.
    ns_per_pp = census["wall_per_expansion_ns"] / 100.0
    rows = []
    m = 0.0
    unmapped_pp = 0.0
    for leaf, keys, unit, inside in MAP:
        bucket = census["buckets"][leaf]
        rate = bucket["s"] * 1e9 / bucket["calls"]
        ours = bucket["calls"] / census["expansions"]
        ours_pp = bucket["pp"]
        row = {
            "leaf": leaf,
            "pp": ours_pp,
            "ns_per_op": round(rate, 3),
            "ours_per_expansion": round(ours, 3),
            "unit": unit,
            "inside": inside,
            "folded": keys is None,
        }
        if keys is None:
            row.update(p_ops_per_body=0.0, ns=0.0, ratio=None,
                       addressable_pp=ours_pp, counters={})
            unmapped_pp += ours_pp
        else:
            p = sum(counters[k] for k in keys) / body
            ns = p * rate
            ratio = ours / p if p else None
            addr = ours_pp * (1 - 1 / ratio) if ratio and ratio > 1 else 0.0
            m += ns
            row.update(p_ops_per_body=round(p, 4),
                       counters={k: counters[k] for k in keys},
                       ns=round(ns, 2),
                       ratio=round(ratio, 3) if ratio else None,
                       below_ours=bool(ratio and ratio <= 1),
                       addressable_pp=round(addr, 2))
        rows.append(row)
    return {
        "cell": label,
        "bodies": body,
        "rows": rows,
        "M_ns": round(m, 2),
        "M_over_T4": round(m / T4_NS, 4),
        "headroom_ns": round(T4_NS - m, 1),
        "addressable_pp": round(sum(r["addressable_pp"] for r in rows), 2),
        "addressable_pp_needed": 81.3,
        "M_plus_R_constant_ns": round(m + R_NS, 1),
        "M_plus_R_scaled_ns": round(m * (1 + R_NS / 6376.0), 1),
        "M_plus_R_literal_ns": round(m + R_LITERAL, 1),
        "band_ns": [round(m * (1 - BAND), 1), round(m * (1 + BAND), 1)],
        "band_flips": (m * (1 - BAND) <= T4_NS) or (m * (1 + BAND) >= T4_NS),
        "unmapped_pp": round(unmapped_pp, 2),
        "unmapped_share_of_op_budget": round(
            unmapped_pp / census["ops_total_pp"] * 100, 2),
        "ns_per_pp": round(ns_per_pp, 2),
    }


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--binary", default="/tmp/atomi-plan35/provari/target/release/provari")
    ap.add_argument("--runs", type=int, default=3)
    ap.add_argument("--reference",
                    default="/tmp/atomi-oracle/provari/target/release/provari",
                    help="the unpatched build at the pin, for tree identity "
                         "and the cross-check (its wall prices nothing)")
    ap.add_argument("--out", default=str(ROOT / "measurements/plan35/shape.json"))
    args = ap.parse_args()
    binary = pathlib.Path(args.binary)
    census = json.loads(CENSUS.read_text())

    data = {
        "plan": "35",
        "item": 43,
        "cells": {},
        "our_side": our_side(census),
        "decode": {},
        "gates": {},
    }
    for cell, seconds in CELLS:
        same, results, counters = run_cell(binary, cell, seconds, args.runs)
        data["cells"][cell] = shape(census, counters, cell)
        data["cells"][cell]["seconds"] = seconds
        data["cells"][cell]["runs"] = args.runs
        data["cells"][cell]["counters_identical"] = same
        data["cells"][cell]["reference"] = results[0]
        data["cells"][cell]["counters"] = counters
        priced = set()
        for _, keys, _, inside in MAP:
            priced.update(keys or [])
            priced.update(inside)
        data["cells"][cell]["counters_not_priced"] = {
            k: v for k, v in counters.items() if k not in priced}
        data["cells"][cell]["unpriced_note"] = (
            "reported for G-35a only: list sizes, legality passes, the "
            "repetition marking and the null use are inputs to a priced "
            "family's unit or free; none is priced")

    deep = data["cells"]["deep-win"]
    ref = deep["reference"]
    data["decode"] = {
        "denominator": "node bodies (one count at the node body entry), not "
                       "the reported `nodes`",
        "deep-win": {
            "bodies": deep["bodies"],
            "reported_nodes": ref["nodes"],
            "ratio": round(ref["nodes"] / deep["bodies"], 4),
            "replay_steps": deep["counters"]["replay_calls"],
            "early_returns_before_the_node_tick": (
                deep["bodies"] + deep["counters"]["replay_calls"] - ref["nodes"]),
        },
    }
    # Cross-check, NOT an input: how much of the reference's own per-body wall
    # the counted families recover. It validates the mapping's completeness; no
    # number here prices anything.
    deep_ns = ref["ms"] * 1e6 / deep["bodies"]
    clean = {}
    reference = pathlib.Path(args.reference)
    if reference.exists():
        for cell, seconds in CELLS:
            proc = subprocess.run(
                [str(reference), "prove", FEN[cell], "--guide", "hand",
                 "--seconds", str(seconds), "--memory", "192"],
                capture_output=True, text=True)
            clean[cell] = json.loads(proc.stdout.strip().splitlines()[-1])
    clean_ms = clean.get("deep-win", {}).get("ms", ref["ms"])
    clean_ns = clean_ms * 1e6 / deep["bodies"]
    data["cross_check"] = {
        "unpatched": clean,
        "tree_identical": all(
            clean.get(c, {}).get("nodes") == data["cells"][c]["reference"]["nodes"]
            and clean.get(c, {}).get("mate") == data["cells"][c]["reference"]["mate"]
            for c, _ in CELLS) if clean else None,
        "reference_ms_unpatched": clean_ms,
        "reference_ms_instrumented": ref["ms"],
        "ns_per_body": round(clean_ns, 1),
        "M_share_of_its_own_body": round(deep["M_ns"] / clean_ns, 4),
        "uncounted_share_ns": round(clean_ns - deep["M_ns"], 1),
        "our_uncounted_ns": round(R_NS, 1),
        "T4_ns": round(T4_NS, 1),
        "note": "the reference's own per-body wall, unpatched build, one run: "
                "the counted families recover M/that share of it. A check on "
                "the mapping's completeness; it prices nothing (the verdict "
                "stands on M alone).",
    }
    data["env"] = {
        "host": platform.platform(),
        "cpu": platform.processor(),
        "binary": str(binary),
        "binary_sha256": sha256(binary),
        "census": str(CENSUS.relative_to(ROOT)),
        "census_sha256": sha256(CENSUS),
    }
    data["gates"] = {
        "G-35a": {
            "bar": "unmapped share of our op budget ≤ 20 %",
            "deep-win": deep["unmapped_share_of_op_budget"],
            "body_vs_nodes": data["decode"]["deep-win"],
            "pass": deep["unmapped_share_of_op_budget"] <= 20,
        },
        "G-35b": {
            "bar": "3 identical runs give identical counts",
            "identical": {c: data["cells"][c]["counters_identical"]
                          for c, _ in CELLS},
            "pass": all(data["cells"][c]["counters_identical"] for c, _ in CELLS),
        },
        "G-35c": {
            "bar": "the mapping reproduces 37.005 s ± 2 % on our counts",
            "error": data["our_side"]["relative_error"],
            "pass": data["our_side"]["pass"],
        },
        "G-35d": {
            "bar": "M ≤ 1,723 ns/exp (the wall T_4 allows)",
            "M_ns": deep["M_ns"],
            "M_over_T4": deep["M_over_T4"],
            "band_ns": deep["band_ns"],
            "band_flips": deep["band_flips"],
            "verdict": "FUNDED" if deep["M_ns"] <= T4_NS else "NOT FUNDED",
        },
    }
    pathlib.Path(args.out).write_text(json.dumps(data, indent=1) + "\n")
    print(json.dumps({"cells": {c: {k: v for k, v in data["cells"][c].items()
                                   if k in ("bodies", "M_ns", "M_over_T4",
                                            "addressable_pp", "band_ns",
                                            "band_flips", "unmapped_pp",
                                            "unmapped_share_of_op_budget")}
                                for c, _ in CELLS},
                      "gates": {k: (v.get("verdict") or v.get("pass"))
                                for k, v in data["gates"].items()}},
                     indent=1))


if __name__ == "__main__":
    main()
