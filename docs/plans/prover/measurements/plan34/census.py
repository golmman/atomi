#!/usr/bin/env python3
"""Plan-34 item 42 — the exhaustive-bucket rdtsc census of one 120 s
`deep-win` seed-0 run.

Three kinds of run, one process at a time, same session, same host:

* **reference** — the shipped binary (the wall), `--guide hand --memory 192
  --seconds 120 --seed 0`;
* **sites** — the instrumented build with the leaf buckets off and only the
  inclusive site aggregates on (overhead ≈ 0.3 s): the *measured*,
  exhaustive site totals;
* **ops** — one instrumented build per leaf group, only that group's leaves
  timed (plus the in-situ empty-interval tax samples): the per-op buckets,
  each with exact call counts.

Why not one build with every leaf timed: measuring a leaf blocks the
inlining/streaming the surrounding code relies on, and the effect grows with
the number of intervals per expansion (mg_legal measures 111 ns/call with
everything instrumented, 69 ns/call measured alone). The isolated builds keep
the observer effect local; the tax of one interval is measured in situ
(an rdtsc pair with an empty body, sampled at every site).

Per-op time = max(0, mean interval − tax at that site) × call count.
`other` = reference wall − Σ buckets; it is checked against the plan gate
(< 10 pp) both from the site totals (independently measured, exhaustive) and
from the op buckets.

    python3 docs/plans/prover/measurements/plan34/census.py \
        --atomi target/release/atomi --scratch /tmp/atomi-plan34 \
        --out docs/plans/prover/measurements/plan34/census_120s.json
"""

import argparse
import hashlib
import json
import pathlib
import subprocess
import sys

FEN = "rnb1kbnr/pppp2pp/5q2/4ppN1/3P4/8/PPP1PPPP/RNBQKB1R w KQkq - 0 1"
EXPECTED_EXPANSIONS = 5_803_257
EXPECTED_CHILD_EVALS = 155_236_740

# The isolated builds: name -> (leaves timed, besides `tax`).
GROUPS = {
    "movegen": ["mg_legal"],
    "status": ["status", "legality", "mover_check"],
    "board": ["do_move", "undo_move", "clone", "key", "mover_new", "null_board"],
    "tables": [
        "proof_probe",
        "tt_probe",
        "filter",
        "prefetch",
        "store",
        "null_cache",
    ],
    "closure": ["closure_frame", "closure_null_p2", "closure_replay"],
    "guide": ["guide_class", "guide_eval", "repetition", "select"],
    "alloc": ["alloc", "dealloc"],
}

# leaf -> bucket (the plan's bucket list; each leaf belongs to exactly one)
BUCKETS = {
    "mg_legal": "movegen (legal)",
    "status": "legality/status/terminality",
    "legality": "legality/status/terminality",
    "mover_check": "legality/status/terminality (needs-board/check test)",
    "do_move": "do_move/undo_move/clone",
    "undo_move": "do_move/undo_move/clone",
    "clone": "do_move/undo_move/clone",
    "key": "key/rule50",
    "mover_new": "key/rule50 (Mover::new)",
    "null_board": "null board (`null_for`)",
    "tt_probe": "TT probe",
    "proof_probe": "proof probe",
    "filter": "filter probe",
    "prefetch": "prefetch hint",
    "store": "table store",
    "null_cache": "exposed-limit cache",
    "closure_frame": "closure: frame pass-1 settle+screen",
    "closure_null_p2": "closure: null pass-2 reply test",
    "closure_replay": "closure: replay/simulation",
    "guide_class": "guide class + start numbers",
    "guide_eval": "guide hand_eval",
    "repetition": "repetition scan",
    "select": "MID select/node_value",
    "alloc": "allocator",
    "dealloc": "allocator",
}

# Site and region aggregates of the `sites` build.
SITES = {
    "A_enter": "Frame::enter (whole site)",
    "A_pass1_loop": "Frame::enter pass 1 (per-move loop)",
    "A_pass3": "Frame::enter pass 3 (repetition + lookup)",
    "A_start_numbers": "Frame::enter start numbers",
    "A_null_replay": "null-replay envelope (per defender node)",
    "A_null_win": "null_win (pass 1 + pass 2)",
    "A_null_pass2_loop": "null pass 2 (loop over the null moves)",
    "A_null_p2": "null pass 2, uncached reply test",
    "A_replay": "replay/simulation (`simulate`, inclusive)",
    "A_siblings": "sibling replay",
    "A_sim": "sibling playout (off in this stack)",
}


def sha256(path):
    h = hashlib.sha256()
    h.update(pathlib.Path(path).read_bytes())
    return h.hexdigest()


def prove(binary, seconds, seed):
    out = subprocess.run(
        [
            str(binary), "prove", FEN,
            "--guide", "hand", "--memory", "192",
            "--seconds", str(seconds), "--seed", str(seed),
        ],
        capture_output=True, text=True, check=True,
    )
    summary = json.loads(out.stdout.strip().splitlines()[-1])
    probe = None
    for line in out.stderr.strip().splitlines():
        if '"probe"' in line:
            probe = json.loads(line)["probe"]
    return summary, probe


def build(script, dest, leaves, stride, only):
    cmd = [sys.executable, str(script), "--dest", str(dest),
           "--leaves", leaves, "--stride", str(stride)]
    if only:
        cmd += ["--only", ",".join(only)]
    subprocess.run(cmd, check=True, capture_output=True)
    subprocess.run(["cargo", "build", "--release"], cwd=dest, check=True,
                   capture_output=True)
    return pathlib.Path(dest) / "target" / "release" / "atomi"


def tsc_hz(repo):
    """The TSC rate from a 1 s sleep, or the recorded plan-33 value."""
    src = repo / "measurements" / "plan34" / "tsc_hz.c"
    exe = pathlib.Path("/tmp/atomi-plan34-tsc_hz")
    try:
        subprocess.run(["gcc", "-O2", "-o", str(exe), str(src)], check=True,
                       capture_output=True)
        out = subprocess.run([str(exe)], check=True, capture_output=True, text=True)
        return float(out.stdout.split()[0])
    except (OSError, subprocess.CalledProcessError):
        return 3_400_013_000.0


def tics_to_s(tics, hz):
    return tics / hz


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--repo", default=str(pathlib.Path(__file__).resolve().parents[5]))
    ap.add_argument("--atomi", default="target/release/atomi")
    ap.add_argument("--scratch", default="/tmp/atomi-plan34")
    ap.add_argument("--seconds", type=int, default=120)
    ap.add_argument("--seed", type=int, default=0)
    ap.add_argument("--out", required=True)
    args = ap.parse_args()
    repo = pathlib.Path(args.repo).resolve()
    scratch = pathlib.Path(args.scratch)
    script = repo / "measurements" / "plan34" / "instrument.py"
    hz = tsc_hz(repo)

    runs = {}

    def record(name, binary, summary, probe):
        assert probe is not None, f"{name}: no probe line"
        tree = (summary["expansions"], summary["child_evals"])
        assert tree == (EXPECTED_EXPANSIONS, EXPECTED_CHILD_EVALS), (
            f"{name}: tree moved {tree}"
        )
        runs[name] = {
            "binary": str(binary),
            "binary_sha256": sha256(binary),
            "ms": summary["ms"],
            "expansions": summary["expansions"],
            "child_evals": summary["child_evals"],
            "length": summary["length"],
            "probe_entries": summary["proof_entries"],
            "probe": probe,
        }
        print(f"{name:14s} {summary['ms'] / 1000:7.3f} s  tree ok", flush=True)

    ref, _ = prove(repo / args.atomi, args.seconds, args.seed)
    assert (ref["expansions"], ref["child_evals"]) == (
        EXPECTED_EXPANSIONS,
        EXPECTED_CHILD_EVALS,
    ), "reference tree is not the plan-33 row"
    runs["reference"] = {
        "binary": str(repo / args.atomi),
        "binary_sha256": sha256(repo / args.atomi),
        "ms": ref["ms"],
        "expansions": ref["expansions"],
        "child_evals": ref["child_evals"],
        "length": ref["length"],
        "probe_entries": ref["proof_entries"],
    }
    print(f"{'reference':14s} {ref['ms'] / 1000:7.3f} s  tree ok", flush=True)
    wall = ref["ms"] / 1000.0

    # 1. The site totals (exhaustive, leaf buckets off).
    binary = build(script, scratch / "sites", "off", 32, None)
    s, p = prove(binary, args.seconds, args.seed)
    record("sites", binary, s, p)
    sites = {}
    sites_total = 0.0
    for key, tics in sorted(runs["sites"]["probe"]["aggregates"].items()):
        name = key.split("@")[0]
        t = tics_to_s(tics["tics"], hz)
        sites[name] = {
            "label": SITES.get(name, name),
            "s": round(t, 3),
            "pp": round(100 * t / wall, 2),
            "ns_per_expansion": round(1e9 * t / runs["sites"]["expansions"], 1),
            "intervals": tics["intervals"],
        }
    # The exclusive regions, by subtraction of the inclusive aggregates:
    # the partition is enter{pass0+setup, pass 1, pass 3, start numbers} +
    # null_replay{null_win{pass 1, pass 2}, per-child replay envelope} +
    # sibling replay + everything outside the frames (= `other`).
    def S(k):
        return sites[k]["s"]

    regions = {
        "enter_pass0": ("Frame::enter pass 0 + setup (status, movegen, guide ctx, allocs)",
                        S("A_enter") - S("A_pass1_loop") - S("A_pass3") - S("A_start_numbers")),
        "enter_pass1": ("Frame::enter pass 1 (per-move loop)", S("A_pass1_loop")),
        "enter_pass3": ("Frame::enter pass 3 (repetition + lookup)", S("A_pass3")),
        "enter_start": ("Frame::enter start numbers + frame build", S("A_start_numbers")),
        "null_pass1": ("null_win pass 1 (key + status + proof scan)",
                       S("A_null_win") - S("A_null_pass2_loop")),
        "null_pass2": ("null_win pass 2 (loop over the null moves)",
                       S("A_null_pass2_loop")),
        "null_envelope": ("null-replay per-child envelope (clone/do + replay call)",
                          S("A_null_replay") - S("A_null_win")),
        "siblings": ("sibling replay", S("A_siblings")),
    }
    top = sum(v[1] for v in regions.values())
    regions["outside"] = ("MID loop + everything outside the frames",
                          wall - top)
    site_rows = {}
    for key, (label, t) in regions.items():
        site_rows[key] = {
            "label": label,
            "s": round(t, 3),
            "pp": round(100 * t / wall, 2),
            "ns_per_expansion": round(1e9 * t / ref["expansions"], 1),
        }
    sites = {"regions": site_rows, "inclusive": sites}

    # 2. One build with every leaf timed: the observer-effect control (a leaf
    #    measured alone vs measured inside the full instrumented body).
    full_bin = build(script, scratch / "full", "on", 1, None)
    s, p = prove(full_bin, args.seconds, args.seed)
    record("full", full_bin, s, p)

    # 3. The op buckets, one build per group.
    ops = {}
    for group, leaves in GROUPS.items():
        binary = build(script, scratch / f"ops-{group}", "on", 1, leaves + ["tax"])
        s, p = prove(binary, args.seconds, args.seed)
        record(f"ops-{group}", binary, s, p)
    for group in GROUPS:
        probe = runs[f"ops-{group}"]["probe"]
        # The tax of one interval, per site, measured in this same build.
        tax = {}
        for key, v in probe["leaves"].items():
            if key.startswith("tax@"):
                site = key.split("@")[1]
                tax[site] = v["tics"] / v["intervals"]
        for key, v in probe["leaves"].items():
            leaf, site = key.split("@")
            if leaf == "tax":
                continue
            mean = v["tics"] / v["intervals"]
            tx = tax.get(site, min(tax.values()) if tax else 0.0)
            net = max(0.0, mean - tx)
            t = tics_to_s(net * v["intervals"], hz)
            row = ops.setdefault(f"{leaf}@{site}", {
                "leaf": leaf,
                "site": site,
                "group": group,
                "calls": v["intervals"],
                "raw_s": 0.0,
                "gross_ns": round(1e9 * mean / hz, 2),
                "tax_ns": round(1e9 * tx / hz, 2),
                "net_ns": round(1e9 * net / hz, 2),
                "clamped": net == 0.0,
            })
            row["calls"] = v["intervals"]
            row["raw_s"] += tics_to_s(v["tics"], hz)
    buckets = {}
    for row in ops.values():
        name = BUCKETS[row["leaf"]]
        t = row["net_ns"] * row["calls"] / 1e9
        b = buckets.setdefault(name, {"s": 0.0, "calls": 0, "leaves": []})
        b["s"] += t
        b["calls"] += row["calls"]
        b["leaves"].append(f"{row['leaf']}@{row['site']}")
    ops_total = 0.0
    for name, b in buckets.items():
        ops_total += b["s"]
        b["s"] = round(b["s"], 3)
        b["pp"] = round(100 * b["s"] / wall, 2)
        b["ns_per_expansion"] = round(1e9 * b["s"] / ref["expansions"], 1)
    buckets = dict(sorted(buckets.items(), key=lambda kv: -kv[1]["s"]))
    tax_table = {}
    for group in GROUPS:
        probe = runs[f"ops-{group}"]["probe"]
        for key, v in probe["leaves"].items():
            if key.startswith("tax@"):
                site = key.split("@")[1]
                ns = 1e9 * v["tics"] / v["intervals"] / hz
                tax_table.setdefault(site, []).append(round(ns, 2))

    # The observer effect: a leaf's net ns/call measured alone (the group
    # build) against measured inside the fully instrumented build.
    def net_ns(probe, leaf, site=None):
        out = {}
        tax = {}
        for key, v in probe["leaves"].items():
            if key.startswith("tax@"):
                tax[key.split("@")[1]] = v["tics"] / v["intervals"]
        for key, v in probe["leaves"].items():
            lf, st = key.split("@")
            if lf == "tax" or (leaf is not None and lf != leaf):
                continue
            if site is not None and st != site:
                continue
            mean = v["tics"] / v["intervals"]
            out[key] = (round(1e9 * max(0.0, mean - tax.get(st, 0.0)) / hz, 2),
                        v["intervals"])
        return out

    observer = {}
    for leaf in ("mg_legal", "proof_probe", "status", "guide_eval"):
        alone = {}
        for group in GROUPS:
            alone.update(net_ns(runs[f"ops-{group}"]["probe"], leaf))
        both = net_ns(runs["full"]["probe"], leaf)
        for key, (ns_alone, calls) in alone.items():
            ns_full = both.get(key, (None, None))[0]
            observer[key] = {
                "calls": calls,
                "ns_alone": ns_alone,
                "ns_all_leaves_timed": ns_full,
                "ratio": round(ns_full / ns_alone, 2) if ns_alone else None,
            }

    # Counts are exact per run; the allocator only counts while its leaf is
    # timed, so take each counter from any run that has it.
    counts = {}
    for run in runs.values():
        for k, v in run.get("probe", {}).get("counts", {}).items():
            counts[k] = max(counts.get(k, 0), v)
    exp = ref["expansions"]

    def per_exp(n):
        return round(counts[n] / exp, 3)

    # The decision numbers (plan34 stop rule): the largest bucket, the
    # removable ceiling, and what T_4/T_5 would require.
    def max_speedup(pp):
        # removing `pp` percent of the wall: wall / (wall * (1 - pp/100))
        return round(1.0 / (1.0 - pp / 100.0), 2)

    largest = max(buckets.items(), key=lambda kv: kv[1]["pp"])
    region_max = max(
        ((k, v) for k, v in sites["regions"].items() if k != "outside"),
        key=lambda kv: kv[1]["pp"],
    )
    op_pp = 100 * ops_total / wall
    need_t4 = round(100 * (1 - 1 / 5.34), 1)
    need_t5 = round(100 * (1 - 1 / 6.61), 1)
    decision = {
        "largest_op_bucket": {
            "bucket": largest[0],
            "s": largest[1]["s"],
            "pp": largest[1]["pp"],
            "max_speedup_if_free": max_speedup(largest[1]["pp"]),
        },
        "largest_region": {
            "region": region_max[0],
            "label": region_max[1]["label"],
            "s": region_max[1]["s"],
            "pp": region_max[1]["pp"],
            "max_speedup_if_free": max_speedup(region_max[1]["pp"]),
        },
        "removable_ceiling": {
            "op_census_pp": round(op_pp, 2),
            "op_census_max_speedup": max_speedup(op_pp),
            "note": "a free-everything-measured bound (every measured op "
                    "call at zero cost), not an estimate of what a rewrite "
                    "can delete",
        },
        "t4_required_pp": need_t4,
        "t4_required_max_speedup": 5.34,
        "t5_required_pp": need_t5,
        "t5_required_max_speedup": 6.61,
        "t4_required_s": round(wall * need_t4 / 100, 2),
        "op_census_s": round(ops_total, 3),
        "bucket_15pp_exists": any(b["pp"] >= 15 for b in buckets.values()),
        "removable_union_reaches_t4": max_speedup(op_pp) >= 5.34,
        "stop_rule_fires": (not any(b["pp"] >= 15 for b in buckets.values()))
        or max_speedup(op_pp) < 5.34,
    }

    out = {
        "plan": "prover plan 34, item 42 (EXPANSION-CENSUS)",
        "fen": FEN,
        "seed": args.seed,
        "seconds": args.seconds,
        "tsc_hz": round(hz),
        "reference_wall_s": wall,
        "expansions": exp,
        "wall_per_expansion_ns": round(1e9 * wall / exp, 1),
        "runs": runs,
        "sites": sites,
        "site_partition": {
            "measured_s": round(wall - sites["regions"]["outside"]["s"], 3),
            "pp": round(100 * (wall - sites["regions"]["outside"]["s"]) / wall, 2),
            "other_s": sites["regions"]["outside"]["s"],
            "other_pp": sites["regions"]["outside"]["pp"],
        },
        "observer_effect": observer,
        "ops": dict(sorted(ops.items(), key=lambda kv: -kv[1]["net_ns"] * kv[1]["calls"])),
        "buckets": buckets,
        "ops_total_s": round(ops_total, 3),
        "ops_total_pp": round(100 * ops_total / wall, 2),
        "other_pp": round(100 * (wall - ops_total) / wall, 2),
        "tax_ns_by_site": {k: sorted(set(v)) for k, v in tax_table.items()},
        "decision": decision,
        "per_expansion": {
            "moves_legal": per_exp("moves_legal"),
            "lists_legal": per_exp("lists_legal"),
            "lists_pseudo": per_exp("lists_pseudo"),
            "moves_pseudo": per_exp("moves_pseudo"),
            "children_built": per_exp("children_built"),
            "tt_probes": per_exp("tt_probes"),
            "proof_probes": per_exp("proof_probes"),
            "filter_probes": per_exp("filter_probes"),
            "prefetches": per_exp("prefetches"),
            "null_calls": per_exp("null_calls"),
            "null_moves": per_exp("null_moves"),
            "hand_evals": per_exp("hand_evals"),
            "replay_calls": per_exp("replay_calls"),
            "stores": per_exp("stores"),
            "allocs": per_exp("allocs"),
            "deallocs": per_exp("deallocs"),
            "child_evals": round(ref["child_evals"] / exp, 3),
        },
        "counts": counts,
        "gates": {
            "tree_identical": "PASS — every run reproduces 5,803,257 expansions "
                              "and 155,236,740 child evals",
            "other_from_regions_pp": round(
                100 * sites["regions"]["outside"]["s"] / wall, 2
            ),
            "op_buckets_total_pp": round(100 * ops_total / wall, 2),
            "untimed_inside_regions_pp": round(
                100 * (wall - ops_total - sites["regions"]["outside"]["s"]) / wall, 2
            ),
        },
    }
    pathlib.Path(args.out).write_text(json.dumps(out, indent=1) + "\n")
    print(f"\nwall {wall:.3f} s, {exp} expansions")
    print(f"{'region':52s} {'s':>8s} {'pp':>7s} {'ns/exp':>9s}")
    for name, row in sites["regions"].items():
        print(f"{row['label'][:52]:52s} {row['s']:8.3f} {row['pp']:7.2f} "
              f"{row['ns_per_expansion']:9.1f}")
    print(f"{'regions sum (== wall by construction)':52s} {wall:8.3f} "
          f"{100.0:7.2f}")
    print(f"\n{'inclusive site':30s} {'s':>8s} {'pp':>7s}")
    for name, row in sites["inclusive"].items():
        print(f"{row['label'][:30]:30s} {row['s']:8.3f} {row['pp']:7.2f}")
    print(f"\n{'bucket':48s} {'s':>8s} {'pp':>7s} {'calls':>14s}")
    for name, b in buckets.items():
        print(f"{name[:48]:48s} {b['s']:8.3f} {b['pp']:7.2f} {b['calls']:14d}")
    print(f"{'ops total (lower bound)':48s} {ops_total:8.3f} "
          f"{100 * ops_total / wall:7.2f}   untimed inside the regions "
          f"{100 * (wall - ops_total - sites['regions']['outside']['s']) / wall:.2f} pp")
    d = decision
    print("\ndecision")
    print(f"  largest op bucket   {d['largest_op_bucket']['bucket']}: "
          f"{d['largest_op_bucket']['pp']} pp "
          f"(<= {d['largest_op_bucket']['max_speedup_if_free']}x if free)")
    print(f"  largest region      {d['largest_region']['region']}: "
          f"{d['largest_region']['pp']} pp "
          f"(<= {d['largest_region']['max_speedup_if_free']}x if free)")
    print(f"  op census ceiling   {d['removable_ceiling']['op_census_pp']} pp "
          f"=> <= {d['removable_ceiling']['op_census_max_speedup']}x")
    print(f"  T_4 needs           >= {d['t4_required_pp']} pp "
          f"({d['t4_required_s']} s); T_5 >= {d['t5_required_pp']} pp")
    print(f"  stop rule fires     {d['stop_rule_fires']}")


if __name__ == "__main__":
    main()
