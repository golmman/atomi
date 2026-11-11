#!/usr/bin/env python3
"""Plan 17 item-24 Phase B: exact-replay applicability and the 20 M isolation.

Phases
------
env           pin host/toolchain/binary/corpus/oracle and the Phase B stacks
applicability B1: for every first move-divergent position in the plan16
              classify data, record whether exact sibling replay can apply
              (defender/AND node, proved child in the next ply, an Open
              sibling). Read-only: no search, no table, no proof run.
isolation     B2: bare vs hand vs hand+replay_exact on core/easy-win/deep-win
              at 20 M child evals, seed 0, --stats; seed 1 for the best
              guided stack. The D29 gate is computed from the seed-0 rows.
d14           an untimed --seconds 600 --proof-out run for every proved
              Phase B row; require equal child_evals and both checkers.
all           env, applicability, isolation, d14

One process at a time. Proof files live under /tmp/atomi-plan17 and are
never committed; only parsed JSON is written here. No raw transcript,
provari artifact or __pycache__ is stored.
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
DEFAULT_ATOMI = REPO / "target/release/atomi"
DEFAULT_PROVARI = Path("/tmp/atomi-oracle/provari/target/release/provari")
DEFAULT_CORPUS = REPO / "bench/corpus.json"
CLASSIFY = REPO / "docs/plans/prover/measurements/plan16/classify.json"
SCRATCH = Path("/tmp/atomi-plan17")

CELLS = {
    "core": "r1b1kbnr/ppp3pp/n4q2/3pppN1/1P1PP3/8/P1P2PPP/RNBQKB1R w KQkq - 0 1",
    "easy-win": "rnbq1b1r/pp1p1kpp/2p1pp1n/8/3PP1P1/2N2N2/PPP2P1P/R1BQKB1R w KQ - 0 1",
    "deep-win": "rnb1kbnr/pppp2pp/5q2/4ppN1/3P4/8/PPP1PPPP/RNBQKB1R w KQkq - 0 1",
}
STACKS = {
    "bare": ["--guide", "none"],
    "hand": ["--guide", "hand"],
    "replay_exact": ["--guide", "hand", "--guide-replay-exact"],
}
EVALS = 20_000_000
MEMORY = 192
SEED = 0
SEED_BEST = 1
BEST_SECONDS = 600.0
D14_SECONDS = 600.0


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--phase", choices=["env", "applicability", "isolation",
                                       "d14", "all"], default="all")
    p.add_argument("--atomi", default=str(DEFAULT_ATOMI))
    p.add_argument("--provari", default=str(DEFAULT_PROVARI))
    p.add_argument("--corpus", default=str(DEFAULT_CORPUS))
    p.add_argument("--classify", default=str(CLASSIFY))
    p.add_argument("--outdir", default=str(HERE))
    p.add_argument("--scratch", default=str(SCRATCH))
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


def git_head(repo: Path) -> str:
    try:
        return subprocess.run(["git", "rev-parse", "HEAD"], cwd=repo,
                              capture_output=True, text=True).stdout.strip()
    except OSError:
        return "unknown"


def worktree(repo: Path) -> str:
    try:
        out = subprocess.run(["git", "status", "--porcelain"], cwd=repo,
                             capture_output=True, text=True).stdout
        return "dirty (plan17 execution; owner commits)" if out else "clean"
    except OSError:
        return "unknown"


def cpu_model() -> str:
    try:
        for line in Path("/proc/cpuinfo").read_text().splitlines():
            if line.startswith("model name"):
                return line.split(":", 1)[1].strip()
    except OSError:
        pass
    return "unknown"


def write_json(path: Path, data: dict) -> None:
    path.write_text(json.dumps(data, indent=1) + "\n")


def node_kind(fen: str, attacker: str) -> str:
    return "attacker (OR)" if fen.split()[1] == attacker else "defender (AND)"


def detail_fields(detail: str) -> dict:
    out = {}
    for part in detail.split():
        if "=" in part:
            k, v = part.split("=", 1)
            out[k] = v
    return out


def env(args: argparse.Namespace) -> dict:
    pv = subprocess.run([args.provari, "--version"], capture_output=True,
                        text=True)
    atomi = Path(args.atomi)
    corpus = Path(args.corpus)
    data = {
        "plan": "plan17",
        "date": "2026-11-10",
        "host": platform.node(),
        "cpu": cpu_model(),
        "kernel": f"{platform.system()} {platform.release()}",
        "machine": platform.machine(),
        "python": platform.python_version(),
        "rustc": subprocess.run(["rustc", "--version"], capture_output=True,
                                text=True).stdout.strip(),
        "git_head": git_head(REPO),
        "worktree": worktree(REPO),
        "atomi_binary": str(atomi),
        "atomi_binary_sha256": sha256(atomi),
        "corpus": str(corpus),
        "corpus_sha256": sha256(corpus),
        "classify_source": str(Path(args.classify)),
        "classify_sha256": sha256(Path(args.classify)),
        "provari": args.provari,
        "provari_version": (pv.stdout or pv.stderr).strip(),
        "cells": CELLS,
        "stacks": STACKS,
        "evals": EVALS,
        "memory_mb": MEMORY,
        "seed": SEED,
        "best_seed": SEED_BEST,
        "isolation_seconds": BEST_SECONDS,
        "d14_seconds": D14_SECONDS,
        "note": ("--seconds 600 only guards the wall; --evals 20000000 is the "
                 "budget, as in plan13 phase C"),
    }
    write_json(Path(args.outdir) / "env.json", data)
    return data


def applicability(args: argparse.Namespace) -> dict:
    source = Path(args.classify)
    classify = json.loads(source.read_text())
    cells = {}
    positions = 0
    sides = 0
    defender_nodes = 0
    child_in_next_ply = 0
    applicable = 0
    for cell, data in classify.items():
        if cell == "summary" or data.get("status") != "ok":
            continue
        rows = []
        for pos in data.get("positions", []):
            positions += 1
            attacker = pos["attacker"]
            kind = node_kind(pos["fen"], attacker)
            defender = kind.startswith("defender")
            if defender:
                defender_nodes += 1
            sides_rows = {}
            for side in ("provari", "atomi"):
                ev = pos.get(side, {})
                detail = ev.get("predicates", {}).get("exact_replay", {})
                fields = detail_fields(detail.get("detail", ""))
                in_next = fields.get("child_in_next_ply") == "True"
                if in_next:
                    child_in_next_ply += 1
                # The "proved child" is the move in this position; exact
                # replay needs a defender node whose child is proved in the
                # next ply and at least one still-Open sibling. At an
                # attacker node the mechanism never fires. The sibling
                # Open/decided state is only available from a proof edge;
                # no defender node occurs here, so it is not evaluated.
                sibling_open = None
                ok = bool(defender and in_next and sibling_open)
                if ok:
                    applicable += 1
                reason = (
                    "not a defender (AND) node"
                    if not defender else
                    "proved child is not in the next ply"
                    if not in_next else
                    "sibling Open/decided state requires a proof edge"
                )
                sides_rows[side] = {
                    "move": ev.get("move"),
                    "class": ev.get("class"),
                    "child": ev.get("child"),
                    "child_in_next_ply": in_next,
                    "sibling_open": sibling_open,
                    "applicable": ok,
                    "reason": reason,
                }
                sides += 1
            rows.append({
                "fen": pos["fen"],
                "ply": pos["ply"],
                "attacker": attacker,
                "node_kind": kind,
                "provari_move": pos.get("provari_move"),
                "atomi_move": pos.get("atomi_move"),
                "sides": sides_rows,
            })
        cells[cell] = {
            "first_divergent_ply": data.get("first_divergent_ply"),
            "first_move_divergent_ply": data.get("first_move_divergent_ply"),
            "positions": rows,
        }
    data = {
        "phase": "applicability",
        "source": str(source),
        "source_sha256": sha256(source),
        "rule": ("exact sibling replay applies only at a defender (AND) node "
                 "when the just-proved child is in the next ply and at least "
                 "one sibling is still Open (src/search/mod.rs)"),
        "cells": cells,
        "summary": {
            "positions": positions,
            "move_sides": sides,
            "defender_nodes": defender_nodes,
            "child_in_next_ply": child_in_next_ply,
            "applicable": applicable,
            "g2_pass": applicable > 0,
            "verdict": (
                "no first move-divergent position is a defender node; exact "
                "replay cannot apply at the first divergence, so B3 is not "
                "implemented and Phase B is re-scoped to the isolation matrix"
                if applicable == 0 else
                "at least one first move-divergent position is an applicable "
                "defender node; B3 is authorised"
            ),
        },
    }
    write_json(Path(args.outdir) / "applicability.json", data)
    return data


def prove_row(args: argparse.Namespace, cell: str, stack: str, seed: int,
              proof_out: Path | None = None) -> dict:
    cmd = [args.atomi, "prove", CELLS[cell], "--seconds", str(int(BEST_SECONDS)),
           "--evals", str(EVALS), "--memory", str(MEMORY),
           "--seed", str(seed), "--stats", *STACKS[stack]]
    if proof_out is not None:
        cmd += ["--proof-out", str(proof_out)]
    payload, wall, stderr, code = run(cmd, BEST_SECONDS)
    st = payload.get("stats") or {}
    row = {
        "cell": cell,
        "stack": stack,
        "seed": seed,
        "flags": STACKS[stack],
        "seconds": int(BEST_SECONDS),
        "evals": EVALS,
        "memory_mb": MEMORY,
        "unique_keys": st.get("unique_keys"),
        "unique_keys_capped": st.get("unique_keys_capped"),
        "re_expansions": st.get("re_expansions"),
        "proved": bool(payload.get("found")),
        "exit": payload.get("exit"),
        "length": payload.get("length"),
        "child_evals": payload.get("child_evals"),
        "expansions": payload.get("expansions"),
        "sim_nodes": payload.get("sim_nodes"),
        "settle_nodes": payload.get("settle_nodes"),
        "aux_nodes": payload.get("aux_nodes"),
        "aux_skipped": payload.get("aux_skipped"),
        "ms": payload.get("ms"),
        "external_wall_s": wall,
        "returncode": code,
        "stderr": stderr or None,
        "proof": payload.get("proof"),
        "proof_positions": payload.get("proof_positions"),
        "proof_length": payload.get("proof_length"),
        "proof_entries": payload.get("proof_entries"),
        "proof_full": payload.get("proof_full"),
        "proof_file_exists": bool(proof_out and proof_out.exists()),
        "proof_file_bytes": (proof_out.stat().st_size
                             if proof_out and proof_out.exists() else None),
    }
    return row


def summarize(rows: list[dict]) -> dict:
    seed0 = [r for r in rows if r["seed"] == SEED]
    by = {(r["stack"], r["cell"]): r for r in seed0}
    base = {}
    for cell in CELLS:
        bare = by.get(("bare", cell))
        if bare and bare.get("unique_keys"):
            base[cell] = bare["unique_keys"]
    ratios = {}
    for stack in STACKS:
        ratios[stack] = {}
        for cell in CELLS:
            row = by.get((stack, cell))
            if row and row.get("unique_keys") and base.get(cell):
                ratios[stack][cell] = round(row["unique_keys"] / base[cell], 3)
            else:
                ratios[stack][cell] = None
    best_core = None
    for stack in ("hand", "replay_exact"):
        row = by.get((stack, "core"))
        if row and row.get("unique_keys"):
            if best_core is None or row["unique_keys"] < best_core[1]:
                best_core = (stack, row["unique_keys"])
    cut = None
    if base.get("core") and by.get(("replay_exact", "core"), {}).get("unique_keys"):
        cut = round(base["core"] / by[("replay_exact", "core")]["unique_keys"], 3)
    easy_regression = None
    if base.get("easy-win") and by.get(("replay_exact", "easy-win"), {}).get("unique_keys"):
        easy_regression = round(
            by[("replay_exact", "easy-win")]["unique_keys"] / base["easy-win"], 3)
    g3 = bool(cut is not None and cut >= 2.0
              and (easy_regression is None or easy_regression <= 1.10))
    return {
        "bare_unique_keys": base,
        "unique_ratio_vs_bare": ratios,
        "core_cut_replay_exact": cut,
        "easy_win_regression_replay_exact": easy_regression,
        "best_guided_stack_core": best_core[0] if best_core else None,
        "best_guided_stack_core_unique": best_core[1] if best_core else None,
        "g3_pass": g3,
        "g3_rule": ("replay_exact cuts core unique expansions >= 2x vs bare "
                    "at 20 M, with no easy-win regression > 10 %"),
    }


def isolation(args: argparse.Namespace) -> dict:
    rows = []
    for stack in STACKS:
        for cell in CELLS:
            row = prove_row(args, cell, stack, SEED)
            rows.append(row)
            print(f"isolation {stack:12s} {cell:9s} s{SEED}: "
                  f"proved={row['proved']} exit={row['exit']} "
                  f"unique={row['unique_keys']} exp={row['expansions']} "
                  f"evals={row['child_evals']} sim={row['sim_nodes']} "
                  f"aux={row['aux_nodes']} len={row['length']}", flush=True)
            write_json(Path(args.outdir) / "isolation.json",
                       {"phase": "isolation", "rows": rows})
    summary = summarize(rows)
    best = summary["best_guided_stack_core"] or "replay_exact"
    for cell in CELLS:
        row = prove_row(args, cell, best, SEED_BEST)
        rows.append(row)
        print(f"isolation {best:12s} {cell:9s} s{SEED_BEST}: "
              f"proved={row['proved']} exit={row['exit']} "
              f"unique={row['unique_keys']} exp={row['expansions']} "
              f"evals={row['child_evals']}", flush=True)
        write_json(Path(args.outdir) / "isolation.json",
                   {"phase": "isolation", "rows": rows})
    data = {
        "phase": "isolation",
        "evals": EVALS,
        "memory_mb": MEMORY,
        "seed": SEED,
        "best_seed": SEED_BEST,
        "best_guided_stack": best,
        "stacks": STACKS,
        "cells": CELLS,
        "rows": rows,
        "summary": summary,
    }
    write_json(Path(args.outdir) / "isolation.json", data)
    return data


def check_proof(args: argparse.Namespace, path: Path) -> dict:
    v, _, _, _ = run([args.atomi, "validate", str(path)], 60)
    c, _, _, _ = run([args.provari, "check", str(path)], 60)
    valid = bool(v.get("valid")) and c.get("valid") is True
    return {
        "atomi_valid": v.get("valid"),
        "atomi_positions": v.get("positions"),
        "provari_valid": c.get("valid"),
        "provari_won_positions": c.get("won_positions"),
        "verdict": "accepted" if valid else "rejected",
    }


def d14(args: argparse.Namespace) -> dict:
    outdir = Path(args.outdir)
    proof_dir = outdir / "proofs"
    proof_dir.mkdir(exist_ok=True)
    data = json.loads((outdir / "isolation.json").read_text())
    rows = data["rows"]
    d14_rows = []
    for row in rows:
        if not row["proved"]:
            continue
        path = proof_dir / (f"d14-{row['stack']}-{row['cell']}"
                            f"-s{row['seed']}.jsonl")
        extra = prove_row(args, row["cell"], row["stack"], row["seed"], path)
        check = None
        if extra["proved"] and extra["proof"] == "written":
            check = check_proof(args, path)
        same = extra["child_evals"] == row["child_evals"]
        verdict = ("accepted" if same and check
                   and check["verdict"] == "accepted" else "rejected")
        d14_rows.append({
            "cell": row["cell"],
            "stack": row["stack"],
            "seed": row["seed"],
            "timed_child_evals": row["child_evals"],
            "proof_child_evals": extra["child_evals"],
            "same_child_evals": same,
            "proof_positions": extra["proof_positions"],
            "proof_entries": extra["proof_entries"],
            "proof_file_bytes": extra["proof_file_bytes"],
            "check": check,
            "verdict": verdict,
        })
        if path.exists():
            path.unlink()
        print(f"d14 {row['stack']:12s} {row['cell']:9s} s{row['seed']}: "
              f"same_evals={same} check={check and check['verdict']}",
              flush=True)
        write_json(outdir / "isolation.json", {**data, "d14": d14_rows})
    if proof_dir.exists() and not any(proof_dir.iterdir()):
        proof_dir.rmdir()
    data["d14"] = d14_rows
    data["d14_all_accepted"] = bool(d14_rows) and all(
        d["verdict"] == "accepted" for d in d14_rows)
    write_json(outdir / "isolation.json", data)
    return data


def main() -> int:
    args = parse_args()
    outdir = Path(args.outdir)
    outdir.mkdir(parents=True, exist_ok=True)
    if args.phase in ("env", "all"):
        env(args)
    if args.phase in ("applicability", "all"):
        data = applicability(args)
        print(json.dumps(data["summary"], indent=1), flush=True)
    if args.phase in ("isolation", "all"):
        data = isolation(args)
        print(json.dumps(data["summary"], indent=1), flush=True)
    if args.phase in ("d14", "all"):
        d14(args)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
