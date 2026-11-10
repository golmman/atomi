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

goal and non-goals are unchanged: atomic chess win prover which is faster than provari.

implementation strategy changes:
* clean-room design
* write high level specification about provari features
* re-implement but use atomic-movegen as move generator
* tune parameters
* get wall time parity with provari
* adopt ideas from atomic_solver for further improvements
