# Report 30 — item 37 (MG15 integration): measured no-go, and what the re-run changed

Plan: [`plan30.md`](plan30.md). Initiative: [`initiative.md`](initiative.md),
item 37, proposed D42. Measurements:
[`measurements/plan30/`](../../../measurements/plan30/README.md). Read-only
session: **no engine change, no flag, no default, no `bench.md` row, no probe
run on the tree.**

## 1. What the plan asked for, and the verdict

Plan30 §3 is a *record* plan: the stop rule (`no-go if MG15 ≥ 1.0× the
build+`outcome()` cost on the class we build`) was declared in the plan
session, the probe fired it there, and the execution session publishes the
evidence instead of a patch. Delivered: `measurements/plan30/{probe/,
probe.json, env.json, README.md}`, the `gotchas.md` row, item 37's closure +
D42 in `initiative.md`, this report, the history entry, and the narrowed
upstream ask [`../movegen/handoff3.md`](../movegen/handoff3.md).

**Verdict: no-go, reproduced.** MG15 is 1.147× today's per-child cost on the
class that needs it (plan session 1.13×), 1.286× on captures, and ≥ 1.0× in
**all five** runs. The stop rule is honoured; the per-node line stays closed.

## 2. The measurements (ns per built-ish child, median of runs)

`measurements/plan30/probe/src/main.rs` — the plan-session probe, copied
verbatim (sha256 `584a3514…`), 1,248 positions / 39,665 legal moves / 7,864
built-ish children, best of 5 timed passes, five runs:

| Class | n | today (`clone`+`do_move`+`outcome()`+`Legal` key) | MG15 | build only | ratio |
| --- | ---: | ---: | ---: | ---: | ---: |
| built-ish | 7,864 | 57.5 | 65.8 | 32.0 | **1.147** (runs 1.095–1.175) |
| captures | 3,957 | 64.8 | 83.3 | 42.4 | 1.286 (1.279–1.330) |
| quiet | 31,801 | 42.6 | 44.6 | 16.4 | 1.038 (0.997–1.154) |

That reproduces plan §1 within host noise (58.0 / 65.3 / 43.0, ratio 1.13;
captures 64.0 / 82.8). The sign is not a corpus artefact: the build alone is
32.0 ns, so MG15 buys nothing on the half of the work it was supposed to
remove and pays a premium on the half it adds.

## 3. What changed against the plan (scope changes)

Three corrections, all recorded in the deliverables:

1. **The plan's per-call rows were not in the probe it committed.** §1's table
   has MG17 (`hash_and_rule50_after` 7.5 / `post_move` 18.1), `child_outcome`
   51.7 and `has_legal_move_after` 45.4; `src/main.rs` times none of them. I
   added `measurements/plan30/probe/src/bin/api_costs.rs` (an
   execution-session addition, labelled as such) and measured them in two loop
   shapes. On this host they do **not** reproduce the plan's absolute values
   (`post_move` 36.2 node / 26.9 flat against 18.1; `hash` 18.2 / 10.6 against
   7.5; `has_legal_move_after` 61.8 / 54.1 against 45.4) — loop shape and code
   layout move per-call cost by up to 1.5× on this machine, and only the
   **sign** is stable. The README states this instead of quietly publishing new
   absolutes as if they were the plan's.
2. **The amortised form the plan §4 wants is worth less than §4 assumes.**
   `PostMove::terminal()` does not exist in 2.7.0 (`PostMove` exposes only
   bitboards, `rule50`, `ep_square`, castling and casualties) and
   `terminal_from` is private — confirmed in the crate source. Because
   `terminal_from` is the single implementation behind `terminal_after`,
   `post_move` + terminality **is** `terminal_after`: measured 62.2 (node) /
   53.6 (flat) against today's 55.1 / 43.5, i.e. parity at best, before the key
   is paid. So plan §4's ceiling (≤ 1.05× wall, gate G-M15 ≤ 0.95×) rests on an
   estimate (`analyze` 18.1 + `terminal_from` 29.9 = 48.0) that this class of
   measurement does not confirm: the amortised path measures as
   `terminal_after`, which is not cheaper than the build. `handoff3` therefore
   pairs the `PostMove::terminal()` ask with a **cost bar** (a specialised
   existential at ≤ 12 ns) and states that the pair, not the API alone, is the
   ask. This is
   a *downgrade* of the follow-up: no plan31 is authorised on the current
   crate.
3. **The probe crate is committed with its `Cargo.lock`** (the plan named only
   `Cargo.toml` + `src/main.rs`): the checksum of `atomic-movegen 2.7.0` is
   part of the record, and `target/` is covered by the existing `.gitignore`.

Ceiling arithmetic stands unchanged from plan §2 (6.3 builds/expansion ≈ 15.5 %
of 2,025 ns/expansion, realisable ~0 % under `--guide hand` because the guide
reads every built child), so nothing here re-opens a rung: `T_3` now, `T_4`
needs `deep-win` 7.17× (D39).

## 4. Gates

`make test`, `make lint`, `make test-full` and `make veto` are **green
unchanged** on the untouched engine; the grouped-engine `make test-full`
includes the oracle veto (all claims accepted, 4/4 controls rejected, checker
agrees). `tests/child_surface_differential.rs` remains the 2.7.0 crate pin;
`tests/effects_differential.rs` keeps MG3 pinned. Verdicts in
[`env.json`](../../../measurements/plan30/env.json).

## 5. Problems and open parts

- **Host noise**: run-to-run spread on the built-ish class is 1.05–1.18× for
  the same binary (load average 0.3–0.4 on 4 visible cores). The record
  therefore publishes medians, per-run minima and the two loop shapes, and
  claims only sign stability. No wall conclusion is drawn from these numbers —
  correctly so, since none is a product-stack measurement.
- **No probe ran on atomi's tree** (plan §3.6, as specified): the corpus is
  random play-outs, so the *share* of built children would differ on the
  product stack. It cannot matter here: the verdict is per-child cost, and the
  plan-session Phase A counters already put the convertible share at ~0 %
  under `--guide hand` (`report29`).
- **`has_legal_move_after` does not behave like an existential query**: it
  costs the same as `terminal_after` (61.8 vs 62.2). That is an upstream
  observation with no downstream consequence yet; it is in `handoff3` §2 as an
  implementation note, not an ask.
- **Why handoff2's answer is not integrable** (owner question in
  [`../notes/notes1.md`](../../notes/notes1.md)): its acceptance bars were all
  *soundness* bars (equality with the built child, precedence, pseudo-legal
  input), and a soundness bar is satisfiable by any implementation — including
  one that re-derives the child view and runs a full view movegen. Nothing in
  the ask named a **cost** or the corpus that would measure it, so the release
  was accepted on evidence that cannot answer "is this cheaper than the build
  it replaces?". Prevention, now in `handoff3` §2: every MG item carries a
  per-child cost bar *and* the corpus/harness that decides it (built-ish
  children, `today = clone+do_move+outcome()+key`, two loop shapes, median of
  runs), and a private helper that the API would need twice is called out as
  part of the ask. The downstream-side version of the same rule: measure the
  cost **before** drafting the integration plan, not after the plan session.
- **Open**: the ordering question (item 37 → formulation fork) is untouched by
  this session. The formulation decision (PDS-PN/PN² or a goal re-pin under
  D30) is the next plan; plan30 §5 option 3 (the *sufficient* "certainly not
  terminal" screen) is only worth a Phase-A probe **after** `child_eval` /
  `classify` stop needing the child board — today they do, so it would be a
  second measured no-go.

## 6. Missing tests

None owed: no code moved, and nothing about MG15 can be pinned as a behaviour
(it is a cost decision). The comparable pin, `tests/child_surface_differential.rs`,
already covers 2.7.0's child surface against our own built child, including the
`terminal_after` contract that atomi would have relied on. A future adoption
plan (only with a crate version clearing `handoff3` §2) needs a differential
against `rules::status` on the built child **and** tree identity — not a unit
test.

## 7. Next steps

1. **Formulation fork** (plan30 §5 option 2): PDS-PN/PN² vs a goal re-pin under
   D30 — the initiative's standing next decision, unblocked by this closure.
2. **Upstream**: nothing to do until a release ships `PostMove::terminal()` +
   a cheap existential; `handoff3` states both bars so a re-proposal is a
   measurement, not a wish.
3. **Kill review due**: this is plan 30; the fixed-prompt review is owed at the
   next session boundary and should re-derive the absolute distance (`T_3`,
   `deep-win` 7.17×) rather than take it from here.
