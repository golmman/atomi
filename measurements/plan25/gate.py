#!/usr/bin/env python3
"""Plan 25 Phase C: the NULL-V1 / NULL-S wall gates.

Sequential, one process at a time, `--guide hand --memory 192 --seconds 120`,
seeds 0-2, three cells. For each cell/seed the control binary and the
candidate binary run back to back. The candidate may carry extra guide flags
(`--guide-null-screen` for NULL-S).

For every proved candidate row one untimed `--seconds 600 --proof-out` run
must match `child_evals` and pass `atomi validate` and `provari check`.
Writes parsed JSON only; raw output and proof files stay in scratch.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import platform
import shlex
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


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--control", default="/tmp/atomi-plan25-control/target/release/atomi")
    p.add_argument("--candidate", default="target/release/atomi")
    p.add_argument("--candidate-flags", default="")
    p.add_argument("--provari-check",
                   default="/tmp/atomi-oracle/provari/target/release/provari")
    p.add_argument("--scratch", default="/tmp/atomi-p25")
    p.add_argument("--out", default="measurements/plan25/gate_v1.json")
    p.add_argument("--deep-threshold", type=float, default=0.80)
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
    extra = shlex.split(args.candidate_flags)
    rows = []
    for cell, fen in CELLS.items():
        for seed in SEEDS:
            name = f"{cell}-seed{seed}"
            pair = {}
            for arm, binary, flags in (
                ("control", args.control, []),
                ("candidate", args.candidate, extra),
            ):
                cmd = [binary, "prove", fen, "--guide", "hand",
                       "--seconds", str(TIMED_SECONDS), "--memory", "192",
                       "--seed", str(seed), "--stats"] + flags
                code, text, wall = run(cmd, TIMED_SECONDS)
                row = last_json(text)
                row.update({"run": name, "cell": cell, "seed": seed,
                            "arm": arm, "code": code, "wall_s": round(wall, 3)})
                pair[arm] = row
                print(arm, name, row.get("exit"), round(wall, 3), flush=True)
            control = pair["control"]
            candidate = pair["candidate"]
            same_tree = (
                control.get("child_evals") == candidate.get("child_evals")
                and control.get("expansions") == candidate.get("expansions")
            )
            row = {
                "run": name, "cell": cell, "seed": seed,
                "control": control, "candidate": candidate,
                "same_tree": same_tree,
            }
            if proved(candidate):
                proof = scratch / f"{name}.pvr"
                if proof.exists():
                    proof.unlink()
                pcmd = [args.candidate, "prove", fen, "--guide", "hand",
                        "--seconds", str(PROOF_SECONDS), "--memory", "192",
                        "--seed", str(seed), "--proof-out", str(proof),
                        "--stats"] + extra
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
                    "proof_child_evals": prow.get("child_evals"),
                    "child_evals_match": prow.get("child_evals") == candidate.get("child_evals"),
                    "checks": (check(proof, args.candidate, args.provari_check)
                               if proof.exists() else
                               {"atomi_validate": None, "provari_check": None}),
                })
            rows.append(row)

    summary = {}
    for cell in CELLS:
        rs = [r for r in rows if r["cell"] == cell]
        cw = [r["control"]["wall_s"] for r in rs]
        kw = [r["candidate"]["wall_s"] for r in rs]
        cmed = statistics.median(cw)
        kmed = statistics.median(kw)
        summary[cell] = {
            "control_walls": cw,
            "candidate_walls": kw,
            "control_median": round(cmed, 3),
            "candidate_median": round(kmed, 3),
            "ratio": round(kmed / cmed, 4) if cmed else None,
            "control_proved": sum(proved(r["control"]) for r in rs),
            "candidate_proved": sum(proved(r["candidate"]) for r in rs),
            "candidate_accepted": sum(accepted(r) for r in rs),
            "same_tree": sum(r["same_tree"] for r in rs),
        }
    deep = summary["deep-win"]
    guards = {
        "deep_win_ratio": deep["ratio"],
        "deep_win_threshold": args.deep_threshold,
        "deep_win_3_proved": deep["candidate_proved"] == 3,
        "deep_win_3_accepted": deep["candidate_accepted"] == 3,
        "small_cells_within_1_1": all(
            summary[c]["ratio"] is not None and summary[c]["ratio"] <= 1.10
            and summary[c]["candidate_proved"] == 3
            and summary[c]["candidate_accepted"] == 3
            for c in ("core", "easy-win")
        ),
        "tree_identical": all(r["same_tree"] for r in rows),
    }
    passed = (
        guards["deep_win_3_proved"]
        and guards["deep_win_3_accepted"]
        and guards["small_cells_within_1_1"]
        and guards["tree_identical"]
        and deep["ratio"] is not None
        and deep["ratio"] <= args.deep_threshold
    )
    data = {
        "date": datetime.now(timezone.utc).strftime("%Y-%m-%d"),
        "control": args.control,
        "control_sha256": hashlib.sha256(Path(args.control).read_bytes()).hexdigest(),
        "candidate": args.candidate,
        "candidate_sha256": hashlib.sha256(Path(args.candidate).read_bytes()).hexdigest(),
        "candidate_flags": extra,
        "host": platform.node(),
        "platform": platform.platform(),
        "machine": platform.machine(),
        "timed_seconds": TIMED_SECONDS,
        "proof_seconds": PROOF_SECONDS,
        "rows": rows,
        "summary": summary,
        "guards": guards,
        "passed": passed,
    }
    Path(args.out).write_text(json.dumps(data, indent=2) + "\n")
    print(json.dumps({"summary": summary, "guards": guards, "passed": passed}, indent=2))
    return 0 if passed else 1


if __name__ == "__main__":
    raise SystemExit(main())
