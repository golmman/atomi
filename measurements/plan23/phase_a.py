#!/usr/bin/env python3
"""Plan 23 Phase A: proof-retention share sweep (parsed results only).

Sequential, one process at a time, `--guide hand --memory 192`:

  * 120 s timed runs for core / easy-win / deep-win, seeds 0-2, at proof
    shares 25 (control), 50 and 75;
  * for every deep-win run that proves inside 120 s, one untimed
    `--seconds 600 --proof-out` run at the same share and seed, whose
    `child_evals` must match the timed run, checked by `atomi validate` and
    `provari check`.

The script writes parsed JSON only; stdout/stderr and proof files stay in a
scratch directory.  The fixed selection rule is evaluated from the rows.
"""

from __future__ import annotations

import argparse
import hashlib
import json
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
SHARES = (25, 50, 75)
SEEDS = (0, 1, 2)
TIMED_SECONDS = 120
PROOF_SECONDS = 600


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--atomi", default="target/release/atomi")
    p.add_argument("--provari-check",
                   default="/tmp/atomi-oracle/provari/target/release/provari")
    p.add_argument("--scratch", default="/tmp/atomi-p23")
    p.add_argument("--out", default="measurements/plan23/sweep.json")
    p.add_argument("--recompute", action="store_true",
                   help="re-evaluate summary/selection from --out, run nothing")
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
    pr = row.get("proof_run") or {}
    c = pr.get("checks") or row.get("checks") or {}
    return (c.get("atomi_validate") is True
            and c.get("provari_check") is True
            and pr.get("child_evals_match", True))


def evaluate(rows: list[dict]) -> tuple[dict, dict, int | None]:
    by = {(r["share"], r["cell"], r["seed"]): r for r in rows}
    summary = {}
    for share in SHARES:
        cells = {}
        for cell in CELLS:
            rs = [by[(share, cell, s)] for s in SEEDS]
            walls = [r["wall_s"] for r in rs]
            cells[cell] = {
                "proved": sum(proved(r) for r in rs),
                "accepted": sum(accepted(r) for r in rs),
                "walls": walls,
                "median_wall": round(statistics.median(walls), 3),
            }
        summary[str(share)] = cells
    control = summary["25"]
    selection = {}
    for share in (50, 75):
        s = summary[str(share)]
        deep_ok = s["deep-win"]["accepted"] == 3
        guards = {}
        for cell in ("core", "easy-win"):
            base = control[cell]["median_wall"]
            med = s[cell]["median_wall"]
            guards[cell] = {
                "control_median": base,
                "median": med,
                "ratio": round(med / base, 4) if base else None,
                "within_10pct": med <= base * 1.10,
                "proved_3": s[cell]["proved"] == 3,
            }
        selection[str(share)] = {
            "deep_win_accepted_3": deep_ok,
            "guards": guards,
            "passes": deep_ok and all(g["within_10pct"] and g["proved_3"]
                                      for g in guards.values()),
        }
    chosen = next((s for s in (50, 75) if selection[str(s)]["passes"]), None)
    return summary, selection, chosen


def recompute(args: argparse.Namespace) -> int:
    out = Path(args.out)
    data = json.loads(out.read_text())
    summary, selection, chosen = evaluate(data["runs"])
    data["summary"] = summary
    data["selection"] = selection
    data["chosen_share"] = chosen
    out.write_text(json.dumps(data, indent=2) + "\n")
    print("summary", json.dumps(summary))
    print("selection", json.dumps(selection))
    print("chosen_share", chosen)
    return 0


def main() -> int:
    args = parse_args()
    if args.recompute:
        return recompute(args)
    scratch = Path(args.scratch)
    scratch.mkdir(parents=True, exist_ok=True)
    rows = []
    for share in SHARES:
        for cell, fen in CELLS.items():
            for seed in SEEDS:
                name = f"s{share}-{cell}-seed{seed}"
                cmd = [args.atomi, "prove", fen, "--guide", "hand",
                       "--seconds", str(TIMED_SECONDS), "--memory", "192",
                       "--proof-share", str(share), "--seed", str(seed),
                       "--stats"]
                code, text, wall = run(cmd, TIMED_SECONDS)
                row = last_json(text)
                row.update({"run": name, "share": share, "cell": cell,
                            "seed": seed, "code": code,
                            "wall_s": round(wall, 3)})
                if cell == "deep-win" and proved(row):
                    proof = scratch / f"{name}.pvr"
                    if proof.exists():
                        proof.unlink()
                    pcmd = [args.atomi, "prove", fen, "--guide", "hand",
                            "--seconds", str(PROOF_SECONDS), "--memory", "192",
                            "--proof-share", str(share), "--seed", str(seed),
                            "--proof-out", str(proof), "--stats"]
                    pcode, ptext, pwall = run(pcmd, PROOF_SECONDS)
                    prow = last_json(ptext)
                    row["proof_run"] = {
                        "code": pcode,
                        "wall_s": round(pwall, 3),
                        "child_evals": prow.get("child_evals"),
                        "proof": prow.get("proof"),
                        "proof_positions": prow.get("proof_positions"),
                        "proof_length": prow.get("proof_length"),
                        "proof_entries": prow.get("proof_entries"),
                        "proof_full": prow.get("proof_full"),
                        "proof_attacker_entries": prow.get("proof_attacker_entries"),
                        "proof_defender_entries": prow.get("proof_defender_entries"),
                        "proof_archive_refused": prow.get("proof_archive_refused"),
                        "child_evals_match": prow.get("child_evals") == row.get("child_evals"),
                        "checks": (check(proof, args.atomi, args.provari_check)
                                   if proof.exists() else
                                   {"atomi_validate": None, "provari_check": None}),
                    }
                rows.append(row)
                print(name, json.dumps({k: row.get(k) for k in (
                    "exit", "code", "wall_s", "child_evals", "expansions",
                    "proof_entries", "proof_full", "proof_attacker_entries",
                    "proof_defender_entries", "proof_archive_refused")}),
                    flush=True)
                if "proof_run" in row:
                    print("  proof", json.dumps(row["proof_run"]), flush=True)

    # Fixed selection rule, evaluated from the rows.
    summary, selection, chosen = evaluate(rows)
    data = {
        "date": datetime.now(timezone.utc).isoformat(),
        "host": platform.platform(),
        "cpu": platform.processor(),
        "atomi": os.path.abspath(args.atomi),
        "atomi_sha256": hashlib.sha256(Path(args.atomi).read_bytes()).hexdigest(),
        "atomi_git": subprocess.run(["git", "rev-parse", "HEAD"],
                                    capture_output=True, text=True).stdout.strip(),
        "provari": args.provari_check,
        "provari_version": subprocess.run([args.provari_check, "--version"],
                                          capture_output=True, text=True).stdout.strip(),
        "guide": "hand",
        "memory_mb": 192,
        "timed_seconds": TIMED_SECONDS,
        "proof_seconds": PROOF_SECONDS,
        "shares": list(SHARES),
        "seeds": list(SEEDS),
        "cells": CELLS,
        "runs": rows,
        "summary": summary,
        "selection": selection,
        "chosen_share": chosen,
    }
    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(data, indent=2) + "\n")
    print("summary", json.dumps(summary))
    print("selection", json.dumps(selection))
    print("chosen_share", chosen)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
