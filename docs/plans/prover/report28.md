# Report 28 — item 35: HOTPATH-V1 misses G-H1; the per-node line closes

Plan: [`plan28.md`](plan28.md). Data:
[`measurements/plan28/`](measurements/plan28/README.md) (`probe.py`,
`gate_h1.json`, `phase_a.json`, `env.json`, README). Decisions: D2, D11,
D14, D21, D24, D26, D30, D34, D36, D38, D39, proposed D40. Host: Ryzen 9
5950X (4 vCPU container), one process at a time. Repo clean at
`atomi@4ac58a9`.

## 0. Verdict

**G-H1 missed: 0.8206× control (1.219×), against the declared ≤ 0.667×
(≥ 1.5×).** The tree was identical in every run: `child_evals` 5,000,034 and
`expansions` 168,910 for the control, the S3-only build and the S1+S2+S3
candidate. Under the plan's stop rule the per-node micro-line is now
**closed for good**; S4 was not started, the 120 s matrix and D14 were not
run, and no `bench.md` row is added. The next substantive plan is the
formulation decision (PDS-PN/PN² or a goal re-pin under D30).

The engine stays at **`T_3`**. `T_4` distance is unchanged from report27:
`deep-win` 71.7 s → 10 s (**7.17×**), or 7.74× on the same-session Cap-120
row (77.4 s). The measured 1.219× would project roughly 63.5 s (≈ 6.4×) if
adopted, but it is not adopted.

## 1. What was built (measurement only)

Nothing from the candidate is committed. The repo is restored to the
published `atomic-movegen 2.4.0` dependency.

- **MG1 `Board::null_for(attacker)`** in a local `atomic-movegen` checkout:
  clone, set side to move, clear en passant, derive the game ply from the
  preserved fullmove number, recompute the hash. A differential test against
  the old `fen_with(Legal)` → split/join → `from_fen` path passes on six
  fixtures and both attackers, comparing FEN, hash, legal-ep hash, side, ep,
  clock, game ply and castling rights.
- **MG2 public `for_each_pseudo_legal`**: the existing internal callback was
  made public; order is the existing `generate_pseudo_legal` order and the
  upstream golden move-order/perft suite stays green.
- **S1**: `guide::null::null_board` now calls `board.null_for(attacker)`.
- **S2**: `guide::threat::immediate_explosion_bits` and
  `guide::null::exposed_limit_uncached` walk the callback stream and stop at
  the first accepted/rejected candidate; no `MoveList` is materialised.
- **S3**: `Tables::lookup` probes the proof table, decodes a hit, then probes
  the TT without constructing `EntrySource`; `lookup_with_source` keeps the
  `--stats` attribution path.

**S2 does not advance `null_win`'s own legal list.** `null_win` sets
`work.moves` to the *full* legal move count, and that number feeds the
auxiliary budget in `Engine::null_replay`; early exit would change the budget
and therefore the tree. This is now a binding gotcha (see `gotchas.md`).
Routing only `exposed_limit_uncached` is tree-identical.

## 2. G-H1 (5 M child evals, `deep-win`, seed 0)

Command shape: `atomi prove <deep-win> --seconds 600 --memory 192 --evals
5000000 --seed 0 --guide hand --stats`, three repeats per side, alternating,
one process at a time.

| Config | `child_evals` | Expansions | Median wall | Ratio | Speedup |
| --- | ---: | ---: | ---: | ---: | ---: |
| control (published 2.4.0) | 5,000,034 | 168,910 | 2.558 s | 1.0000 | 1.000× |
| S3 only | 5,000,034 | 168,910 | 2.560 s | 1.0008 | 0.999× |
| S1+S2+S3 candidate | 5,000,034 | 168,910 | 2.099 s | **0.8206** | **1.219×** |

The control reproduces plan27's 168,910 expansions and the expected
`null_calls` 105,120 / `null_replies` 2,782,739. S3 alone is within noise
(1.002×). The safe stack's gain is real but below the gate.

## 3. Attribution (Phase A)

4 s `perf` self-time, `--memory 192 --seed 0 --guide hand`:

| Control | Self | Candidate | Self |
| --- | ---: | --- | ---: |
| `generate_pseudo_legal` | 20.7 % | `immediate_explosion_bits` | 14.0 % |
| `Tables::lookup_with_source` | 12.7 % | `ProofTable::get` | 13.3 % |
| `Board::legal` | 9.3 % | `Board::legal` | 9.9 % |
| `immediate_explosion_bits` | 6.8 % | `Board::do_move` | 6.5 % |
| `Board::do_move` | 6.5 % | `compute_checkers` | 6.1 % |
| `Frame::enter` | 4.9 % | `Frame::enter` | 5.1 % |
| `compute_checkers` | 4.7 % | `null_win` | 5.0 % |
| `null_win` | 3.7 % | `generate_legal_with_state` | 4.3 % |

`from_fen`, `fen_with` and the allocator traffic drop out, confirming S1.
The movegen symbol is folded into `immediate_explosion_bits` by inlining, but
its share falls with the old `generate_pseudo_legal` share. What remains is
the per-child board/legality/terminality path (`legal`, `do_move`,
`compute_checkers`, `populate_state`, `generate_legal_with_state`): the
surface S4 would have attacked. S4 was correctly not started after the miss.

## 4. Gates and tests

- Upstream local checkout: `cargo test --release` green, including the
  golden move-order and perft suites; new `null_for` differential green.
- Candidate copy: `cargo test --release` green (5 M contract, validator,
  search, guide and null differentials).
- Repo (no adopted change): `make test` green, `make lint` green (goal
  check, fmt, clippy `-D warnings`, doc).
- `make test-full` / `make veto`: not run; there is no adopted change to
  gate. No D14 claim is made.
- Licence gate unchanged; no provari/shakmaty material, no committed path
  dependency, no `unsafe`.

## 5. Open parts and missing tests

- The additional Phase A counters (`needs_board` children, per-node child
  lookup hits/misses, `generate_pseudo_legal` calls per expansion) were not
  added: the safe-stack gate missed before they would have influenced a
  decision. The profile and the existing `--stats` counters carry the
  attribution.
- `null_win`'s legal-list early-exit is blocked by the `work.moves` contract;
  a future reformulation would need to change the auxiliary accounting and
  prove tree identity separately.
- No upstream release was published; MG1/MG2 remain prototypes in the
  movegen initiative. The prover plan does not block on a release.

## 6. Next steps

1. **Formulation decision** (the plan's follow-up option 3): choose between
   PDS-PN/PN² and a goal re-pin, with the usual kill review. The per-node
   line is closed; do not reopen it.
2. Movegen initiative: keep MG1/MG2 as measured prototypes; MG3–MG7 stay
   proposed. No prover dependency change until a release exists.
3. Add the `null_win`/`work.moves` gotcha so a later plan cannot "optimise"
   it into a tree change.
