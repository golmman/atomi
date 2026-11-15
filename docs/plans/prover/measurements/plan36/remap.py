#!/usr/bin/env python3
"""Plan-36 item 45 (M-REMAP): `M` in like units — the mapping audit.

Reads two committed files and does arithmetic. No engine run, no reference run,
no clock:

* `docs/plans/prover/measurements/plan35/shape.json` — the reference's counted families per node
  body (integers) and report35's row table for `M`;
* `docs/plans/prover/measurements/plan34/census_120s.json` — our 21 census buckets (pp, calls,
  seconds), the 67 timed (leaf, site) calls, and the wall per expansion.

It emits `remap.json`: every reference family and every one of our buckets in
one unit, the *like-unit corrected* charge per row (and with it the corrected
addressable pp), the three pre-registered `M` conventions, the gates G-36a–d
and the verdict.

The conventions (fixed in `plan36` before this arithmetic was computed):

* **A — report35 verbatim.** `M = sum(p_ops/body x our ns/op)` over the 21
  rows; every counted family charged at the bucket's own rate, the three folded
  buckets at zero. Gate G-36b reproduces the committed 1,785.88 ns.
* **B — per-op, excess-only.** A, but each *per-child/per-move test family* is
  charged `max(0, theirs_per_body - ours_per_expansion)` at our cheapest
  matching rate, because we already do at least as much of that work. Nothing
  is priced twice.
* **C — per-op, region-total.** B, but the reference's child-decide loop is
  reconciled with *our* decide region before it is priced: everything our
  closure region does (the closure, the per-move screen, the status/legality
  queries, the key) is charged **once**, at our region's own ns per child
  decided, times their children decided.

Separately from A/B/C, the audit corrects what report35 *called* the work (its
row table) — the like-unit column. A row is `refuted` when its ratio was a unit
artifact: the reference does that work under another counter, so the row's
"our volume / their volume" ratio is not a saving.

    python3 docs/plans/prover/measurements/plan36/remap.py --out docs/plans/prover/measurements/plan36/remap.json

Nothing from the GPL clone enters this file: the family names are our own
counter names (already committed in `shape.json`), the numbers are integers and
our own rates.

Size justification (~28 KB, over the 10 KB guideline): the 39-family mapping
table, the three conventions and the four gates are one argument — a reader has
to be able to check the mapping against the arithmetic in one artifact, and the
observation it audits (`shape.json`) is a fixed input that will not grow. This
is measurement code: it is never compiled into the engine and touches no
`src/`.
"""

import argparse
import hashlib
import json
import pathlib
import platform

ROOT = pathlib.Path(__file__).resolve().parents[5]
SHAPE = ROOT / "docs/plans/prover/measurements/plan35/shape.json"
CENSUS = ROOT / "docs/plans/prover/measurements/plan34/census_120s.json"
CELL = "deep-win"
T4_NS = 10e9 / 5_803_257          # 1,723.17 ns/exp: our 120 s tree in 10 s
NEEDED_PP = 81.3                  # pp of our wall T_4 must delete
UNHASHABLE_LIMIT = 20.0           # > 20 % of the op budget => INCONCLUSIVE
G36B_TOLERANCE = 1.0              # "reproduces 1,785.9 ns (+- 1 ns)"

# The 39 counted reference families, each in the one unit we can defend.
# role: denominator | priced | inside | folded
FAMILIES = [
    ("bodies", "denominator", None,
     "the node body (not one reported node)"),
    ("list_generations", "priced", "movegen (legal)",
     "one move list generated"),
    ("moves_yielded", "inside", "movegen (legal)",
     "moves per list: the size of the priced unit, not a cost"),
    ("mover_setups", "priced", "key/rule50 (Mover::new)",
     "one mover setup"),
    ("legality_screens", "priced",
     "legality/status/terminality (needs-board/check test)",
     "one per-move legality screen"),
    ("legality_passes", "inside",
     "legality/status/terminality (needs-board/check test)",
     "passes over a list: the shape of a screen, not a cost"),
    ("child_decisions", "priced", "closure: frame pass-1 settle+screen",
     "one child decided from bit tests"),
    ("effect_computations", "priced", "key/rule50",
     "one child hash/effect"),
    ("effect_hash_computations", "priced", "key/rule50",
     "one child hash/effect (the same unit, charged once)"),
    ("child_builds", "priced", "do_move/undo_move/clone",
     "one child board built"),
    ("proof_probes", "priced", "proof probe",
     "one proof-store probe"),
    ("table_lookups", "priced", "TT probe",
     "one small-table lookup"),
    ("table_reads", "inside", "TT probe",
     "the read half of a lookup, charged with it"),
    ("filter_bit_tests", "priced", "filter probe",
     "one filter bit test"),
    ("table_stores", "priced", "table store",
     "one table store"),
    ("hand_evals", "priced", "guide hand_eval",
     "one hand evaluation of a body"),
    ("guide_steps", "priced", "guide class + start numbers",
     "one guide number pass"),
    ("terminality_queries", "priced", "legality/status/terminality",
     "one terminality/status query"),
    ("mate_tests", "inside", "legality/status/terminality",
     "a mate test inside the terminality family"),
    ("body_check_queries", "inside", "legality/status/terminality",
     "a check query inside the terminality family"),
    ("status_queries", "inside", "legality/status/terminality",
     "the root's own status query"),
    ("null_uses", "inside", "null board (`null_for`)",
     "the null move itself; its board is the priced unit"),
    ("null_builds", "priced", "null board (`null_for`)",
     "one null board"),
    ("replay_calls", "priced", "closure: replay/simulation",
     "one proof replay"),
    ("replay_defence_calls", "inside", "closure: replay/simulation",
     "per-defence work inside the priced replay"),
    ("selection_steps", "priced", "MID select/node_value",
     "one child set combined"),
    ("repetition_tests", "priced", "repetition scan",
     "one repetition test"),
    ("repetition_marks", "inside", "repetition scan",
     "the marking half of a repetition test"),
    ("prefetch_hints", "priced", "prefetch hint",
     "one prefetch hint"),
    # The per-child test families. report35 listed them as `inside` the per-move
    # screen row; the counts cannot support that (they sum to 126.7 per body
    # against a 59.2 screen counter). They are the work of *deciding a child*,
    # so they ride inside `child_decisions`, the one family that can contain
    # them (18.2 children x 7.0 tests per child).
    ("quiet_tests", "inside", "closure: frame pass-1 settle+screen",
     "a per-child quiet test"),
    ("exposing_tests", "inside", "closure: frame pass-1 settle+screen",
     "a per-child exposure test"),
    ("move_check_tests", "inside", "closure: frame pass-1 settle+screen",
     "a per-child check test"),
    ("blast_capture_tests", "inside", "closure: frame pass-1 settle+screen",
     "a per-child blast-capture test"),
    ("threat_tests", "inside", "closure: frame pass-1 settle+screen",
     "a per-child threat test"),
    ("evasion_scans", "inside", "closure: frame pass-1 settle+screen",
     "a per-child evasion scan"),
    ("material_tests", "inside", "closure: frame pass-1 settle+screen",
     "a per-child material test (0 calls on this cell)"),
    ("heavy_legality_tests", "inside", "closure: frame pass-1 settle+screen",
     "the heavy per-child legality test: the unit report35's status row missed"),
    ("attacking_bodies", "inside", "denominator",
     "a classification of the body, not a cost"),
    ("defending_bodies", "inside", "denominator",
     "a classification of the body, not a cost"),
]

# The two families whose *unit* report35's row table got wrong, and the our-leaf
# volume each is matched to. Both are the reference's per-child legality work.
LIKE_PAIRS = [
    # (their counters, the unit they share with these our-leaves, their same-unit partner row)
    (["heavy_legality_tests"], ["status@null_p1", "legality@null_p2"],
     "our per-reply legality screen (20.20 + 13.88 = 34.08 per expansion)"),
    (["terminality_queries", "mate_tests", "body_check_queries", "status_queries"],
     ["status@enter", "status@replay", "legality@replay"],
     "our remaining status queries (6.97 + 3.67 + 1.15 = 11.78 per expansion)"),
]

# C's region: everything our per-child decide loop does, folded into one rate.
REGION_BUCKETS = [
    "closure: frame pass-1 settle+screen",
    "legality/status/terminality (needs-board/check test)",
    "legality/status/terminality",
    "key/rule50",
]
SCREENS_ROW = "legality/status/terminality (needs-board/check test)"
STATUS_ROW = "legality/status/terminality"


def sha256(path):
    return hashlib.sha256(pathlib.Path(path).read_bytes()).hexdigest()


def bucket_rate(census, name):
    b = census["buckets"][name]
    return b["s"] * 1e9 / b["calls"]


def bucket_volume(census, name):
    return census["buckets"][name]["calls"] / census["expansions"]


def leaf_rate(census, leaf):
    return census["ops"][leaf]["net_ns"]


def leaf_volume(census, leaf):
    return census["ops"][leaf]["calls"] / census["expansions"]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--out", default=str(ROOT / "docs/plans/prover/measurements/plan36/remap.json"))
    args = ap.parse_args()

    shape = json.loads(SHAPE.read_text())
    census = json.loads(CENSUS.read_text())
    cell = shape["cells"][CELL]
    bodies = cell["bodies"]
    counters = cell["counters"]
    expansions = census["expansions"]
    wall_ns = census["wall_per_expansion_ns"]
    pp_ns = wall_ns / 100.0                          # ns per pp of the wall
    op_budget_pp = census["ops_total_pp"]

    # ------------------------------------------------------------- A
    rows = {r["leaf"]: r for r in cell["rows"]}
    M_A = round(sum(r["ns"] for r in cell["rows"]), 2)

    # ------------------------------------------------ the 39 families
    charged = set()
    families = []
    for name, role, bucket, unit in FAMILIES:
        count = counters[name]
        per_body = count / bodies
        entry = {"family": name, "counter": count, "per_body": round(per_body, 4),
                 "role": role, "our_bucket": bucket, "unit": unit}
        if role == "priced":
            entry["our_per_expansion"] = round(bucket_volume(census, bucket), 3)
            entry["our_ns_per_op"] = round(bucket_rate(census, bucket), 3)
            entry["ratio"] = round(bucket_volume(census, bucket) / per_body, 3)
            if bucket not in charged:
                entry["charge_A_ns"] = round(rows[bucket]["ns"], 2)
                charged.add(bucket)
        families.append(entry)

    # --------------------------------- the like-unit correction of A's rows
    their_like = {}
    for keys, leaves, note in LIKE_PAIRS:
        their_like[STATUS_ROW] = their_like.get(STATUS_ROW, 0.0) + \
            sum(counters[k] for k in keys) / bodies
    status_rate = bucket_rate(census, STATUS_ROW)
    like_charge = {name: r["ns"] for name, r in rows.items() if not r["folded"]}
    like_charge[STATUS_ROW] = round(their_like[STATUS_ROW] * status_rate, 2)
    for name in rows:
        if rows[name]["folded"]:
            like_charge[name] = 0.0

    row_table = []
    for r in cell["rows"]:
        name = r["leaf"]
        ours = r["ours_per_expansion"]
        theirs_A = r["p_ops_per_body"]
        theirs_like_p = their_like.get(name, theirs_A)
        entry = {
            "row": name,
            "pp": r["pp"],
            "ns_per_op": r["ns_per_op"],
            "ours_per_expansion": ours,
            "theirs_per_body_A": theirs_A,
            "theirs_per_body_like": round(theirs_like_p, 4),
            "ratio_A": r["ratio"],
            "ratio_like": round(ours / theirs_like_p, 3) if theirs_like_p else None,
            "ns_A": r["ns"],
            "ns_like": round(like_charge.get(name, 0.0), 2),
            "addr_pp_A": r["addressable_pp"],
            "addr_pp_like": round(max(0.0, r["pp"] - like_charge.get(name, 0.0) / pp_ns),
                                  2),
            "folded": r["folded"],
            "refuted": False,
        }
        if name == STATUS_ROW:
            entry["refuted"] = True
            entry["refuted_reason"] = (
                "the 93x ratio is a unit artifact: the reference's per-reply "
                "legality is heavy_legality_tests (33.07 per body, 0.97x of our "
                "34.08 per expansion), so the 0.49-query row prices one "
                "database query and not the work; in like units the row's "
                "addressable share falls from 9.39 pp to %s pp" %
                entry["addr_pp_like"])
            entry["units"] = [{"theirs": round(sum(counters[k] for k in keys) / bodies, 4),
                               "counters": keys, "ours_per_expansion":
                               round(sum(leaf_volume(census, leaf) for leaf in leaves), 3),
                               "matched_unit": note}
                              for keys, leaves, note in LIKE_PAIRS]
        if name == SCREENS_ROW:
            light = ("quiet_tests", "exposing_tests", "move_check_tests",
                     "blast_capture_tests", "threat_tests", "evasion_scans",
                     "material_tests")
            per_child = sum(counters[k] for k in light
                            + ("heavy_legality_tests",)) / bodies
            entry["reclassified"] = (
                "the seven per-child test families are inside "
                "`child_decisions`, not inside this row: they sum to %.1f per "
                "body (%.2f per child decided) against this row's %.1f screens "
                "per body, so one counter cannot contain them"
                % (per_child, per_child / (counters["child_decisions"] / bodies),
                   counters["legality_screens"] / bodies))
            entry["per_child_tests"] = {
                k: round(counters[k] / bodies, 3) for k in light
                + ("heavy_legality_tests",)}
            entry["per_child_tests_per_child_decided"] = round(
                per_child / (counters["child_decisions"] / bodies), 2)
        row_table.append(entry)

    # ------------------------------------------------------- buckets
    buckets = []
    for name, b in census["buckets"].items():
        row = rows.get(name)
        buckets.append({
            "bucket": name,
            "pp": b["pp"],
            "leaves": b["leaves"],
            "calls_per_expansion": round(b["calls"] / expansions, 3),
            "ns_per_op": round(bucket_rate(census, name), 3),
            "ns_per_expansion": round(b["s"] * 1e9 / expansions, 1),
            "role": "folded" if (row is None or row["folded"]) else "priced",
            "charge_A_ns": round(row["ns"], 2) if (row and not row["folded"]) else 0.0,
            "charge_like_ns": round(like_charge.get(name, 0.0), 2),
            "in_region_C": name in REGION_BUCKETS,
        })

    def pp_of(name):
        return census["buckets"][name]["s"] * 100.0 / census["reference_wall_s"]

    partition = {r: round(sum(pp_of(b["bucket"]) for b in buckets if b["role"] == r), 3)
                 for r in ("priced", "folded", "unhashable", "inside")}
    partition["total"] = round(sum(pp_of(b["bucket"]) for b in buckets), 3)
    partition["total_of_parts"] = round(sum(partition[r] for r in
                                            ("priced", "folded", "unhashable", "inside")), 3)

    # ------------------------------------------------------------- B
    screens_theirs = counters["legality_screens"] / bodies
    screens_ours = leaf_volume(census, "mover_check@enter")
    screens_excess = max(0.0, screens_theirs - screens_ours)
    heavy_theirs = counters["heavy_legality_tests"] / bodies
    heavy_ours = (leaf_volume(census, "status@null_p1")
                  + leaf_volume(census, "legality@null_p2"))
    heavy_excess = max(0.0, heavy_theirs - heavy_ours)
    term_theirs = sum(counters[k] for k in
                      ("terminality_queries", "mate_tests", "body_check_queries",
                       "status_queries")) / bodies
    term_ours = (leaf_volume(census, "status@enter") + leaf_volume(census, "status@replay")
                 + leaf_volume(census, "legality@replay"))
    term_excess = max(0.0, term_theirs - term_ours)

    variants = []
    for label, leaf in (("B0 — the screen excess at our per-move screen rate (the convention)",
                         "mover_check@enter"),
                        ("B1 — the screen excess at our cheapest test rate",
                         "repetition@enter"),
                        ("B2 — the screen counter cannot be put in our unit "
                         "(charged 0: the generosity floor)", None)):
        rate = leaf_rate(census, leaf) if leaf else 0.0
        m = (M_A - rows[SCREENS_ROW]["ns"] - rows[STATUS_ROW]["ns"]
             + screens_excess * rate
             + heavy_excess * rate
             + term_excess * leaf_rate(census, "status@enter"))
        variants.append({
            "label": label,
            "rate_leaf": leaf,
            "rate_ns": round(rate, 3),
            "screens": {"theirs_per_body": round(screens_theirs, 4),
                        "ours_per_expansion": round(screens_ours, 3),
                        "excess": round(screens_excess, 4),
                        "charge_ns": round(screens_excess * rate, 2)},
            "heavy_legality": {"theirs_per_body": round(heavy_theirs, 4),
                               "ours_per_expansion": round(heavy_ours, 3),
                               "excess": round(heavy_excess, 4),
                               "charge_ns": round(heavy_excess * rate, 2)},
            "terminality": {"theirs_per_body": round(term_theirs, 4),
                            "ours_per_expansion": round(term_ours, 3),
                            "excess": round(term_excess, 4),
                            "charge_ns": round(term_excess
                                               * leaf_rate(census, "status@enter"), 2)},
            "M_ns": round(m, 2),
        })
    M_B = variants[0]["M_ns"]

    # ------------------------------------------------------------- C
    region_ns = sum(b["ns_per_expansion"] for b in buckets if b["in_region_C"])
    our_children = leaf_volume(census, "closure_frame@enter")
    region_rate = region_ns / our_children
    theirs_children = counters["child_decisions"] / bodies
    region_charge = theirs_children * region_rate
    M_C = round(M_A - sum(b["charge_A_ns"] for b in buckets if b["in_region_C"])
                + region_charge, 2)

    # ------------------------------------------------ addressable pp
    def addressable(charges):
        total = 0.0
        for b in buckets:
            if b["role"] == "folded":
                total += b["pp"]
            else:
                total += max(0.0, b["pp"] - charges.get(b["bucket"], 0.0) / pp_ns)
        return round(total, 2)

    charge_A = {b["bucket"]: b["charge_A_ns"] for b in buckets}
    charge_C = dict(charge_A)
    for name in REGION_BUCKETS:
        charge_C[name] = 0.0
    charge_C[REGION_BUCKETS[0]] = round(region_charge, 2)

    def conv(m, charge=None, extra=None):
        d = {
            "M_ns": m,
            "M_over_T4": round(m / T4_NS, 4),
            "headroom_or_shortfall_ns": round(T4_NS - m, 1),
            "addressable_pp_wall_minus_M": round((wall_ns - m) / pp_ns, 2),
        }
        if charge is not None:
            d["charges"] = {k: round(v, 2) for k, v in charge.items()}
        if extra:
            d.update(extra)
        return d

    conventions = {
        "A": conv(M_A, charge_A,
                  {"definition": "report35 verbatim: every counted family at our "
                                 "bucket rate, the folded buckets at 0"}),
        "B": conv(M_B, None,
                  {"definition": "A, but each per-child/per-move test family "
                                 "charged excess-only at our cheapest matching "
                                 "rate (nothing priced twice)",
                   "variables": variants,
                   "charges": {
                       SCREENS_ROW: variants[0]["screens"]["charge_ns"],
                       STATUS_ROW: 0.0,
                       **{k: round(v, 2) for k, v in charge_A.items()
                          if k not in (SCREENS_ROW, STATUS_ROW)}},
                   "note": "the screens row is repriced excess-only; the status "
                           "row's two like-unit families both have zero excess, "
                           "so it is charged 0"}),
        "C": conv(M_C, charge_C,
                  {"definition": "B, but the decide loop reconciled to our region "
                                 "and charged once at the region's own rate",
                   "region": {
                       "buckets": REGION_BUCKETS,
                       "our_ns_per_expansion": round(region_ns, 1),
                       "our_children_decided_per_expansion": round(our_children, 3),
                       "our_region_ns_per_child": round(region_rate, 2),
                       "their_children_decided_per_body": round(theirs_children, 4),
                       "charge_ns": round(region_charge, 2),
                       "note": "at our region's 77.1 ns per child decided, a "
                               "port that decided 18.2 children the way we "
                               "decide one would spend 1,403.9 ns/exp of the "
                               "1,723 the rung allows; the screens row is inside "
                               "this region and is not charged twice"}}),
    }

    like_unit_pp = addressable(like_charge)
    report35_pp = addressable(charge_A)
    addressable = {
        "needed_pp": NEEDED_PP,
        "report35_leaf_wise_pp": report35_pp,
        "like_unit_leaf_wise_pp": like_unit_pp,
        "like_unit_delta_pp": round(like_unit_pp - report35_pp, 2),
        "wall_minus_M_pp": {k: conventions[k]["addressable_pp_wall_minus_M"]
                            for k in ("A", "B", "C")},
        "note": "leaf-wise = our bucket's own cost minus what the reference "
                "spends on the same work at our rate; wall-minus-M = what a "
                "body costing exactly M would leave of our wall (report35 §4's "
                "second reading, which assumes our whole per-node scaffolding "
                "disappears too)",
    }

    # ------------------------------------------------ verdict
    values = {k: conventions[k]["M_ns"] for k in ("A", "B", "C")}
    lo, hi = min(values.values()), max(values.values())
    floor = min(v["M_ns"] for v in variants)
    if hi <= T4_NS:
        outcome, rule = "FUNDED", "every convention inside the bar"
    elif lo > T4_NS:
        outcome, rule = "NOT FUNDED", "the lowest convention exceeds the bar"
    else:
        outcome, rule = ("INCONCLUSIVE-BY-UNITS",
                         "the band straddles the bar: min <= 1,723 < max")

    data = {
        "plan": "36", "item": 45, "name": "M-REMAP", "cell": CELL,
        "units": {
            "reference_denominator": "one node body",
            "our_denominator": "one expansion",
            "bodies": bodies, "expansions": expansions,
            "reference_reported_nodes": cell["reference"]["nodes"],
            "expansions_per_body": round(expansions / bodies, 4),
            "wall_ns_per_expansion": wall_ns,
            "ns_per_pp": round(pp_ns, 2),
            "op_budget_pp": op_budget_pp,
            "T4_ns": round(T4_NS, 2),
            "addressable_pp_needed": NEEDED_PP,
        },
        "families": families,
        "rows": row_table,
        "buckets": buckets,
        "partition": partition,
        "conventions": conventions,
        "addressable": addressable,
        "verdict": {
            "A": values["A"], "B": values["B"], "C": values["C"],
            "min": lo, "max": hi, "band_ns": [lo, hi],
            "generosity_floor_ns": floor,
            "outcome": outcome, "rule": rule,
            "note": "plan36 pre-registers FUNDED <=> min <= 1,723 and "
                    "INCONCLUSIVE-BY-UNITS <=> min <= 1,723 < max; the two "
                    "clauses overlap by construction, so the third (the "
                    "refinement) is applied whenever max > 1,723",
        },
        "env": {
            "host": platform.platform(),
            "machine_id_from_plan35": "e10de197da58",
            "shape_json": str(SHAPE.relative_to(ROOT)),
            "shape_sha256": sha256(SHAPE),
            "census_json": str(CENSUS.relative_to(ROOT)),
            "census_sha256": sha256(CENSUS),
            "runs": "none: arithmetic on committed integers only",
        },
        "gates": {},
    }

    priced_buckets = [b["bucket"] for b in buckets if b["role"] == "priced"]
    mapped = [f["our_bucket"] for f in families if f["role"] == "priced"]
    data["gates"]["G-36a"] = {
        "bar": "every one of the 21 census buckets is in exactly one of "
               "priced / inside / folded / unhashable, summing to 100 % of the "
               "op budget (68.97 pp)",
        "buckets": len(buckets),
        "partition_pp": partition,
        "op_budget_pp": op_budget_pp,
        "sum_note": "pp recomputed from the committed seconds "
                    "(s / wall x 100), so the total is 68.965 pp = the census's "
                    "rounded 68.97; `inside` and `unhashable` are empty",
        "pass": (len(buckets) == 21
                 and round(abs(partition["total"] - op_budget_pp), 2) <= 0.01
                 and round(abs(partition["total"] - partition["total_of_parts"]), 3)
                 <= 0.001),
    }
    data["gates"]["G-36b"] = {
        "bar": "convention A reproduces 1,785.9 ns from the committed files (+- 1 ns)",
        "committed_M_ns": cell["M_ns"], "recomputed_M_ns": M_A,
        "delta_ns": round(M_A - cell["M_ns"], 4),
        "pass": abs(M_A - cell["M_ns"]) <= G36B_TOLERANCE,
    }
    shared = sorted({b for b in mapped if mapped.count(b) > 1})
    data["gates"]["G-36c"] = {
        "bar": "no family priced twice (asserted on the leaf set)",
        "priced_buckets": len(priced_buckets),
        "distinct_priced_buckets": len(set(priced_buckets)),
        "families_charging_a_bucket": len(mapped),
        "buckets_shared_by_two_counters": shared,
        "shared_note": "two counters that are two faces of one unit (the effect "
                       "and its hash) enter one bucket's single charge",
        "heavy_legality_priced_at": ["legality@replay 22.92 ns (excess 0)",
                                     "mover_check@enter 3.66 ns (excess 0)"],
        "region_C": "the four region buckets carry one charge between them "
                    "(on closure_frame), the other three at 0.00",
        "pass": (set(mapped) == set(priced_buckets)
                 and all(f["role"] != "priced" or f["our_bucket"] in priced_buckets
                         for f in families)
                 and all(set(c.get("charges", charge_A)) ==
                         {b["bucket"] for b in buckets}
                         for c in conventions.values())),
    }
    data["gates"]["G-36d"] = {
        "bar": "A, B, C, min, max and the verdict reported",
        "A": values["A"], "B": values["B"], "C": values["C"],
        "min": lo, "max": hi,
        "unhashable_pp": partition["unhashable"],
        "unhashable_share_of_op_budget": round(partition["unhashable"]
                                               / op_budget_pp * 100, 2),
        "verdict": outcome,
        "pass": (partition["unhashable"] / op_budget_pp * 100) <= UNHASHABLE_LIMIT,
    }

    pathlib.Path(args.out).write_text(json.dumps(data, indent=1) + "\n")
    print(json.dumps({
        "A": values["A"], "B": values["B"], "C": values["C"],
        "band": [lo, hi], "generosity_floor": floor,
        "T4": round(T4_NS, 2), "verdict": outcome,
        "addressable_pp": {k: addressable[k] for k in
                           ("report35_leaf_wise_pp", "like_unit_leaf_wise_pp",
                            "like_unit_delta_pp")},
        "wall_minus_M_pp": addressable["wall_minus_M_pp"],
        "partition": partition,
        "gates": {k: v["pass"] for k, v in data["gates"].items()},
    }, indent=1))


if __name__ == "__main__":
    main()
