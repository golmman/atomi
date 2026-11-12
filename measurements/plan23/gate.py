#!/usr/bin/env python3
"""Plan 23 Phase C: the T_3 gate at the selected default proof share.

Sequential, one process at a time:

  1. reference: pinned provari `prove <FEN> --guide hand --seconds 10
     --memory 192`, three repeats per cell, same session;
  2. candidate: `atomi prove <FEN> --guide hand --seconds 120 --memory 192
     --seed N` (no `--proof-share`: the default is the selected share),
     seeds 0-2;
  3. D14: for every proved candidate row, one untimed `--seconds 600
     --proof-out` run with the same flags and seed; `child_evals` must
     match and both checkers must accept the proof.

Reads Phase A's 25 % control medians from `sweep.json` for the wall guard.
Writes parsed JSON only; raw output and proof files stay in scratch.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
import platform
import statistics
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

sys.dont_write_bytecode = True

CELLS = {
    "core": "r1b1kbnr/ppp3pp/n4q2/3pppN1/1P1PP3/8/P1P2PPP/RNBQKB1R w KQkq - 0 1",
    "easy-win": "rnbq1b1r/pp1p1kpp/2p1pp1n/8/3PP1P1/2N2N2/PPP2P1P/R1BQKB1R w KQ - 0 1",
    "deep-win": "rnb1kbnr/pppp2pp/5q2/4ppN1/3P4/8/PPP1PPPP/RNBQKB1R w KQkq - 0 1",
}
SEEDS = (0, 1, 2)
TIMED_SECONDS = 120
PROOF_SECONDS = 600
REFERENCE_SECONDS = 10


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--atomi", default="target/release/atomi")
    p.add_argument("--provari",
                   default="/tmp/atomi-oracle/provari/target/release/provari")
    p.add_argument("--sweep", default="measurements/plan23/sweep.json")
    p.add_argument("--scratch", default="/tmp/atomi-p23c")
    p.add_argument("--out", default="measurements/plan23/gate.json")
    return p.parse_args()


def last_json(text: str) -> dict:
    for line in reversed(text.splitlines()):
        line = line.strip()
        if line.startswith("{"):
            try:
                return json.loads(line)
            except json.JSONDecodeError:
                continue
    return {}


def run(cmd: list[str], timeout: int) -> tuple[int, str, float]:
    start = time.perf_counter()
    try:
        proc = subprocess.run(cmd, capture_output=True, text=True,
                              timeout=timeout + 120, check=False)
        return proc.returncode, proc.stdout + proc.stderr, time.perf_counter() - start
    except subprocess.TimeoutExpired:
        return 124, "", time.perf_counter() - start


def check(path: Path, atomi: str, provari: str) -> dict:
    v = subprocess.run([atomi, "validate", str(path)], capture_output=True,
                       text=True, check=False)
    c = subprocess.run([provari, "check", str(path)], capture_output=True,
                       text=True, check=False)
    return {
        "atomi_validate": last_json(v.stdout + v.stderr).get("valid"),
        "provari_check": last_json(c.stdout + c.stderr).get("valid"),
    }


def proved(row: dict) -> bool:
    return bool(row.get("found")) and row.get("exit") == "proved"


def accepted(row: dict) -> bool:
    c = row.get("checks") or {}
    return (c.get("atomi_validate") is True
            and c.get("provari_check") is True
            and row.get("child_evals_match") is True)


def main() -> int:
    args = parse_args()
    scratch = Path(args.scratch)
    scratch.mkdir(parents=True, exist_ok=True)
    sweep = json.loads(Path(args.sweep).read_text())
    control = sweep["summary"]["25"]

    references = []
    for cell, fen in CELLS.items():
        for repeat in range(3):
            cmd = [args.provari, "prove", fen, "--guide", "hand",
                   "--seconds", str(REFERENCE_SECONDS), "--memory", "192"]
            code, text, wall = run(cmd, REFERENCE_SECONDS)
            row = last_json(text)
            references.append({
                "cell": cell, "repeat": repeat + 1, "code": code,
                "wall_s": round(wall, 3), "found": row.get("found"),
                "mate": row.get("mate"), "nodes": row.get("nodes"),
                "ms": row.get("ms"),
            })
            print("ref", cell, repeat + 1, round(wall, 3), flush=True)

    runs = []
    for cell, fen in CELLS.items():
        for seed in SEEDS:
            name = f"{cell}-seed{seed}"
            cmd = [args.atomi, "prove", fen, "--guide", "hand",
                   "--seconds", str(TIMED_SECONDS), "--memory", "192",
                   "--seed", str(seed), "--stats"]
            code, text, wall = run(cmd, TIMED_SECONDS)
            row = last_json(text)
            row.update({"run": name, "cell": cell, "seed": seed, "code": code,
                        "wall_s": round(wall, 3)})
            if proved(row):
                proof = scratch / f"{name}.pvr"
                if proof.exists():
                    proof.unlink()
                pcmd = [args.atomi, "prove", fen, "--guide", "hand",
                        "--seconds", str(PROOF_SECONDS), "--memory", "192",
                        "--seed", str(seed), "--proof-out", str(proof),
                        "--stats"]
                pcode, ptext, pwall = run(pcmd, PROOF_SECONDS)
                prow = last_json(ptext)
                row.update({
                    "proof_run_code": pcode,
                    "proof_run_wall_s": round(pwall, 3),
                    "proof": prow.get("proof"),
                    "proof_positions": prow.get("proof_positions"),
                    "proof_length": prow.get("proof_length"),
                    "proof_entries": prow.get("proof_entries"),
                    "proof_full": prow.get("proof_full"),
                    "proof_attacker_entries": prow.get("proof_attacker_entries"),
                    "proof_defender_entries": prow.get("proof_defender_entries"),
                    "proof_archive_refused": prow.get("proof_archive_refused"),
                    "proof_child_evals": prow.get("child_evals"),
                    "child_evals_match": prow.get("child_evals") == row.get("child_evals"),
                    "checks": (check(proof, args.atomi, args.provari)
                               if proof.exists() else
                               {"atomi_validate": None, "provari_check": None}),
                })
            runs.append(row)
            print(name, json.dumps({k: row.get(k) for k in (
                "exit", "code", "wall_s", "child_evals", "expansions",
                "proof_entries", "proof_full", "proof_attacker_entries",
                "proof_defender_entries", "proof_archive_refused",
                "proof", "proof_positions", "proof_length",
                "child_evals_match", "checks")}), flush=True)

    ref_walls = {}
    for cell in CELLS:
        ws = [r["wall_s"] for r in references if r["cell"] == cell]
        ref_walls[cell] = round(statistics.median(ws), 3)
    summary = {}
    for cell in CELLS:
        rs = [r for r in runs if r["cell"] == cell]
        walls = [r["wall_s"] for r in rs]
        med = statistics.median(walls)
        summary[cell] = {
            "proved": sum(proved(r) for r in rs),
            "accepted": sum(accepted(r) for r in rs),
            "walls": walls,
            "median_wall": round(med, 3),
            "reference_median": ref_walls[cell],
            "factor": round(med / ref_walls[cell], 4),
            "control_median": control[cell]["median_wall"],
            "guard_ratio": round(med / control[cell]["median_wall"], 4),
            "guard_ok": med <= control[cell]["median_wall"] * 1.10,
        }
    claimed = [c for c in CELLS if summary[c]["proved"] == 3
               and summary[c]["accepted"] == 3]
    geomean = (round(math.exp(sum(math.log(summary[c]["factor"])
                                  for c in claimed) / len(claimed)), 4)
               if claimed else None)
    t3 = all(summary[c]["proved"] == 3 for c in CELLS)
    d14 = all(summary[c]["accepted"] == 3 for c in CELLS)
    guards = all(summary[c]["guard_ok"] for c in ("core", "easy-win"))
    data = {
        "date": datetime.now(timezone.utc).isoformat(),
        "host": platform.platform(),
        "cpu": platform.processor(),
        "atomi": os.path.abspath(args.atomi),
        "atomi_sha256": hashlib.sha256(Path(args.atomi).read_bytes()).hexdigest(),
        "atomi_git": subprocess.run(["git", "rev-parse", "HEAD"],
                                    capture_output=True, text=True).stdout.strip(),
        "provari": args.provari,
        "provari_version": subprocess.run([args.provari, "--version"],
                                          capture_output=True, text=True).stdout.strip(),
        "guide": "hand",
        "memory_mb": 192,
        "proof_share": "default (75)",
        "timed_seconds": TIMED_SECONDS,
        "proof_seconds": PROOF_SECONDS,
        "reference_seconds": REFERENCE_SECONDS,
        "seeds": list(SEEDS),
        "cells": CELLS,
        "references": references,
        "reference_medians": ref_walls,
        "runs": runs,
        "summary": summary,
        "claimed_cells": claimed,
        "geomean": geomean,
        "t3_proved_all": t3,
        "d14_all": d14,
        "guards_ok": guards,
        "gate_passed": bool(t3 and d14 and guards),
    }
    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(data, indent=2) + "\n")
    print("summary", json.dumps(summary))
    print("claimed", claimed, "geomean", geomean)
    print("t3", t3, "d14", d14, "guards", guards,
          "gate", data["gate_passed"])
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
