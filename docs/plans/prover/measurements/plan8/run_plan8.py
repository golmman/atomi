#!/usr/bin/env python3
"""Plan 8 item-17 driver: comparable per-expansion diagnosis + wall bracket.

Phases
------
env        pin host/toolchain/binary, corpus/guidance hashes and provari
pernode    phase A: provari hand repeats and the bare/neither/no-sim/hand
           stacks on easy-win/core, seeds 0-2, fixed 10 M evals and 10 s
longbudget phase B: product point at 60 s on core/deep-win/easy-win and
           --guide none on core, seeds 0-2, D14 checks for claimed wins
all        env, pernode, longbudget

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
SEEDS = [0, 1, 2]
FIXED_EVALS = 10_000_000
CANON_SECONDS = 10.0
LONG_SECONDS = 60.0
CAL = ["--guide-scale", "64", "--guide-ef-weight", "0"]
HAND = ["--guide", "hand", *CAL]
STACKS = {
    "bare": ["--guide", "none"],
    "neither": HAND + ["--guide-off", "sim,closures"],
    "no-sim": HAND + ["--guide-off", "sim"],
    "hand": HAND,
}
A_CELLS = ["easy-win", "core"]
B_CELLS = ["core", "deep-win", "easy-win"]


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--phase", choices=["env", "pernode", "longbudget", "provari-b",
                                       "d14", "all"],
                   default="all")
    p.add_argument("--atomi", default=str(DEFAULT_ATOMI))
    p.add_argument("--provari", default=str(DEFAULT_PROVARI))
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
        return "dirty (plan8 execution; owner commits)" if out else "clean"
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
        "plan": "plan8",
        "date": "2026-11-09",
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
        "stacks": STACKS,
        "fixed_evals": FIXED_EVALS,
        "canonical_seconds": CANON_SECONDS,
        "long_seconds": LONG_SECONDS,
        "memory_mb": MEMORY,
        "seeds": SEEDS,
        "provari_repeats": args.provari_repeats,
    }
    write_json(Path(args.outdir) / "env.json", env)
    return env


def atomi_row(args: argparse.Namespace, cell: dict, seed: int, flags: list[str],
              seconds: float, evals: int | None, stack: str, budget: str,
              proof_out: Path | None = None) -> dict:
    cmd = [args.atomi, "prove", cell["fen"], "--seconds", str(int(seconds)),
           "--memory", str(MEMORY), "--seed", str(seed), *flags]
    if evals is not None:
        cmd += ["--evals", str(evals)]
    if proof_out is not None:
        cmd += ["--proof-out", str(proof_out)]
    payload, wall, stderr, code = run(cmd, seconds)
    ms = payload.get("ms")
    evals_done = payload.get("child_evals")
    exps = payload.get("expansions")
    aux = None
    if payload.get("sim_nodes") is not None and payload.get("settle_nodes") is not None:
        aux = payload["sim_nodes"] + payload["settle_nodes"]
    return {
        "cell": cell["name"], "seed": seed, "stack": stack, "budget": budget,
        "flags": flags, "seconds": seconds, "evals_limit": evals,
        "proved": bool(payload.get("found")),
        "exit": payload.get("exit"),
        "length": payload.get("length"),
        "child_evals": evals_done,
        "expansions": exps,
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
        "ns_per_exp": (round(ms * 1e6 / exps, 1)
                       if ms and exps else None),
        "exp_per_s": (round(exps * 1000 / ms, 1) if ms and exps else None),
        "children_per_exp": (round(evals_done / exps, 3)
                             if evals_done is not None and exps else None),
        "aux_share": (round(aux / (evals_done + aux), 4)
                      if aux is not None and evals_done is not None
                      and (evals_done + aux) else None),
        "proof_file_exists": bool(proof_out and proof_out.exists()),
        "proof_file_bytes": (proof_out.stat().st_size
                             if proof_out and proof_out.exists() else None),
    }


def provari_row(args: argparse.Namespace, cell: dict, repeat: int) -> dict:
    cmd = [args.provari, "prove", cell["fen"], "--seconds",
           str(int(CANON_SECONDS)), "--memory", str(MEMORY),
           "--guide", "hand"]
    payload, wall, stderr, code = run(cmd, CANON_SECONDS)
    ms = payload.get("ms")
    nodes = payload.get("nodes")
    return {
        "cell": cell["name"], "repeat": repeat, "proved": bool(payload.get("found")),
        "nodes": nodes, "ms": ms, "external_wall_s": wall,
        "nodes_per_s": (round(nodes * 1000 / ms, 1) if ms and nodes else None),
        "ns_per_node": (round(ms * 1e6 / nodes, 1) if ms and nodes else None),
        "returncode": code, "stderr": stderr or None,
    }


def phase_pernode(args: argparse.Namespace) -> dict:
    outdir = Path(args.outdir)
    cells = cells_by_name(args)
    provari_rows = []
    for cell in A_CELLS:
        for repeat in range(args.provari_repeats):
            row = provari_row(args, cells[cell], repeat)
            provari_rows.append(row)
            print(f"provari {cell:9s} r{repeat}: proved={row['proved']} "
                  f"nodes={row['nodes']} ms={row['ms']} wall={row['external_wall_s']}",
                  flush=True)
            write_json(outdir / "pernode.json",
                       {"phase": "pernode", "provari": provari_rows})
    ref = {}
    for cell in A_CELLS:
        rs = [r for r in provari_rows if r["cell"] == cell and r["proved"]]
        ref[cell] = {
            "repeats": rs,
            "median_ms": median([r["ms"] for r in rs]),
            "median_nodes": median([r["nodes"] for r in rs]),
            "median_ns_per_node": median([r["ns_per_node"] for r in rs]),
        }
    rows = []
    for stack, flags in STACKS.items():
        for cell in A_CELLS:
            for seed in SEEDS:
                for budget, seconds, evals in (
                        ("fixed", 600.0, FIXED_EVALS),
                        ("canonical", CANON_SECONDS, None)):
                    row = atomi_row(args, cells[cell], seed, flags, seconds,
                                    evals, stack, budget)
                    rows.append(row)
                    print(f"{stack:8s} {cell:9s} s{seed} {budget:9s}: "
                          f"proved={row['proved']} exit={row['exit']} "
                          f"exp={row['expansions']} evals={row['child_evals']} "
                          f"aux={row['aux_nodes']} ms={row['ms']} "
                          f"ns/exp={row['ns_per_exp']} wall={row['external_wall_s']}",
                          flush=True)
                    write_json(outdir / "pernode.json",
                               {"phase": "pernode", "provari": provari_rows,
                                "atomi": rows})
    ratios = []
    for stack in STACKS:
        for cell in A_CELLS:
            rs = [r for r in rows if r["stack"] == stack
                  and r["cell"] == cell and r["budget"] == "fixed"]
            if not rs or not ref[cell]["median_ms"]:
                continue
            atomi_ms = median([r["ms"] for r in rs])
            atomi_exp = median([r["expansions"] for r in rs])
            ref_ns = ref[cell]["median_ns_per_node"]
            ns_exp = (round(atomi_ms * 1e6 / atomi_exp, 1)
                      if atomi_ms and atomi_exp else None)
            ratios.append({
                "stack": stack, "cell": cell, "budget": "fixed",
                "atomi_median_ms": atomi_ms,
                "atomi_median_expansions": atomi_exp,
                "atomi_ns_per_exp": ns_exp,
                "provari_median_ms": ref[cell]["median_ms"],
                "provari_median_nodes": ref[cell]["median_nodes"],
                "provari_ns_per_node": ref_ns,
                "R": (round(ns_exp / ref_ns, 3)
                      if ns_exp and ref_ns else None),
                "children_per_exp": median([r["children_per_exp"] for r in rs
                                            if r["children_per_exp"] is not None]),
                "ns_per_1m_child_evals": ns_per_1m_child_evals(rs, atomi_ms),
            })
    neither_bad = [r for r in ratios if r["stack"] == "neither"
                   and r["R"] is not None and r["R"] > 2]
    bare_easy = [r for r in ratios if r["stack"] == "bare"
                 and r["cell"] == "easy-win"]
    canon_bare_easy = [r for r in rows if r["stack"] == "bare"
                       and r["cell"] == "easy-win" and r["budget"] == "canonical"]
    zero_floor = None
    if ref["easy-win"]["median_ms"] and canon_bare_easy:
        wall = median([r["ms"] for r in canon_bare_easy])
        if wall:
            zero_floor = {
                "median_ms": wall,
                "proved": all(r["proved"] for r in canon_bare_easy),
                "factor_vs_provari": round(wall / ref["easy-win"]["median_ms"], 3),
            }
    data = {
        "phase": "pernode",
        "stacks": STACKS,
        "fixed_evals": FIXED_EVALS,
        "canonical_seconds": CANON_SECONDS,
        "memory_mb": MEMORY,
        "seeds": SEEDS,
        "provari": ref,
        "provari_rows": provari_rows,
        "atomi": rows,
        "ratios": ratios,
        "decision": {
            "rule": "R > 2 on easy-win or core for neither => core throughput in item 18",
            "neither_over_2": [{"cell": r["cell"], "R": r["R"]}
                               for r in neither_bad],
            "core_is_first_order": bool(neither_bad),
            "bare_easy_win_fixed": bare_easy,
            "zero_guide_floor": zero_floor,
        },
    }
    write_json(outdir / "pernode.json", data)
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


def ns_per_1m_child_evals(rows: list[dict], ms: float | None) -> float | None:
    evals = median([r["child_evals"] for r in rows
                    if r["child_evals"] is not None])
    if not ms or not evals:
        return None
    return round(ms * 1000 / (evals / 1e6), 1)


def phase_longbudget(args: argparse.Namespace) -> dict:
    outdir = Path(args.outdir)
    proof_dir = outdir / "proofs"
    proof_dir.mkdir(exist_ok=True)
    cells = cells_by_name(args)
    rows = []
    configs = [("hand", HAND, ["core", "deep-win", "easy-win"]),
               ("bare", ["--guide", "none"], ["core"])]
    for stack, flags, targets in configs:
        for cell in targets:
            seeds = [0] if (cell == "easy-win") else SEEDS
            for seed in seeds:
                row = atomi_row(args, cells[cell], seed, flags, LONG_SECONDS,
                                None, stack, "long")
                rows.append(row)
                print(f"long {stack:5s} {cell:9s} s{seed}: "
                      f"proved={row['proved']} exit={row['exit']} "
                      f"exp={row['expansions']} evals={row['child_evals']} "
                      f"length={row['length']} ms={row['ms']} "
                      f"wall={row['external_wall_s']}", flush=True)
                write_json(outdir / "longbudget.json",
                           {"phase": "longbudget", "rows": rows})
    # D14 checks for every proved long-budget row, product and bare floor.
    d14 = []
    for row in rows:
        if not row["proved"]:
            continue
        flags = STACKS.get(row["stack"], HAND)
        path = proof_dir / f"d14-{row['stack']}-{row['cell']}-s{row['seed']}.jsonl"
        extra = atomi_row(args, cells[row["cell"]], row["seed"], flags, 600.0,
                          None, row["stack"], "d14", path)
        check = None
        if extra["proved"] and extra["proof"] == "written":
            check = check_proof(args, Path(args.atomi), path)
        same = extra["child_evals"] == row["child_evals"]
        d14.append({
            "cell": row["cell"], "seed": row["seed"], "stack": row["stack"],
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
        print(f"d14 {row['stack']:5s} {row['cell']:9s} s{row['seed']}: "
              f"same_evals={same} check={check and check['verdict']}", flush=True)
        write_json(outdir / "longbudget.json",
                   {"phase": "longbudget", "rows": rows, "d14": d14})
    if proof_dir.exists() and not any(proof_dir.iterdir()):
        proof_dir.rmdir()
    ref = {}
    for cell in B_CELLS:
        rs = [r for r in rows if r["cell"] == cell and r["stack"] == "hand"]
        ref[cell] = {
            "median_ms": median([r["ms"] for r in rs]),
            "proved_all": bool(rs) and all(r["proved"] for r in rs),
            "runs": len(rs),
        }
    # Same-session provari references from phase A, if present.
    prev = {}
    pfile = outdir / "pernode.json"
    if pfile.exists():
        prev = json.loads(pfile.read_text()).get("provari", {})
    factors = {}
    for cell in B_CELLS:
        pref = prev.get(cell, {}).get("median_ms")
        wall = ref[cell]["median_ms"]
        factors[cell] = {
            "atomi_ms": wall,
            "provari_ms": pref,
            "factor": (round(wall / pref, 3) if wall and pref else None),
            "proved_all": ref[cell]["proved_all"],
        }
    near_term = all(factors[c]["proved_all"] and factors[c]["factor"] is not None
                    and factors[c]["factor"] <= 3.0
                    for c in ["core", "deep-win"])
    data = {
        "phase": "longbudget",
        "seconds": LONG_SECONDS,
        "memory_mb": MEMORY,
        "seeds": SEEDS,
        "rows": rows,
        "d14": d14,
        "factors": factors,
        "decision": {
            "rule": "core and deep-win proved at 60 s with factor <= 3 => wall goal near-term",
            "near_term": near_term,
            "recommendation": "continue (item 18)" if near_term else "re-pin (item 19)",
            "d14_all_accepted": all(d["verdict"] == "accepted" for d in d14),
        },
    }
    write_json(outdir / "longbudget.json", data)
    return data




def phase_provari_b(args: argparse.Namespace) -> dict:
    """Same-session provari hand reference for every phase-B cell."""
    outdir = Path(args.outdir)
    cells = cells_by_name(args)
    data = json.loads((outdir / "longbudget.json").read_text())
    prov = {}
    for cell in B_CELLS:
        rs = []
        for repeat in range(args.provari_repeats):
            row = provari_row(args, cells[cell], repeat)
            rs.append(row)
            print(f"provari-b {cell:9s} r{repeat}: proved={row['proved']} "
                  f"nodes={row['nodes']} ms={row['ms']} "
                  f"wall={row['external_wall_s']}", flush=True)
        prov[cell] = {
            "repeats": rs,
            "median_ms": median([r["ms"] for r in rs]),
            "median_nodes": median([r["nodes"] for r in rs]),
            "median_ns_per_node": median([r["ns_per_node"] for r in rs]),
        }
    data["provari"] = prov
    factors = {}
    for cell in B_CELLS:
        ref = prov[cell]["median_ms"]
        rs = [r for r in data["rows"] if r["cell"] == cell and r["stack"] == "hand"]
        wall = median([r["ms"] for r in rs])
        factors[cell] = {
            "atomi_ms": wall,
            "provari_ms": ref,
            "factor": (round(wall / ref, 3) if wall and ref else None),
            "proved_all": bool(rs) and all(r["proved"] for r in rs),
            "censored": not (bool(rs) and all(r["proved"] for r in rs)),
        }
    data["factors"] = factors
    data["decision"]["near_term"] = all(
        factors[c]["proved_all"] and factors[c]["factor"] is not None
        and factors[c]["factor"] <= 3.0 for c in ["core", "deep-win"])
    data["decision"]["recommendation"] = (
        "continue (item 18)" if data["decision"]["near_term"]
        else "re-pin (item 19)")
    write_json(outdir / "longbudget.json", data)
    return data

def phase_d14(args: argparse.Namespace) -> dict:
    """Fill D14 checks for proved rows already recorded by longbudget."""
    outdir = Path(args.outdir)
    cells = cells_by_name(args)
    data = json.loads((outdir / "longbudget.json").read_text())
    rows = data["rows"]
    proof_dir = outdir / "proofs"
    proof_dir.mkdir(exist_ok=True)
    d14 = []
    for row in rows:
        if not row["proved"]:
            continue
        flags = STACKS.get(row["stack"], HAND)
        path = proof_dir / f"d14-{row['stack']}-{row['cell']}-s{row['seed']}.jsonl"
        extra = atomi_row(args, cells[row["cell"]], row["seed"], flags, 600.0,
                          None, row["stack"], "d14", path)
        check = None
        if extra["proved"] and extra["proof"] == "written":
            check = check_proof(args, Path(args.atomi), path)
        same = extra["child_evals"] == row["child_evals"]
        d14.append({
            "cell": row["cell"], "seed": row["seed"], "stack": row["stack"],
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
        print(f"d14 {row['stack']:5s} {row['cell']:9s} s{row['seed']}: "
              f"same_evals={same} check={check and check['verdict']}", flush=True)
    if proof_dir.exists() and not any(proof_dir.iterdir()):
        proof_dir.rmdir()
    data["d14"] = d14
    data["decision"]["d14_all_accepted"] = all(
        d["verdict"] == "accepted" for d in d14)
    write_json(outdir / "longbudget.json", data)
    return data

def main() -> None:
    args = parse_args()
    outdir = Path(args.outdir)
    outdir.mkdir(parents=True, exist_ok=True)
    if args.phase in ("env", "all"):
        phase_env(args)
    if args.phase in ("pernode", "all"):
        phase_pernode(args)
    if args.phase in ("longbudget", "all"):
        phase_longbudget(args)
    if args.phase == "provari-b":
        phase_provari_b(args)
    if args.phase == "d14":
        phase_d14(args)


if __name__ == "__main__":
    main()
