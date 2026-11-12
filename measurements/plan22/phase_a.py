#!/usr/bin/env python3
"""Plan 22 Phase A: proof-table composition probe (parsed results only).

Four sequential runs, one process at a time, on the `deep-win` FEN with
`--guide hand` and `--stats`:

  A1  total 384 MB, default proof share (control for "the label is enough")
  A2  total 192 MB, default share (the store-full control)
  A3  total 192 MB, --proof-share 50
  A4  total 192 MB, --proof-share 75

Every written proof is checked by `atomi validate` and `provari check`.
The script writes parsed JSON only; stdout/stderr and proof files stay in a
scratch directory.  The stop rule is evaluated from A1's live attacker
entries against 9/10 of the 192 MB default-share slot count.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

sys.dont_write_bytecode = True

DEEP_WIN = "rnb1kbnr/pppp2pp/5q2/4ppN1/3P4/8/PPP1PPPP/RNBQKB1R w KQkq - 0 1"
SLOTS_192 = 3_145_728  # split(192).proof_slots

RUNS = {
    "A1": {"memory": 384, "extra": []},
    "A2": {"memory": 192, "extra": []},
    "A3": {"memory": 192, "extra": ["--proof-share", "50"]},
    "A4": {"memory": 192, "extra": ["--proof-share", "75"]},
}


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--atomi", default="target/release/atomi")
    p.add_argument("--provari-check",
                   default="/tmp/atomi-oracle/provari/target/release/provari")
    p.add_argument("--seconds", type=int, default=600)
    p.add_argument("--scratch", default="/tmp/atomi-p22")
    p.add_argument("--out", default="measurements/plan22/composition.json")
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


def main() -> int:
    args = parse_args()
    scratch = Path(args.scratch)
    scratch.mkdir(parents=True, exist_ok=True)
    rows = {}
    for name, cfg in RUNS.items():
        proof = scratch / f"{name.lower()}.pvr"
        if proof.exists():
            proof.unlink()
        cmd = [args.atomi, "prove", DEEP_WIN, "--guide", "hand",
               "--seconds", str(args.seconds), "--memory", str(cfg["memory"]),
               "--proof-out", str(proof), "--stats", *cfg["extra"]]
        code, text, wall = run(cmd, args.seconds)
        row = last_json(text)
        row.update({"code": code, "wall_s": round(wall, 3)})
        row["checks"] = (check(proof, args.atomi, args.provari_check)
                         if proof.exists() else
                         {"atomi_validate": None, "provari_check": None})
        rows[name] = row
        print(name, json.dumps({k: row.get(k) for k in (
            "exit", "code", "wall_s", "child_evals", "expansions",
            "proof", "proof_positions", "proof_length", "proof_entries",
            "proof_full", "proof_attacker_entries", "proof_defender_entries",
            "proof_archive_refused", "checks")}), flush=True)
    threshold = SLOTS_192 * 9 // 10
    a1_attacker = rows["A1"].get("proof_attacker_entries", 0)
    data = {
        "date": datetime.now(timezone.utc).isoformat(),
        "host": platform.platform(),
        "cpu": platform.processor(),
        "atomi": os.path.abspath(args.atomi),
        "atomi_sha256": hashlib.sha256(Path(args.atomi).read_bytes()).hexdigest(),
        "fen": DEEP_WIN,
        "guide": "hand",
        "seconds": args.seconds,
        "runs": rows,
        "stop_rule": {
            "a1_attacker_entries": a1_attacker,
            "threshold_9_10_of_192mb_slots": threshold,
            "a1_truncated_by_refusal": bool(rows["A1"].get("proof_full")),
            "fires": a1_attacker > threshold,
        },
    }
    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(data, indent=2) + "\n")
    print("stop_rule", json.dumps(data["stop_rule"]))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
