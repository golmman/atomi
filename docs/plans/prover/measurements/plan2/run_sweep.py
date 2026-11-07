#!/usr/bin/env python3
"""plan2 M3: the 10 M fixture sweep with both checkers (G4).

Runs every `tests/fixtures/wins.tsv` and `tests/fixtures/never_prove.tsv`
row with

    atomi prove FEN --evals N --memory 192 --seconds 600 --proof-out <tmp>

for the side `--atomi` (named `m1`) and, optionally, `--baseline-atomi`
(named `m0`), one process at a time. Every written proof goes through
`atomi validate` (the same side's binary) and `provari check`. Proof files
live in a temporary directory outside the repo and are deleted afterwards
(`--keep-proofs DIR` keeps them there instead); they are never committed.

Regression rule (G4): a `wins.tsv` row that `m0` proved and `m1` did not
(exit evals/timeout) is re-run for `m1` at `2 x` m0's `child_evals`; the
re-run is recorded under `rerun`. It fails the gate only if the re-run
does not prove it either.

`sweep.json` holds per row and side: exit, child_evals, expansions, length,
proof, proof_length, proof_positions, proof_entries, proof_full, both
verdicts and `positions` / `won_positions`; plus a per-side summary and the
G4 verdict block.

Usage:
    python3 docs/plans/prover/measurements/plan2/run_sweep.py \\
        --atomi target/release/atomi \\
        --baseline-atomi /tmp/atomi-m0/target/release/atomi \\
        --provari /tmp/atomi-oracle/provari/target/release/provari \\
        --out docs/plans/prover/measurements/plan2/sweep.json

Stdlib only. No transcripts; no `__pycache__`.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import platform
import shutil
import subprocess
import sys
import tempfile
import time
from datetime import datetime, timezone
from pathlib import Path

sys.dont_write_bytecode = True

REPO = Path(__file__).resolve().parents[5]
FIXTURES = REPO / "tests" / "fixtures"
CENSORED = ("evals", "timeout", "depth")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--atomi", required=True, help="the m1 binary")
    parser.add_argument("--baseline-atomi", help="the m0 binary")
    parser.add_argument("--provari", required=True,
                        help="provari binary for `provari check`")
    parser.add_argument("--evals", type=int, default=10_000_000)
    parser.add_argument("--memory", type=int, default=192)
    parser.add_argument("--seconds", type=int, default=600)
    parser.add_argument("--keep-proofs", metavar="DIR",
                        help="keep proof files in DIR (outside the repo)")
    parser.add_argument("--out", required=True)
    return parser.parse_args()


def read_tsv(path: Path) -> list[tuple[str, str]]:
    rows = []
    for line in path.read_text().splitlines():
        if not line.strip() or line.startswith("#") or line.startswith("name\t"):
            continue
        f = line.split("\t")
        rows.append((f[0], f[1]))
    return rows


def last_json(text: str) -> dict:
    for line in reversed(text.splitlines()):
        line = line.strip()
        if line.startswith("{"):
            try:
                return json.loads(line)
            except json.JSONDecodeError:
                continue
    return {}


def prove(binary: str, fen: str, evals: int, args: argparse.Namespace,
          proof: Path) -> dict:
    cmd = [binary, "prove", fen, "--evals", str(evals),
           "--memory", str(args.memory), "--seconds", str(args.seconds),
           "--proof-out", str(proof)]
    start = time.perf_counter()
    done = subprocess.run(cmd, capture_output=True, text=True)
    wall = time.perf_counter() - start
    p = last_json(done.stdout)
    row = {
        "returncode": done.returncode, "wall_s": round(wall, 3),
        "evals_limit": evals,
        **{k: p.get(k) for k in (
            "exit", "found", "length", "child_evals", "expansions", "ms",
            "proof", "proof_positions", "proof_length", "proof_entries",
            "proof_full", "error")},
    }
    if p.get("proof") == "written" and proof.exists():
        ours = last_json(subprocess.run([binary, "validate", str(proof)],
                                        capture_output=True, text=True).stdout)
        chk = last_json(subprocess.run([args.provari, "check", str(proof)],
                                       capture_output=True, text=True).stdout)
        row["validate"] = ours.get("valid")
        row["validate_error"] = ours.get("error")
        row["positions"] = ours.get("positions")
        row["checker"] = chk.get("valid")
        row["won_positions"] = chk.get("won_positions")
        if not row["validate"]:
            row["verdict"] = "validator-reject"
        elif row["checker"] is None:
            row["verdict"] = "unchecked"
        elif not row["checker"]:
            row["verdict"] = "checker-reject"
        elif row["positions"] != row["won_positions"]:
            row["verdict"] = "disagreement"
        else:
            row["verdict"] = "accepted"
    elif p.get("exit") == "proved":
        row["verdict"] = "export-failure"
    return row


def summarize(rows: list[dict], side: str) -> dict:
    wins = [r[side] for r in rows if r["file"] == "wins" and side in r]
    never = [r[side] for r in rows if r["file"] == "never_prove" and side in r]
    written = [x for x in wins + never if x.get("proof") == "written"]
    return {
        "wins_rows": len(wins),
        "wins_proved": sum(x.get("exit") == "proved" for x in wins),
        "wins_censored": sum(x.get("exit") in CENSORED for x in wins),
        "wins_refuted": sum(x.get("exit") == "refuted" for x in wins),
        "wins_error": sum(x.get("exit") is None for x in wins),
        "never_rows": len(never),
        "never_found": sum(x.get("exit") == "proved" for x in never),
        "never_error": sum(x.get("exit") is None for x in never),
        "proofs_written": len(written),
        "proofs_accepted": sum(x.get("verdict") == "accepted"
                               for x in written),
        "export_failures": sum(x.get("verdict") == "export-failure"
                               for x in wins + never),
        "proof_full_runs": sum(bool(x.get("proof_full"))
                               for x in wins + never),
    }


def sha256(path: str) -> str:
    try:
        return hashlib.sha256(Path(path).read_bytes()).hexdigest()
    except OSError:
        return "unknown"


def version(binary: str) -> str:
    return subprocess.run([binary, "--version"], capture_output=True,
                          text=True, check=False).stdout.strip()


def main() -> int:
    args = parse_args()
    sides = [("m1", args.atomi)]
    if args.baseline_atomi:
        sides.append(("m0", args.baseline_atomi))
    if args.keep_proofs:
        proof_dir = Path(args.keep_proofs).resolve()
        if REPO in proof_dir.parents or proof_dir == REPO:
            raise SystemExit("--keep-proofs must be outside the repo")
        proof_dir.mkdir(parents=True, exist_ok=True)
    else:
        proof_dir = Path(tempfile.mkdtemp(prefix="atomi-plan2-sweep-"))

    rows: list[dict] = []
    for file in ("wins", "never_prove"):
        for name, fen in read_tsv(FIXTURES / f"{file}.tsv"):
            row = {"file": file, "name": name, "fen": fen}
            for side, binary in sides:
                proof = proof_dir / f"{file}-{name}-{side}.jsonl"
                row[side] = prove(binary, fen, args.evals, args, proof)
                print(json.dumps({"file": file, "name": name, "side": side,
                                  **{k: row[side].get(k) for k in (
                                      "exit", "child_evals", "length",
                                      "proof", "verdict")}}), flush=True)
            rows.append(row)

    # G4 regression re-runs: m0 proved, m1 censored.
    regressions = []
    if args.baseline_atomi:
        for row in rows:
            m0, m1 = row.get("m0", {}), row["m1"]
            if (row["file"] == "wins" and m0.get("exit") == "proved"
                    and m1.get("exit") in CENSORED):
                cap = 2 * int(m0["child_evals"])
                proof = proof_dir / f"rerun-{row['name']}-m1.jsonl"
                row["rerun"] = prove(args.atomi, row["fen"], cap, args, proof)
                regressions.append({
                    "name": row["name"],
                    "m0_child_evals": m0["child_evals"],
                    "m1_child_evals": m1.get("child_evals"),
                    "rerun_evals_limit": cap,
                    "rerun_exit": row["rerun"].get("exit"),
                    "rerun_child_evals": row["rerun"].get("child_evals"),
                    "rerun_verdict": row["rerun"].get("verdict"),
                })

    summary = {side: summarize(rows, side) for side, _ in sides}
    s = summary["m1"]
    all_runs = [r["m1"] for r in rows] + [r["rerun"] for r in rows
                                           if "rerun" in r]
    written = [x for x in all_runs if x.get("proof") == "written"]
    g4 = {
        "wins_refuted": s["wins_refuted"],
        "never_found": s["never_found"],
        "proofs_written": len(written),
        "proofs_accepted": sum(x.get("verdict") == "accepted"
                               for x in written),
        "export_failures": sum(x.get("verdict") == "export-failure"
                               for x in all_runs),
        "regressions": regressions,
        "regressions_failing": [r["name"] for r in regressions
                                if r["rerun_exit"] != "proved"],
    }
    g4["pass"] = (g4["wins_refuted"] == 0 and g4["never_found"] == 0
                  and g4["proofs_accepted"] == g4["proofs_written"]
                  and g4["export_failures"] == 0
                  and not g4["regressions_failing"])

    result = {
        "provenance": {
            "date": datetime.now(timezone.utc).strftime("%Y-%m-%d"),
            "host": platform.node(), "platform": platform.platform(),
            "budgets": {"evals": args.evals, "memory_mb": args.memory,
                        "seconds": args.seconds},
            "sides": {side: {"binary": b, "version": version(b),
                             "binary_sha256": sha256(b)}
                      for side, b in sides},
            "provari": {"binary": args.provari, "version": version(args.provari),
                        "binary_sha256": sha256(args.provari)},
            "fixtures": {f: sha256(str(FIXTURES / f"{f}.tsv"))
                         for f in ("wins", "never_prove")},
        },
        "summary": summary,
        "g4": g4,
        "rows": rows,
    }
    if not args.keep_proofs:
        shutil.rmtree(proof_dir, ignore_errors=True)
    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(result, indent=1) + "\n")
    print(f"wrote {out}")
    print(json.dumps({"summary": summary, "g4": g4}, indent=1))
    return 0


if __name__ == "__main__":
    sys.exit(main())
