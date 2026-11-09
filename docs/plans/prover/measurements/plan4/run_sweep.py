#!/usr/bin/env python3
"""Plan4 calibration sweep and probe driver (stdlib only).

Phases
------
probe  record the plan3/plan4 baselines on the two gate cells
sweep  scale x ef_weight on `core` (50 M evals) with an `easy-win` guard
       (10 M evals), seed 0, 192 MB; pick the Pareto point by expansions

The script runs `atomi` only. The pinned provari oracle is never read; the
reference numbers in the plan are hard-coded here as `PROVARI_NODES`.
"""

from __future__ import annotations

import argparse
import json
import statistics
import subprocess
import time
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[4]
SET = HERE.parent / "plan3" / "guidance-v1.tsv"

PROVARI_NODES = {"core": 342_289, "easy-win": 72_523}
SCALES = [1, 4, 16, 64, 256]
EF_WEIGHTS = [0, 8, 16, 32, 64]


def args() -> argparse.Namespace:
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--phase", required=True, choices=["probe", "sweep"])
    p.add_argument("--atomi", default=str(REPO / "target/release/atomi"))
    p.add_argument("--out", default=str(HERE / "sweep.json"))
    p.add_argument("--memory", type=int, default=192)
    p.add_argument("--core-evals", type=int, default=50_000_000)
    p.add_argument("--guard-evals", type=int, default=10_000_000)
    p.add_argument("--seconds", type=int, default=600)
    p.add_argument("--seed", type=int, default=0)
    p.add_argument("--scales", default=",".join(map(str, SCALES)))
    p.add_argument("--ef-weights", default=",".join(map(str, EF_WEIGHTS)))
    p.add_argument("--cells", default="core,easy-win")
    p.add_argument("--stacks", default="bare=--guide none|hand=--guide hand")
    p.add_argument("--dry-run", action="store_true")
    return p.parse_args()


def load_cells() -> dict[str, str]:
    out = {}
    for raw in SET.read_text().splitlines():
        if not raw.strip() or raw.startswith("name\t"):
            continue
        f = raw.split("\t")
        out[f[0]] = f[1]
    return out


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
        return (last_json(proc.stdout + proc.stderr),
                round(time.perf_counter() - start, 3), proc.returncode)
    except subprocess.TimeoutExpired:
        return {}, round(time.perf_counter() - start, 3), 124


def atomi_cmd(a: argparse.Namespace, fen: str, evals: int,
              flags: list[str], extra: list[str]) -> list[str]:
    return [a.atomi, "prove", fen, "--seconds", str(a.seconds),
            "--memory", str(a.memory), "--evals", str(evals),
            "--seed", str(a.seed), *flags, *extra]


def row(a: argparse.Namespace, cell: str, fen: str, evals: int,
        stack: str, flags: list[str], extra: list[str]) -> dict:
    cmd = atomi_cmd(a, fen, evals, flags, extra)
    if a.dry_run:
        print("RUN", " ".join(cmd))
        return {}
    payload, wall, code = run(cmd, a.seconds)
    return {
        "cell": cell, "stack": stack, "label": " ".join(extra) or "default",
        "seed": a.seed, "evals_cap": evals, "wall_s": wall,
        "returncode": code, "found": bool(payload.get("found")),
        "exit": payload.get("exit"), "length": payload.get("length"),
        "child_evals": payload.get("child_evals"),
        "expansions": payload.get("expansions"),
        "settle_nodes": payload.get("settle_nodes"),
        "sim_nodes": payload.get("sim_nodes"),
        "proof_entries": payload.get("proof_entries"),
        "proof_full": payload.get("proof_full"),
    }


def main() -> int:
    a = args()
    cells = load_cells()
    wanted = [c.strip() for c in a.cells.split(",") if c.strip()]
    stacks = [(s.partition("=")[0], s.partition("=")[2].split())
              for s in a.stacks.split("|") if s]
    rows = []
    if a.phase == "probe":
        for cell in wanted:
            for stack, flags in stacks:
                rows.append(row(a, cell, cells[cell], a.core_evals,
                                stack, flags, []))
        data = {"phase": "probe", "provari_nodes": PROVARI_NODES,
                "memory": a.memory, "evals": a.core_evals,
                "seed": a.seed, "rows": rows}
    else:
        scales = [int(x) for x in a.scales.split(",") if x]
        weights = [int(x) for x in a.ef_weights.split(",") if x]
        for scale in scales:
            for weight in weights:
                flags = ["--guide", "hand", "--guide-scale", str(scale),
                         "--guide-ef-weight", str(weight)]
                label = f"scale={scale},ef_weight={weight}"
                core = row(a, "core", cells["core"], a.core_evals,
                           "hand", flags, [])
                guard = row(a, "easy-win", cells["easy-win"], a.guard_evals,
                            "hand", flags, [])
                if core and guard:
                    core["label"] = guard["label"] = label
                    rows.extend([core, guard])
                if not a.dry_run:
                    print(f"{label}: core={core.get('exit')} "
                          f"{core.get('expansions')} easy={guard.get('exit')} "
                          f"{guard.get('expansions')}", flush=True)
        # Pareto: smallest core expansions among guard cells that prove (or
        # the smallest guard expansions when all are censored).
        pairs = {}
        for r in rows:
            if r:
                pairs.setdefault(r["label"], {})[r["cell"]] = r
        eligible = []
        for label, pair in pairs.items():
            core, guard = pair.get("core", {}), pair.get("easy-win", {})
            if core.get("expansions") is None:
                continue
            ok = bool(guard.get("found")) and (
                guard.get("expansions") is not None
                and guard["expansions"] <= 145_046)
            eligible.append({
                "label": label,
                "core_expansions": core.get("expansions"),
                "easy_expansions": guard.get("expansions"),
                "easy_found": bool(guard.get("found")),
                "easy_pass": ok,
                "core_found": bool(core.get("found")),
            })
        eligible.sort(key=lambda e: (not e["easy_pass"],
                                     e["core_expansions"] or 1 << 60,
                                     e["easy_expansions"] or 1 << 60))
        chosen = eligible[0] if eligible else None
        data = {"phase": "sweep", "memory": a.memory,
                "core_evals": a.core_evals, "guard_evals": a.guard_evals,
                "seed": a.seed, "provari_nodes": PROVARI_NODES,
                "rows": rows, "ranking": eligible, "chosen": chosen}
    Path(a.out).write_text(json.dumps(data, indent=1) + "\n")
    print(json.dumps({"out": a.out, "chosen": data.get("chosen"),
                      "rows": len(rows)}, indent=1))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
