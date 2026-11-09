#!/usr/bin/env python3
"""Plan 6 item-15 GROUP kill-test driver (stdlib only).

Phases
------
env        pin host/toolchain/binary and the frozen input hashes
group      core/easy-win x seeds 0-2, flag off/on, --evals 20M (12 runs + repeats)
soundness  guidance-v1 seed 0 with GROUP: refutations, D14 proofs and both checkers
scale      core/easy-win auxiliary D14 proofs (the plan7 scale-case re-check)
all        env, group, soundness, scale

One atomi process at a time. Only parsed JSON-line fields are stored; no raw
transcript, provari artifact or bytecode is written.
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
GROUPS = ["core", "easy-win"]
AUX = ["--guide", "hand", "--guide-scale", "64", "--guide-ef-weight", "0",
       "--guide-max-sim", "1", "--guide-max-settle", "1"]
HAND = ["--guide", "hand"]
SEEDS = [0, 1, 2]
SECONDS = 600.0
EVALS = 20_000_000
MEMORY = 192


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--phase", choices=["env", "group", "soundness", "scale",
                                       "all"], default="all")
    p.add_argument("--atomi", default=str(DEFAULT_ATOMI))
    p.add_argument("--provari-check", default=str(DEFAULT_PROVARI))
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


def run(cmd: list[str], timeout: float) -> tuple[dict, float, int]:
    start = time.perf_counter()
    try:
        proc = subprocess.run(cmd, capture_output=True, text=True,
                              timeout=timeout + 120, check=False)
        return last_json(proc.stdout + proc.stderr), round(
            time.perf_counter() - start, 3), proc.returncode
    except subprocess.TimeoutExpired:
        return {}, round(time.perf_counter() - start, 3), 124


def load_set(path: Path) -> list[dict]:
    rows = []
    for raw in path.read_text().splitlines():
        if not raw.strip() or raw.startswith("name\t"):
            continue
        f = raw.split("\t")
        rows.append({"name": f[0], "fen": f[1], "source": f[2],
                     "ref_nodes": int(f[3]), "ref_wall_s": float(f[4])})
    return rows


def git_head() -> str:
    try:
        return subprocess.run(["git", "rev-parse", "HEAD"], cwd=REPO,
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
        return "dirty (plan6 execution; owner commits)" if out else "clean"
    except OSError:
        return "unknown"


def write_json(path: Path, data: dict) -> None:
    path.write_text(json.dumps(data, indent=1) + "\n")


def phase_env(args: argparse.Namespace) -> dict:
    atomi = Path(args.atomi)
    set_path = Path(args.set)
    corpus = Path(args.corpus)
    pv = subprocess.run([args.provari_check, "--version"], capture_output=True,
                        text=True)
    env = {
        "plan": "plan6",
        "date": "2026-11-09",
        "host": platform.node(),
        "cpu": cpu_model(),
        "kernel": f"{platform.system()} {platform.release()}",
        "machine": platform.machine(),
        "python": platform.python_version(),
        "rustc": subprocess.run(["rustc", "--version"], capture_output=True,
                                text=True).stdout.strip(),
        "git_head": git_head(),
        "worktree": worktree(),
        "atomi_binary": str(atomi),
        "atomi_binary_sha256": sha256(atomi),
        "guidance_set": str(set_path),
        "guidance_sha256": sha256(set_path),
        "corpus": str(corpus),
        "corpus_sha256": sha256(corpus),
        "provari_check": args.provari_check,
        "provari_version": (pv.stdout or pv.stderr).strip(),
        "stack_off": HAND,
        "stack_on": HAND + ["--guide-group"],
        "group_budget": {"seconds": SECONDS, "evals": EVALS,
                         "memory_mb": MEMORY, "seeds": SEEDS},
        "aux_stack": AUX,
    }
    write_json(Path(args.outdir) / "env.json", env)
    return env


def atomi_row(args: argparse.Namespace, cell: dict, seed: int, flags: list[str],
              seconds: float, evals: int | None,
              proof_out: Path | None = None) -> dict:
    cmd = [args.atomi, "prove", cell["fen"], "--seconds", str(int(seconds)),
           "--memory", str(MEMORY), "--seed", str(seed), *flags]
    if evals is not None:
        cmd += ["--evals", str(evals)]
    if proof_out is not None:
        cmd += ["--proof-out", str(proof_out)]
    payload, wall, code = run(cmd, seconds)
    return {
        "cell": cell["name"], "seed": seed, "flags": flags,
        "seconds": seconds, "evals_limit": evals,
        "proved": bool(payload.get("found")),
        "exit": payload.get("exit"),
        "length": payload.get("length"),
        "child_evals": payload.get("child_evals"),
        "expansions": payload.get("expansions"),
        "sim_nodes": payload.get("sim_nodes"),
        "settle_nodes": payload.get("settle_nodes"),
        "proof_full": payload.get("proof_full"),
        "proof": payload.get("proof"),
        "proof_positions": payload.get("proof_positions"),
        "proof_length": payload.get("proof_length"),
        "proof_entries": payload.get("proof_entries"),
        "ms": payload.get("ms"),
        "wall_s": wall,
        "returncode": code,
        "proof_file_exists": bool(proof_out and proof_out.exists()),
        "proof_file_bytes": (proof_out.stat().st_size
                             if proof_out and proof_out.exists() else None),
    }


def cells_by_name(args: argparse.Namespace) -> dict[str, dict]:
    return {c["name"]: c for c in load_set(Path(args.set))}


def phase_group(args: argparse.Namespace) -> dict:
    cells = cells_by_name(args)
    rows = []
    for name in GROUPS:
        for seed in SEEDS:
            for tag, flags in (("off", HAND), ("on", HAND + ["--guide-group"])):
                row = atomi_row(args, cells[name], seed, flags, SECONDS, EVALS)
                row["tag"] = tag
                rows.append(row)
                print(f"{name} s{seed} {tag}: proved={row['proved']} "
                      f"exit={row['exit']} exp={row['expansions']} "
                      f"evals={row['child_evals']} wall={row['wall_s']}",
                      flush=True)
    # G4: repeat seed 0 for both cells and both tags.
    repeats = []
    for name in GROUPS:
        for tag, flags in (("off", HAND), ("on", HAND + ["--guide-group"])):
            row = atomi_row(args, cells[name], 0, flags, SECONDS, EVALS)
            row["tag"] = tag
            repeats.append(row)
    index = {(r["cell"], r["seed"], r["tag"]): r for r in rows}
    repeat_equal = {}
    for r in repeats:
        base = index[(r["cell"], r["seed"], r["tag"])]
        repeat_equal[f"{r['cell']}-{r['tag']}"] = (
            (base["exit"], base["length"], base["child_evals"],
             base["expansions"]) ==
            (r["exit"], r["length"], r["child_evals"], r["expansions"])
        )
    summary = {}
    for name in GROUPS:
        off = [r for r in rows if r["cell"] == name and r["tag"] == "off"]
        on = [r for r in rows if r["cell"] == name and r["tag"] == "on"]
        summary[name] = {
            "off_expansions": [r["expansions"] for r in off],
            "on_expansions": [r["expansions"] for r in on],
            "off_proved": [r["proved"] for r in off],
            "on_proved": [r["proved"] for r in on],
            "off_child_evals": [r["child_evals"] for r in off],
            "on_child_evals": [r["child_evals"] for r in on],
            "on_max_expansions": max(r["expansions"] for r in on),
            "off_mean_expansions": sum(r["expansions"] for r in off) / len(off),
            "on_mean_expansions": sum(r["expansions"] for r in on) / len(on),
            "cut_vs_off_mean": (sum(r["expansions"] for r in off) / len(off)
                                / (sum(r["expansions"] for r in on) / len(on))),
        }
    core = summary["core"]
    easy = summary["easy-win"]
    gates = {
        "g1_literal_under_1m": core["on_max_expansions"] < 1_000_000,
        "g1_fivefold_cut": core["cut_vs_off_mean"] >= 5.0,
        "g2_easy_proved_all_seeds": all(easy["on_proved"]),
        "g2_no_expansion_regression": all(
            o <= b for o, b in zip(easy["on_expansions"],
                                   easy["off_expansions"])
        ),
        "g4_repeats_identical": all(repeat_equal.values()),
    }
    data = {
        "phase": "group",
        "stack_off": HAND,
        "stack_on": HAND + ["--guide-group"],
        "seconds": SECONDS, "evals_limit": EVALS, "memory_mb": MEMORY,
        "seeds": SEEDS,
        "rows": rows, "repeats": repeats, "repeat_equal": repeat_equal,
        "summary": summary, "gates": gates,
    }
    write_json(Path(args.outdir) / "group.json", data)
    return data


def check_proof(args: argparse.Namespace, atomi: Path, path: Path) -> dict:
    v, _, _ = run([str(atomi), "validate", str(path)], 60)
    c, _, _ = run([args.provari_check, "check", str(path)], 60)
    valid = bool(v.get("valid")) and c.get("valid") is True
    return {
        "atomi_valid": v.get("valid"),
        "atomi_positions": v.get("positions"),
        "provari_valid": c.get("valid"),
        "provari_won_positions": c.get("won_positions"),
        "verdict": "accepted" if valid else "rejected",
    }


def phase_soundness(args: argparse.Namespace) -> dict:
    outdir = Path(args.outdir)
    proof_dir = outdir / "proofs"
    proof_dir.mkdir(exist_ok=True)
    cells = load_set(Path(args.set))
    rows = []
    refuted = []
    proof_rows = []
    for cell in cells:
        path = proof_dir / f"group-{cell['name']}-s0.jsonl"
        row = atomi_row(args, cell, 0, HAND + ["--guide-group"], 10.0,
                        5_000_000, path)
        row["tag"] = "on"
        rows.append(row)
        if row["exit"] == "refuted":
            refuted.append(cell["name"])
        if row["proved"] and row["proof"] == "written":
            check = check_proof(args, Path(args.atomi), path)
            proof_rows.append({"cell": cell["name"], "check": check,
                               "positions": row["proof_positions"],
                               "entries": row["proof_entries"]})
        if path.exists():
            path.unlink()
        print(f"sound {cell['name']}: exit={row['exit']} "
              f"proved={row['proved']}", flush=True)
    if proof_dir.exists() and not any(proof_dir.iterdir()):
        proof_dir.rmdir()
    rejected = [p for p in proof_rows if p["check"]["verdict"] != "accepted"]
    data = {
        "phase": "soundness",
        "stack": HAND + ["--guide-group"],
        "seconds": 10.0, "evals_limit": 5_000_000, "memory_mb": MEMORY,
        "seed": 0,
        "rows": rows,
        "refuted": refuted,
        "proofs": proof_rows,
        "gates": {
            "no_guidance_refutation": not refuted,
            "all_proved_proofs_accepted": not rejected,
        },
    }
    write_json(outdir / "soundness.json", data)
    return data


def phase_scale(args: argparse.Namespace) -> dict:
    outdir = Path(args.outdir)
    proof_dir = outdir / "proofs"
    proof_dir.mkdir(exist_ok=True)
    cells = cells_by_name(args)
    cases = []
    for name, seconds in (("core", 120.0), ("easy-win", 600.0)):
        path = proof_dir / f"scale-{name}-s0.jsonl"
        row = atomi_row(args, cells[name], 0, AUX, seconds, None, path)
        check = None
        if row["proved"] and row["proof"] == "written":
            check = check_proof(args, Path(args.atomi), path)
        if path.exists():
            path.unlink()
        cases.append({"cell": name, "run": row, "check": check})
        print(f"scale {name}: proved={row['proved']} positions="
              f"{row['proof_positions']} entries={row['proof_entries']} "
              f"check={check and check['verdict']}", flush=True)
    if proof_dir.exists() and not any(proof_dir.iterdir()):
        proof_dir.rmdir()
    data = {"phase": "scale", "aux_stack": AUX, "memory_mb": MEMORY,
            "cases": cases}
    write_json(outdir / "scale.json", data)
    return data


def main() -> None:
    args = parse_args()
    outdir = Path(args.outdir)
    outdir.mkdir(parents=True, exist_ok=True)
    if args.phase in ("env", "all"):
        phase_env(args)
    if args.phase in ("group", "all"):
        phase_group(args)
    if args.phase in ("soundness", "all"):
        phase_soundness(args)
    if args.phase in ("scale", "all"):
        phase_scale(args)


if __name__ == "__main__":
    main()
