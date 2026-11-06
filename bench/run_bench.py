#!/usr/bin/env python3
"""Bench driver for `docs/plans/prover/bench.md`.

from atomic_solver@5f85133ea9d751368a682e53e354bd3fc0016d00:docs/plans/winprover/measurements/bench/run_bench.py
(adapted: reference/stretch provari sides, atomi + legacy stacks with no
implicit default, corpus sha256 gate, --proof-check cells (D14/R6), the
atomi/legacy parsers and the expanded provenance block)

Measures the sides of the bench ledger on one host, sequentially, one
process at a time:

- **reference:** the pinned provari oracle under one guide
  (`--guide hand`, the initiative's primary column), labelled
  `provari-<guide>`;
- **stretch:** provari v0.2.0 under the same guide, measured in the same
  session but reported ONLY in a separate `stretch` block, never in the
  ledger/geomean;
- **candidates:** `atomi` under named flag stacks
  (`--atomi-stack NAME=FLAGS`, repeatable) and the legacy `atomic_solver`
  under `--legacy-stack NAME=FLAGS`, all versus the reference.

Every side gets the same nominal budget (10 s, 192 MB label) and repeat
count. Only parsed rows are written; raw stdout/stderr stay in memory (no
transcripts, no `__pycache__`).

`--proof-check` (D14/R6): for every atomi cell where all timed repeats
won, one extra run writes a proof (`--seconds 600 --proof-out`), which
must validate under `atomi validate` AND `provari check` with identical
child_evals across every repeat; any deviation censors the cell with the
verdict class as its note.

Usage:
    python3 bench/run_bench.py \\
        --provari /tmp/atomi-oracle/provari/target/release/provari \\
        --provari-commit 48686bc \\
        --atomi ./target/release/atomi --atomi-stack m0=

`--dry-run` prints the exact commands without running anything.

Justification for a single >10 KB file: corpus loading, command
construction, output parsing, provenance capture, the derived per-cell
ledger and the proof-check contract are one measurement contract (same
reasoning as the source driver).
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

HERE = Path(__file__).resolve().parent
REPO = HERE.parent
DEFAULT_CORPUS = HERE / "corpus.json"
DEFAULT_OUT = HERE / "results.json"
DEFAULT_PROOF_DIR = Path("/tmp/atomi-bench-proofs")
CORE_ID = "core"
CORPUS_SHA256 = (
    "9cd0169a30bb7d5b162b6a1ec119fe194b776c8a24a46cdf1582c07062847d0e"
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--provari", required=True, help="reference provari binary")
    parser.add_argument("--provari-commit", default="unknown")
    parser.add_argument("--provari-stretch", default=None,
                        help="stretch provari (v0.2.0) binary")
    parser.add_argument("--provari-stretch-commit", default="unknown")
    parser.add_argument("--atomi", default=None, help="atomi binary")
    parser.add_argument(
        "--atomi-stack", action="append", default=[], metavar="NAME=FLAGS",
        help="named atomi flag stack (repeatable)")
    parser.add_argument("--legacy-solver", default=None,
                        help="legacy atomic_solver binary")
    parser.add_argument("--legacy-commit", default="unknown")
    parser.add_argument(
        "--legacy-stack", action="append", default=[], metavar="NAME=FLAGS",
        help="named legacy flag stack (repeatable)")
    parser.add_argument("--corpus", default=str(DEFAULT_CORPUS))
    parser.add_argument("--out", default=str(DEFAULT_OUT))
    parser.add_argument("--seconds", type=float, default=10.0)
    parser.add_argument("--memory", type=int, default=192, metavar="MB")
    parser.add_argument("--repeats", type=int, default=3)
    parser.add_argument(
        "--guide", default="hand",
        help="provari evaluation guide (default: hand, the primary column)")
    parser.add_argument(
        "--cases", default="",
        help="comma-separated case ids (default: the whole corpus)")
    parser.add_argument("--proof-check", action="store_true",
                        help="run the D14 proof contract on won atomi cells")
    parser.add_argument("--provari-check", default=None,
                        help="checker binary for --proof-check")
    parser.add_argument("--proof-dir", default=str(DEFAULT_PROOF_DIR))
    parser.add_argument("--date", default=None)
    parser.add_argument("--dry-run", action="store_true")
    return parser.parse_args()


def split_stacks(specs: list[str]) -> list[tuple[str, list[str]]]:
    return [(spec.partition("=")[0], spec.partition("=")[2].split())
            for spec in specs]


def verify_corpus(path: Path) -> str:
    digest = hashlib.sha256(path.read_bytes()).hexdigest()
    if digest != CORPUS_SHA256:
        raise SystemExit(
            f"corpus sha256 mismatch: {digest} != {CORPUS_SHA256}")
    return digest


def load_cases(path: Path) -> list[dict]:
    data = json.loads(path.read_text())
    return [data["core"], *data["sample20"]]


def provari_command(binary: str, guide: str, seconds: float, memory: int,
                    fen: str) -> list[str]:
    return [binary, "prove", fen,
            "--seconds", str(int(seconds)),
            "--memory", str(memory),
            "--guide", guide]


def atomi_command(binary: str, seconds: float, memory: int, fen: str,
                  flags: list[str]) -> list[str]:
    return [binary, "prove", fen,
            "--seconds", str(int(seconds)),
            "--memory", str(memory),
            *flags]


def legacy_command(binary: str, seconds: float, memory: int, fen: str,
                   flags: list[str]) -> list[str]:
    return [binary,
            "--fen", fen,
            "--timeout", str(int(seconds)),
            "--tt-size", str(memory),
            "--first-outcome",
            "--outcome-only",
            *flags]


def run_process(cmd: list[str], timeout: float) -> tuple[int, str, float]:
    start = time.perf_counter()
    try:
        proc = subprocess.run(
            cmd, capture_output=True, text=True, timeout=timeout + 60,
            check=False)
        wall = time.perf_counter() - start
        return proc.returncode, proc.stdout + proc.stderr, wall
    except subprocess.TimeoutExpired:
        return 124, "", time.perf_counter() - start


def last_json(text: str) -> dict | None:
    for line in reversed(text.splitlines()):
        line = line.strip()
        if line.startswith("{"):
            try:
                return json.loads(line)
            except json.JSONDecodeError:
                continue
    return None


def parse_legacy(text: str) -> dict:
    row = {"outcome": "none", "evals": None, "exit": None, "pv_status": None}
    for line in text.splitlines():
        line = line.strip()
        if line.startswith("outcome:"):
            # `outcome: win length: 29` / `outcome: draw length: 0`
            words = line.split(":", 1)[1].split()
            row["outcome"] = words[0] if words else "none"
            row["pv_length"] = int(words[-1]) if words[-1].isdigit() else None
        elif line.startswith("evals:"):
            row["evals"] = int(line.split(":", 1)[1].strip())
        elif line.startswith("pv_status:"):
            row["pv_status"] = line.split(":", 1)[1].strip()
        elif line.startswith("pre_exit:"):
            for part in line.split(":", 1)[1].split():
                if "=" in part:
                    key, value = part.split("=", 1)
                    # The pre_exit key is `reason` (was `exit_reason` in
                    # older revisions; legacy report5 §10).
                    if key == "reason":
                        row["exit"] = value
    return row


def reference_row(binary: str, guide: str, side: str, args: argparse.Namespace,
                  case: dict) -> list[dict]:
    rows = []
    for index in range(args.repeats):
        cmd = provari_command(binary, guide, args.seconds, args.memory,
                              case["fen"])
        if args.dry_run:
            print("REF  " + " ".join(cmd))
            continue
        code, text, wall = run_process(cmd, args.seconds)
        payload = last_json(text) or {}
        rows.append({
            "case": case["id"], "side": side, "run": index + 1,
            "wall_s": round(wall, 3), "returncode": code,
            "win": bool(payload.get("found")), "mate": payload.get("mate"),
            "nodes": payload.get("nodes"), "ms": payload.get("ms"),
        })
    return rows


def atomi_rows(args: argparse.Namespace, case: dict, name: str,
               flags: list[str]) -> list[dict]:
    rows = []
    for index in range(args.repeats):
        cmd = atomi_command(args.atomi, args.seconds, args.memory,
                            case["fen"], flags)
        if args.dry_run:
            print("CAND " + " ".join(cmd))
            continue
        code, text, wall = run_process(cmd, args.seconds)
        payload = last_json(text) or {}
        rows.append({
            "case": case["id"], "side": f"atomi:{name}", "run": index + 1,
            "wall_s": round(wall, 3), "returncode": code,
            "win": bool(payload.get("found")),
            "length": payload.get("length"), "exit": payload.get("exit"),
            "child_evals": payload.get("child_evals"),
            "expansions": payload.get("expansions"), "ms": payload.get("ms"),
            "flags": flags,
        })
    return rows


def legacy_rows(args: argparse.Namespace, case: dict, name: str,
                flags: list[str]) -> list[dict]:
    rows = []
    for index in range(args.repeats):
        cmd = legacy_command(args.legacy_solver, args.seconds, args.memory,
                             case["fen"], flags)
        if args.dry_run:
            print("CAND " + " ".join(cmd))
            continue
        code, text, wall = run_process(cmd, args.seconds)
        parsed = parse_legacy(text)
        rows.append({
            "case": case["id"], "side": f"atomic_solver:{name}",
            "run": index + 1, "wall_s": round(wall, 3), "returncode": code,
            "win": parsed["outcome"] == "win",
            "outcome": parsed["outcome"], "pv_length": parsed.get("pv_length"),
            "evals": parsed["evals"], "exit": parsed["exit"],
            "pv_status": parsed["pv_status"], "flags": flags,
        })
    return rows


def medians(rows: list[dict], side: str) -> dict[str, dict]:
    by_case: dict[str, list[dict]] = {}
    for row in rows:
        if row["side"] == side:
            by_case.setdefault(row["case"], []).append(row)
    return {
        case: {
            "runs": len(group),
            "wall_s": statistics.median(r["wall_s"] for r in group),
            # A censored run must not be read as a latency: a side's cell is
            # a win only if every repeat won.
            "win": all(r["win"] for r in group),
            "nodes": next((r["nodes"] for r in group if r.get("nodes")), None),
            "mate": next((r["mate"] for r in group if r.get("mate")), None),
            "evals": next((r["evals"] for r in group if r.get("evals")), None),
            "child_evals": next(
                (r["child_evals"] for r in group if r.get("child_evals")),
                None),
            "exit": next((r["exit"] for r in group if r.get("exit")), None),
            "returncodes": sorted({r["returncode"] for r in group}),
        }
        for case, group in by_case.items()
    }


def proof_check(args: argparse.Namespace, case_id: str, stack: str,
                flags: list[str], runs: list[dict], fen: str) -> tuple[dict, str]:
    """The D14 cell contract: one proof run, both validators, identical
    child_evals. Returns (extra fields, verdict)."""
    path = Path(args.proof_dir) / f"{case_id}-{stack}.jsonl"
    cmd = atomi_command(args.atomi, 600, args.memory, fen,
                        [*flags, "--proof-out", str(path)])
    code, text, wall = run_process(cmd, 600)
    payload = last_json(text) or {}
    fields = {"proof_run": {
        "wall_s": round(wall, 3), "returncode": code,
        "exit": payload.get("exit"),
        "child_evals": payload.get("child_evals"),
        "proof": payload.get("proof"),
        "proof_positions": payload.get("proof_positions"),
        "proof_length": payload.get("proof_length"),
    }}
    if payload.get("exit") != "proved":
        return fields, "not-proved"
    expected = {r["child_evals"] for r in runs}
    if len(expected) != 1 or payload.get("child_evals") not in expected:
        return fields, "nondeterministic"
    if payload.get("proof") != "written":
        return fields, "export-failure"
    ours = last_json(
        subprocess.run([args.atomi, "validate", str(path)],
                       capture_output=True, text=True).stdout) or {}
    if not ours.get("valid"):
        return fields, "validator-reject"
    chk = last_json(
        subprocess.run([args.provari_check, "check", str(path)],
                       capture_output=True, text=True).stdout) or {}
    if chk.get("valid") is None:
        return fields, "unchecked"
    if not chk["valid"]:
        return fields, "checker-reject"
    if ours.get("positions") != chk.get("won_positions"):
        return fields, "disagreement"
    return fields, "accepted"


def ledger(case_ids: list[str], ref: dict[str, dict],
           cands: dict[str, dict[str, dict]], seconds: float) -> dict:
    """Per-case factors and the shared-win geometric mean per stack."""
    cells: dict[str, dict] = {}
    for case in case_ids:
        cell: dict = {"reference": ref[case]}
        for name, table in cands.items():
            entry = dict(table[case])
            if not ref[case]["win"]:
                entry["factor"] = None
                entry["note"] = "no reference win (provari hand censored)"
            elif not entry["win"]:
                # Lower bound only: we needed more than the budget.
                entry["factor"] = None
                entry["factor_lower_bound"] = round(
                    seconds / ref[case]["wall_s"], 2)
                entry["note"] = "candidate censored"
            else:
                entry["factor"] = round(
                    entry["wall_s"] / ref[case]["wall_s"], 3)
                entry["note"] = "shared win"
            cell[name] = entry
        cells[case] = cell

    summary = {}
    for name in cands:
        shared = [c for c in case_ids
                  if cells[c][name]["note"] == "shared win"]
        factors = [cells[c][name]["factor"] for c in shared]
        geomean = None
        if factors:
            geomean = round(
                statistics.geometric_mean(factors), 3) if len(factors) > 1 \
                else factors[0]
        summary[name] = {
            "shared_cells": shared,
            "shared_count": len(shared),
            "geomean_factor": geomean,
            "coverage_reference": sum(1 for c in case_ids if ref[c]["win"]),
            "coverage_candidate": sum(1 for c in case_ids
                                      if cells[c][name]["win"]),
            "censored_cells": [c for c in case_ids
                               if not cells[c][name]["win"]],
        }
    return {"cells": cells, "summary": summary}


def stretch_block(case_ids: list[str], ref: dict[str, dict],
                  stretch: dict[str, dict], seconds: float) -> dict:
    """v0.2.0 vs v0.1.0, reported separately — never in the ledger."""
    per_cell = {}
    ratios = []
    for case in case_ids:
        r, s = ref[case], stretch.get(case, {"win": False, "wall_s": None})
        if r["win"] and s.get("win"):
            ratio = round(s["wall_s"] / r["wall_s"], 3)
            ratios.append(ratio)
            per_cell[case] = {
                "reference_wall_s": r["wall_s"], "stretch_wall_s": s["wall_s"],
                "ratio": ratio,
            }
        else:
            per_cell[case] = {
                "reference_wall_s": r["wall_s"],
                "stretch_wall_s": s.get("wall_s"),
                "note": ("stretch censored" if r["win"]
                         else "reference censored"),
            }
    geomean = (round(statistics.geometric_mean(ratios), 3)
               if len(ratios) > 1 else (ratios[0] if ratios else None))
    return {"cells": per_cell, "shared_ratio_geomean": geomean,
            "shared_count": len(ratios)}


def cpu_model() -> str:
    try:
        for line in Path("/proc/cpuinfo").read_text().splitlines():
            if line.startswith("model name"):
                return line.split(":", 1)[1].strip()
    except OSError:
        pass
    return "unknown"


def sha256_file(path: str) -> str:
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def rustc_version() -> str:
    return subprocess.run(["rustc", "-V"], capture_output=True, text=True,
                          check=False).stdout.strip()


def movegen_version() -> str:
    text = Path(REPO / "Cargo.lock").read_text()
    idx = text.find('name = "atomic-movegen"')
    if idx >= 0:
        for line in text[idx:].splitlines():
            if line.startswith("version"):
                return line.split('"')[1]
    return "unknown"


def tool_out(cmd: list[str]) -> str:
    proc = subprocess.run(cmd, capture_output=True, text=True, check=False)
    if proc.returncode != 0 or not proc.stdout.strip():
        return "unknown"
    return proc.stdout.strip().splitlines()[0]


def main() -> int:
    args = parse_args()
    corpus = Path(args.corpus)
    corpus_sha = verify_corpus(corpus)

    atomi_stacks = split_stacks(args.atomi_stack)
    legacy_stacks = split_stacks(args.legacy_stack)
    if args.atomi and not atomi_stacks:
        raise SystemExit("--atomi given without --atomi-stack")
    if args.legacy_solver and not legacy_stacks:
        raise SystemExit("--legacy-solver given without --legacy-stack")
    if args.proof_check and not args.provari_check:
        raise SystemExit("--proof-check needs --provari-check")
    if args.proof_check:
        Path(args.proof_dir).mkdir(parents=True, exist_ok=True)

    head = subprocess.run(
        ["git", "-C", str(REPO), "rev-parse", "HEAD"],
        capture_output=True, text=True, check=False).stdout.strip() or "unknown"
    dirty = bool(subprocess.run(
        ["git", "-C", str(REPO), "status", "--porcelain"],
        capture_output=True, text=True, check=False).stdout.strip())

    cases = load_cases(corpus)
    if args.cases:
        wanted = [name.strip() for name in args.cases.split(",") if name.strip()]
        known = {case["id"] for case in cases}
        missing = [name for name in wanted if name not in known]
        if missing:
            raise SystemExit(f"unknown case ids: {missing}")
        cases = [case for case in cases if case["id"] in wanted]
    case_ids = [case["id"] for case in cases]
    fen_of = {case["id"]: case["fen"] for case in cases}

    ref_side = f"provari-{args.guide}"
    stretch_side = f"provari-v0.2.0-{args.guide}"

    rows: list[dict] = []
    for case in cases:
        # Order per case: reference, stretch, each atomi stack, each legacy
        # stack — sequential, one process at a time.
        rows += reference_row(args.provari, args.guide, ref_side, args, case)
        if args.provari_stretch:
            rows += reference_row(args.provari_stretch, args.guide,
                                  stretch_side, args, case)
        for name, flags in atomi_stacks:
            rows += atomi_rows(args, case, name, flags)
        for name, flags in legacy_stacks:
            rows += legacy_rows(args, case, name, flags)
    if args.dry_run:
        return 0

    ref = medians(rows, ref_side)
    stretch = medians(rows, stretch_side) if args.provari_stretch else {}
    atomi = {name: medians(rows, f"atomi:{name}")
             for name, _ in atomi_stacks}
    legacy = {name: medians(rows, f"atomic_solver:{name}")
              for name, _ in legacy_stacks}

    proof_verdicts: dict[str, dict] = {}
    if args.proof_check:
        for name, table in atomi.items():
            for case in case_ids:
                cell = table.get(case)
                if not cell or not cell["win"]:
                    continue
                group = [r for r in rows
                         if r["side"] == f"atomi:{name}"
                         and r["case"] == case]
                flags = group[0].get("flags", [])
                fields, verdict = proof_check(
                    args, case, name, flags, group, fen_of[case])
                cell.update(fields)
                cell["proof_verdict"] = verdict
                proof_verdicts[f"atomi:{name}"] = {
                    c: atomi[name][c].get("proof_verdict") for c in case_ids}
                if verdict != "accepted":
                    cell["win"] = False
                    cell["note"] = verdict

    cands = {**atomi, **legacy}
    result = {
        "provenance": {
            "date": args.date or datetime.now(timezone.utc).strftime("%Y-%m-%d"),
            "host": platform.node(),
            "platform": platform.platform(),
            "cpu_model": cpu_model(),
            "cpu_count": os.cpu_count(),
            "rustc": rustc_version(),
            "atomic_movegen": movegen_version(),
            "atomi": {
                "version": tool_out([args.atomi, "--version"])
                if args.atomi else None,
                "binary_sha256": sha256_file(args.atomi)
                if args.atomi else None,
                "binary": args.atomi,
            },
            "repo_head": head,
            "repo_dirty": dirty,  # the owner has not committed; honest flag
            "provari": {
                "commit": args.provari_commit,
                "guide": args.guide,
                "version": tool_out([args.provari, "version"]),
                "binary": args.provari,
            },
            "provari_stretch": {
                "commit": args.provari_stretch_commit,
                "guide": args.guide,
                "version": tool_out([args.provari_stretch, "version"])
                if args.provari_stretch else None,
                "binary": args.provari_stretch,
            },
            "legacy": {"commit": args.legacy_commit,
                       "binary": args.legacy_solver},
            "stacks": {
                "atomi": dict(atomi_stacks),
                "legacy": dict(legacy_stacks),
            },
            "budgets": {"seconds": args.seconds, "memory_mb": args.memory,
                        "repeats": args.repeats},
            "corpus": {"path": args.corpus, "sha256": corpus_sha},
            "proof_verdicts": proof_verdicts,
        },
        "runs": rows,
        "ledger": ledger(case_ids, ref, cands, args.seconds),
    }
    if args.provari_stretch:
        result["stretch"] = stretch_block(case_ids, ref, stretch,
                                          args.seconds)
    Path(args.out).write_text(json.dumps(result, indent=1) + "\n")
    print(f"wrote {args.out}")
    for name, summary in result["ledger"]["summary"].items():
        print(f"{name}: shared={summary['shared_count']} "
              f"geomean={summary['geomean_factor']} "
              f"coverage={summary['coverage_candidate']}"
              f"/{summary['coverage_reference']}")
    if args.provari_stretch:
        s = result["stretch"]
        print(f"stretch: shared={s['shared_count']} "
              f"geomean_ratio={s['shared_ratio_geomean']}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
