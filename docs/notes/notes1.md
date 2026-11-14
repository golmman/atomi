Bootstrap the successor to `atomic_solver`'s closed `winprover` initiative
in this new repository (working name `prover`). Clone the legacy repo
<https://github.com/golmman/atomic_solver> into a sibling directory as a
read-only reference (never a submodule or dependency; legacy code pin
`5f85133ea9d751368a682e53e354bd3fc0016d00`, handoff from `main`). Read its
`docs/plans/winprover/HANDOFF.md` (revision 2026-11-06), `report5.md` and
`bench.md` next to it, `docs/plans/movegen/report_update_2.4.0.md`, and the
pinned provari source at commit `48686bc6c786d6ea70568f0b2ad4002cf0520206`
(scratch clone; never commit it; ideas only). This is a plan session. Repo
location (D1), evidence strategy (D9) and bench ledger (D10) are settled in
the handoff; decide the remaining open questions in §9. Write the
initiative's `initiative.md`, a small `AGENTS.md` with the GPL boundary and
the copy-with-provenance rule, and `plan1.md` for milestone M0 (bootstrap,
correctness, minimal JSONL proof output with own validator +
`provari check` veto, adapted bench driver with a re-measured baseline row,
per-node cost baseline). No crate dependency on `atomic_solver`. Do not
implement yet.

---

Please revise the decisions of `docs/plans/prover/plan1.md`.

---

suggestion for atomic-movegen in prover/plan2.md

---

Let's have a scrum-like retrospective.

`atomic_solver` was meant to be the fastest winprover for atomic chess.
We put a lot of effort into refining ideas, taking measurements, analyzing papers, etc..
Then we found `provari` which is two orders of magnitude faster and started over, working on `atomi`.

Some of my questions are:

- Is there some secret sauce in `provari` that was easy to miss?
- Was the formulation of the goal not clear enough?
- Did i instruct the coding agents in a wrong way?
- What can i do to prevent drift like this in future sessions with coding agents?

Please be direct, no need to be polite. I want to learn something.

---

Please summarize this into three lists that help me improve.

1. Long term and general improvements to the way i communicate, focus, structurize.
2. Checklist of methods i can use to prevent drift from my intended goal.
3. Immediate todos i should apply for `atomi`.

---

I asked multiple times "where could we improve, think out of the box" and i expected (in hindsight) a response like "we could drop solving for draws". I also asked "should we abandon solving for draws to get more performance", apparently dropping draw-solving conflicted with the goals so much that it was never convincingly suggested by the coding agent.

I explicitly want this sort of push back and ideation process though, i thought i even added this in the AGENTS.md.

---

Plan session for atomi. Read docs/notes/retro.md , docs/plans/prover/initiative.md , report2.md and bench.md . Record an owner decision replacing "one mechanism per plan" with stack-level gating. Merge M2+M3 into one "hand-guide stack" item: move classes, check evasions, quiet grouping, EF-PN, per-child static closures including the one-ply explosion settle, defender priors ( UNLIKELY , defence history), history tie-breaks, sibling simulation and the null-move threat replay. Gate it on the whole stack: easy-win ≤ 2× provari hand nodes, core proved at 10 s, measured over ≥ 20 cells and several seeds, with ablations in both directions afterwards. Write plan3.md .
Do not implement.

---

Read docs/plans/prover/report5.md, plan5.md, measurements/plan5/, plan6.md and initiative.md. Execute plan6 (item 15 GROUP kill-test); then re-check the scale case from the plan5/report4 data before plan7 phase A.

---

Are we still on track to beat provari? If not, how should we change the plan?

---

Read report8.md, measurements/plan8/ and the plan6/item-15 erratum in initiative.md. Run the plans 7–8 kill review with the fixed prompt. Then, unless the review overturns it, write plan9.md for item 19: the coverage-first tier, plus (if the owner wants the wall) one bounded proof-replay stack with the pre-registered stop rule. Update bench.md, the success condition, the ledger and D26 status together.

---

How come we have not a single improvement in the last 10 plans? Why do the provari ideas work there but not here?

We didn't improve wall times in the last 10 plans, no additions to the bench ledger.
What is holding us back? Are there any constraints we should rethink?

Addressing the "All-or-nothing stack gate"-problem: we could add provari ideas even if they fail their individual gates with cli feature flags. Gate fails => default off, gate succeeds => default on. Then we could test feature interaction properly.

---

Read docs/plans/prover/report13.md and measurements/plan13/. The three cheap probes failed: no proof-share is 3/3 on deep-win, no ply cap passes, and the conformance stack cuts core by only 1.22×. The hand-guide line is closed. Run the kill review and write a plan for a deep-win tree mechanism outside the hand-guide family, or an owner decision on the formulation (proof limit / budget). Do not reopen the hand-guide/auxiliary line, do not raise the 120 s budget, and do not build a portfolio.

---

Brainstorming session. No code.

For the prover initiative i want back to the initial goal (non-goals unchanged):

**Be the fastest atomic win prover on the fair, reproducible shared-win
corpus, with every claimed win checked by our own replay validator and by an
independent second rules implementation.**

But i propose the following implementation strategy changes:

1. clean-room design
2. write high level specification about provari ideas/features
3. re-implement but use atomic-movegen as move generator
4. tune parameters
5. get wall time parity with provari
6. adopt ideas from atomic_solver for further improvements

Would this be a valid approach?

---

I want the `prover` initiative file shortened and organized.

Some ideas to make it shorter:

- status -> reduce to one line plus the summary table, everything else is redundant with decisions, milestones and history, see below
- decisions -> shorten significantly, outsource details to dedicated file.
- milestones -> shorten significantly, details live in the plan-files.
- history -> outsource to dedicated file, then shorten: details live in the report-files.

---

We identified the current search tree size to be the main culprit for not reaching the initiatives goals.
So a sub-goal would be to get close to search tree parity with provari.
Are we working on this? What are the ideas to get there?

---

Decisions that touched tier formulation should be re-formulated or deprecated then.

---

Read `https://www.schadd.com/Papers/2011PNeval.pdf`.
Can we adopt something for the `prover` initiative?

---

Should we add node count instead of wall time as a new goal rung?

---

the most recent version of provari is 0.4.0. Should we repin?

---

I'd rather explore the df-pn "secret sauce" of provari instead of trying a completely new algorithm that moves us away from adopting more ideas. The risk i see is this: pds-pn proves to be somewhat more efficient first, we adopt it, then we try to adopt more provari ideas but this fails since they are based on the df-pn base.

---

Our project `atomi` is still not on par with the search tree reduction of `provari`.
Analyze the `provari` search in depth and compare it to the current state of `atomi`.
Compile a complete table with concise descriptions of ideas/features that improve the search of `provari` compared to `atomi`.

Where is the "secret sauce"? What ideas can we adopt?

Store your findings in `docs/plans/prover/analysis1.md`.

---

Is the "Ledger" section in `docs/plans/prover/bench.md` supposed to show the plans wall time factors compared to provari?
It looks very convoluted, to much information mixed up, little context given, no common theme found.

---

tune `--guide hand` via ...

---

Looking at the `docs/plans/prover/bench.md` ledger, why did we go worse?
Row 3 shows a 2/3 coverage and 6.3 geomean, the most recent row is so much worse. What happened?

---

Last session we finished with `docs/plans/prover/report23.md`.
The last revisit analysis of `provari` (`docs/plans/prover/analysis1.md`) apparently proved to be very fruitful.
Should we add this to the regular kill reviews?

---

Analysis:
At each expansion we do the same conceptual work provari does, but with more board copies, more movegen, and more auxiliary passes. The M0 baseline was ≈ 5.18 µs per expansion; provari's equivalent is a fraction of that. That's why the initiative says the remaining wall is per-expansion cost, not tree size — and why the closed "per-node micro-line" (NULL-V1/V2) was trying to shave exactly this number.

---

Let's align the goals and handoff in the `movegen` initiative.

The main goal in `movegen` should be to provide feedback and properly integrate the dependency `atomic-movegen`.
The `atomic-movegen` crate is maintained by the owner of `atomi` so we can create "wishlists" which will then be integrated upstream. So the tasks from `atomi` perspective: find out what we need here, integrate a new version when it is released.

The handoff file was created to close the remaining gaps analysed in `docs/plans/prover/analysis2.md`, in particular P1.
Let's refine the handoff by thinking about this imaginary scenario: What would provari need if it replaced `shakmaty` with `atomic-movegen`?

---

Read docs/plans/prover/initiative.md, docs/plans/prover/report28.md,  
docs/plans/movegen/report1.md and measurements/movegen/plan1/README.md,  
then write docs/plans/prover/plan29.md (plan session, ≤ 8 KB): adopt  
atomic-movegen 2.6.0's MG3 (Board::effect_context / EffectContext::is_legal /  
outcome_after) in the search's per-child classification, behind one named  
stack and one wall gate. Replace per-child Board::outcome/has_legal_move with  
outcome_after on a board-free child path; gate on same-session wall vs the  
current tree at identical child_evals/expansions (5 M deep-win probe seed 0  
= 168,910 expansions), make test-full + make veto, and a differential of  
outcome_after vs the moved board on the new  
tests/fixtures/movegen_2.4.0.tsv corpus. Baseline: 2,045 ns/expansion,  
paired-control methodology in measurements/movegen/plan1/bench.py.

---

Last session we finished with ...

Create a new `atomic-movegen` wishlist in `docs/plans/movegen/handoff2.md`.

---

here is how i see it: handoff1 is merely the ask for 14 items. it does not lie, it was a wishlist. plan1 should note the changelog (only 13/14 landed) bump the version, check for regressions. report1 should document implementation challenges and give a final verdict and again should note that only 13/14 features landed in 2.6.0.  
The actual integration of the new api should then happen in dedicated plans in the `prover` initiative. The report1 can be taken as the initial context needed to kickoff the first plan session there.

---

Plan session — prover plan30 (MG15 integration, kickoff from movegen report2 §7).  
Read docs/plans/prover/initiative.md, gotchas.md, report29.md, analysis2.md §5,  
docs/plans/prover/bench.md (reading rules), docs/plans/movegen/report2.md §5/§7 and  
measurements/movegen/plan2/README.md (flat wall, 2,025 ns/expansion, 21.3 % needs_board).  
Then write docs/plans/prover/plan30.md (<= 8 KB, one sitting): replace the  
clone + do_move + terminal fallback in src/classify.rs and src/mover.rs with  
EffectContext::terminal_after, keeping child_outcome where the Illegal/Undecided  
split matters and mapping TerminalReason::MaterialDraw to atomi's own clause  
(tests/child_surface_differential.rs pins both). Gate: tree identity at fixed  
evals, same-session wall vs a control binary, make test-full + make veto,  
proof trajectory unchanged; keep/revert with data. Read-only probes only, no  
implementation in this session.

---

why did the last handoff iteration fail? how can we prevent missing details in the upstream handoff files in the future?

---

we tested `shakmaty` versus `atomic-movegen` and found that `atomic-movegen` is significantly faster.
so where exactly is the gap now?
