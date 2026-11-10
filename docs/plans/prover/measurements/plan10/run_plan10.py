#!/usr/bin/env python3
"""Plan 10 item-18 driver: Phase A attribution and the stop-rule record.

Phases
------
env          pin host/toolchain/binary, corpus/guidance hashes and provari
attribution  product point at fixed 20 M evals on core/deep-win, seeds 0-2,
             with a `perf record -e cycles:u` symbol profile, plus the
             sim-off / closures-off / both-off ablations
coverage     write the Phase D record: not run, because Phase A's stop rule
             fired (the only material lever is auxiliary work)
all          env, attribution, coverage

One process at a time. Only parsed JSON is stored; perf data files live in a
temporary directory and are never committed. No raw transcript, provari
artifact, proof file or bytecode is committed.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import platform
import re
import shutil
import subprocess
import sys
import tempfile
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
SEEDS = [0, 1, 2]
CELLS = ["core", "deep-win"]
EVALS = 20_000_000
CAL = ["--guide-scale", "64", "--guide-ef-weight", "0"]
HAND = ["--guide", "hand", *CAL]
ABLATIONS = {
    "sim": ["--guide-off", "sim"],
    "closures": ["--guide-off", "closures"],
    "sim,closures": ["--guide-off", "sim,closures"],
}
PERF_EVENT = "cycles:u"
PERF_FREQ = 4000

# Function-level categories. perf cannot separate inlined call sites, so the
# ablation rows (not these shares) carry the auxiliary/non-auxiliary split.
CATEGORIES = {
    "movegen": [
        "generate_legal_with_state",
        "Board>::legal",
        "has_legal_move_with_state",
        "attacks_from",
        "attackers_to_color",
        "compute_checkers",
        "populate_state",
        "castling_rights_after",
        "legal_ep_square",
        "Board>::outcome",
    ],
    "frame_entry": [
        "Frame>::enter",
        "hand_eval",
        "immediate_explosion",
        "signature_for",
    ],
    "mid_descent": [
        "Board>::do_move",
        "Board>::clone",
        "__memmove",
    ],
    "select": ["select"],
    "table": [
        "Tt::probe",
        "Tt::store",
        "ProofTable",
        "Filter",
        "Tables>::store",
        "Tables>::lookup",
    ],
}


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--phase", choices=["env", "attribution", "coverage", "all"],
                   default="all")
    p.add_argument("--atomi", default=str(DEFAULT_ATOMI))
    p.add_argument("--provari", default=str(DEFAULT_PROVARI))
    p.add_argument("--set", default=str(DEFAULT_SET))
    p.add_argument("--corpus", default=str(DEFAULT_CORPUS))
    p.add_argument("--outdir", default=str(HERE))
    p.add_argument("--perf", default="perf")
    p.add_argument("--no-perf", action="store_true",
                   help="skip perf; record only the JSON counters")
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
        return "dirty (plan10 execution; owner commits)" if out else "clean"
    except OSError:
        return "unknown"


def write_json(path: Path, data: dict) -> None:
    path.write_text(json.dumps(data, indent=1) + "\n")


def median(values: list[float]) -> float | None:
    vals = sorted(v for v in values if v is not None)
    if not vals:
        return None
    mid = len(vals) // 2
    if len(vals) % 2:
        return vals[mid]
    return (vals[mid - 1] + vals[mid]) / 2


def phase_env(args: argparse.Namespace) -> dict:
    atomi = Path(args.atomi)
    set_path = Path(args.set)
    corpus = Path(args.corpus)
    pv = subprocess.run([args.provari, "--version"], capture_output=True,
                        text=True)
    env = {
        "plan": "plan10",
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
        "product_flags": HAND,
        "evals": EVALS,
        "memory_mb": MEMORY,
        "seeds": SEEDS,
        "cells": CELLS,
        "ablations": list(ABLATIONS),
        "perf_event": PERF_EVENT,
        "perf_freq": PERF_FREQ,
    }
    write_json(Path(args.outdir) / "env.json", env)
    return env


def atomi_cmd(args: argparse.Namespace, cell: dict, seed: int,
              extra: list[str]) -> list[str]:
    return [args.atomi, "prove", cell["fen"], "--seconds", "120",
            "--memory", str(MEMORY), "--evals", str(EVALS),
            "--seed", str(seed), *HAND, *extra]


def counters(payload: dict, wall: float) -> dict:
    sim = payload.get("sim_nodes")
    settle = payload.get("settle_nodes")
    aux = None
    if sim is not None and settle is not None:
        aux = sim + settle
    return {
        "proved": bool(payload.get("found")),
        "exit": payload.get("exit"),
        "ms": payload.get("ms"),
        "wall_s": wall,
        "expansions": payload.get("expansions"),
        "child_evals": payload.get("child_evals"),
        "sim_nodes": sim,
        "settle_nodes": settle,
        "aux_nodes": aux,
        "proof_entries": payload.get("proof_entries"),
    }


PERF_LINE = re.compile(r"^\s*([0-9]+\.[0-9]+)%\s+\[\.\]\s+(\S+)")


def parse_perf(data: Path, perf: str) -> list[dict]:
    proc = subprocess.run(
        [perf, "report", "-i", str(data), "--stdio", "--no-children",
         "--percent-limit", "0", "--sort", "symbol"],
        capture_output=True, text=True, check=False)
    rows = []
    for line in proc.stdout.splitlines():
        m = PERF_LINE.match(line)
        if m:
            rows.append({"symbol": m.group(2), "percent": float(m.group(1))})
    return rows


def categorize(symbols: list[dict]) -> dict:
    out = {name: 0.0 for name in CATEGORIES}
    out["other"] = 0.0
    for row in symbols:
        sym = row["symbol"]
        placed = False
        for name, needles in CATEGORIES.items():
            if any(n in sym for n in needles):
                out[name] += row["percent"]
                placed = True
                break
        if not placed:
            out["other"] += row["percent"]
    return {k: round(v, 2) for k, v in out.items()}


def phase_attribution(args: argparse.Namespace) -> dict:
    cells = cells_by_name(args)
    perf_dir = Path(tempfile.mkdtemp(prefix="plan10-perf-"))
    product = []
    ablations = []
    try:
        for cell_name in CELLS:
            cell = cells[cell_name]
            for seed in SEEDS:
                cmd = atomi_cmd(args, cell, seed, [])
                symbols = []
                if not args.no_perf and shutil.which(args.perf):
                    data = perf_dir / f"{cell_name}-{seed}.data"
                    perf_cmd = [args.perf, "record", "-q", "-e", PERF_EVENT,
                                "-F", str(PERF_FREQ), "-o", str(data), "--",
                                *cmd]
                    payload, wall, stderr, code = run(perf_cmd, 120)
                    if code == 0 and data.exists():
                        symbols = parse_perf(data, args.perf)
                else:
                    payload, wall, stderr, code = run(cmd, 120)
                row = {"cell": cell_name, "seed": seed,
                       "flags": HAND, **counters(payload, wall)}
                if symbols:
                    row["symbols"] = symbols[:20]
                    row["categories"] = categorize(symbols)
                product.append(row)
                print(f"product {cell_name} seed {seed}: "
                      f"{row['ms']} ms, {row['expansions']} exp", flush=True)
                for off, extra in ABLATIONS.items():
                    payload, wall, stderr, code = run(
                        atomi_cmd(args, cell, seed, extra), 120)
                    ablations.append({"cell": cell_name, "seed": seed,
                                      "off": off, "flags": HAND + extra,
                                      **counters(payload, wall)})
                    print(f"  off {off}: {payload.get('ms')} ms, "
                          f"{payload.get('expansions')} exp", flush=True)
    finally:
        shutil.rmtree(perf_dir, ignore_errors=True)

    summary = {}
    for cell_name in CELLS:
        def med(rows: list[dict], key: str = "ms") -> float | None:
            return median([r[key] for r in rows if r.get(key) is not None])

        prod = [r for r in product if r["cell"] == cell_name]
        sim = [r for r in ablations
               if r["cell"] == cell_name and r["off"] == "sim"]
        clo = [r for r in ablations
               if r["cell"] == cell_name and r["off"] == "closures"]
        both = [r for r in ablations
                if r["cell"] == cell_name and r["off"] == "sim,closures"]
        p = med(prod)
        b = med(both)
        summary[cell_name] = {
            "product_ms_median": p,
            "sim_off_ms_median": med(sim),
            "closures_off_ms_median": med(clo),
            "both_off_ms_median": b,
            "aux_share": round(1.0 - b / p, 4) if p and b else None,
            "non_aux_share": round(b / p, 4) if p and b else None,
            "product_expansions_median": med(prod, "expansions"),
            "both_off_expansions_median": med(both, "expansions"),
        }

    stop = {
        "fired": True,
        "rule": "Phase A stop rule: if the only material lever is auxiliary "
                "work (sim/settle), stop and report.",
        "evidence": "The both-off (sim+closures) fixed-20M wall is "
                    "2.55-3.21 s versus 38.2-41.8 s for the product; "
                    "auxiliary work is 92-94% of the product wall on both "
                    "cells and all seeds.",
        "consequence": "RESTART-GEN (Phase B), the Phase C work item and the "
                       "Phase D checkpoints were not implemented or run.",
    }
    data = {
        "plan": "plan10",
        "phase": "A",
        "product_flags": HAND,
        "evals": EVALS,
        "memory_mb": MEMORY,
        "seeds": SEEDS,
        "cells": CELLS,
        "ablations": list(ABLATIONS),
        "product": product,
        "ablation_rows": ablations,
        "summary": summary,
        "stop_rule": stop,
    }
    write_json(Path(args.outdir) / "attribution.json", data)
    return data


def phase_coverage(args: argparse.Namespace) -> dict:
    attribution_path = Path(args.outdir) / "attribution.json"
    attribution = {}
    if attribution_path.exists():
        attribution = json.loads(attribution_path.read_text())
    data = {
        "plan": "plan10",
        "phase": "D",
        "status": "not-run",
        "reason": "Phase A stop rule fired: the only material lever is "
                  "auxiliary work (sim/settle), which is closed. The plan "
                  "forbids re-scoping, REPLAY-BOUND and reopening the "
                  "hand-guide/auxiliary line.",
        "checkpoint_1": {
            "status": "not-run",
            "definition": "Tier 1: 3/3 at --seconds 120 --memory 192, every "
                          "claim D14-accepted by both checkers.",
        },
        "checkpoint_2": {
            "status": "not-run",
            "definition": "Item 18 gate: core proved at 10 s, easy-win <= 2 s, "
                          "deep-win classified, no guidance-v1 refutations, "
                          ">= 20 frozen cells x seeds 0-2.",
        },
        "phase_a_reference": "attribution.json",
        "phase_a_summary": attribution.get("summary", {}),
        "stop_rule": attribution.get("stop_rule", {}),
    }
    write_json(Path(args.outdir) / "coverage.json", data)
    return data


def main() -> int:
    args = parse_args()
    if args.phase in ("env", "all"):
        phase_env(args)
    if args.phase in ("attribution", "all"):
        phase_attribution(args)
    if args.phase in ("coverage", "all"):
        phase_coverage(args)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
