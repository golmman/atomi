#!/usr/bin/env python3
"""Plan 16 item-24 Phase A: proof-tree differential and read-only classify.

Phases
------
env     pin host/toolchain/binary/corpus/oracle
diff    provari hand (10 s) and atomi bare (120 s) proofs for the three
        bench cells; normalise FENs, group by attacker ply, find the first
        divergent ply and the first ply with a different chosen move
classify  run `atomi classify` at the move-divergent positions under bare,
        hand and each mechanism toggle; test the five item-15 predicates
flags   A3: re-run atomi with each approximate flag toggled and record
        whether the first divergent ply moves later (evidence only)
all     env, diff, classify, flags

One process at a time. Proof files live under /tmp/atomi-plan16 and are
never committed; only parsed JSON is written here.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import platform
import subprocess
import sys
import time
from collections import defaultdict
from pathlib import Path

sys.dont_write_bytecode = True

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[4]
ATOMI = REPO / "target/release/atomi"
PROVARI = Path("/tmp/atomi-oracle/provari/target/release/provari")
CORPUS = REPO / "bench/corpus.json"
SCRATCH = Path("/tmp/atomi-plan16")
CELLS = {
    "core": "r1b1kbnr/ppp3pp/n4q2/3pppN1/1P1PP3/8/P1P2PPP/RNBQKB1R w KQkq - 0 1",
    "easy-win": "rnbq1b1r/pp1p1kpp/2p1pp1n/8/3PP1P1/2N2N2/PPP2P1P/R1BQKB1R w KQ - 0 1",
    "deep-win": "rnb1kbnr/pppp2pp/5q2/4ppN1/3P4/8/PPP1PPPP/RNBQKB1R w KQkq - 0 1",
}
SEEDS = {"core": 0, "easy-win": 0, "deep-win": 1}
PROVARI_SECONDS = 10
ATOMI_SECONDS = 120
MEMORY = 192
FLAGS = {
    "hand": ["--guide", "hand"],
    "group": ["--guide", "hand", "--guide-group"],
    "fresh_dn": ["--guide", "hand", "--guide-fresh-dn"],
    "replay_exact": ["--guide", "hand", "--guide-replay-exact"],
    "ef_off": ["--guide", "hand", "--guide-scale", "1", "--guide-ef-weight", "0"],
    "quiet_off": ["--guide", "hand", "--guide-off", "quiet"],
}
CLASSIFY_VARIANTS = {
    "bare": ["--guide", "none"],
    "hand": ["--guide", "hand"],
    "off_quiet": ["--guide", "hand", "--guide-off", "quiet"],
    "fresh_dn": ["--guide", "hand", "--guide-fresh-dn"],
    "off_evasions": ["--guide", "hand", "--guide-off", "evasions"],
    "off_efpn": ["--guide", "hand", "--guide-off", "efpn"],
    "replay_exact": ["--guide", "hand", "--guide-replay-exact"],
}


def parse_args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--phase", choices=["env", "diff", "classify", "flags", "all"],
                   default="all")
    p.add_argument("--atomi", default=str(ATOMI))
    p.add_argument("--provari", default=str(PROVARI))
    p.add_argument("--corpus", default=str(CORPUS))
    p.add_argument("--outdir", default=str(HERE))
    p.add_argument("--scratch", default=str(SCRATCH))
    p.add_argument("--deep-seconds", type=int, default=ATOMI_SECONDS,
                   help="atomi budget for deep-win (default 120; a longer "
                        "proof-extraction budget is recorded as a deviation)")
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
                              timeout=timeout, check=False)
        return (last_json(proc.stdout + proc.stderr),
                round(time.perf_counter() - start, 3),
                proc.stderr.strip(), proc.returncode)
    except subprocess.TimeoutExpired:
        return {}, round(time.perf_counter() - start, 3), "timeout", 124


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
                return line.split(":", 1)[1].strip()
    except OSError:
        pass
    return "unknown"


def norm(fen: str) -> str:
    f = fen.split()
    return " ".join([f[0], f[1], f[2], f[5]])


def side(fen: str) -> str:
    return fen.split()[1]


def fullmove(fen: str) -> int:
    return int(fen.split()[5])


def parse_proof(path: Path) -> tuple[str, list[dict]]:
    lines = [json.loads(line) for line in path.read_text().splitlines() if line.strip()]
    header = lines[0]
    root = header["root"]
    root_full = fullmove(root)
    entries = []
    for line in lines[1:]:
        fen = line["fen"]
        entries.append({
            "fen": fen,
            "norm": norm(fen),
            "move": line["move"],
            "ply": fullmove(fen) - root_full,
        })
    return root, entries


def group(entries: list[dict]) -> dict[int, dict[str, list[str]]]:
    out: dict[int, dict[str, set[str]]] = defaultdict(lambda: defaultdict(set))
    for e in entries:
        out[e["ply"]][e["norm"]].add(e["move"])
    return {p: {f: sorted(m) for f, m in v.items()} for p, v in out.items()}


def first_set_divergence(a: dict[int, dict], b: dict[int, dict]) -> int | None:
    for ply in sorted(set(a) | set(b)):
        if set(a.get(ply, {})) != set(b.get(ply, {})):
            return ply
    return None


def first_move_divergence(a: dict[int, dict], b: dict[int, dict]) -> int | None:
    for ply in sorted(set(a) & set(b)):
        for fen in set(a[ply]) & set(b[ply]):
            if a[ply][fen] != b[ply][fen]:
                return ply
    return None


def ply_summary(a: dict[int, dict], b: dict[int, dict], ply: int,
                raw_a: dict[str, str], raw_b: dict[str, str]) -> dict:
    an = set(a.get(ply, {}))
    bn = set(b.get(ply, {}))
    common = an & bn
    return {
        "ply": ply,
        "provari": len(an),
        "atomi": len(bn),
        "common": len(common),
        "provari_only": sorted(an - bn),
        "atomi_only": sorted(bn - an),
        "common_different_move": [
            {"fen": f, "raw_fen": raw_a.get(f) or raw_b.get(f),
             "provari": sorted(a[ply][f]), "atomi": sorted(b[ply][f])}
            for f in sorted(common) if a[ply][f] != b[ply][f]
        ],
    }


def run_proofs(args: argparse.Namespace) -> dict:
    scratch = Path(args.scratch)
    (scratch / "provari").mkdir(parents=True, exist_ok=True)
    (scratch / "atomi").mkdir(parents=True, exist_ok=True)
    out = {}
    for cell, fen in CELLS.items():
        pv_path = scratch / "provari" / f"{cell}.jsonl"
        at_path = scratch / "atomi" / f"{cell}.jsonl"
        pv_cmd = [args.provari, "prove", fen, "--seconds", str(PROVARI_SECONDS),
                  "--memory", str(MEMORY), "--guide", "hand", "--proofs", str(pv_path)]
        pv, pv_wall, pv_err, pv_rc = run(pv_cmd, PROVARI_SECONDS + 60)
        seconds = args.deep_seconds if cell == "deep-win" else ATOMI_SECONDS
        at_cmd = [args.atomi, "prove", fen, "--seconds", str(seconds),
                  "--memory", str(MEMORY), "--guide", "none", "--seed",
                  str(SEEDS[cell]), "--proof-out", str(at_path)]
        at, at_wall, at_err, at_rc = run(at_cmd, seconds + 60)
        row = {
            "fen": fen,
            "seed": SEEDS[cell],
            "provari": {"cmd": pv_cmd, "result": pv, "wall_s": pv_wall,
                        "rc": pv_rc, "stderr": pv_err},
            "atomi": {"cmd": at_cmd, "result": at, "wall_s": at_wall,
                      "rc": at_rc, "stderr": at_err},
        }
        if pv_path.exists() and pv.get("found"):
            root, entries = parse_proof(pv_path)
            row["provari"]["root"] = root
            row["provari"]["positions"] = len(entries)
            row["provari"]["by_ply"] = group(entries)
            row["provari"]["raw_by_norm"] = {e["norm"]: e["fen"] for e in entries}
        if at_path.exists() and at.get("found"):
            root, entries = parse_proof(at_path)
            row["atomi"]["root"] = root
            row["atomi"]["positions"] = len(entries)
            row["atomi"]["by_ply"] = group(entries)
            row["atomi"]["raw_by_norm"] = {e["norm"]: e["fen"] for e in entries}
        out[cell] = row
    return out


def diff_rows(proofs: dict) -> dict:
    out = {}
    for cell, row in proofs.items():
        pv = row["provari"].get("by_ply")
        at = row["atomi"].get("by_ply")
        if pv is None or at is None:
            out[cell] = {"status": "missing-proof",
                         "provari_found": row["provari"]["result"].get("found"),
                         "atomi_exit": row["atomi"]["result"].get("exit")}
            continue
        set_ply = first_set_divergence(pv, at)
        move_ply = first_move_divergence(pv, at)
        plies = sorted(set(pv) | set(at))
        out[cell] = {
            "status": "ok",
            "root": row["provari"].get("root"),
            "provari_positions": row["provari"]["positions"],
            "atomi_positions": row["atomi"]["positions"],
            "provari_length": row["provari"]["result"].get("mate"),
            "atomi_length": row["atomi"]["result"].get("length"),
            "first_divergent_ply": set_ply,
            "first_move_divergent_ply": move_ply,
            "ply_counts": {str(p): {"provari": len(pv.get(p, {})),
                                    "atomi": len(at.get(p, {}))} for p in plies},
            "first_divergent": ply_summary(
                pv, at, set_ply,
                row["provari"].get("raw_by_norm", {}),
                row["atomi"].get("raw_by_norm", {})) if set_ply is not None else None,
            "first_move_divergent": ply_summary(
                pv, at, move_ply,
                row["provari"].get("raw_by_norm", {}),
                row["atomi"].get("raw_by_norm", {})) if move_ply is not None else None,
        }
    return out


def classify(fen: str, args: list[str], atomi: str, attacker: str) -> dict:
    attacker_arg = "white" if attacker == "w" else "black"
    cmd = [atomi, "classify", fen, "--attacker", attacker_arg] + args
    result, wall, err, rc = run(cmd, 60)
    if rc != 0 or not result:
        return {"error": err or f"rc={rc}", "cmd": cmd}
    return result


def row_by_move(result: dict) -> dict[str, dict]:
    return {m["move"]: m for m in result.get("moves", [])}


def predicate_table(fen: str, ply: int, provari_move: str | None,
                    atomi_move: str | None, attacker: str, atomi: str,
                    atomi_by_ply: dict[int, dict]) -> dict:
    variants = {name: classify(fen, args, atomi, attacker)
                for name, args in CLASSIFY_VARIANTS.items()}
    hand = variants["hand"]
    if "moves" not in hand:
        return {"fen": fen, "ply": ply, "error": hand.get("error", "classify failed")}
    hand_rows = row_by_move(hand)
    quiet_count = sum(1 for m in hand["moves"] if m["class"] == "Quiet")
    legal = hand["legal"]
    in_check = hand["in_check"]
    attacker_node = side(fen) == attacker
    next_norms = set(atomi_by_ply.get(ply + 1, {}))

    def order(name: str, move: str | None) -> int | None:
        if move is None:
            return None
        r = row_by_move(variants[name]).get(move)
        return None if r is None else r["order"]

    def evaluate(move: str | None) -> dict:
        if move is None or move not in hand_rows:
            return {"move": move, "applicable": False}
        r = hand_rows[move]
        base_pn, base_dn = r["base_pn"], r["base_dn"]
        start_pn, start_dn = r["start_pn"], r["start_dn"]
        child_norm = norm(r["child"])
        preds = {
            "quiet_grouping": {
                "holds": r["class"] == "Quiet" and quiet_count >= 2
                         and base_pn == 9 and base_dn == 9,
                "specific": order("hand", move) != order("off_quiet", move),
                "detail": f"class={r['class']} quiet_count={quiet_count} base=({base_pn},{base_dn})",
            },
            "fresh_dn": {
                "holds": base_dn != 1,
                "specific": order("hand", move) != order("fresh_dn", move),
                "detail": f"base_dn={base_dn}",
            },
            "check_evasions": {
                "holds": in_check and base_pn == legal,
                "specific": order("hand", move) != order("off_evasions", move),
                "detail": f"in_check={in_check} base_pn={base_pn} legal={legal}",
            },
            "ef_pn": {
                "holds": start_pn != base_pn or start_dn != base_dn,
                "specific": order("hand", move) != order("off_efpn", move),
                "detail": f"base=({base_pn},{base_dn}) start=({start_pn},{start_dn})",
            },
            "exact_replay": {
                "holds": (not attacker_node) and child_norm not in next_norms,
                "specific": None,
                "detail": f"attacker_node={attacker_node} child_in_next_ply={child_norm in next_norms}",
            },
        }
        return {
            "move": move,
            "applicable": True,
            "class": r["class"],
            "base": [base_pn, base_dn],
            "start": [start_pn, start_dn],
            "order": r["order"],
            "child": r["child"],
            "predicates": preds,
        }

    return {
        "fen": fen,
        "ply": ply,
        "attacker": attacker,
        "provari_move": provari_move,
        "atomi_move": atomi_move,
        "hand_order": {m: r["order"] for m, r in hand_rows.items()},
        "provari": evaluate(provari_move),
        "atomi": evaluate(atomi_move),
        "variants": {name: {"legal": v.get("legal"), "in_check": v.get("in_check")}
                     for name, v in variants.items()},
    }


def classify_rows(diff: dict, proofs: dict, atomi: str) -> dict:
    out = {}
    for cell, d in diff.items():
        if d.get("status") != "ok":
            out[cell] = {"status": d.get("status")}
            continue
        attacker = side(d["root"])
        at_by_ply = {int(k): v for k, v in proofs[cell]["atomi"]["by_ply"].items()}
        positions = []
        move_ply = d["first_move_divergent_ply"]
        if move_ply is not None:
            for item in d["first_move_divergent"]["common_different_move"]:
                pv_moves = item["provari"]
                at_moves = item["atomi"]
                pv_move = pv_moves[0] if len(pv_moves) == 1 else None
                at_move = at_moves[0] if len(at_moves) == 1 else None
                positions.append(predicate_table(
                    item.get("raw_fen") or item["fen"], move_ply, pv_move,
                    at_move, attacker, atomi, at_by_ply))
        # The set-divergent ply is recorded for G2; its positions are not
        # move-divergent nodes, so they are not used for G3.
        out[cell] = {
            "status": "ok",
            "first_divergent_ply": d["first_divergent_ply"],
            "first_move_divergent_ply": move_ply,
            "positions": positions,
        }
    return out


def summarize(classified: dict) -> dict:
    names = ["quiet_grouping", "fresh_dn", "check_evasions", "ef_pn", "exact_replay"]
    out = {}
    for cell, c in classified.items():
        if c.get("status") != "ok":
            out[cell] = {"status": c.get("status")}
            continue
        summary = {n: {"holds": 0, "specific": 0, "positions": 0} for n in names}
        for pos in c["positions"]:
            for side_name in ("provari", "atomi"):
                ev = pos.get(side_name, {})
                if not ev.get("applicable"):
                    continue
                for n in names:
                    p = ev["predicates"][n]
                    summary[n]["positions"] += 1
                    if p["holds"]:
                        summary[n]["holds"] += 1
                    if p["specific"]:
                        summary[n]["specific"] += 1
        out[cell] = summary
    return out


def flag_rows(args: argparse.Namespace, diff: dict) -> dict:
    scratch = Path(args.scratch) / "flags"
    scratch.mkdir(parents=True, exist_ok=True)
    out = {}
    for cell in ("core", "easy-win"):
        fen = CELLS[cell]
        pv = diff.get(cell, {}).get("first_divergent_ply")
        rows = {}
        for name, flags in FLAGS.items():
            path = scratch / f"{cell}-{name}.jsonl"
            cmd = [args.atomi, "prove", fen, "--seconds", str(ATOMI_SECONDS),
                   "--memory", str(MEMORY), "--seed", str(SEEDS[cell]),
                   "--proof-out", str(path)] + flags
            result, wall, err, rc = run(cmd, ATOMI_SECONDS + 60)
            row = {"cmd": cmd, "result": result, "wall_s": wall, "rc": rc,
                   "stderr": err}
            if path.exists() and result.get("found"):
                _, entries = parse_proof(path)
                by_ply = group(entries)
                pv_by_ply = diff[cell].get("provari_by_ply")
                # The diff JSON does not carry the full maps; recompute from
                # the scratch proof files.
                pv_path = Path(args.scratch) / "provari" / f"{cell}.jsonl"
                if pv_path.exists():
                    _, pv_entries = parse_proof(pv_path)
                    pv_by_ply = group(pv_entries)
                if pv_by_ply is not None:
                    row["first_divergent_ply"] = first_set_divergence(pv_by_ply, by_ply)
                    row["first_move_divergent_ply"] = first_move_divergence(pv_by_ply, by_ply)
                row["positions"] = len(entries)
            rows[name] = row
        out[cell] = {"bare_first_divergent_ply": pv, "runs": rows}
    return out


def env(args: argparse.Namespace) -> dict:
    provari_version = subprocess.run([args.provari, "--version"],
                                     capture_output=True, text=True).stdout.strip()
    return {
        "host": platform.node(),
        "cpu": cpu_model(),
        "platform": platform.platform(),
        "python": platform.python_version(),
        "rustc": subprocess.run(["rustc", "--version"], capture_output=True,
                                text=True).stdout.strip(),
        "git_head": git_head(),
        "atomi": str(args.atomi),
        "atomi_sha256": sha256(Path(args.atomi)),
        "provari": str(args.provari),
        "provari_version": provari_version,
        "corpus": str(args.corpus),
        "corpus_sha256": sha256(Path(args.corpus)),
        "cells": CELLS,
        "seeds": SEEDS,
        "provari_seconds": PROVARI_SECONDS,
        "atomi_seconds": ATOMI_SECONDS,
        "deep_win_seconds": args.deep_seconds,
        "memory_mb": MEMORY,
        "flags": FLAGS,
        "classify_variants": CLASSIFY_VARIANTS,
    }


def main() -> int:
    args = parse_args()
    outdir = Path(args.outdir)
    outdir.mkdir(parents=True, exist_ok=True)
    if args.phase in ("env", "all"):
        (outdir / "env.json").write_text(json.dumps(env(args), indent=2) + "\n")
    if args.phase in ("diff", "all"):
        proofs = run_proofs(args)
        diff = diff_rows(proofs)
        (outdir / "diff.json").write_text(json.dumps(diff, indent=2) + "\n")
        # Keep the parsed proof maps for the classify phase in a scratch file.
        Path(args.scratch).mkdir(parents=True, exist_ok=True)
        (Path(args.scratch) / "proofs.json").write_text(json.dumps(proofs) + "\n")
    if args.phase in ("classify", "all"):
        proofs_path = Path(args.scratch) / "proofs.json"
        if not proofs_path.exists():
            print("classify needs the diff phase first", file=sys.stderr)
            return 2
        proofs = json.loads(proofs_path.read_text())
        diff = json.loads((outdir / "diff.json").read_text())
        classified = classify_rows(diff, proofs, args.atomi)
        classified["summary"] = summarize(classified)
        (outdir / "classify.json").write_text(json.dumps(classified, indent=2) + "\n")
    if args.phase in ("flags", "all"):
        diff = json.loads((outdir / "diff.json").read_text())
        flags = flag_rows(args, diff)
        (outdir / "flags.json").write_text(json.dumps(flags, indent=2) + "\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
