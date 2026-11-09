# Prover Plan 5: item 5 (M4) — scaled proof output

Initiative: [`prover`](initiative.md), backlog **item 5 (M4)**. Drafted
2026-11-10 in plan session 5 after [`report4.md`](report4.md), on the plan4
tree. No impl. Item 3 is closed; M4-first is the recorded pivot;
M5 waits for M4's proof path.

## Absolute distance (probes, 2026-11-10)

Goal: geomean wall `< 1.0` vs provari hand, coverage 3/3. `easy-win` proves at
10 s (42× wall), `core` only at 120 s with the auxiliary budget, `deep-win`
not proved.

| Case | Search result | Proof output today |
| --- | --- | --- |
| `easy-win`, chosen 10 s | 121,866 expansions, 6.04 s, length 18 | 2,493 positions, 10,550 proof entries, 210 KB JSONL |
| `core`, aux 120 s (report4) | 278.5 M evals, 59.9 s, length 20 | 14,332 positions, ≈691 k entries; both checkers accept |
| `deep-win`, aux 120 s | censored: 562.7 M evals, 20.3 M expansions, 1.22 M entries | no proof |

No `deep-win` proof exists, so `core` (14,332 positions) is the scale case
and `deep-win` stays a censored stretch cell. The exporter is still the D18
proof table: its 25% `--memory` share refuses inserts at 90% load, and export
then fails `store-full`; that is the M4 stop to remove. `.pvr` does not exist;
`provari check` reads our JSONL.

## Scope

**In:** an event-fed, disk-spooled proof worker with bounded memory and no
proof-table cap; a replay protocol that keeps `--proof-out` trajectory-neutral;
JSONL from the worker; `.pvr` export; tests, measurements and docs.

**Out:** guide/search/value/key/validator changes, default-flag changes,
restarts, portfolio, tablebases, model guide, a goal claim, and any use of
provari code or binaries in the repo.

## Proposed D24

**Proof output is replay-fed; the proof table is not the export source.** A
proved run is followed by a deterministic replay (same FEN, seed, guide
config/params, memory; no wall deadline; `--evals = recorded child_evals +
1`) feeding a bounded worker. The replay must reproduce `child_evals`,
`expansions`, `exit` and `length`, else `replay-mismatch`. The worker's disk
budget (`--proof-budget`, default 256 MiB) is the only output cap;
proof-table refusal does not block export. JSONL stays default; `.pvr` is
opt-in. Owner to confirm.

## Design

1. **Worker (`strategy/worker.rs`).** `WorkerSink` appends 16-byte records
   `(key, mv bits, length, limit)` to a 16 MiB run buffer. A full buffer is
   sorted by `(key, length, limit, mv)`, compacted with the D18 non-dominated
   rule, and written as a run file. At end a k-way merge produces one
   fixed-record file with the same rule (ties by move bits, so any input
   order gives the same file). Peak memory is the run buffer plus one record
   per run, independent of the proof table.
2. **Source.** `FileSource` implements `ProofSource` by binary-searching the
   fixed-record file for `key`. It uses `Mutex<File>` to keep the writer's
   `Sync` contract. `is_full` means the worker budget was exhausted
   (`worker-full`), reported as `proof_worker_bytes` / `proof_worker_runs`.
3. **Replay.** The proving run keeps `NullSink`, so timed trajectories are
   untouched. On `--proof-out` + proved, rerun `search::prove` with the same
   flags and `WorkerSink`, no deadline, and an eval cap one above the
   recorded count. The replay reaches root proof before any cap; assert equal
   counters/exit/length, else fail closed. The proof run cannot change the
   proving run's outcome, even at a wall budget.
4. **Writer.** The existing DFS, cycle check, per-`(position, counter)`
   choice and fail-closed rules stay; it reads `FileSource` when the worker
   is used. `--proof-out` still writes `PATH.tmp` then renames.
5. **`.pvr` (`strategy/pvr.rs`, opt-in `--proof-format pvr`).** Walk with the
   format's settled and "already proven at this counter" rules, so a position
   proven earlier at a counter ≥ the current one writes no move. Header:
   version 1, variant atomic, root FEN, tablebases 0, length, positions; no
   `attacker` field. Move index: checks first, then `(from, to, promotion)`
   by square; unsigned LEB128. Frame is zstd with checksum; `positions`
   equals the numbers written. Add `zstd` only if the licence gate stays
   green; otherwise stop and defer `.pvr` (JSONL is the veto path).

## Measurement

- **P0.** Run the plan4 aux point (`scale 64 / ef 0 / max_sim 1 /
  max_settle 1`) on `guidance-v1` at 10 s × seeds 0–2 with and without
  `--proof-out`; record wins, counters, positions, worker bytes and wall.
  `deep-win` gets one 120 s proof attempt; a censor is recorded.
- **A.** Worker tests: random event streams, multiple runs, domination, byte
  cap, deterministic bytes, binary-search source, table-refusal export.
- **B.** Replay tests: proved timed run vs replay counters identical;
  injected mismatch fails `replay-mismatch`; no-guide counters unchanged;
  non-proved `--proof-out` writes nothing.
- **C.** `core` aux 120 s and `easy-win` chosen 10 s JSONL, both accepted by
  `atomi validate` and `provari check`, D14 counters identical.
- **D.** `.pvr` for both cases, checker-accepted, JSONL as control; then
  `make test-full`/`lint`/licence gate; update docs and `report5.md`.

## Gates

- **G1 scale.** `core` aux 120 s exports a JSONL proof accepted by both
  checkers, with `proof_full` allowed and `worker-full` false; `easy-win`
  10 s exports the same. Worker peak memory and disk are within
  `--proof-budget`; a table-refusal run still exports.
- **G2 `.pvr`.** `easy-win` and `core` `.pvr` files are accepted by
  `provari check`; corrupted-frame and wrong-index controls are `valid:
  false`; JSONL controls still accepted. If the licence gate rejects zstd,
  G2 is reported deferred, not passed.
- **G3 determinism.** The proving run's counters/exit/length are identical
  with and without `--proof-out`; replay counters equal the proving run;
  no-guide counters unchanged from m1. Any mismatch is `replay-mismatch`.
- **G4 soundness.** No refutation/never-prove; every claimed proof accepted
  by both checkers; `--proof-out` writes nothing on non-proved runs.
- **G5 build/docs.** `make test`, `make lint`, `make test-full` green; no
  GPL-family licence; parsed data and README under `measurements/plan5/`;
  no raw transcripts, provari artifacts or `__pycache__`.

## Tasks (stop only at a phase boundary)

- **P0** probes and `measurements/plan5/{probes.json,README.md,env.json}`;
  `run_m4.py` (stdlib only) for the guidance runs.
- **A** `strategy/worker.rs`, CLI `--proof-budget`, `FileSource`, tests.
- **B** replay path in `main.rs`/`cli.rs`, failure classes, D14 tests.
- **C** scaled JSONL gate on `core`/`easy-win`, table-refusal control.
- **D** `.pvr` writer + zstd licence check, checker controls; then docs and
  `report5.md`. If G1 fails, stop before `.pvr` and report the pivot.

## Risks

| Risk | Mitigation |
| --- | --- |
| Replay changes trajectory or doubles a hot run | Replay only on `--proof-out` + proved; eval guard; counter equality gate |
| Worker spill is slow or disk-bound | Bounded runs, k-way merge, `--proof-budget`; measure bytes and wall |
| `.pvr` walk differs from the checker's settled rule | Dedicated walk state, `positions` count, checker controls |
| zstd brings a GPL-family transitive dependency | Licence gate first; defer `.pvr` rather than weaken the gate |
| Proof table full during a proving search | Worker gets all events; table-refusal export test |
| `deep-win` scale unavailable | `core` is the scale case; record the censor |

## Report and deliverables

`report5.md`: headline, provenance, commands, probe/coverage table, worker
bytes, replay evidence, JSONL/`.pvr` verdicts, gates, not measured, problems,
missing tests, next steps and a closure block. State whether M4 is done and
whether M5 can start.

- **Engine:** `src/strategy/{worker.rs,pvr.rs,mod.rs,write.rs}`,
  `src/{cli.rs,main.rs}`, `Cargo.toml`/`Cargo.lock`, tests.
- **Measurements:** `measurements/plan5/{run_m4.py,README.md,env.json,
  probes.json,coverage.json,proofs.json,worker.json}`.
- **Docs:** `initiative.md` (D24, status, history), `bench.md` (M4 row; no
  goal claim), `gotchas.md`/`AGENTS.md` if needed, `report5.md`.
