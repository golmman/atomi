# Plan 35 measurements — item 43 (SHAPE-DIFF): the reference's body, counted

Plan: [`plan35.md`](../../plan35.md). Report:
[`report35.md`](../../report35.md). Host, binary shas, pins:
[`env.json`](env.json). One execution session; **counts only, nothing timed for
the verdict**: no `ns` here is the reference's — every rate is ours
(report34 §4 verbatim).

| File | Contents |
| --- | --- |
| `shape.py` | the driver: 3 runs per cell with the count build + 1 run with the unpatched pin build (tree identity, cross-check), then the mapping and the arithmetic. Writes `shape.json` |
| `shape.json` | every counter (integers), per-body rates, the leaf table (`p_ops/body`, `ours/exp`, volume ratio, `p_ops × ns/op`, pp, addressable pp), `M` for both cells, the gates, the cross-check |
| `env.json` | host, pins, the two binaries and their sha256, the census input, the scratch patch script's sha256 |

## Boundary (GPL, hard rule)

The reference's pin tree is read and patched **only** in `/tmp`
(`/tmp/atomi-plan35/provari`, a copy of `/tmp/atomi-oracle/provari` at
`48686bc`), by a patch script that also lives only in `/tmp`. Nothing from it
enters this repo: no source fragment, diff, comment or doc text. What is
committed is integer counts and **our own** operation names; `shape.py` only
runs the binary and does arithmetic. `PROVENANCE.md` untouched.

## The count build

A copy of the pin tree with a thread-local array of 39 `Cell<u64>` counters
(no clock, no atomics, one thread), bumped at 35 call sites and printed as one
JSON line on stderr when `ATOMI_COUNTS` is set; the patch script's sha256 is in
`env.json`. Counters never change the search: both cells report the same
`nodes`, mate length and wall as the unpatched build (`cross_check.unpatched`,
`tree_identical: true`).

Six families, each counter named for **our own** op:

1. **moves** — list generations; moves yielded per list (pseudo and legal); the
   best-move generator's own setup.
2. **lists** — list generations per body and their sizes.
3. **probes** — the proof store, the small table, the filter bit, the prefetch
   hint, stores.
4. **per-move and per-child tests** — the legality screen, the quiet-move test,
   check/exposure/explosion/threat tests, the heavy legality test, the
   repetition test, the replay's reply helpers.
5. **child builds** — child boards built, child hash/effect computations, the
   per-child decision, the terminality and status queries, the number
   combination per selection step, the replay calls.
6. **null scans** — null uses, null boards, replay steps and their defence
   loops.

The denominator is the **node body** (one counter at the body entry), not the
reported node count: on `deep-win` the reference reports 3,024,117 nodes for
2,099,100 bodies, the difference being 926,564 replay steps and 1,547 early
returns before its node tick (`decode` in `shape.json`).

## Reproduce

    python3 docs/plans/prover/measurements/plan35/shape.py --out docs/plans/prover/measurements/plan35/shape.json

≈ 15 s: 3 count runs per cell (`deep-win` 4.5 s, `core` 0.5 s) and 1 unpatched
run per cell. The counters are byte-identical across the 3 runs of each cell
(G-35b).

## The runs

| Cell | Runs | Bodies | Reported nodes | Counters |
| --- | ---: | ---: | ---: | --- |
| `deep-win` (`--guide hand --seconds 120 --memory 192`) | 3 + 1 | 2,099,100 | 3,024,117 | 3/3 identical |
| `core` (`--guide hand --seconds 10 --memory 192`) | 3 + 1 | 189,751 | 342,289 | 3/3 identical |

## Per body (the reference) against per expansion (ours, report34)

| Quantity | there / body | ours / expansion | ratio ours:theirs |
| --- | ---: | ---: | ---: |
| list generations | 2.39 | 13.51 | 5.65× |
| moves yielded per list | 31.0 pseudo | 31.5 pseudo | — |
| legal moves | 36.4 | 53.1 | 1.46× |
| per-move legality screens | 59.2 | 26.8 | 0.45× |
| children decided | 18.20 | 8.17 built / 26.75 evaluated | — |
| child boards built | 3.56 | 128.8 do/undo/clone | 36× |
| proof-store probes | 5.22 | 23.73 | 4.5× |
| small-table lookups | 3.76 | 22.26 | 5.9× |
| filter bit tests | 45.5 | 2.14 | 0.05× |
| table stores | 1.00 | 2.99 | 3.0× |
| hand evaluations | 1.00 | 3.68 | 3.7× |
| null uses | 0.217 | 0.591 | 2.7× |
| replay steps | 0.441 | 0.819 | 1.9× |
| number combinations | 2.03 | 4.00 | 2.0× |

Nothing here is a benchmark: the reference's body is **2.4× fewer** nodes than
ours on `deep-win` (2,099,100 bodies vs 5,803,257 expansions) and does fewer
lists, probes, stores and guide passes per node.
