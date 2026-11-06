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
