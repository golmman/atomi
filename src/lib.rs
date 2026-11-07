//! `atomi` — an attacker-only atomic chess win prover.
//!
//! The engine proves that the side to move (the attacker) can force a win
//! under the checker's rules and emits a JSONL proof accepted by this crate's
//! own validator and by the external `provari check` veto.
//!
//! Layer rules: `validate` uses only `std`, `serde_json` and `atomic_movegen`,
//! never other `crate::` modules (enforced by a test); `search` depends only
//! on `events`, `mover`, `rules`, `table` and `value`; `strategy` depends on
//! `table::proofs` (read only) and `rules`; `mover` depends only on
//! `rules`.

pub mod cli;
pub mod events;
pub mod mover;
pub mod rules;
pub mod search;
pub mod strategy;
pub mod table;
#[cfg(test)]
mod testutil;
pub mod validate;
