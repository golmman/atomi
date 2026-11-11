#!/usr/bin/env python3
"""Plan 18 item-25: DF-PN exactness stack (simulation + null-threat replay).

Phases
------
env     pin host/toolchain/binary/corpus/oracle and the Phase C stacks
diff    Phase A read-only mechanism differential (no search): the portable
        exactness gaps between provari 0.1.0 and atomi.
probe   a single 20 M isolation run of the combined stack on deep-win, with
        the parsed counters (the G2 applicability probe's numeric record).
matrix  Phase C: bare, hand, sim_exact, null_exact and sim+null on
        core/easy-win/deep-win at 20 M child evals, seed 0, --stats. The G3
        gate is computed from the seed-0 rows.
d14     an untimed --seconds 600 --proof-out run for every proved row;
        require equal child_evals and both checkers.
all     env, diff, probe, matrix, d14

One process at a time. Proof files live under /tmp/atomi-plan18 and are
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
DEFAULT_PROVARI_SRC = Path("/tmp/atomi-oracle/provari")
DEFAULT_CORPUS = REPO / "bench/corpus.json"
SCRATCH = Path("/tmp/atomi-plan18")

CELLS = {
    "core": "r1b1kbnr/ppp3pp/n4q2/3pppN1/1P1PP3/8/P1P2PPP/RNBQKB1R w KQkq - 0 1",
    "easy-win": "rnbq1b1r/pp1p1kpp/2p1pp1n/8/3PP1P1/2N2N2/PPP2P1P/R1BQKB1R w KQ - 0 1",
    "deep-win": "rnb1kbnr/pppp2pp/5q2/4ppN1/3P4/8/PPP1PPPP/RNBQKB1R w KQkq - 0 1",
}
STACKS = {
    "bare": ["--guide", "none"],
    "hand": ["--guide", "hand"],
    "sim_exact": ["--guide", "hand", "--guide-sim-exact"],
    "null_exact": ["--guide", "hand", "--guide-null-exact"],
    "sim_null": ["--guide", "hand", "--guide-sim-exact", "--guide-null-exact"],
}
EVALS = 20_000_000
MEMORY = 192
SEED = 0
BEST_SECONDS = 600.0
D14_SECONDS = 600.0


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--phase", choices=["env", "diff", "probe", "matrix", "d14",
                                       "all"], default="all")
    p.add_argument("--atomi", default=str(DEFAULT_ATOMI))
    p.add_argument("--provari", default=str(DEFAULT_PROVARI))
    p.add_argument("--provari-src", default=str(DEFAULT_PROVARI_SRC))
    p.add_argument("--corpus", default=str(DEFAULT_CORPUS))
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
        return "dirty (plan18 execution; owner commits)" if out else "clean"
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


def provari_version(args: argparse.Namespace) -> str:
    pv = subprocess.run([args.provari, "--version"], capture_output=True,
                        text=True)
    return (pv.stdout or pv.stderr).strip()


def env(args: argparse.Namespace) -> dict:
    atomi = Path(args.atomi)
    corpus = Path(args.corpus)
    data = {
        "plan": "plan18",
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
        "provari": args.provari,
        "provari_version": provari_version(args),
        "provari_source": args.provari_src,
        "cells": CELLS,
        "stacks": STACKS,
        "evals": EVALS,
        "memory_mb": MEMORY,
        "seed": SEED,
        "isolation_seconds": BEST_SECONDS,
        "d14_seconds": D14_SECONDS,
        "note": ("--seconds 600 only guards the wall; --evals 20000000 is the "
                 "budget, as in plan13/plan17 phase C"),
    }
    write_json(Path(args.outdir) / "env.json", data)
    return data


def diff(args: argparse.Namespace) -> dict:
    """Phase A: the read-only mechanism differential.

    The provari readings are from the pinned scratch clone; only their
    structure is recorded (no source text is copied). The atomi readings name
    the local functions that implement the corresponding path.
    """
    data = {
        "plan": "plan18",
        "phase": "A-differential",
        "provari": {
            "version": provari_version(args),
            "source": args.provari_src,
            "sibling_simulation": {
                "site": "dfpn.rs mid(), after a defence is proved",
                "guard": "!attacking && value.proof == 0; sibling open and not already simulated",
                "guide": "the just-proved defence position, looked up in the proof table",
                "budget": "a fixed simulation counter per sibling",
                "success": "the sibling child becomes WON; the replay proofs are kept",
            },
            "simulate": {
                "attacker_move": "the guide's stored proof move, or the guide's immediate explosion / dropped mate",
                "defences": "every legal reply; a reply that explodes the attacker fails, an exposing reply is a one-move proof, a stored proof is reused, else the same reply in the guide is replayed",
                "exposing_shortcut": "moves::exploding_capture(next, attacker) -> (1, LAST)",
                "cycle": "the search path and the replay line are both checked",
            },
            "replay_threat": {
                "site": "dfpn.rs mid(), defender node, no known value, not in check",
                "null": "pos.null(): side to move passed, en passant cleared",
                "win": "immediate explosion, a check every evasion loses, a double threat no defence parries, or a move into a known proof",
                "replay": "the null proof is replayed on every open defence via simulate()",
                "miss": "a fresh defence starts at half its proof number",
            },
        },
        "atomi": {
            "sibling_simulation": {
                "site": "src/search/mod.rs replay_siblings(), src/guide/replay.rs",
                "guard": "defender node, proved child, replay_exact or sim_exact",
                "guide": "the just-proved child position, looked up in the proof table",
                "budget": "REPLAY_BUDGET",
                "success": "the sibling child becomes Proved; the replay proofs are stored and emitted",
            },
            "greedy_simulation": {
                "site": "src/guide/simulate.rs, src/search/frame.rs pass 1",
                "what": "a bounded 6-ply greedy playout that adjusts fresh start numbers; no proof",
            },
            "null_replay": {
                "site": "src/guide/threat.rs Guide::replay, src/search/frame.rs pass 1",
                "what": "a 64-bit explosion-threat signature; parried/created only lowers the start pair via UNLIKELY",
                "null_position": "never built; the signature is a pure function of the board",
            },
        },
        "gaps": [
            {
                "id": "G2-a",
                "name": "exact null-threat replay",
                "provari": "build the passed position, find a winning move, replay it on every open defence, lower a fresh parrying defence",
                "atomi": "signature-only prior; no null proof and no replay",
                "portability": "same DF-PN base and the same exact replay module; no new search algorithm",
                "portable": True,
            },
            {
                "id": "G2-b",
                "name": "exposing one-move shortcut",
                "provari": "simulate_defences treats a defence after which the attacker can explode at once as a one-move proof",
                "atomi": "replay::simulate_defences had no such shortcut; it needed a stored proof or the guide's reply",
                "portability": "a local branch in guide/replay.rs using threat::immediate_explosion",
                "portable": True,
            },
            {
                "id": "G2-c",
                "name": "guide explosion / dropped mate fallback",
                "provari": "simulate falls back to the guide's immediate explosion or dropped mate when the guide has no stored proof",
                "atomi": "simulate requires a stored guide proof",
                "portability": "local, but only useful once the null guide proof is supplied",
                "portable": True,
            },
            {
                "id": "G2-d",
                "name": "top-level guide proof override",
                "provari": "the null proof is stored in the table and looked up like any other",
                "atomi": "the null position must not be keyed or stored, so the top guide move is passed explicitly",
                "portability": "a new simulate_with_guide_proof entry point",
                "portable": True,
            },
        ],
        "g2": {
            "pass": True,
            "rule": "at least one concrete exactness gap portable without changing the DF-PN base",
            "verdict": "G2-a..G2-d are portable on the existing DF-PN and replay base; Phase B is authorised",
        },
        "not_ported": [
            "provari's check-every-evasion and double-threat null-win cases (only the immediate explosion, stored proof and exposed mate-in-two are ported)",
            "provari's recomputed proofs (atomi has no recomputed proof table)",
        ],
    }
    write_json(Path(args.outdir) / "diff.json", data)
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
    return {
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


def print_row(row: dict) -> None:
    print(f"matrix {row['stack']:12s} {row['cell']:9s} s{row['seed']}: "
          f"proved={row['proved']} exit={row['exit']} "
          f"unique={row['unique_keys']} exp={row['expansions']} "
          f"evals={row['child_evals']} sim={row['sim_nodes']} "
          f"aux={row['aux_nodes']} len={row['length']}", flush=True)


def probe(args: argparse.Namespace) -> dict:
    row = prove_row(args, "deep-win", "sim_null", SEED)
    print_row(row)
    data = {
        "plan": "plan18",
        "phase": "probe",
        "purpose": ("a single parsed 20 M run of the combined stack on the "
                    "gate cell; the mechanism applicability is read from the "
                    "sim/aux counters and the unique-expansion trajectory"),
        "row": row,
    }
    write_json(Path(args.outdir) / "probe.json", data)
    return data


def summarize(rows: list[dict]) -> dict:
    by = {(r["stack"], r["cell"]): r for r in rows if r["seed"] == SEED}
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
    combined = by.get(("sim_null", "deep-win"))
    cut = None
    if base.get("deep-win") and combined and combined.get("unique_keys"):
        cut = round(base["deep-win"] / combined["unique_keys"], 3)
    regressions = {}
    for cell in ("core", "easy-win"):
        row = by.get(("sim_null", cell))
        if base.get(cell) and row and row.get("unique_keys"):
            regressions[cell] = round(row["unique_keys"] / base[cell], 3)
        else:
            regressions[cell] = None
    no_regression = all(r is not None and r <= 1.10
                        for r in regressions.values())
    g3 = bool(cut is not None and cut >= 2.0 and no_regression)
    # Best individual mechanism for the ablation record.
    best = None
    for stack in ("sim_exact", "null_exact", "hand"):
        row = by.get((stack, "deep-win"))
        if row and row.get("unique_keys"):
            if best is None or row["unique_keys"] < best[1]:
                best = (stack, row["unique_keys"])
    return {
        "bare_unique_keys": base,
        "unique_ratio_vs_bare": ratios,
        "deep_win_cut_sim_null": cut,
        "core_easy_regression_sim_null": regressions,
        "best_individual_deep_win": best[0] if best else None,
        "best_individual_deep_win_unique": best[1] if best else None,
        "g3_pass": g3,
        "g3_rule": ("sim_null cuts deep-win unique expansions >= 2x vs bare "
                    "at 20 M, with no core/easy-win regression > 10 %"),
    }


def matrix(args: argparse.Namespace) -> dict:
    rows = []
    for stack in STACKS:
        for cell in CELLS:
            row = prove_row(args, cell, stack, SEED)
            rows.append(row)
            print_row(row)
            write_json(Path(args.outdir) / "matrix.json",
                       {"phase": "matrix", "rows": rows})
    summary = summarize(rows)
    data = {
        "phase": "matrix",
        "evals": EVALS,
        "memory_mb": MEMORY,
        "seed": SEED,
        "stacks": STACKS,
        "cells": CELLS,
        "rows": rows,
        "summary": summary,
    }
    write_json(Path(args.outdir) / "matrix.json", data)
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
    proof_dir = Path(args.scratch) / "proofs"
    proof_dir.mkdir(parents=True, exist_ok=True)
    data = json.loads((outdir / "matrix.json").read_text())
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
        write_json(outdir / "matrix.json", {**data, "d14": d14_rows})
    if proof_dir.exists() and not any(proof_dir.iterdir()):
        proof_dir.rmdir()
    data["d14"] = d14_rows
    data["d14_all_accepted"] = bool(d14_rows) and all(
        d["verdict"] == "accepted" for d in d14_rows)
    write_json(outdir / "matrix.json", data)
    return data


def main() -> int:
    args = parse_args()
    outdir = Path(args.outdir)
    outdir.mkdir(parents=True, exist_ok=True)
    if args.phase in ("env", "all"):
        env(args)
    if args.phase in ("diff", "all"):
        data = diff(args)
        print(json.dumps(data["g2"], indent=1), flush=True)
    if args.phase in ("probe", "all"):
        probe(args)
    if args.phase in ("matrix", "all"):
        data = matrix(args)
        print(json.dumps(data["summary"], indent=1), flush=True)
    if args.phase in ("d14", "all"):
        d14(args)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
