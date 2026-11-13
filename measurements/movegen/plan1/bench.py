#!/usr/bin/env python3
"""Plan 1 bench: `atomic-movegen` 2.4.0 (`m0`) vs 2.6.0 (`m1`), same tree.

The only difference between the two sides is the dependency pin: both
binaries are built from the identical source tree (only `Cargo.toml` +
`Cargo.lock` differ), so any wall difference is the crate's.

Per (case, cap) each repeat runs **every** side once — provari `--guide hand`
at 10 s (the reference, fixed by `docs/plans/prover/bench.md`), then `m0`,
then `m1` — one process at a time, so the sides share drift (reading rule 1).
Command construction, output parsing and the ledger arithmetic are imported
from [`bench/run_bench.py`](../../bench/run_bench.py): one measurement
contract, no second parser.

Writes parsed JSON only; no transcripts, no proofs.

Usage:
    python3 measurements/movegen/plan1/bench.py \\
        --m0 /tmp/atomi-m0-plan1/target/release/atomi \\
        --m1 target/release/atomi \\
        --provari /tmp/atomi-oracle/provari/target/release/provari \\
        --provari-commit 48686bc --caps 10,120 --repeats 3 \\
        --out measurements/movegen/plan1/bench.json
"""

from __future__ import annotations

import argparse
import json
import statistics
import sys
from pathlib import Path

sys.dont_write_bytecode = True

REPO = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(REPO / "bench"))

import run_bench as rb  # noqa: E402

# The three bench cells (`docs/plans/prover/bench.md`): short name -> cell id.
CELLS = {"core": "core", "easy-win": "1eb80b92f92c", "deep-win": "db6a741c723f"}


def atomi_run(binary: str, seconds: float, memory: int, fen: str) -> dict:
    cmd = rb.atomi_command(binary, seconds, memory, fen, [])
    code, text, wall = rb.run_process(cmd, seconds)
    payload = rb.last_json(text) or {}
    return {
        "wall_s": round(wall, 3),
        "returncode": code,
        "win": bool(payload.get("found")),
        "length": payload.get("length"),
        "exit": payload.get("exit"),
        "child_evals": payload.get("child_evals"),
        "expansions": payload.get("expansions"),
        "ms": payload.get("ms"),
    }


def provari_run(binary: str, seconds: float, memory: int, fen: str,
                guide: str) -> dict:
    cmd = rb.provari_command(binary, guide, seconds, memory, fen)
    code, text, wall = rb.run_process(cmd, seconds)
    payload = rb.last_json(text) or {}
    return {
        "wall_s": round(wall, 3),
        "returncode": code,
        "win": bool(payload.get("found")),
        "mate": payload.get("mate"),
        "nodes": payload.get("nodes"),
    }


def median(rows: list[dict], keys: tuple[str, ...]) -> dict:
    out: dict = {"runs": len(rows),
                 "wall_s": statistics.median(r["wall_s"] for r in rows)}
    out["win"] = all(r["win"] for r in rows)
    for key in keys:
        values = {r[key] for r in rows}
        out[key] = values.pop() if len(values) == 1 else sorted(
            v for v in values if v is not None)
    return out


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--m0", required=True, help="atomi built on 2.4.0")
    parser.add_argument("--m1", required=True, help="atomi built on 2.6.0")
    parser.add_argument("--provari", required=True)
    parser.add_argument("--provari-commit", default="unknown")
    parser.add_argument("--guide", default="hand")
    parser.add_argument("--caps", default="10,120", help="candidate seconds")
    parser.add_argument("--reference-seconds", type=float, default=10.0)
    parser.add_argument("--memory", type=int, default=192)
    parser.add_argument("--repeats", type=int, default=3)
    parser.add_argument("--corpus", default=str(REPO / "bench/corpus.json"))
    parser.add_argument("--out", required=True)
    args = parser.parse_args()

    corpus_sha = rb.verify_corpus(Path(args.corpus))
    cases = {c["id"]: c for c in rb.load_cases(Path(args.corpus))}
    caps = [float(x) for x in args.caps.split(",")]

    runs: list[dict] = []
    ledger: dict[str, dict] = {}
    for cap in caps:
        for short, cell_id in CELLS.items():
            fen = cases[cell_id]["fen"]
            ref_rows, m0_rows, m1_rows = [], [], []
            for index in range(args.repeats):
                ref = provari_run(args.provari, args.reference_seconds,
                                  args.memory, fen, args.guide)
                ref["side"] = f"provari-{args.guide}"
                m0 = atomi_run(args.m0, cap, args.memory, fen)
                m0["side"] = "m0"
                m1 = atomi_run(args.m1, cap, args.memory, fen)
                m1["side"] = "m1"
                for row, bucket in ((ref, ref_rows), (m0, m0_rows),
                                    (m1, m1_rows)):
                    row.update({"cell": short, "cap_s": cap,
                                "run": index + 1})
                    bucket.append(row)
                    runs.append(row)
            reference = median(ref_rows, ("nodes", "mate"))
            cands = {
                "m0": median(m0_rows, ("child_evals", "expansions", "ms")),
                "m1": median(m1_rows, ("child_evals", "expansions", "ms")),
            }
            cell: dict = {"reference": reference, "m0": cands["m0"],
                          "m1": cands["m1"]}
            for name, row in cands.items():
                if row["win"] and reference["win"]:
                    row["factor"] = round(row["wall_s"] / reference["wall_s"], 3)
                else:
                    row["factor"] = None
                    row["factor_lower_bound"] = round(
                        cap / reference["wall_s"], 2)
            if cands["m0"]["win"] and cands["m1"]["win"]:
                cell["m1_over_m0"] = round(
                    cands["m1"]["wall_s"] / cands["m0"]["wall_s"], 4)
                cell["tree_identical"] = (
                    cands["m0"]["child_evals"] == cands["m1"]["child_evals"]
                    and cands["m0"]["expansions"] == cands["m1"]["expansions"])
            else:
                cell["m1_over_m0"] = None
                cell["tree_identical"] = None
            ledger[f"{short}@{int(cap)}s"] = cell

    summary: dict[str, dict] = {}
    for cap in caps:
        for name in ("m0", "m1"):
            shared = [k for k in ledger
                      if k.endswith(f"@{int(cap)}s") and ledger[k][name]["factor"]]
            factors = [ledger[k][name]["factor"] for k in shared]
            summary[f"{name}@{int(cap)}s"] = {
                "claimed_cells": [k.split("@")[0] for k in shared],
                "claimed_count": len(shared),
                "geomean_factor": (round(statistics.geometric_mean(factors), 3)
                                    if factors else None),
            }
    paired = [ledger[k]["m1_over_m0"] for k in ledger
              if ledger[k]["m1_over_m0"]]
    summary["m1_over_m0"] = {
        "cells": [k for k in ledger if ledger[k]["m1_over_m0"]],
        "ratio_geomean": (round(statistics.geometric_mean(paired), 4)
                          if len(paired) > 1 else (paired[0] if paired else None)),
        "tree_identical_everywhere": all(
            ledger[k]["tree_identical"] for k in ledger
            if ledger[k]["tree_identical"] is not None),
    }

    payload = {
        "plan": "docs/plans/movegen/plan1.md",
        "cell_set": CELLS,
        "caps_s": caps,
        "reference": {"binary": args.provari,
                      "commit": args.provari_commit,
                      "guide": args.guide,
                      "version": rb.tool_out([args.provari, "version"])},
        "sides": {
            "m0": {"binary": args.m0, "sha256": rb.sha256_file(args.m0),
                   "dependency": "atomic-movegen 2.4.0 (published)"},
            "m1": {"binary": args.m1, "sha256": rb.sha256_file(args.m1),
                   "dependency": "atomic-movegen 2.6.0 (published)"},
        },
        "budgets": {"memory_mb": args.memory, "repeats": args.repeats,
                    "reference_seconds": args.reference_seconds},
        "corpus": {"path": args.corpus, "sha256": corpus_sha},
        "runs": runs,
        "ledger": ledger,
        "summary": summary,
    }
    Path(args.out).write_text(json.dumps(payload, indent=1) + "\n")
    print(f"wrote {args.out}")
    for key, row in summary.items():
        print(f"{key}: {row}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
