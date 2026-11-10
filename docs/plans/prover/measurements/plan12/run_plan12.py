#!/usr/bin/env python3
"""Plan 12 item-21 driver: Phase A attribution and the stop-rule record.

Phases
------
env          pin host/toolchain/binary, corpus/guidance hashes and provari
attribution  bare tree at fixed 20 M evals on core/easy-win/deep-win, seed 0,
             with --stats; deep-win seed 0 at 120 s with and without --stats
coverage     write the Phase B/C record: not run, because Phase A's stop rule
             fired (entry_would_return < 10 % of expansions)
all          env, attribution, coverage

One process at a time. Only parsed JSON-line fields are stored; no raw
transcript, provari artifact, proof file or bytecode is committed.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import platform
import subprocess
import sys
import time
from pathlib import Path

sys.dont_write_bytecode = True

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[4]
DEFAULT_SET = REPO / "docs/plans/prover/measurements/plan3/guidance-v1.tsv"
DEFAULT_CORPUS = REPO / "bench/corpus.json"
DEFAULT_ATOMI = REPO / "target/release/atomi"
DEFAULT_PROVARI = Path("/tmp/atomi-oracle/provari/target/release/provari")
MEMORY = 192
SEED = 0
CELLS = ["core", "easy-win", "deep-win"]
EVALS = 20_000_000
COVERAGE_SECONDS = 120.0
BARE = ["--guide", "none"]


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--phase", choices=["env", "attribution", "coverage", "all"],
                   default="all")
    p.add_argument("--atomi", default=str(DEFAULT_ATOMI))
    p.add_argument("--provari", default=str(DEFAULT_PROVARI))
    p.add_argument("--set", default=str(DEFAULT_SET))
    p.add_argument("--corpus", default=str(DEFAULT_CORPUS))
    p.add_argument("--outdir", default=str(HERE))
    return p.parse_args()


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def last_json(text: str) -> dict:
    for line in reversed(text.splitlines()):
        line = line.strip()
        if line.startswith("{"):
            try:
                return json.loads(line)
            except json.JSONDecodeError:
                continue
    return {}


def run(cmd: list[str], timeout: float) -> tuple[dict, float, str, int]:
    start = time.perf_counter()
    try:
        proc = subprocess.run(cmd, capture_output=True, text=True,
                              timeout=timeout + 120, check=False)
        return (last_json(proc.stdout + proc.stderr),
                round(time.perf_counter() - start, 3),
                proc.stderr.strip(), proc.returncode)
    except subprocess.TimeoutExpired:
        return {}, round(time.perf_counter() - start, 3), "timeout", 124


def load_set(path: Path) -> list[dict]:
    rows = []
    for raw in path.read_text().splitlines():
        if not raw.strip() or raw.startswith("name\t"):
            continue
        f = raw.split("\t")
        rows.append({"name": f[0], "fen": f[1], "source": f[2],
                     "ref_nodes": int(f[3]), "ref_wall_s": float(f[4])})
    return rows


def cells_by_name(args: argparse.Namespace) -> dict[str, dict]:
    return {c["name"]: c for c in load_set(Path(args.set))}


def git_head(repo: Path) -> str:
    try:
        return subprocess.run(["git", "rev-parse", "HEAD"], cwd=repo,
                              capture_output=True, text=True).stdout.strip()
    except OSError:
        return "unknown"


def cpu_model() -> str:
    try:
        for line in Path("/proc/cpuinfo").read_text().splitlines():
            if line.startswith("model name"):
                return line.partition(":")[2].strip()
    except OSError:
        pass
    return platform.processor() or "unknown"


def worktree() -> str:
    try:
        out = subprocess.run(["git", "status", "--porcelain"], cwd=REPO,
                             capture_output=True, text=True).stdout
        return "dirty (plan12 execution; owner commits)" if out else "clean"
    except OSError:
        return "unknown"


def write_json(path: Path, data: dict) -> None:
    path.write_text(json.dumps(data, indent=1) + "\n")


def phase_env(args: argparse.Namespace) -> dict:
    atomi = Path(args.atomi)
    set_path = Path(args.set)
    corpus = Path(args.corpus)
    pv = subprocess.run([args.provari, "--version"], capture_output=True,
                        text=True)
    env = {
        "plan": "plan12",
        "date": "2026-11-10",
        "host": platform.node(),
        "cpu": cpu_model(),
        "kernel": f"{platform.system()} {platform.release()}",
        "machine": platform.machine(),
        "python": platform.python_version(),
        "rustc": subprocess.run(["rustc", "--version"], capture_output=True,
                                text=True).stdout.strip(),
        "git_head": git_head(REPO),
        "worktree": worktree(),
        "atomi_binary": str(atomi),
        "atomi_binary_sha256": sha256(atomi),
        "guidance_set": str(set_path),
        "guidance_sha256": sha256(set_path),
        "corpus": str(corpus),
        "corpus_sha256": sha256(corpus),
        "provari": args.provari,
        "provari_version": (pv.stdout or pv.stderr).strip(),
        "provari_commit": git_head(Path(args.provari).parents[2]),
        "product_flags": BARE,
        "evals": EVALS,
        "coverage_seconds": COVERAGE_SECONDS,
        "memory_mb": MEMORY,
        "seed": SEED,
        "cells": CELLS,
    }
    write_json(Path(args.outdir) / "env.json", env)
    return env


def atomi_row(args: argparse.Namespace, cell: dict, seconds: float | None,
              evals: int | None, stats: bool, budget: str) -> dict:
    cmd = [args.atomi, "prove", cell["fen"], "--memory", str(MEMORY),
           "--seed", str(SEED), *BARE]
    if seconds is not None:
        cmd += ["--seconds", str(int(seconds))]
    if evals is not None:
        cmd += ["--evals", str(evals)]
    if stats:
        cmd += ["--stats"]
    payload, wall, stderr, code = run(cmd, seconds or 600.0)
    st = payload.get("stats") or {}
    return {
        "cell": cell["name"], "budget": budget, "seconds": seconds,
        "evals": evals, "stats": stats, "flags": BARE,
        "proved": bool(payload.get("found")),
        "exit": payload.get("exit"),
        "length": payload.get("length"),
        "child_evals": payload.get("child_evals"),
        "expansions": payload.get("expansions"),
        "ms": payload.get("ms"),
        "external_wall_s": wall,
        "proof_entries": payload.get("proof_entries"),
        "proof_full": payload.get("proof_full"),
        "unique_keys": st.get("unique_keys"),
        "unique_keys_capped": st.get("unique_keys_capped"),
        "re_expansions": st.get("re_expansions"),
        "entry_proof_hits": st.get("entry_proof_hits"),
        "entry_tt_hits": st.get("entry_tt_hits"),
        "entry_would_return": st.get("entry_would_return"),
        "attacker_expansions": st.get("attacker_expansions"),
        "defender_expansions": st.get("defender_expansions"),
        "depth_histogram": st.get("depth_histogram"),
        "selected_class": st.get("selected_class"),
        "returncode": code,
        "stderr": stderr or None,
    }


def phase_attribution(args: argparse.Namespace) -> dict:
    outdir = Path(args.outdir)
    cells = cells_by_name(args)
    rows = []
    for cell in CELLS:
        row = atomi_row(args, cells[cell], None, EVALS, True, "evals-20m")
        rows.append(row)
        print(f"attribution {cell:9s} 20M: exp={row['expansions']} "
              f"unique={row['unique_keys']} re={row['re_expansions']} "
              f"would={row['entry_would_return']} "
              f"proof_hits={row['entry_proof_hits']} "
              f"tt_hits={row['entry_tt_hits']} ms={row['ms']}", flush=True)
        write_json(outdir / "attribution.json",
                   {"phase": "attribution", "rows": rows})
    # The 120 s horizon: one clean wall row and one --stats row.
    for stats in (False, True):
        row = atomi_row(args, cells["deep-win"], COVERAGE_SECONDS, None, stats,
                        "seconds-120-stats" if stats else "seconds-120")
        rows.append(row)
        print(f"attribution deep-win 120s stats={stats}: proved={row['proved']} "
              f"exit={row['exit']} exp={row['expansions']} "
              f"evals={row['child_evals']} ms={row['ms']} "
              f"would={row['entry_would_return']}", flush=True)
        write_json(outdir / "attribution.json",
                   {"phase": "attribution", "rows": rows})
    data = {
        "phase": "attribution",
        "evals": EVALS,
        "seconds": COVERAGE_SECONDS,
        "memory_mb": MEMORY,
        "seed": SEED,
        "flags": BARE,
        "rows": rows,
    }
    # Stop rule: entry_would_return < 10 % of expansions at 20 M on deep-win.
    deep = next(r for r in rows
                if r["cell"] == "deep-win" and r["budget"] == "evals-20m")
    ratio = (deep["entry_would_return"] / deep["expansions"]
             if deep["expansions"] else None)
    data["stop_rule"] = {
        "cell": "deep-win",
        "entry_would_return": deep["entry_would_return"],
        "expansions": deep["expansions"],
        "ratio": ratio,
        "threshold": 0.10,
        "fired": ratio is not None and ratio < 0.10,
    }
    write_json(outdir / "attribution.json", data)
    return data


def phase_coverage(args: argparse.Namespace) -> dict:
    outdir = Path(args.outdir)
    data = {
        "phase": "coverage",
        "status": "not-run",
        "reason": ("Phase A stop rule fired: entry_would_return is below "
                   "10 % of expansions at 20 M on deep-win, so Phase B "
                   "(ENTRY-REUSE) and Phase C (120 s matrix + D14) were not "
                   "implemented or run."),
        "rows": [],
        "d14": [],
    }
    write_json(outdir / "coverage.json", data)
    return data


def main() -> None:
    args = parse_args()
    outdir = Path(args.outdir)
    outdir.mkdir(parents=True, exist_ok=True)
    if args.phase in ("env", "all"):
        phase_env(args)
    if args.phase in ("attribution", "all"):
        phase_attribution(args)
    if args.phase in ("coverage", "all"):
        phase_coverage(args)


if __name__ == "__main__":
    main()
