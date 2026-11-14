# Plan 32 measurements — item 39 (PROOF-FILTER): the filter is net-zero, G1 is the residue

Plan: [`plan32.md`](../../docs/plans/prover/plan32.md). Host, binaries and pins:
[`env.json`](env.json). Every number belongs to the control
`target/release/atomi` (`sha256 3b6b3f6b…`, the plan-31 candidate, unchanged in
this session) and to two **scratch** builds outside the repo. No engine change
was made in the repo; the oracle was never run (no proof was claimed).

| File | Contents |
| --- | --- |
| `phase0.py` | the driver, two modes: `counts` (how a lookup resolves, per site) and `time` (interleaved control vs prototype) |
| `counts.json` | `--stats` + counter build, 3 cells at 5 M evals and at 120 s |
| `time_deep_win_120s.json` | G1 and three filter fundings, 120 s, 2 interleaved pairs |
| `time_small_120s.json` | G1 and the 256 KiB filter on `core`/`easy-win`, 120 s, 3 pairs |
| `time_deep_win_sizes_120s.json` | the 512 KiB and 4 MiB fundings, 120 s, 1 pair |
| `profile_120s.json` | `perf` self time, control, 120 s `deep-win` (the profile plan31 did at 5 M) |

## The question the item demanded

How often is the 144 MiB proof table probed for a child that only has a TT
entry? (`counts.json`, `deep-win`, `--guide hand`; a lookup is counted once, so
a proof hit never reaches the TT probe.)

| 5 M evals — 168,910 expansions, 5,384,659 lookups | lookups | proof hit | proof miss → **TT hit** | proof miss → TT miss |
| --- | ---: | ---: | ---: | ---: |
| frame child (pass 3) | 602,926 | 142,323 (23.6 %) | **242,406 (40.2 %)** | 218,197 |
| guide null replay | 4,135,356 | 652 | 171,613 | 3,963,091 |
| guide replay | 646,377 | 122,214 (18.9 %) | 33,057 | 491,106 |
| **total** | 5,384,659 | **265,189 (4.9 %)** | **447,076 (8.3 %)** | 4,672,394 (86.8 %) |

At 120 s (5,803,257 expansions, 269,098,870 lookups): proof hit **7,560,290
(2.8 %)**, TT-only **17,847,542 (6.6 %)**, nothing at all **243,691,038
(90.6 %)**. Keys present in the proof table but unusable at the counter: **0**
in both runs. So a negative-exact proof filter *could* skip 95.1 % / 97.2 % of
the proof probes — the hit rate is not the binding constraint.

## What the existing filter does (and when it stops)

2^22 bits (512 KiB), permanent drop above 1/4 fill. `deep-win` @120 s: dropped
at **store 1,539,730 of 17,355,189** (8.9 %), `filter_bits` 1,048,563;
`core` @120 s likewise at store 1,518,556 of 2,040,597; `easy-win` never drops
(96,444 bits of 111,195 stores). Consequence: only **12,403,676 gate calls** on
`deep-win` @120 s (≈4.6 % of the lookups) ever consult it, and
`lookup_with_source` becomes the top self-time row — 21.55 % at 120 s
(`profile_120s.json`) against 16.61 % at 5 M (`plan31/profile_candidate.json`).

## Does a second filter pay? (measured, not assumed)

`time` mode runs one process at a time, control and variant interleaved; the
prototype's switches are `ATOMI_PF_KB`/`ATOMI_PF_MB`, `ATOMI_PF_FROM`
(`tt`|`proof`), `ATOMI_PF_DROP` (percent full at which it drops).

| Variant (all on top of G1) | deep-win @120 s | core @120 s | easy-win @120 s | tree identical |
| --- | --- | --- | --- | --- |
| G1 alone | 0.9822, 0.9743 | 0.9765, 0.9839, 0.9866 | 0.9766, 0.9664, 1.0132 | 8/8 |
| + 256 KiB proof filter (the label's free slack) | 0.9749, 0.9772 | 0.9935, 1.0096, 0.9962 | 0.9896, 0.9897, 1.0106 | 8/8 |
| + 512 KiB, funded from the proof table | 0.9892, 1.0041 | 1.0085 | 1.0026 | yes |
| + 2 MiB, funded from the proof table | 1.0009, 1.0162 | — | — | yes |
| + 4 MiB, funded from the proof table | 1.0386, 1.0261 | — | — | yes |
| + 2 MiB, funded from the **TT** | 0.7341, 0.7424 | — | — | **no** (118,249,051 vs 155,236,740 child_evals) |

At 5 M `deep-win`: G1 0.9836, +2 MiB 1.0044, +4 MiB 1.0191 (tree identical).

Reading: the filter is **net zero to negative** at every size that keeps the
tree (a filter probe is a dependent random access; the probe it skips is issued
early and overlaps the loop), and the one positive row is a **different tree**
(chaotic sensitivity of TT evictions), which is not the filter's gain and is
forbidden by the item's tree-identity gate.

## Scratch builds (never committed)

Both are `git archive HEAD | tar -x` copies outside the repo.

1. **Counter build** (`/tmp/atomi-plan32-probe`): `src/table/probe.rs` with
   relaxed `AtomicU64` counters; `Tables::lookup_site(key, counter, site)`
   (site 0 = frame pass 3, 1 = `guide/null`, 2 = `guide/replay`, 3 =
   `search/mod.rs`'s `--stats` attribution, which is measurement-only);
   `Tables::may_hold` counts calls/opens; `Tables::store` counts stores, proof
   inserts and the store at which the filter stops answering; `Filter::bits_set`;
   `main.rs` prints the counters as an extra `"probe"` object.
2. **Prototype build** (`/tmp/atomi-plan32-proto`): `Tables::proved(key,
   counter)` (proof table only) at the five guide lookups plus the collapsed
   duplicate in `replay.rs` = G1; on top, an optional second `Filter` over proof
   keys set on every proof insert (`store`'s `Proved` arm), gating `proved` and
   the `lookup_with_source` proof probe, sized/funded by the env switches above.
   A `ATOMI_PF_DIAG=1` check counted zero real entries behind a filter "no"
   before any timing was taken (the first draft forgot to set the bit on insert
   and produced 34,624 hidden entries — the tree moved; the check is the reason
   that run is not in this table).

## Reproduce

    python3 measurements/plan32/phase0.py --mode counts \
        --probe /tmp/atomi-plan32-probe/target/release/atomi \
        --evals 5000000 --seconds 120 --out /tmp/counts.json
    python3 measurements/plan32/phase0.py --mode time \
        --control target/release/atomi \
        --proto /tmp/atomi-plan32-proto/target/release/atomi \
        --cells deep-win --seconds 120 --repeats 2 \
        --variants G1,G1+pf256k_proof,G1+pf2m_proof,G1+pf2m_tt --out /tmp/time.json
    perf record -q -o /tmp/p.data -- target/release/atomi prove \
        "rnb1kbnr/pppp2pp/5q2/4ppN1/3P4/8/PPP1PPPP/RNBQKB1R w KQkq - 0 1" \
        --guide hand --seconds 120
    perf report -i /tmp/p.data --no-children --stdio --sort symbol --percent-limit 1.5
