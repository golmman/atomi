#!/usr/bin/env python3
"""Plan 13 item-22 driver: cheap probes before any conformance stack.

Phases
------
env      pin host/toolchain/binary, corpus/guidance hashes and provari
phase_a  proof-share sweep: core/easy-win/deep-win, seeds 0-2, shares
         25/40/50/60 at --seconds 120 --memory 192 --guide none
d14      --seconds 600 --proof-out for every proved Phase A row; require
         equal child_evals and both checkers
phase_b  (only if A fails) --max-depth ply-cap probe at 20 M evals
phase_c  (only if A and B fail) conformance probe
all      env, phase_a, d14, then the conditional phases

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
CELLS = ["core", "easy-win", "deep-win"]
SHARES = [25, 40, 50, 60]
COVERAGE_SECONDS = 120.0
D14_SECONDS = 600.0
BARE = ["--guide", "none"]


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--phase", choices=["env", "phase_a", "d14", "phase_b",
                                       "phase_c", "d14_c", "all"],
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
        return "dirty (plan13 execution; owner commits)" if out else "clean"
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
        "plan": "plan13",
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
        "coverage_seconds": COVERAGE_SECONDS,
        "d14_seconds": D14_SECONDS,
        "memory_mb": MEMORY,
        "seeds": SEEDS,
        "cells": CELLS,
        "shares": SHARES,
    }
    write_json(Path(args.outdir) / "env.json", env)
    return env


def atomi_row(args: argparse.Namespace, cell: dict, seed: int, seconds: float,
              budget: str, share: int, proof_out: Path | None = None,
              extra: list[str] | None = None) -> dict:
    cmd = [args.atomi, "prove", cell["fen"], "--seconds", str(int(seconds)),
           "--memory", str(MEMORY), "--seed", str(seed), *BARE,
           "--proof-share", str(share)]
    if extra:
        cmd += extra
    if proof_out is not None:
        cmd += ["--proof-out", str(proof_out)]
    payload, wall, stderr, code = run(cmd, seconds)
    st = payload.get("stats") or {}
    cap = None
    if extra and "--max-depth" in extra:
        cap = int(extra[extra.index("--max-depth") + 1])
    return {
        "cell": cell["name"], "seed": seed, "budget": budget,
        "flags": BARE, "seconds": seconds, "share": share,
        "extra": extra or [], "cap": cap,
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
        "proof_full": payload.get("proof_full"),
        "proof": payload.get("proof"),
        "proof_positions": payload.get("proof_positions"),
        "proof_length": payload.get("proof_length"),
        "proof_entries": payload.get("proof_entries"),
        "ms": payload.get("ms"),
        "external_wall_s": wall,
        "returncode": code,
        "stderr": stderr or None,
        "proof_file_exists": bool(proof_out and proof_out.exists()),
        "proof_file_bytes": (proof_out.stat().st_size
                             if proof_out and proof_out.exists() else None),
    }


def phase_a(args: argparse.Namespace) -> dict:
    outdir = Path(args.outdir)
    cells = cells_by_name(args)
    rows = []
    for share in SHARES:
        for cell in CELLS:
            for seed in SEEDS:
                row = atomi_row(args, cells[cell], seed, COVERAGE_SECONDS,
                                "phase-a", share)
                rows.append(row)
                print(f"phase_a share={share:2d} {cell:9s} s{seed}: "
                      f"proved={row['proved']} exit={row['exit']} "
                      f"len={row['length']} exp={row['expansions']} "
                      f"evals={row['child_evals']} "
                      f"entries={row['proof_entries']} "
                      f"full={row['proof_full']} ms={row['ms']} "
                      f"wall={row['external_wall_s']}", flush=True)
                write_json(outdir / "phase_a.json",
                           {"phase": "phase_a", "rows": rows})
    data = {
        "phase": "phase_a",
        "seconds": COVERAGE_SECONDS,
        "memory_mb": MEMORY,
        "seeds": SEEDS,
        "cells": CELLS,
        "shares": SHARES,
        "flags": BARE,
        "rows": rows,
    }
    data["summary"] = summarize_a(rows)
    data["stop_rule"] = stop_rule_a(data["summary"])
    write_json(outdir / "phase_a.json", data)
    return data


def summarize_a(rows: list[dict]) -> dict:
    out = {}
    for share in SHARES:
        per = {}
        for cell in CELLS:
            rs = [r for r in rows
                  if r["share"] == share and r["cell"] == cell]
            per[cell] = {
                "proved": sum(1 for r in rs if r["proved"]),
                "runs": len(rs),
                "all_proved": bool(rs) and all(r["proved"] for r in rs),
                "median_ms": median([r["ms"] for r in rs]),
                "median_expansions": median([r["expansions"] for r in rs]),
                "median_child_evals": median([r["child_evals"] for r in rs]),
                "lengths": [r["length"] for r in rs],
                "proof_full": [r["proof_full"] for r in rs],
            }
        out[str(share)] = per
    return out


def stop_rule_a(summary: dict) -> dict:
    control = summary[str(SHARES[0])]
    best = None
    for share in SHARES:
        per = summary[str(share)]
        deep3 = per["deep-win"]["all_proved"]
        others3 = per["core"]["all_proved"] and per["easy-win"]["all_proved"]
        # Wall guard: core/easy-win median wall not worse than the 25 % control
        # by more than 10 %.
        guard = True
        ratios = {}
        for cell in ("core", "easy-win"):
            base = control[cell]["median_ms"]
            cur = per[cell]["median_ms"]
            if base and cur:
                ratios[cell] = round(cur / base, 3)
                if cur > base * 1.10:
                    guard = False
        if deep3 and others3 and guard:
            best = {"share": share, "ratios": ratios}
            break
    return {
        "control_share": SHARES[0],
        "passing_share": best["share"] if best else None,
        "ratios": best["ratios"] if best else None,
        "fired": best is not None,
        "rule": ("some share proves deep-win 3/3 and core/easy-win stay 3/3 "
                 "with median wall not worse than the 25 % control by more "
                 "than 10 %"),
    }


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


def phase_d14(args: argparse.Namespace) -> dict:
    outdir = Path(args.outdir)
    proof_dir = outdir / "proofs"
    proof_dir.mkdir(exist_ok=True)
    cells = cells_by_name(args)
    data = json.loads((outdir / "phase_a.json").read_text())
    rows = data["rows"]
    d14 = []
    for row in rows:
        if not row["proved"]:
            continue
        path = proof_dir / (f"d14-{row['cell']}-s{row['seed']}"
                            f"-share{row['share']}.jsonl")
        extra = atomi_row(args, cells[row["cell"]], row["seed"], D14_SECONDS,
                          "d14", row["share"], path)
        check = None
        if extra["proved"] and extra["proof"] == "written":
            check = check_proof(args, Path(args.atomi), path)
        same = extra["child_evals"] == row["child_evals"]
        d14.append({
            "cell": row["cell"], "seed": row["seed"], "share": row["share"],
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
        print(f"d14 {row['cell']:9s} s{row['seed']} share={row['share']}: "
              f"same_evals={same} check={check and check['verdict']}",
              flush=True)
        write_json(outdir / "phase_a.json", {**data, "d14": d14})
    if proof_dir.exists() and not any(proof_dir.iterdir()):
        proof_dir.rmdir()
    data["d14"] = d14
    data["d14_all_accepted"] = bool(d14) and all(
        d["verdict"] == "accepted" for d in d14)
    write_json(outdir / "phase_a.json", data)
    return data


EVALS_B = 20_000_000
CAPS = [16, 20, 24, 28, 32]


def phase_d14_c(args: argparse.Namespace) -> dict:
    outdir = Path(args.outdir)
    proof_dir = outdir / "proofs"
    proof_dir.mkdir(exist_ok=True)
    cells = cells_by_name(args)
    data = json.loads((outdir / "phase_c.json").read_text())
    rows = data["rows"]
    d14 = []
    for row in rows:
        if not row["proved"]:
            continue
        stack = row["stack"]
        path = proof_dir / (f"d14c-{stack}-{row['cell']}-s{row['seed']}"
                            f".jsonl")
        extra = ["--evals", str(EVALS_C), *STACKS[stack]]
        extra_row = atomi_row(args, cells[row["cell"]], row["seed"], D14_SECONDS,
                              "d14-c", 25, path, extra=extra)
        check = None
        if extra_row["proved"] and extra_row["proof"] == "written":
            check = check_proof(args, Path(args.atomi), path)
        same = extra_row["child_evals"] == row["child_evals"]
        d14.append({
            "cell": row["cell"], "seed": row["seed"], "stack": stack,
            "timed_child_evals": row["child_evals"],
            "proof_child_evals": extra_row["child_evals"],
            "same_child_evals": same,
            "proof_positions": extra_row["proof_positions"],
            "proof_entries": extra_row["proof_entries"],
            "proof_file_bytes": extra_row["proof_file_bytes"],
            "check": check,
            "verdict": (check["verdict"] if same and check
                        and check["verdict"] == "accepted"
                        else "rejected"),
        })
        if path.exists():
            path.unlink()
        print(f"d14_c {stack:10s} {row['cell']:9s} s{row['seed']}: "
              f"same_evals={same} check={check and check['verdict']}",
              flush=True)
        write_json(outdir / "phase_c.json", {**data, "d14": d14})
    if proof_dir.exists() and not any(proof_dir.iterdir()):
        proof_dir.rmdir()
    data["d14"] = d14
    data["d14_all_accepted"] = bool(d14) and all(
        d["verdict"] == "accepted" for d in d14)
    write_json(outdir / "phase_c.json", data)
    return data


def phase_b(args: argparse.Namespace) -> dict:
    outdir = Path(args.outdir)
    cells = cells_by_name(args)
    rows = []
    # Bare control at 20 M, seed 0, with --stats for the unique-key count.
    for cell in ("deep-win", "core"):
        row = atomi_row(args, cells[cell], 0, 600.0, "phase-b-bare", 25,
                        extra=["--evals", str(EVALS_B), "--stats"])
        rows.append(row)
        print(f"phase_b bare {cell:9s} s0: proved={row['proved']} "
              f"exit={row['exit']} exp={row['expansions']} "
              f"unique={row.get('unique_keys')} len={row['length']}",
              flush=True)
        write_json(outdir / "phase_b.json",
                   {"phase": "phase_b", "rows": rows})
    # Caps on deep-win and core, seed 0.
    for cap in CAPS:
        for cell in ("deep-win", "core"):
            row = atomi_row(args, cells[cell], 0, 600.0, "phase-b-cap", 25,
                            extra=["--evals", str(EVALS_B), "--stats",
                                   "--max-depth", str(cap)])
            rows.append(row)
            print(f"phase_b cap={cap:2d} {cell:9s} s0: proved={row['proved']} "
                  f"exit={row['exit']} exp={row['expansions']} "
                  f"unique={row.get('unique_keys')} len={row['length']}",
                  flush=True)
            write_json(outdir / "phase_b.json",
                       {"phase": "phase_b", "rows": rows})
    data = {
        "phase": "phase_b",
        "evals": EVALS_B,
        "memory_mb": MEMORY,
        "seed": 0,
        "caps": CAPS,
        "flags": BARE,
        "rows": rows,
    }
    data["stop_rule"] = stop_rule_b(rows)
    write_json(outdir / "phase_b.json", data)
    return data


def stop_rule_b(rows: list[dict]) -> dict:
    # Bare proof lengths from Phase A (share 25 control): deep-win seed 1 = 25.
    bare_len = {"deep-win": 25, "core": 14}
    bare_unique = {}
    for cell in ("deep-win", "core"):
        r = next((r for r in rows
                  if r["budget"] == "phase-b-bare" and r["cell"] == cell), None)
        bare_unique[cell] = r.get("unique_keys") if r else None
    per_cap = {}
    passing = []
    for cap in CAPS:
        per_cap[str(cap)] = {}
        for cell in ("deep-win", "core"):
            r = next((r for r in rows if r["budget"] == "phase-b-cap"
                      and r["cell"] == cell and r["cap"] == cap), None)
            if r is None:
                continue
            u = r.get("unique_keys")
            base = bare_unique[cell]
            ratio = (base / u) if base and u else None
            reaches = bool(r["proved"]) or (
                r["length"] is not None and r["length"] == bare_len[cell])
            ok = bool(reaches and ratio is not None and ratio >= 2.0)
            per_cap[str(cap)][cell] = {
                "proved": r["proved"], "length": r["length"],
                "unique_keys": u, "bare_unique_keys": base,
                "unique_ratio": round(ratio, 3) if ratio else None,
                "reaches_bare_length": reaches, "passes": ok,
            }
            if ok:
                passing.append((cap, cell))
    return {
        "bare_proof_length": bare_len,
        "bare_unique_keys": bare_unique,
        "per_cap": per_cap,
        "passing": [{"cap": c, "cell": cell} for c, cell in passing],
        "fired": bool(passing),
        "drop_remaining": not passing,
        "rule": ("a cap passes only if it reaches the bare proof length (or "
                 "proves) with >= 2x fewer unique expansions at 20 M"),
    }


EVALS_C = 20_000_000
# The documented divergences, assembled behind existing/new flags.
FULL = ["--guide", "hand", "--guide-group", "--guide-scale", "1",
        "--guide-ef-weight", "0", "--guide-fresh-dn",
        "--guide-replay-exact"]
STACKS = {
    "bare": ["--guide", "none"],
    "hand": ["--guide", "hand"],
    "full": FULL,
    "lo_group": ["--guide", "hand", "--guide-scale", "1",
                 "--guide-ef-weight", "0", "--guide-fresh-dn",
                 "--guide-replay-exact"],
    "lo_efpn": ["--guide", "hand", "--guide-group", "--guide-fresh-dn",
                "--guide-replay-exact"],
    "lo_fresh": ["--guide", "hand", "--guide-group", "--guide-scale", "1",
                 "--guide-ef-weight", "0", "--guide-replay-exact"],
    "lo_replay": ["--guide", "hand", "--guide-group", "--guide-scale", "1",
                  "--guide-ef-weight", "0", "--guide-fresh-dn"],
    "add_group": ["--guide", "hand", "--guide-group"],
    "add_efpn": ["--guide", "hand", "--guide-scale", "1",
                 "--guide-ef-weight", "0"],
    "add_fresh": ["--guide", "hand", "--guide-fresh-dn"],
    "add_replay": ["--guide", "hand", "--guide-replay-exact"],
}


def phase_c(args: argparse.Namespace) -> dict:
    outdir = Path(args.outdir)
    cells = cells_by_name(args)
    rows = []
    for name, flags in STACKS.items():
        for cell in CELLS:
            row = atomi_row(args, cells[cell], 0, 600.0, "phase-c", 25,
                            extra=["--evals", str(EVALS_C), "--stats", *flags])
            row["stack"] = name
            rows.append(row)
            print(f"phase_c {name:10s} {cell:9s} s0: proved={row['proved']} "
                  f"exit={row['exit']} exp={row['expansions']} "
                  f"unique={row.get('unique_keys')} len={row['length']} "
                  f"ms={row['ms']}", flush=True)
            write_json(outdir / "phase_c.json",
                       {"phase": "phase_c", "rows": rows})
    data = {
        "phase": "phase_c",
        "evals": EVALS_C,
        "memory_mb": MEMORY,
        "seed": 0,
        "stacks": STACKS,
        "flags": BARE,
        "rows": rows,
    }
    data["stop_rule"] = stop_rule_c(rows)
    # Seed 1 for the best stack (fewest core unique expansions, excluding
    # bare; ties by stack name for determinism).
    best = data["stop_rule"].get("best_stack")
    if best:
        for cell in CELLS:
            row = atomi_row(args, cells[cell], 1, 600.0, "phase-c-seed1", 25,
                            extra=["--evals", str(EVALS_C), "--stats",
                                   *STACKS[best]])
            row["stack"] = best
            rows.append(row)
            print(f"phase_c {best:10s} {cell:9s} s1: proved={row['proved']} "
                  f"exit={row['exit']} exp={row['expansions']} "
                  f"unique={row.get('unique_keys')} len={row['length']}",
                  flush=True)
            write_json(outdir / "phase_c.json",
                       {"phase": "phase_c", "rows": rows})
    data["rows"] = rows
    write_json(outdir / "phase_c.json", data)
    return data


def stop_rule_c(rows: list[dict]) -> dict:
    bare = next((r for r in rows
                 if r["stack"] == "bare" and r["cell"] == "core"), None)
    bare_unique = bare.get("unique_keys") if bare else None
    per_stack = {}
    best = None
    for name in STACKS:
        r = next((r for r in rows
                  if r["stack"] == name and r["cell"] == "core"), None)
        if r is None:
            continue
        u = r.get("unique_keys")
        ratio = (bare_unique / u) if bare_unique and u else None
        per_stack[name] = {
            "core_unique_keys": u,
            "bare_core_unique_keys": bare_unique,
            "cut": round(ratio, 3) if ratio else None,
            "proved": r["proved"],
            "length": r["length"],
        }
        if name != "bare" and u is not None:
            if best is None or u < per_stack[best]["core_unique_keys"]:
                best = name
    passing = [n for n, v in per_stack.items()
               if n != "bare" and v["cut"] is not None and v["cut"] >= 2.0]
    return {
        "bare_core_unique_keys": bare_unique,
        "per_stack": per_stack,
        "passing": passing,
        "best_stack": best,
        "fired": bool(passing),
        "close_line": not passing,
        "rule": ("no assembled stack cuts core unique expansions by >= 2x "
                 "vs bare at 20 M -> close the hand-guide line"),
    }


def main() -> None:
    args = parse_args()
    outdir = Path(args.outdir)
    outdir.mkdir(parents=True, exist_ok=True)
    if args.phase in ("env", "all"):
        phase_env(args)
    if args.phase in ("phase_a", "all"):
        phase_a(args)
    if args.phase in ("d14", "all"):
        phase_d14(args)
    if args.phase in ("phase_b", "all"):
        phase_b(args)
    if args.phase in ("phase_c", "all"):
        phase_c(args)
    if args.phase in ("d14_c", "all"):
        phase_d14_c(args)


if __name__ == "__main__":
    main()
