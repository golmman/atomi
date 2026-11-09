#!/usr/bin/env python3
"""Guidance census, frozen-set gate and both-direction ablations (plan3).

Stdlib only. The census runs the pinned provari oracle as an external
process; no provari artifact is read into the repository. The gate and
ablation phases read a frozen ``guidance-v1.tsv`` and refuse a hash mismatch
against ``env.json``.

Phases
------
census    run provari --guide hand on every candidate (30 s, 60 s retry)
freeze    select the first >=20 shared candidates into guidance-v1.tsv
gate      bare/hand stacks x seeds x cells (deterministic eval cap)
wall      core/easy-win x seeds x repeats (10 s product budget)
ablation  leave-one-out and add-one-in over the frozen set
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import statistics
import subprocess
import sys
import time
from pathlib import Path

sys.dont_write_bytecode = True

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[3]
SHORT = {"core": "core", "1eb80b92f92c": "easy-win", "db6a741c723f": "deep-win"}
MECHANISMS = [
    "classes", "evasions", "quiet", "efpn", "closures",
    "priors", "history", "sim", "replay",
]


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--phase", required=True,
                   choices=["census", "freeze", "gate", "wall", "ablation"])
    p.add_argument("--provari")
    p.add_argument("--provari-commit", default="unknown")
    p.add_argument("--atomi")
    p.add_argument("--candidates", default="")
    p.add_argument("--set", default=str(HERE / "guidance-v1.tsv"))
    p.add_argument("--census", default=str(HERE / "census.json"))
    p.add_argument("--out", default=str(HERE / "guidance.json"))
    p.add_argument("--freeze-out", default=str(HERE / "guidance-v1.tsv"))
    p.add_argument("--seconds", type=float, default=30.0)
    p.add_argument("--retry-seconds", type=float, default=60.0)
    p.add_argument("--memory", type=int, default=192)
    p.add_argument("--evals", type=int, default=None)
    p.add_argument("--seeds", default="0,1,2,3")
    p.add_argument("--repeats", type=int, default=3)
    p.add_argument("--cells", default="")
    p.add_argument("--stack", action="append", default=[],
                   metavar="NAME=FLAGS")
    p.add_argument("--leave-one-out", default="")
    p.add_argument("--add-one-in", default="")
    p.add_argument("--proof-check", action="store_true")
    p.add_argument("--provari-check")
    p.add_argument("--proof-dir", default="/tmp/atomi-plan3-proofs")
    p.add_argument("--dry-run", action="store_true")
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


def run(cmd: list[str], timeout: float) -> tuple[dict, float, int]:
    start = time.perf_counter()
    try:
        proc = subprocess.run(cmd, capture_output=True, text=True,
                              timeout=timeout + 60, check=False)
        wall = time.perf_counter() - start
        return last_json(proc.stdout + proc.stderr), wall, proc.returncode
    except subprocess.TimeoutExpired:
        return {}, time.perf_counter() - start, 124


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def candidates(args: argparse.Namespace) -> list[dict]:
    seen: set[str] = set()
    rows: list[dict] = []
    for spec in [s for s in args.candidates.split(",") if s]:
        path = Path(spec)
        if path.suffix == ".json":
            data = json.loads(path.read_text())
            for case in [data["core"], *data["sample20"]]:
                if case["fen"] in seen:
                    continue
                seen.add(case["fen"])
                rows.append({"name": case["id"], "fen": case["fen"],
                             "source": path.name})
        else:
            for raw in path.read_text().splitlines():
                if not raw.strip() or raw.startswith("#") or raw.startswith("name\t"):
                    continue
                f = raw.split("\t")
                if len(f) < 2 or f[1] in seen:
                    continue
                seen.add(f[1])
                rows.append({"name": f[0], "fen": f[1], "source": path.name})
    return rows


def order_key(row: dict, index: int) -> tuple[int, int]:
    name = SHORT.get(row["name"], row["name"])
    if name == "core":
        return (0, 0)
    if name == "easy-win":
        return (1, 0)
    if name == "deep-win":
        return (2, 0)
    if row["source"] == "corpus.json":
        return (3, index)
    return (4, index)


def census(args: argparse.Namespace) -> dict:
    rows = []
    for i, row in enumerate(candidates(args)):
        budget = args.seconds
        cmd = [args.provari, "prove", row["fen"], "--seconds", str(int(budget)),
               "--memory", str(args.memory), "--guide", "hand"]
        if args.dry_run:
            print("CENSUS", " ".join(cmd))
            continue
        payload, wall, code = run(cmd, budget)
        used = budget
        if not payload.get("found"):
            used = args.retry_seconds
            cmd[3] = str(int(used))
            payload, wall, code = run(cmd, used)
        rows.append({
            **row, "index": i, "found": bool(payload.get("found")),
            "nodes": payload.get("nodes"), "ms": payload.get("ms"),
            "wall_s": round(wall, 3), "returncode": code,
            "selection_budget_s": used,
        })
        print(f"[{i}] {row['name']} found={payload.get('found')} "
              f"nodes={payload.get('nodes')} budget={used}", flush=True)
    return {"provari_commit": args.provari_commit, "rows": rows}


def freeze(census_path: Path, out_path: Path) -> list[dict]:
    data = json.loads(census_path.read_text())["rows"]
    shared = [r for r in data if r.get("found")]
    shared.sort(key=lambda r: order_key(r, r.get("index", 0)))
    take = shared[:24]
    if len(take) < 20:
        raise SystemExit(f"only {len(take)} shared candidates (need >=20)")
    lines = ["name\tfen\tsource\tref_nodes\tref_wall_s\tselection_budget_s"]
    for r in take:
        lines.append("\t".join([
            SHORT.get(r["name"], r["name"]), r["fen"], r["source"],
            str(r["nodes"]), str(r["wall_s"]), str(r["selection_budget_s"]),
        ]))
    out_path.write_text("\n".join(lines) + "\n")
    return take


def load_set(args: argparse.Namespace) -> list[dict]:
    path = Path(args.set)
    rows = []
    for raw in path.read_text().splitlines():
        if not raw.strip() or raw.startswith("name\t"):
            continue
        f = raw.split("\t")
        rows.append({"name": f[0], "fen": f[1], "source": f[2],
                     "ref_nodes": int(f[3]), "ref_wall_s": float(f[4]),
                     "selection_budget_s": float(f[5])})
    env = HERE / "env.json"
    if env.exists():
        expected = json.loads(env.read_text()).get("guidance_sha256")
        if expected and sha256(path) != expected:
            raise SystemExit("guidance set hash mismatch (freeze changed)")
    return rows


def split_stacks(specs: list[str]) -> list[tuple[str, list[str]]]:
    return [(s.partition("=")[0], s.partition("=")[2].split()) for s in specs]


def reference(args: argparse.Namespace, cells: list[dict], repeats: int,
              budgeted: bool) -> dict:
    out = {}
    for cell in cells:
        runs = []
        budget = (max(30.0, cell["selection_budget_s"])
                  if budgeted else args.seconds)
        for _ in range(repeats):
            cmd = [args.provari, "prove", cell["fen"],
                   "--seconds", str(int(budget)),
                   "--memory", str(args.memory), "--guide", "hand"]
            if args.dry_run:
                print("REF", " ".join(cmd))
                continue
            payload, wall, code = run(cmd, budget)
            runs.append({"wall_s": round(wall, 3), "win": bool(payload.get("found")),
                         "nodes": payload.get("nodes"), "mate": payload.get("mate"),
                         "returncode": code})
        out[cell["name"]] = {
            "runs": runs,
            "win": all(r["win"] for r in runs) if runs else False,
            "wall_s": statistics.median(r["wall_s"] for r in runs) if runs else None,
            "nodes": next((r["nodes"] for r in runs if r.get("nodes")), None),
        }
    return out


def atomi_rows(args: argparse.Namespace, cells: list[dict],
               stacks: list[tuple[str, list[str]]], seeds: list[int],
               repeats: int) -> list[dict]:
    rows = []
    for cell in cells:
        for name, flags in stacks:
            for seed in seeds:
                for rep in range(repeats):
                    cmd = [args.atomi, "prove", cell["fen"],
                           "--seconds", str(int(args.seconds)),
                           "--memory", str(args.memory),
                           "--seed", str(seed), *flags]
                    if args.evals:
                        cmd += ["--evals", str(args.evals)]
                    if args.dry_run:
                        print("CAND", " ".join(cmd))
                        continue
                    payload, wall, code = run(cmd, args.seconds)
                    rows.append({
                        "cell": cell["name"], "stack": name, "seed": seed,
                        "repeat": rep + 1, "wall_s": round(wall, 3),
                        "win": bool(payload.get("found")),
                        "exit": payload.get("exit"),
                        "length": payload.get("length"),
                        "child_evals": payload.get("child_evals"),
                        "expansions": payload.get("expansions"),
                        "settle_nodes": payload.get("settle_nodes"),
                        "sim_nodes": payload.get("sim_nodes"),
                        "returncode": code,
                    })
    return rows


def summarize(cells: list[dict], rows: list[dict], ref: dict,
              stacks: list[tuple[str, list[str]]], seeds: list[int],
              seconds: float) -> dict:
    summary = {}
    for name, _ in stacks:
        per_cell = {}
        for cell in cells:
            group = [r for r in rows if r["stack"] == name
                     and r["cell"] == cell["name"]]
            if not group:
                continue
            wins = [r for r in group if r["win"]]
            med = lambda k: (statistics.median(r[k] for r in group)
                             if all(r.get(k) is not None for r in group) else None)
            entry = {
                "runs": len(group), "wins": len(wins),
                "win_all": len(wins) == len(group) and bool(group),
                "wall_s": med("wall_s"),
                "expansions": med("expansions"),
                "child_evals": med("child_evals"),
                "settle_nodes": med("settle_nodes"),
                "sim_nodes": med("sim_nodes"),
                "by_seed": {},
            }
            for seed in seeds:
                sg = [r for r in group if r["seed"] == seed]
                if sg:
                    entry["by_seed"][str(seed)] = {
                        "win": all(r["win"] for r in sg),
                        "wall_s": statistics.median(r["wall_s"] for r in sg),
                        "expansions": statistics.median(
                            r["expansions"] for r in sg
                            if r["expansions"] is not None),
                        "child_evals": statistics.median(
                            r["child_evals"] for r in sg
                            if r["child_evals"] is not None),
                    }
            if ref[cell["name"]]["win"] and entry["win_all"]:
                entry["wall_factor"] = round(
                    entry["wall_s"] / ref[cell["name"]]["wall_s"], 3)
            elif ref[cell["name"]]["win"]:
                entry["wall_factor_lower_bound"] = round(
                    seconds / ref[cell["name"]]["wall_s"], 2)
            per_cell[cell["name"]] = entry
        summary[name] = per_cell
    return summary


def gate(summary: dict, seeds: list[int]) -> dict:
    easy = summary.get("hand", {}).get("easy-win")
    core = summary.get("hand", {}).get("core")
    easy_ok = bool(easy and all(
        easy["by_seed"][str(s)]["win"]
        and easy["by_seed"][str(s)]["expansions"] is not None
        and easy["by_seed"][str(s)]["expansions"] <= 145_046
        for s in seeds))
    core_ok = bool(core and all(
        core["by_seed"][str(s)]["win"] for s in seeds))
    return {
        "easy_win_expansions_limit": 145_046,
        "easy_win": easy,
        "core": core,
        "easy_win_pass": easy_ok,
        "core_pass": core_ok,
        "pass": easy_ok and core_ok,
    }


def main() -> int:
    args = parse_args()
    seeds = [int(s) for s in args.seeds.split(",") if s]
    if args.phase == "census":
        data = census(args)
        Path(args.out).write_text(json.dumps(data, indent=1) + "\n")
        return 0
    if args.phase == "freeze":
        take = freeze(Path(args.census), Path(args.freeze_out))
        print(json.dumps({"kept": len(take),
                          "sha256": sha256(Path(args.freeze_out))}, indent=1))
        return 0

    cells = load_set(args)
    if args.phase == "ablation":
        wanted = [c.strip() for c in args.cells.split(",") if c.strip()]
        if wanted:
            cells = [c for c in cells if c["name"] in wanted]
        stacks = [("full", ["--guide", "hand"])]
        for m in [x for x in args.leave_one_out.split(",") if x]:
            stacks.append((f"lo_{m}", ["--guide", "hand", "--guide-off", m]))
        for m in [x for x in args.add_one_in.split(",") if x]:
            stacks.append((f"ai_{m}", ["--guide", "none", "--guide-on", m]))
    else:
        stacks = split_stacks(args.stack) or [("hand", ["--guide", "hand"])]
        wanted = [c.strip() for c in args.cells.split(",") if c.strip()]
        if wanted:
            cells = [c for c in cells if c["name"] in wanted]
    if args.phase == "wall":
        cells = [c for c in cells if c["name"] in ("core", "easy-win")]

    budgeted = args.phase in ("gate", "ablation")
    ref_repeats = 1 if budgeted else args.repeats
    if args.dry_run:
        reference(args, cells, ref_repeats, budgeted)
        atomi_rows(args, cells, stacks, seeds, args.repeats)
        return 0
    ref = reference(args, cells, ref_repeats, budgeted)
    rows = atomi_rows(args, cells, stacks, seeds, args.repeats)
    if args.proof_check and args.provari_check:
        Path(args.proof_dir).mkdir(parents=True, exist_ok=True)
        for row in rows:
            if not row["win"] or row["seed"] != seeds[0]:
                continue
            path = Path(args.proof_dir) / (
                f"{row['cell']}-{row['stack']}-s{row['seed']}.jsonl")
            flags = dict(stacks)[row["stack"]]
            cmd = [args.atomi, "prove", next(c["fen"] for c in cells
                                             if c["name"] == row["cell"]),
                   "--seconds", "600", "--memory", str(args.memory),
                   "--seed", str(row["seed"]), *flags,
                   "--proof-out", str(path)]
            payload, _, _ = run(cmd, 600)
            row["proof_run"] = payload
            if payload.get("exit") != "proved":
                row["proof_verdict"] = "not-proved"
                continue
            if payload.get("child_evals") != row["child_evals"]:
                row["proof_verdict"] = "nondeterministic"
                continue
            ours = last_json(subprocess.run(
                [args.atomi, "validate", str(path)],
                capture_output=True, text=True).stdout)
            chk = last_json(subprocess.run(
                [args.provari_check, "check", str(path)],
                capture_output=True, text=True).stdout)
            if not ours.get("valid"):
                row["proof_verdict"] = "validator-reject"
            elif chk.get("valid") is None:
                row["proof_verdict"] = "unchecked"
            elif not chk.get("valid"):
                row["proof_verdict"] = "checker-reject"
            elif ours.get("positions") != chk.get("won_positions"):
                row["proof_verdict"] = "disagreement"
            else:
                row["proof_verdict"] = "accepted"

    summary = summarize(cells, rows, ref, stacks, seeds, args.seconds)
    out = {
        "phase": args.phase,
        "provari_commit": args.provari_commit,
        "guidance_sha256": sha256(Path(args.set)),
        "seeds": seeds, "seconds": args.seconds, "memory": args.memory,
        "evals": args.evals,
        "reference": ref, "rows": rows, "summary": summary,
    }
    if args.phase in ("gate", "wall"):
        out["gate"] = gate(summary, seeds)
    Path(args.out).write_text(json.dumps(out, indent=1) + "\n")
    print(json.dumps({"phase": args.phase, "out": args.out,
                      "gate": out.get("gate")}, indent=1))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
