# Initiative: `movegen` — upstream feedback and deliberate integration of `atomic-movegen`

## Status

**Open (2026-11-12; goal revised 2026-11-13).** Created from the `prover`
hot-path profile ([`../prover/plan28.md`](../prover/plan28.md)) and
[`../prover/analysis2.md`](../prover/analysis2.md) §5: atomi spends 15.5 µs per
expansion against provari's 1.42 µs per node, ~54 % of it in movegen, legality
and terminality. The crate is MIT, maintained by the same owner; this
initiative is the **feedback loop**, not a fork and not a second engine
project. The previous success clause ("the S1–S3 stack reaches ≥1.5× at an
identical tree") is **retired**: the stack was measured at 1.219× and not
adopted ([`handoff1.md`](handoff1.md) §7, prover D40), so a per-node wall gate
never was this initiative's decision to make.

Current artifacts: **`plan3.md`** (2.8.0 adoption, drafted 2026-11-13: a pin —
MG19/MG20 shipped but missed both cost bars, so no `prover` plan follows),
**`handoff3.md`** (answered by 2.8.0 and now history: MG19 the terminal state on
`PostMove`, MG20 a specialised existential legality test, filed 2026-11-13 by
prover plan30 with the measured cost bars), **`handoff2.md`**
(MG15–MG18: child-view terminality, filed 2026-11-13 by prover plan29;
answered 4/4 by 2.7.0 and now history), **`plan2.md`**
(2.7.0 adoption, executed 2026-11-13) with **`report2.md`** (its outcomes,
challenges, and the kickoff for the `prover` plans that integrate MG15–MG18),
**`handoff1.md`** (history now that 2.6.0 answered it), **`plan1.md`** (2.6.0
adoption, executed 2026-11-13) and **`report1.md`**. 2.6.0
landed **13 of the 14** MG items (MG5 merged into MG3, MG7 withdrawn) and is
**neutral** on atomi's wall at an identical tree (plan1 §3, report1 §2). MG3
is now *measured* at a call site, not merely adopted: the prover's
EFFECT-CHILD-V1 ([`../prover/report29.md`](../prover/report29.md)) closed as
`no effect` — the effect decided **0.00 %** of 1,067,066 built children on
the `--guide hand` stack (ceiling 0.08 %) and 100 % of the fallback share is
the mate/stalemate question MG15 asks about. The adopted dependency is
published **2.7.0** (owner commits; 2.6.0 answered handoff1, 2.7.0 answered
handoff2); integrating the new API is **`prover`** work.

## Goal

Two halves, one owner, one lifecycle:

1. **Requirements — find out what we need.** Derive the prover's needs from
   measurement (profile, probe, differential), state them as a *generally
   useful* upstream specification, and file them as a handoff. Requests are
   numbered **MG\***; each must cite the requirement it serves
   ([`handoff1.md`](handoff1.md) §2, R1–R8).
2. **Integration — adopt a release deliberately.** When the crate publishes a
   version, plan the adoption, execute it, and keep or revert **with data**.

Success (all four; each is a process fact, not a performance hope):

| # | Condition |
| --- | --- |
| S1 | Every atomi need is **filed** in a handoff (never only in a plan or a report), and every resolved ask records its **outcome** — landed, rejected upstream with reasons, or withdrawn downstream — either in the handoff that asked or in the integration `reportN.md` the ledger names for the release that answered it (D-MG7). |
| S2 | The committed dependency is a **published** version — never a fork, never a `[patch]`, never a committed path dependency. A published release gets an **integration plan** within one plan of us noticing it, and the plan is executed, not parked. |
| S3 | Every integration plan carries the same gate (see **Gates**): the crate's harness as published, the golden rules differential, `make test-full`, `make veto`, and a same-session bench. Any rules divergence or outcome regression is reverted in the same session and reported. |
| S4 | The per-expansion cost is recorded per adopted version (`make pernode` + a 120 s bench row), so the P1 gap is a **trend line** with an owner, not an unexplained constant. |

**Not success.** Moving a rung from atomi's side by squeezing the dependency:
the prover's rung decisions are its own (D30); this initiative supplies the
per-node lever or the measurement, it does not claim the rung. A release that
lands MG1–MG3 is valuable even if no rung moves, and a rung that moves on the
prover's tree is not this initiative's gain.

**Not success, second form — waiting.** A release does not block the prover
formulation decision; the prover may measure with a local checkout
(measurement only) and adopt after the release.

## Flow and file structure

Four steps, in this order. Only steps 1, 3 and 4 produce a file here; step 2
produces none and needs no atomi action.

| Step | Artifact | Where | Who | Notes |
| --- | --- | --- | --- | --- |
| 1. Wishlist | **`handoffN.md`** | `docs/plans/movegen/` | atomi sessions, addressed to upstream | Standalone: it must be readable in the `atomic-movegen` repo with no atomi context. `N` is a handoff number (never a plan number); a revised wishlist of **new** asks is a new handoff; a release that answers an existing one produces no new handoff (D-MG7) |
| 2. Release | — | crates.io / the crate's repo | the owner | **No atomi action.** No branch, no patch, no path dependency in a commit |
| 3. Integration plan | **`planN.md`** | `docs/plans/movegen/` | atomi sessions | One plan per adoption. Names the handoff it consumes, the version, the gate, the measurement and the keep/revert rule. Plan numbers are independent of handoff numbers (AGENTS.md); this initiative's first is `plan1.md` |
| 4. Implementation report | **`reportN.md`** | `docs/plans/movegen/` | atomi sessions | Tools, problems, open parts, missing tests, next steps; the adopted version, ratios and the release's **MG outcomes** (D-MG7); a one-line history entry here. It is also the kickoff context for the `prover` plans that integrate the new API |

Ledger of the lifecycle:

| Handoff | Release | Plan | Report | State |
| --- | --- | --- | --- | --- |
| [`handoff1.md`](handoff1.md) (2026-11-13) | 2.4.0 is the baseline; **no new release** | — | — | wishlist filed, awaiting a release |
| [`handoff1.md`](handoff1.md) | **2.6.0** (published 2026-11-13; no 2.5.0 exists) | [`plan1.md`](plan1.md) | — | superseded by the executed row below |
| [`handoff1.md`](handoff1.md) | **2.6.0** | [`plan1.md`](plan1.md) | [`report1.md`](report1.md) | **executed 2026-11-13, kept** — 13/14 landed (MG5 merged, MG7 withdrawn), gates 1–4 green, wall flat (paired 0.9999, tree identical) |
| [`handoff1.md`](handoff1.md) §4/MG3 | **2.6.0** | — (consumed by a `prover` plan) | — | **consumed and measured 2026-11-13** — the prover's EFFECT-CHILD-V1 is `no effect` (0.00 % decided, [`../prover/report29.md`](../prover/report29.md)); its differential is committed as `tests/effects_differential.rs` |
| [`handoff2.md`](handoff2.md) (2026-11-13) | **no release yet** | — | — | superseded by the row below |
| [`handoff2.md`](handoff2.md) | **2.7.0** (published 2026-11-13T16:20:30Z) | [`plan2.md`](plan2.md) | [`report2.md`](report2.md) | **executed 2026-11-13, kept** — 4/4 landed (MG15–MG18), gates 1–4 green, frozen differential diff-empty, tree identical, wall flat (paired `m1/m0` 0.987, probe 0.9992) |
| [`handoff3.md`](handoff3.md) (2026-11-13) | **2.7.0** is the baseline | — | — | filed by prover plan30: MG19 `PostMove::terminal()` (+ the key on the same summary), MG20 specialised existential legality (≤ 12 ns bar); both P2 |
| [`handoff3.md`](handoff3.md) | **2.8.0** (published 2026-11-13T22:45:11Z, `.crate` `ac68cf10…f2006b`) | [`plan3.md`](plan3.md) | — | **drafted 2026-11-13, not executed** — MG19/MG20 landed as API, **0 of 2 met the cost bar** (MG20 25–30 ns against ≤ 12 ns; with the key the composite is 1.24× built-ish), so the prover stack stays unauthorised (handoff3 §3): the plan is a pin, not an integration |

A plan may be drafted **early** (to have the gate and the measurement script
ready) but it is executed only against a published version. A pre-release
measurement is a probe inside the plan, never an adoption.

## Division of work

| Who | Does | Does not |
| --- | --- | --- |
| atomi sessions (this initiative) | Profile; write the requirement, the differential and the measurement; write `handoffN.md`; plan and execute integrations | Edit the crate's repo, commit a path dependency, or open the upstream PR |
| The crate's owner (same owner) | Implement the handoff upstream, decide API names and release | Ask atomi to carry a fork |
| atomi sessions (prover) | Consume the published version; measure the effect; keep or revert | Depend on an unreleased crate |

## Non-goals

- No provari, shakmaty or other GPL code, dependency, binary or text — and
  **no GPL interop types in the crate either**: an MIT crate must not depend
  on a GPL crate, so interchange with such libraries stays data-level (FEN,
  UCI, SAN, piece lists). See [`handoff1.md`](handoff1.md) R9.
- No rules change without a differential against the previous version and
  atomi's fixture corpus (D-MG1, Gates).
- No atomi-only API (D-MG5): an item must be wanted by a second consumer of
  the same class — concretely, an engine that replaces a GPL rules library
  with this crate.
- No framework scope: no variant system, tablebase probing, eval/NNUE, UCI
  loop, threads, `async`.
- No `unsafe` without a measured win and a `// SAFETY:` argument.
- No network, model or tablebase work.

## Relationship to `prover`

`prover` consumes the crate; this initiative changes the crate's supply. The
prover's per-node line is closed (D40) as a *micro* line, so the only remaining
per-node lever is upstream. Measurements use a local checkout through a
**measurement-only** path dependency; the committed dependency stays published
(D-MG4).

## Generality test (D-MG5)

Every MG item is tested against one question: **what would a prover-class
engine need if it replaced its GPL rules library with `atomic-movegen` and
kept everything else?** The answer is written down as R1–R8 in
[`handoff1.md`](handoff1.md) §2; each MG row cites its R. The three verdicts:

| Verdict | Means | Examples |
| --- | --- | --- |
| both | atomi needs it *and* the requirement set wants it | MG1, MG2, MG3, MG10, MG11 |
| upstream value | the second consumer wants it, atomi does not — filed, never a blocker | MG8, MG9, MG12, MG13 |
| `atomi-only` | only atomi wants it — filed as a comment, never P0 | none today |

This is the cheapest defence against the wishlist drifting into atomi's
private search design.

## Wishlist

The live wishlist is [`handoff3.md`](handoff3.md) (MG19–MG20) plus every
handoff1/handoff2 item that has not landed — as of **2.7.0** every row below has
landed (MG7 withdrawn), so MG19–MG20 are the only live asks (D-MG7); this
table is the atomi-side index (evidence, priority, acceptance). Priorities are relative to
*atomi's* next plan, not to the crate's roadmap.

| # | Item | R | Why (evidence) | Risk | Acceptance | Pri |
| --- | --- | --- | --- | --- | --- | --- |
| MG1 | `Board::null_for(attacker)` | R4 | `null_board` does `fen_with` → split/join → `from_fen` per null replay; ~1.6 % + allocator; also needed by any null-move search | low | Differential: same key, side, ep, clock as the FEN path; no rules change | P0 |
| MG2 | public `for_each_pseudo_legal` (callback + early exit) | R3 | `generate_pseudo_legal` is 21 % of atomi's wall; scans materialise a `MoveList` | low | Same order as `generate_pseudo_legal`; differential + early-exit test | P0 |
| MG3 | **Node effect context** + per-move effect query (optionally `outcome_after`) | R3 | Per-child `outcome` runs `has_legal_move`; ~15–20 % of atomi's wall; the flagship — see handoff1 §2/R3 and §4 | **high** | Differential vs `legal`/`outcome` on the corpus, the three bench cells and random play-outs; no divergence | P1 |
| MG4 | Expand the trivial-legality ladder (captures/checks) | R3 | `Board::legal` is 8.4 % of atomi's wall; the fast path covers only quiet, non-capture, non-check, non-pinned moves | **high** | Differential vs `legal` on the corpus; no divergence | P2 |
| MG5 | `Board::legal` / node-constant attack data | R3 | Same as MG4 | high | Differential | P2 — **merge into MG3** unless a separate measurement says otherwise |
| MG6 | `generate_pseudo_legal` micro-optimisation | R3 | Still the largest movegen symbol | medium | Perft + movegen differential; same order | P2 |
| MG7 | `from_fen` / `fen_with` performance | R1 | Fallback only if MG1 does not land | low | Round-trip differential | P3 |
| MG8 | SAN parse/format | R1 | Any engine with a lines/PV surface needs it; atomi does not | low | Round-trip + ambiguity cases vs UCI | P3 |
| MG9 | Construction/inspection from a piece list (`Setup`-like) | R1 | Interchange without a FEN string; needed to build from another library's types | low | Round-trip through FEN for every piece set | P3 |
| MG10 | Terminal reason (extinction / no-moves-in-check / stalemate / material / clock) | R6 | `Outcome` is WDL only; a consumer reporting why has to re-derive the rules | low | Every rule branch covered by a fixture | P2 |
| MG11 | Key contract: FSF parity, stability across versions, ep modes | R5 | A consumer that migrates keeps its key space; a silent Zobrist change invalidates stored data | low | Documented; a cross-version test in the crate | P1 |
| MG12 | Generation order as a public contract | R3 | Proofs, transcripts and golden tests depend on it; today it is only internal tests | low | Documented order + existing golden tests as the guard | P2 |
| MG13 | Conformance corpus as data (FEN → moves, outcome, key) | R7 | Every downstream differential needs a reference set; it is also the crate's own regression net | low | Corpus published in-repo with a generator and a checker | P2 |
| MG14 | Engineering contract: `Clone`, `Send + Sync`, no hidden allocation, semver policy | R8 | A threaded or copy-make consumer relies on all four; none is stated | low | Static assertions + a doc section | P2 |

## Gates

Every integration plan contains these, in this order (D-MG6):

1. **Pin and lock** — bump `Cargo.toml` + `Cargo.lock`; run the crate's harness
   as published (tests + perft).
2. **Golden rules differential** — freeze and diff the crate's observable
   behaviour: perft to depth 4–5, full legal move lists *in order*, `outcome`,
   `is_terminal`, `hash`/`hash_with`, `rule50`/`game_ply` after every legal
   move, on a fixed corpus. Any difference that is not an announced rules fix
   is a **veto**, not a note. atomi has no perft test today, so the first
   integration plan **adds this differential**; MG13 would replace the frozen
   data with the crate's own corpus.
3. **Downstream gates** — `make test-full` and `make veto` (an independent
   rules implementation).
4. **Measure** — same-session bench at the standard cap, a 10 s row, and
   per-expansion cost; three repeats, alternating builds, one process at a
   time; parsed results only, under `measurements/<plan>/` with `env.json`.
5. **Keep or revert with data** — one line: version, gate result, ratios,
   reason. A regression, a divergence or a missing harness is a revert.

## Decisions

| # | Decision | Resolution |
| --- | --- | --- |
| D-MG1 | Scope | Performance and API only; no rules change without a differential |
| D-MG2 | GPL boundary | Ideas only; no provari/shakmaty code, dependency or text; no GPL interop types in the crate |
| D-MG3 | Adoption gate | The crate's tests + perft + the golden rules differential + the prover's `make test-full` + veto |
| D-MG4 | Dependency | The prover ships a published version; a path dependency is measurement-only |
| D-MG5 | Generality test | Every MG item is justified by the shakmaty-replacement requirement set (handoff1 §2); atomi-only items are marked and never P0 |
| D-MG6 | Lifecycle | Wishlist `handoffN.md` → release (no atomi action) → `planN.md` → `reportN.md`; the plan carries the Gates section verbatim and no commit depends on an unreleased crate |
| D-MG7 | A handoff is an ask | A handoff is a **point-in-time ask**, not a living document: when a release answers it, it stays as history and is neither edited nor replaced by a "close-out" handoff. The release's outcomes are recorded in the integration report (2.6.0: 13 of 14 landed, MG5 merged into MG3, MG7 withdrawn). A handoff number is spent only when a session has **new** asks, so `handoff2.md` is unspent |

## Risks

| Risk | Mitigation |
| --- | --- |
| A performance release changes rules silently | The golden rules differential freezes moves, perft, keys and outcomes; a divergence is a veto, not a bench note |
| The wishlist becomes atomi's private API | D-MG5 and R1–R8; every row cites its requirement |
| atomi blocks on an upstream release | Measurement-only checkout; the prover's next plan never depends on a release date (D-MG4) |
| Handoffs and plans drift apart | The ledger above names handoff → release → plan → report; a plan that does not name its handoff is rejected |
| The crate's own harness is not run | Gate 1 names it explicitly, including perft, before atomi's gates |
| The owner is both sides, so review is thin | The handoff is written for an outside reader; the golden differential and the prover's veto are independent checks |

## History

- **2026-11-12** — initiative created from the `prover` hot-path profile
  ([`../prover/plan28.md`](../prover/plan28.md)); MG1–MG7 proposed.
- **2026-11-12** — plan28 prototyped MG1 (`Board::null_for`) and MG2
  (`for_each_pseudo_legal`) in a local checkout and measured them in the
  prover's S1+S2+S3 stack. The stack was tree-identical but reached only
  1.219×, below the prover's 1.5× gate, so the prover did not adopt it and MG3
  was not started. The prototypes remain unreleased; the prover dependency
  stays on published 2.4.0. Standalone upstream wishlist:
  [`handoff1.md`](handoff1.md).
- **2026-11-13** — goal revised: the initiative is now **requirements +
  integration**, not a performance gate (S1–S4). Added the generality test
  (D-MG5), the requirement set R1–R8 derived from the shakmaty-replacement
  scenario, MG8–MG14 (upstream value that atomi does not need today), and
  D-MG6. The wishlist became `handoff1.md`; the file structure is now
  `handoffN.md` → release → `planN.md` → `reportN.md` with a lifecycle ledger.
- **2026-11-13** — `atomic-movegen` **2.6.0** is published (no 2.5.0; the list
  goes 2.4.0 → 2.6.0) and S2 triggers: [`plan1.md`](plan1.md) is drafted. The
  tarball was fetched and verified (sha256 matches the index, MIT, MSRV 1.85,
  `src/zobrist.rs` byte-identical to 2.4.0, **no `Fixed` section**), and a
  pre-plan probe showed the crate harness green, atomi compiling **unchanged**
  (no call-site adaptation), atomi's 191 tests green on both versions, and an
  **identical tree with a flat wall (1.002×)** on the `deep-win` 5 M probe — so
  the adoption is neutral for the prover's rung and D40 stands. 2.6.0 lands
  **13 of 14** MG items (MG1–MG6, MG8–MG14; MG7 withdrawn), including the
  flagship MG3 (`EffectContext`/`MoveEffect`/`outcome_after`). Session note:
  crates.io answers **403 without a `User-Agent`** — that, not a network block,
  was what made a first draft of the plan call the release unreachable.
- **2026-11-13** — lifecycle settled (D-MG7, S1 amended): a handoff is an
  **ask**, and `handoff1.md` is not edited or superseded when a release answers
  it. `plan1.md` notes what 2.6.0 contains (13/14), bumps, and checks for
  regressions; `report1.md` records the implementation challenges, the 13/14
  outcome and the verdict, and is the **kickoff context for the first `prover`
  plan** — API integration belongs to that initiative, not to an adoption plan.
  `handoff2.md` stays unspent until a session has a new ask.
- **2026-11-13** — `atomic-movegen` **2.7.0** is published (16:20Z, MIT, MSRV
  1.85, `.crate` `cd69dc00…d38e1ad3`) and answers **all four** items of
  [`handoff2.md`](handoff2.md): MG15 `terminal_after` (+ `is_terminal_after`,
  `has_legal_move_after`), MG16 `AfterState`/`child_outcome`, MG17
  `PostMove`/`effect_and_post`, MG18 `MoveClassification`/`classification`, plus
  an unasked `CHILD_CASES`. S2 triggers: [`plan2.md`](plan2.md) is drafted. The
  probe (one session, `/tmp` only) shows the crate harness green, atomi compiling
  **unchanged** with its 193 tests green (frozen differential diff-empty), an
  identical tree at 5 M evals on both stacks and a **flat** wall (1.006
  `--guide none`, 0.997 `--guide hand`) — so, as with 2.6.0, the adoption is a
  pin plus evidence, and the wall lever (21.3 % of hand-stack children built only
  for terminality) is a `prover` claim. Also recorded there: handoff2 §3's >10 %
  probe gate is **superseded, not passed** — MG15 decides every child, so its
  successor is the probe's `needs_board` share.
- **2026-11-13** — **handoff3 filed (MG19–MG20), triggered by a downstream
  no-go.** The prover measured MG15's *cost* before integrating it
  ([`../prover/report30.md`](../prover/report30.md)): `hash_and_rule50_after` +
  `terminal_after` is **1.147×** today's `clone`+`do_move`+`outcome()`+key on
  the class of child that needs it (1.29× on captures), because the view
  movegen is not cheaper than `do_move` + the board movegen and `analyze` is
  paid on top; `terminal_from` is private, so the best amortised composite a
  consumer can assemble *is* `terminal_after`. The ask therefore narrows: MG19
  `PostMove::terminal()` (+ the post-move key on the same summary) and MG20 a
  specialised existential-legality test at **≤ 12 ns** per child — the pair,
  not the API alone, is what could reach ≈0.84× the build. MG16–MG18 need
  nothing further. No plan is drafted: a release must clear the two bars in
  `handoff3` §2 first.
- **2026-11-13** — **plan2 executed, 2.7.0 kept.** Gates 1–4 green: tarball
  sha256 = index = `Cargo.lock` (`cd69dc00…d38e1ad3`), crate harness as
  published 163 + 45 + 12 green; the frozen differential is **byte-identical**
  on 2.7.0 (the writer regenerates the same sha256); the new
  `tests/child_surface_differential.rs` pins MG15–MG18 against our own built
  child (94 positions + 2,620 play-out nodes, 51,334 children, every
  `TerminalReason` branch, plus the crate-`MaterialDraw` → atomi-clause
  mapping); `make test-full` (slow tier 4/4) and `make veto` (8/8 accepted,
  4/4 controls rejected) green. Same-session bench, both sides from the
  identical tree: paired `m1/m0` geomean **0.987** at tree identity 3/3,
  5 M `deep-win` probe **0.9992×**, per-expansion **2,025 ns** (2.6.0's plan1
  point 2,054, cross-session) — the S4 trend point. Two dated
  `prover/bench.md` rows appended; data in `measurements/movegen/plan2/`. No
  rung claimed (D30/D40 stand); the API integration is `prover` work, and
  handoff2's §3 >10 % gate is superseded by the probe's `needs_board` share.
- **2026-11-13** — **plan1 executed, 2.6.0 kept.** Gates 1–4 green: tarball
  sha256 = index = `Cargo.lock` (`075cd3bf…`), crate harness 156 + 26 + 9
  green as published; the **frozen differential**
  `tests/fixtures/movegen_2.4.0.tsv` (94 positions, 1,605 legal moves, 4 perft
  rows) generated on 2.4.0 is byte-identical on 2.6.0 (diff empty — no rules
  change); `make test-full` and `make veto` (8/8 accepted, 4/4 controls
  rejected) green. Same-session bench, both sides from the identical tree:
  paired `m1/m0` geomean **0.9999** (tree identical 3/3), 5 M `deep-win`
  probe **1.0152×** (flat, inside the ±2–3 % session spread),
  per-expansion **2,045 ns** (`m0` 2,064) — the S4 trend point. Two dated
  `prover/bench.md` rows (Cap 10 s / 120 s, bare stack) appended; data in
  `measurements/movegen/plan1/`. No rung claimed (D30/D40 stand). Two atomi-
  side fixes went with it: the broken `make pernode` target and a writer that
  cannot silently repair the fixture (`report1.md` §4).
- **2026-11-13** — **`atomic-movegen` 2.8.0 is published** (22:45:11Z, MIT, MSRV
  1.85, `.crate` `ac68cf10…f2006b`) and answers **both** items of
  [`handoff3.md`](handoff3.md) as API — `PostMove::terminal()`,
  `post_move_and_key()` (+ `rule50()`), and a positive screen in front of the
  unchanged `has_legal_move` ladder — while its own `Measured` section, and a
  probe in this session on atomi's corpus, show **neither cost bar is met**
  (MG20 25–30 ns against ≤ 12 ns; composite **with** the key 1.24× on the
  crate's built-ish class / 0.97× on atomi's fixture corpus). S2 triggers, so
  [`plan3.md`](plan3.md) is **drafted as a pin,** not an integration: handoff3
  §3's stack gate is **failed, not superseded**, and no `prover` plan follows.
  The probe (one session, `/tmp` only): crate harness green (173 + 51 + 16),
  atomi compiling unchanged with 195 passed / 5 ignored and the frozen
  differential green, `deep-win` 5 M tree identical and wall flat (`m1/m0`
  0.986, medians 2551 vs 2587 ms). Note the screen is the first pin with a
  plausible wall effect: it serves `has_legal_move_with_state`, which
  `rules::status` reaches through `Board::outcome()`.
