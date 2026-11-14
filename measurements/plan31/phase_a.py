#!/usr/bin/env python3
"""Plan-31 Phase 0 — attribution counters (read-only, scratch build).

The counters are **not** in this repo: they were added to a throwaway copy of
the tree (`src/probe.rs` + one `explosion(kind, site, list_len, scanned, hit)`
call per explosion-test exit, the `null_board`/`null_win`/`exposed_limit`
counters, and a `probe` object appended to `--stats`). The scratch copy is at
`/tmp/atomi-plan31-p0` and was never committed; this driver and its output
(`phase_a.json`) are the record.

Runs one `atomi prove --guide hand --stats` per cell against a **scratch**
build that carries plan-31's temporary counters (`src/probe.rs`, never
committed), and derives the per-expansion multiplicity the plan claims:
explosion-test calls/lists/moves, null-board FEN round-trips, reply lists,
and the total moves read from move lists.

Usage:

    python3 phase_a.py --probe /tmp/atomi-plan31-p0/target/release/atomi \
        --control target/release/atomi --evals 1000000,5000000 \
        --out measurements/plan31/phase_a.json
"""

import argparse
import json
import subprocess
import sys

CELLS = [
    ("core", "r1b1kbnr/ppp3pp/n4q2/3pppN1/1P1PP3/8/P1P2PPP/RNBQKB1R w KQkq - 0 1"),
    ("easy-win", "rnbq1b1r/pp1p1kpp/2p1pp1n/8/3PP1P1/2N2N2/PPP2P1P/R1BQKB1R w KQ - 0 1"),
    ("deep-win", "rnb1kbnr/pppp2pp/5q2/4ppN1/3P4/8/PPP1PPPP/RNBQKB1R w KQkq - 0 1"),
]

# site -> counter key prefix
SITES = {
    "class": "expl_class_calls",
    "null": "expl_null_calls",
    "replay": "expl_replay_calls",
    "settle": "expl_settle_calls",
    "settle_bits": "expl_settle_bits_calls",
    "other": "expl_other_calls",
}


def run(binary, fen, evals):
    cmd = [
        binary, "prove", fen, "--evals", str(evals), "--memory", "192",
        "--guide", "hand", "--stats", "--seed", "0",
    ]
    out = subprocess.run(cmd, capture_output=True, text=True, check=True)
    return json.loads(out.stdout)


def derive(probe, result, evals):
    e = result["expansions"]
    p = probe
    per = lambda v: round(v / e, 3) if e else None
    expl_calls = sum(p[k] for k in SITES.values())
    # moves read from every move list the guide+search build
    moves_total = (
        p["site_frame_moves"] + p["site_replay_moves"] + p["site_sim_moves"]
        + p["null_first_moves"] + p["expl_list_len"] + p["exposed_pseudo"]
    )
    expl_moves = p["expl_list_len"]
    return {
        "cell": None,
        "evals_cap": evals,
        "child_evals": result["child_evals"],
        "expansions": e,
        "ms": result["ms"],
        "counts": {k: v for k, v in sorted(p.items())},
        "per_expansion": {
            "frame_lists": per(p["site_frame_list"]),
            "frame_moves": per(p["site_frame_moves"]),
            "replay_lists": per(p["site_replay_list"]),
            "replay_moves": per(p["site_replay_moves"]),
            "explosion_calls": per(expl_calls),
            "explosion_lists": per(p["expl_lists"]),
            "explosion_moves": per(expl_moves),
            "explosion_scanned": per(p["expl_moves"]),
            "explosion_hits": per(p["expl_hits"]),
            "explosion_avg_list_len": (
                round(p["expl_list_len"] / p["expl_lists"], 2) if p["expl_lists"] else None
            ),
            "null_boards": per(p["null_board_calls"]),
            "null_first_moves": per(p["null_first_moves"]),
            "reply_lists": per(p["exposed_lists"]),
            "reply_pseudo_moves": per(p["exposed_pseudo"]),
            "reply_legal_moves": per(p["exposed_legal"]),
            "moves_total": per(moves_total),
        },
        "explosion_calls_by_site": {s: p[k] for s, k in SITES.items()},
        "explosion_calls_by_site_per_expansion": {
            s: per(p[k]) for s, k in SITES.items()
        },
        "shares_of_moves_total": {
            "explosion": round(expl_moves / moves_total, 4) if moves_total else None,
            "frame": round(p["site_frame_moves"] / moves_total, 4) if moves_total else None,
            "replay": round(p["site_replay_moves"] / moves_total, 4) if moves_total else None,
            "null_first": round(p["null_first_moves"] / moves_total, 4) if moves_total else None,
            "reply_streams": round(p["exposed_pseudo"] / moves_total, 4) if moves_total else None,
        },
        "null_replies_reported": result["stats"]["null_replies"],
        "aux_nodes": result["aux_nodes"],
        "settle_nodes": result["settle_nodes"],
        "sim_nodes": result["sim_nodes"],
    }


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--probe", required=True)
    ap.add_argument("--control", default=None)
    ap.add_argument("--evals", default="1000000,5000000")
    ap.add_argument("--cells", default=",".join(c for c, _ in CELLS))
    ap.add_argument("--out", required=True)
    args = ap.parse_args()
    caps = [int(x) for x in args.evals.split(",")]
    wanted = set(args.cells.split(","))
    out = {"probe_binary": args.probe, "control_binary": args.control, "cells": []}
    for cap in caps:
        for name, fen in CELLS:
            if name not in wanted:
                continue
            res = run(args.probe, fen, cap)
            row = derive(res["stats"]["probe"], res, cap)
            row["cell"] = name
            row["fen"] = fen
            if args.control:
                ctrl = run(args.control, fen, cap)
                row["control"] = {
                    "child_evals": ctrl["child_evals"],
                    "expansions": ctrl["expansions"],
                    "ms": ctrl["ms"],
                    "identical_tree": (
                        ctrl["child_evals"] == res["child_evals"]
                        and ctrl["expansions"] == res["expansions"]
                    ),
                }
            out["cells"].append(row)
            print(
                f"{name:9s} cap={cap:>8d} evals={row['child_evals']} "
                f"exp={row['expansions']} ms={row['ms']} "
                f"expl/exp={row['per_expansion']['explosion_calls']} "
                f"expl_moves/exp={row['per_expansion']['explosion_moves']} "
                f"lists/exp={row['per_expansion']['reply_lists']} "
                f"null/exp={row['per_expansion']['null_boards']}",
                file=sys.stderr,
            )
    with open(args.out, "w") as f:
        json.dump(out, f, indent=2, sort_keys=False)
        f.write("\n")


if __name__ == "__main__":
    main()
