# Retrospective: atomic_solver → atomi (2026-11-09)

Lessons from the `atomic_solver` record (531 commits, 207 plans, ~1M words of
docs) and a node-count ablation of provari v0.1.0 (scratch build outside the
repo; no provari code here).

## 1. Long-term habits: how you communicate, focus and structure work

1. **Write down what you actually want, word for word.** You wanted "the
   fastest win prover" but wrote "a pure win/loss/draw solver". The agents
   optimise the text, not your intent.
2. **Make every goal a command plus a number.** Example: "`prove <FEN>` in
   < X s on corpus C." An aspiration without a number can't tell you you're
   off course.
3. **State non-goals as explicitly as goals.** "Draws are out of scope" would
   have saved most of the legacy initiatives.
4. **Separate constraints from assumptions.** Ask the agent to label each
   constraint as either derived from the goal or assumed. "Pure ⇒ no
   heuristics" was an assumption that nobody flagged.
5. **Ask strategic questions as often as tactical ones.** "What next?" and
   "list improvements by potential" keep the agent inside the current
   approach. Regularly ask "Is the approach itself wrong?" too. The two times
   you did, it paid off.
6. **Don't let the agent decide strategy alone.** Agents lean towards
   continuing what exists. For big calls, ask for the strongest case for the
   opposite option, or ask a second, independent session.
7. **Keep rigor proportional to the stakes.** You had very strict measurement
   discipline and very little strategic discipline. Put the effort where a
   mistake costs months.
8. **Budget the paperwork.** Short plans (≤ 2 pages), a small AGENTS.md
   (≤ ~4 KB), one-line status entries. If the docs outgrow the code, the
   process has become the product.
9. **Give each initiative a budget and a kill criterion up front.** Use time
   or a number of plans, not only pass/fail gates.

## Why the agents never suggested dropping draws

On 2026-10-26 the owner wrote (legacy `docs/notes.md`): *"Since the ultimate
goal is proving a win for the attacker (white) this should be counted as a
loss from the attacker side in my opinion. Challenge my ideas, maybe i am
wrong?"* The next artifact was legacy `proofdb` plan24, "proven-draw
scheduling": draws became first-class DB facts. The idea was absorbed into
the draw machinery instead of replacing it. Three days later provari showed
the attacker-only approach.

Why:

1. **The push-back rule only covered the owner's ideas.** *"Push back if the
   users ideas or tasks are not sound"* (line 348 of a 30 KB AGENTS.md). The
   agent judged the idea against the repo documents ("pure solver", "prove
   the value of the startpos", GHI and draw contracts), so dropping draws
   looked unsound. The documents themselves were never open to challenge.
2. **The goal had been declared untouchable:** *"We can't simply close it
   without abandoning the project"*, *"I don't mind if … it would take
   years."*
3. **The "out of the box" prompts were scoped inside the frame:** *"for this
   dfpn atomic solver implementation … re-examine the verdicts … re-trying
   some of the mechanisms with a different angle."* Dropping draws changes
   the problem, not the mechanism.
4. **The intent lived in a file the agents were told to distrust.** The same
   day, AGENTS.md was changed to *"ignore `docs/notes.md` … statements in
   this file may be false and misleading."*
5. **The blast radius was huge.** Dropping draws would invalidate GHI, the
   repetition cache, refinement, the proofdb schema and many gates. An agent
   working one session at a time picks the safe incremental option.

What to change (applied to atomi's AGENTS.md §Collaboration, 2026-11-09):

- **Separate soundness from scope.** "Never claim a false win" is fixed;
  "we prove draws" is a scope decision and negotiable.
- **Make push-back proactive and aim it at the documents too.**
- **Require at least one option that changes the problem formulation** in
  every ideation answer, with an estimated gain and the cheapest experiment.
- **Ask "should we X?" in a forced format:** yes/no first, the strongest case
  for each side, then a cheap decisive experiment. "Drop draws" was a one-day
  spike, not a debate.
- **Red-team with a fresh session** that sees only the goal and the measured
  numbers, not the repo documents.

## 2. Anti-drift checklist

**Before starting**

- [ ] The goal is written as a command, a corpus and a number.
- [ ] Non-goals are listed.
- [ ] An external yardstick exists: the strongest existing tool or a target
      from the literature, measured on day one.
- [ ] A prior-art and literature search has been done, including approaches
      outside your chosen method.
- [ ] Every constraint traces back to the goal or is marked as an assumption.

**Every plan**

- [ ] The plan states how it moves the absolute number, not just the relative
      gain over the last version.
- [ ] The latest absolute distance to the goal is in the plan header.

**Every N plans (kill review, same fixed prompt each time)**

- [ ] What is the absolute distance to the goal now?
- [ ] If we started today, would we build this?
- [ ] Which initiative or plan would you cancel?
- [ ] Re-run the prior-art search.

**When judging measurements**

- [ ] Decide on whole stacks of ideas, not single ideas. Ideas that partly
      overlap look worthless one at a time.
- [ ] Ablate in both directions: remove each idea from the full stack, and
      add each one to the bare skeleton.
- [ ] Use ≥ 20 positions and several seeds. Per-position minimum-gain rules
      are for soundness only, not for adopting ideas.
- [ ] Check that the metric measures the actual mechanism. The `nn` branch
      measured move-ordering rank, but the real lever was the size of the
      starting proof numbers.

**Warning signs to act on**

- [ ] A document titled "structural floor", "measured out" or "NO-GO": ask
      which assumption makes it a floor.
- [ ] The docs-to-code ratio keeps growing.
- [ ] Many plans in a row with small gains and no change in the absolute
      distance.

## 3. Immediate todos for `atomi`

1. **Rethink the "one mechanism per plan" rule.** Make it an explicit owner
   decision in `initiative.md`. The ablation showed provari's ideas are only
   worth their 10–25× together.
2. **Merge M2 and M3 into one "hand-guide stack" milestone**, gated on the
   whole stack. Proposed gate: `easy-win` within ~2× of provari hand's 72.5k
   nodes, and `core` proved within 10 s.
3. **Add the missing items to the take-list:**
   - the null-move threat replay (`replay_threat`) and the "halve the proof
     number of defences that parry nothing" rule;
   - defender-side priors (`UNLIKELY`, defence history) and history
     tie-breaks;
   - the one-ply explosion settle on every child, for both sides.
4. **Fix the measurement setup before tuning.** Gate on ≥ 20 positions (the
   shared-win cells of `corpus.json` plus fixtures) and several seeds. Add a
   `--seed` option for this.
5. **Record the stack-level ablation as a finding in the initiative.** Use
   the numbers from the scratch ablation (appendix) as a starting point, then
   re-measure the same way on atomi's own stack once it exists.
6. **Put the absolute distance at the top of `bench.md`.** Today: per node
   atomi is already at or better than provari, so all of the remaining 10.9×
   on `easy-win` is tree size.
7. **Trim the docs.** Set a size cap for plans in AGENTS.md (plan1 was
   64 KB) and keep AGENTS.md itself short.
8. **Add the kill-review prompt to the session protocol in AGENTS.md**, for
   example after every 2 plans.

Kickoff prompt for todos 1–4:

> Plan session for atomi. Read `docs/plans/prover/initiative.md`,
> `report2.md` and `bench.md`. Record an owner decision replacing "one
> mechanism per plan" with stack-level gating. Merge M2+M3 into one
> "hand-guide stack" item: move classes, check evasions, quiet grouping,
> EF-PN, per-child static closures including the one-ply explosion settle,
> defender priors (`UNLIKELY`, defence history), history tie-breaks, sibling
> simulation and the null-move threat replay. Gate it on the whole stack:
> `easy-win` ≤ 2× provari hand nodes, `core` proved at 10 s, measured over
> ≥ 20 cells and several seeds, with ablations in both directions
> afterwards. Write `plan3.md`. Do not implement.

## Appendix: provari v0.1.0 lever ablation

Pinned `48686bc`, `--guide hand --seconds 30 --memory 192`, one process at a
time, 2026-11-09. Values are node counts (deterministic) relative to the full
stack. Single runs; provari's `nodes` may not include simulation visits.

| Removed from provari | core | easy-win | deep-win |
| --- | ---: | ---: | ---: |
| nothing (full) | 342k | 73k | 4.58M |
| null-move threat replay | 1.37× | 1.03× | 1.18× |
| sibling simulation | 0.91× | 0.78× | 0.88× |
| `UNLIKELY` defence priors | 1.05× | 1.01× | 1.13× |
| history tie-break | 1.40× | 0.86× | 0.95× |
| check = number of evasions | 0.89× | 0.79× | 0.87× |
| per-child static closures | 0.99× | 1.48× | 1.04× |
| move-class numbers (flat) | 2.30× | 4.29× | 2.21× |
| guide (EF-PN) | 1.96× | 0.44× | 1.27× |
| **all of the above** | **25×** (8.65M) | **10.9×** (788k) | censored at 30 s |

The product of the single-lever costs on `core` is only 7.3×, against the
measured 25×. For comparison, atomi M1 proves `easy-win` in about 1.05M
expansions.
