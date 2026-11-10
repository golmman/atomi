#!/usr/bin/env python3
"""Plan 15 item-6 (M5) driver: Tier 1 confirmation and Tier 2 head-to-head.

Phases
------
env        pin host/toolchain/binary, corpus hash, provari and controls
tier1      --guide none at 120 s on core/easy-win/deep-win, seeds 0-2
tier1_d14  --seconds 600 --proof-out for every proved Tier 1 row; require
           equal child_evals and both checkers
tier2      --guide none at 10 s, 3 repeats per cell; same-session provari
           hand references; legacy default/opt and v0.2.0 controls if the
           pinned binaries exist, otherwise recorded unavailable
tier2_d14  --seconds 600 --proof-out for every Tier 2 row that wins
all        env, tier1, tier1_d14, tier2, tier2_d14

One process at a time. Only parsed JSON-line fields are stored; no raw
transcript, provari artifact, proof file or bytecode is committed.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
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
DEFAULT_V020 = Path("/tmp/atomi-oracle-v020/provari/target/release/provari")
DEFAULT_LEGACY = Path("/tmp/atomic_solver-5f85133/target/release/atomic_solver")
MEMORY = 192
SEEDS = [0, 1, 2]
CELLS = ["core", "easy-win", "deep-win"]
TIER1_SECONDS = 120.0
TIER2_SECONDS = 10.0
D14_SECONDS = 600.0
BARE = ["--guide", "none"]
TIER2_REPEATS = 3


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--phase", choices=["env", "tier1", "tier1_d14", "tier2",
                                       "tier2_d14", "all"], default="all")
    p.add_argument("--atomi", default=str(DEFAULT_ATOMI))
    p.add_argument("--provari", default=str(DEFAULT_PROVARI))
    p.add_argument("--v020", default=str(DEFAULT_V020))
    p.add_argument("--legacy", default=str(DEFAULT_LEGACY))
    p.add_argument("--set", default=str(DEFAULT_SET))
    p.add_argument("--corpus", default=str(DEFAULT_CORPUS))
    p.add_argument("--outdir", default=str(HERE))
    p.add_argument("--provari-repeats", type=int, default=3)
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
        return "dirty (plan15 execution; owner commits)" if out else "clean"
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


def geomean(values: list[float]) -> float | None:
    vals = [v for v in values if v is not None and v > 0]
    if not vals:
        return None
    return round(math.exp(sum(math.log(v) for v in vals) / len(vals)), 3)


def phase_env(args: argparse.Namespace) -> dict:
    atomi = Path(args.atomi)
    set_path = Path(args.set)
    corpus = Path(args.corpus)
    pv = subprocess.run([args.provari, "--version"], capture_output=True,
                        text=True)
    v020 = Path(args.v020)
    legacy = Path(args.legacy)
    env = {
        "plan": "plan15",
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
        "v020": str(v020),
        "v020_available": v020.exists(),
        "legacy": str(legacy),
        "legacy_available": legacy.exists(),
        "product_flags": BARE,
        "tier1_seconds": TIER1_SECONDS,
        "tier2_seconds": TIER2_SECONDS,
        "d14_seconds": D14_SECONDS,
        "memory_mb": MEMORY,
        "seeds": SEEDS,
        "cells": CELLS,
        "tier2_repeats": TIER2_REPEATS,
        "provari_repeats": args.provari_repeats,
    }
    write_json(Path(args.outdir) / "env.json", env)
    return env


def atomi_row(args: argparse.Namespace, cell: dict, seed: int, seconds: float,
              budget: str, proof_out: Path | None = None) -> dict:
    cmd = [args.atomi, "prove", cell["fen"], "--seconds", str(int(seconds)),
           "--memory", str(MEMORY), "--seed", str(seed), *BARE]
    if proof_out is not None:
        cmd += ["--proof-out", str(proof_out)]
    payload, wall, stderr, code = run(cmd, seconds)
    ms = payload.get("ms")
    aux = None
    if payload.get("sim_nodes") is not None and payload.get("settle_nodes") is not None:
        aux = payload["sim_nodes"] + payload["settle_nodes"]
    return {
        "cell": cell["name"], "seed": seed, "budget": budget,
        "flags": BARE, "seconds": seconds,
        "proved": bool(payload.get("found")),
        "exit": payload.get("exit"),
        "length": payload.get("length"),
        "child_evals": payload.get("child_evals"),
        "expansions": payload.get("expansions"),
        "sim_nodes": payload.get("sim_nodes"),
        "settle_nodes": payload.get("settle_nodes"),
        "aux_nodes": aux,
        "proof_full": payload.get("proof_full"),
        "proof": payload.get("proof"),
        "proof_positions": payload.get("proof_positions"),
        "proof_length": payload.get("proof_length"),
        "proof_entries": payload.get("proof_entries"),
        "ms": ms,
        "external_wall_s": wall,
        "returncode": code,
        "stderr": stderr or None,
        "proof_file_exists": bool(proof_out and proof_out.exists()),
        "proof_file_bytes": (proof_out.stat().st_size
                             if proof_out and proof_out.exists() else None),
    }


def provari_row(args: argparse.Namespace, cell: dict, repeat: int,
                binary: str | None = None) -> dict:
    cmd = [binary or args.provari, "prove", cell["fen"], "--seconds",
           str(int(TIER2_SECONDS)), "--memory", str(MEMORY),
           "--guide", "hand"]
    payload, wall, stderr, code = run(cmd, TIER2_SECONDS)
    ms = payload.get("ms")
    nodes = payload.get("nodes")
    return {
        "cell": cell["name"], "repeat": repeat,
        "proved": bool(payload.get("found")),
        "nodes": nodes, "ms": ms, "external_wall_s": wall,
        "nodes_per_s": (round(nodes * 1000 / ms, 1) if ms and nodes else None),
        "ns_per_node": (round(ms * 1e6 / nodes, 1) if ms and nodes else None),
        "returncode": code, "stderr": stderr or None,
    }


def phase_tier1(args: argparse.Namespace) -> dict:
    outdir = Path(args.outdir)
    cells = cells_by_name(args)
    rows = []
    for cell in CELLS:
        for seed in SEEDS:
            row = atomi_row(args, cells[cell], seed, TIER1_SECONDS, "tier1")
            rows.append(row)
            print(f"tier1 {cell:9s} s{seed}: proved={row['proved']} "
                  f"exit={row['exit']} exp={row['expansions']} "
                  f"evals={row['child_evals']} length={row['length']} "
                  f"ms={row['ms']} wall={row['external_wall_s']}", flush=True)
            write_json(outdir / "tier1.json",
                       {"phase": "tier1", "rows": rows})
    data = {
        "phase": "tier1",
        "seconds": TIER1_SECONDS,
        "memory_mb": MEMORY,
        "seeds": SEEDS,
        "cells": CELLS,
        "flags": BARE,
        "rows": rows,
        "coverage": {
            cell: {
                "proved": sum(1 for r in rows
                              if r["cell"] == cell and r["proved"]),
                "runs": sum(1 for r in rows if r["cell"] == cell),
                "all_proved": all(r["proved"] for r in rows
                                  if r["cell"] == cell),
            }
            for cell in CELLS
        },
    }
    data["coverage"]["total_proved"] = sum(
        1 for c in CELLS if data["coverage"][c]["all_proved"])
    data["coverage"]["tier1_pass"] = data["coverage"]["total_proved"] >= 2
    write_json(outdir / "tier1.json", data)
    return data


def check_proof(args: argparse.Namespace, atomi: Path, path: Path) -> dict:
    v, _, _, _ = run([str(atomi), "validate", str(path)], 60)
    c, _, _, _ = run([args.provari, "check", str(path)], 60)
    valid = bool(v.get("valid")) and c.get("valid") is True
    return {
        "atomi_valid": v.get("valid"),
        "atomi_positions": v.get("positions"),
        "provari_valid": c.get("valid"),
        "provari_won_positions": c.get("won_positions"),
        "verdict": "accepted" if valid else "rejected",
    }


def phase_tier1_d14(args: argparse.Namespace) -> dict:
    outdir = Path(args.outdir)
    proof_dir = outdir / "proofs"
    proof_dir.mkdir(exist_ok=True)
    cells = cells_by_name(args)
    data = json.loads((outdir / "tier1.json").read_text())
    d14 = []
    for row in data["rows"]:
        if not row["proved"]:
            continue
        path = proof_dir / f"d14-{row['cell']}-s{row['seed']}.jsonl"
        extra = atomi_row(args, cells[row["cell"]], row["seed"], D14_SECONDS,
                          "d14", path)
        check = None
        if extra["proved"] and extra["proof"] == "written":
            check = check_proof(args, Path(args.atomi), path)
        same = extra["child_evals"] == row["child_evals"]
        d14.append({
            "cell": row["cell"], "seed": row["seed"],
            "timed_child_evals": row["child_evals"],
            "proof_child_evals": extra["child_evals"],
            "same_child_evals": same,
            "proof_positions": extra["proof_positions"],
            "proof_entries": extra["proof_entries"],
            "proof_file_bytes": extra["proof_file_bytes"],
            "check": check,
            "verdict": (check["verdict"] if same and check
                        and check["verdict"] == "accepted"
                        else "rejected"),
        })
        if path.exists():
            path.unlink()
        print(f"tier1_d14 {row['cell']:9s} s{row['seed']}: "
              f"same_evals={same} check={check and check['verdict']}",
              flush=True)
        write_json(outdir / "tier1.json", {**data, "d14": d14})
    if proof_dir.exists() and not any(proof_dir.iterdir()):
        proof_dir.rmdir()
    data["d14"] = d14
    data["d14_all_accepted"] = bool(d14) and all(
        d["verdict"] == "accepted" for d in d14)
    data["tier1_pass"] = (data["coverage"]["tier1_pass"]
                          and data["d14_all_accepted"])
    write_json(outdir / "tier1.json", data)
    return data


def phase_tier2(args: argparse.Namespace) -> dict:
    outdir = Path(args.outdir)
    cells = cells_by_name(args)
    rows = []
    for cell in CELLS:
        for repeat in range(TIER2_REPEATS):
            row = atomi_row(args, cells[cell], 0, TIER2_SECONDS, "tier2")
            row["repeat"] = repeat
            rows.append(row)
            print(f"tier2 {cell:9s} r{repeat}: proved={row['proved']} "
                  f"exit={row['exit']} exp={row['expansions']} "
                  f"evals={row['child_evals']} length={row['length']} "
                  f"ms={row['ms']} wall={row['external_wall_s']}", flush=True)
            write_json(outdir / "tier2.json",
                       {"phase": "tier2", "rows": rows})
    prov = {}
    for cell in CELLS:
        rs = []
        for repeat in range(args.provari_repeats):
            row = provari_row(args, cells[cell], repeat)
            rs.append(row)
            print(f"provari {cell:9s} r{repeat}: proved={row['proved']} "
                  f"nodes={row['nodes']} ms={row['ms']} "
                  f"wall={row['external_wall_s']}", flush=True)
        prov[cell] = {
            "repeats": rs,
            "median_ms": median([r["ms"] for r in rs]),
            "median_nodes": median([r["nodes"] for r in rs]),
            "median_ns_per_node": median([r["ns_per_node"] for r in rs]),
        }
    controls = {}
    v020 = Path(args.v020)
    legacy = Path(args.legacy)
    if v020.exists():
        controls["provari-v0.2.0-hand"] = {
            cell: [provari_row(args, cells[cell], r, str(v020))
                   for r in range(args.provari_repeats)]
            for cell in CELLS
        }
    else:
        controls["provari-v0.2.0-hand"] = "unavailable"
    if legacy.exists():
        controls["legacy"] = "available but not run by this driver"
    else:
        controls["legacy-default"] = "unavailable"
        controls["legacy-opt"] = "unavailable"
    factors = {}
    for cell in CELLS:
        ref = prov[cell]["median_ms"]
        rs = [r for r in rows if r["cell"] == cell]
        wall = median([r["ms"] for r in rs])
        proved_all = bool(rs) and all(r["proved"] for r in rs)
        factors[cell] = {
            "atomi_ms": wall,
            "provari_ms": ref,
            "factor": (round(wall / ref, 3) if proved_all and wall and ref
                       else None),
            "proved_all": proved_all,
            "censored": not proved_all,
            "lower_bound": (round(TIER2_SECONDS * 1000 / ref, 3)
                            if not proved_all and ref else None),
        }
    shared = [c for c in CELLS if factors[c]["proved_all"]]
    data = {
        "phase": "tier2",
        "seconds": TIER2_SECONDS,
        "memory_mb": MEMORY,
        "seed": 0,
        "repeats": TIER2_REPEATS,
        "cells": CELLS,
        "flags": BARE,
        "rows": rows,
        "provari": prov,
        "controls": controls,
        "factors": factors,
        "shared_cells": shared,
        "coverage": f"{len(shared)}/3",
        "geomean_shared": geomean([factors[c]["factor"] for c in shared]),
        "tier2_pass": (len(shared) == 3
                       and geomean([factors[c]["factor"] for c in shared]) is not None
                       and geomean([factors[c]["factor"] for c in shared]) < 1.0),
    }
    write_json(outdir / "tier2.json", data)
    return data


def phase_tier2_d14(args: argparse.Namespace) -> dict:
    outdir = Path(args.outdir)
    proof_dir = outdir / "proofs"
    proof_dir.mkdir(exist_ok=True)
    cells = cells_by_name(args)
    data = json.loads((outdir / "tier2.json").read_text())
    d14 = []
    for row in data["rows"]:
        if not row["proved"]:
            continue
        path = proof_dir / f"d14-tier2-{row['cell']}-r{row['repeat']}.jsonl"
        extra = atomi_row(args, cells[row["cell"]], row["seed"], D14_SECONDS,
                          "d14", path)
        check = None
        if extra["proved"] and extra["proof"] == "written":
            check = check_proof(args, Path(args.atomi), path)
        same = extra["child_evals"] == row["child_evals"]
        d14.append({
            "cell": row["cell"], "repeat": row["repeat"], "seed": row["seed"],
            "timed_child_evals": row["child_evals"],
            "proof_child_evals": extra["child_evals"],
            "same_child_evals": same,
            "proof_positions": extra["proof_positions"],
            "proof_entries": extra["proof_entries"],
            "proof_file_bytes": extra["proof_file_bytes"],
            "check": check,
            "verdict": (check["verdict"] if same and check
                        and check["verdict"] == "accepted"
                        else "rejected"),
        })
        if path.exists():
            path.unlink()
        print(f"tier2_d14 {row['cell']:9s} r{row['repeat']}: "
              f"same_evals={same} check={check and check['verdict']}",
              flush=True)
        write_json(outdir / "tier2.json", {**data, "d14": d14})
    if proof_dir.exists() and not any(proof_dir.iterdir()):
        proof_dir.rmdir()
    data["d14"] = d14
    data["d14_all_accepted"] = bool(d14) and all(
        d["verdict"] == "accepted" for d in d14)
    write_json(outdir / "tier2.json", data)
    return data


def main() -> None:
    args = parse_args()
    outdir = Path(args.outdir)
    outdir.mkdir(parents=True, exist_ok=True)
    if args.phase in ("env", "all"):
        phase_env(args)
    if args.phase in ("tier1", "all"):
        phase_tier1(args)
    if args.phase in ("tier1_d14", "all"):
        phase_tier1_d14(args)
    if args.phase in ("tier2", "all"):
        phase_tier2(args)
    if args.phase in ("tier2_d14", "all"):
        phase_tier2_d14(args)


if __name__ == "__main__":
    main()
