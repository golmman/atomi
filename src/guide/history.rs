//! Fixed history tables, seeded tie-break and the deterministic order key
//! (D21/D22).

use atomic_movegen::types::{Color, Move, Square};

/// A fixed `[side][from][to]` table of saturating counters. Cleared per run;
/// never iterated, so it cannot make the search non-deterministic.
pub struct History {
    table: [[[u16; 64]; 64]; 2],
}

impl Default for History {
    fn default() -> Self {
        Self::new()
    }
}

impl History {
    /// A zeroed table.
    pub fn new() -> History {
        History {
            table: [[[0; 64]; 64]; 2],
        }
    }

    /// The bounded score for `mv` by `side` (`<= 16`).
    pub fn score(&self, side: Color, mv: Move) -> u16 {
        self.table[side as usize][mv.from_sq() as usize][mv.to_sq() as usize].min(16)
    }

    /// The raw counter (tests/diagnostics).
    pub fn raw(&self, side: Color, from: Square, to: Square) -> u16 {
        self.table[side as usize][from as usize][to as usize]
    }

    /// Record a move that produced its mover's goal: an attacker move that
    /// was `Proved`, or a defender reply that was `Refuted`.
    pub fn observe(&mut self, side: Color, mv: Move, good: bool) {
        let cell = &mut self.table[side as usize][mv.from_sq() as usize][mv.to_sq() as usize];
        if good {
            *cell = cell.saturating_add(2).min(64);
        } else {
            *cell = cell.saturating_sub(1);
        }
    }

    /// The scorer's scale for a starting number: 16 is neutral, lower is
    /// better for the mover (bounded 8..=32).
    pub fn factor(&self, side: Color, mv: Move) -> u32 {
        16 + u32::from(self.score(side, mv))
    }

    /// `splitmix64(seed ^ key ^ move_bits)`; seed 0 is handled by the
    /// caller (it never calls this).
    pub fn seeded(seed: u64, key: u64, mv: Move) -> u64 {
        let mut x = seed ^ key ^ u64::from(mv.to_bits()) ^ 0x9E37_79B9_7F4A_7C15;
        x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        x ^ (x >> 31)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use atomic_movegen::types::Move;

    fn mv(a: Square, b: Square) -> Move {
        Move::make_move(a, b)
    }

    #[test]
    fn counters_saturate_and_are_side_separated() {
        let mut h = History::new();
        let m = mv(Square::A2, Square::A3);
        for _ in 0..100 {
            h.observe(Color::White, m, true);
        }
        assert_eq!(h.raw(Color::White, Square::A2, Square::A3), 64);
        assert_eq!(h.score(Color::White, m), 16);
        assert_eq!(h.score(Color::Black, m), 0);
        for _ in 0..100 {
            h.observe(Color::White, m, false);
        }
        assert_eq!(h.raw(Color::White, Square::A2, Square::A3), 0);
    }

    #[test]
    fn seeded_hash_is_stable_and_seed_dependent() {
        let m = mv(Square::E2, Square::E4);
        let a = History::seeded(7, 123, m);
        assert_eq!(a, History::seeded(7, 123, m));
        assert_ne!(a, History::seeded(8, 123, m));
        assert_ne!(a, History::seeded(7, 124, m));
    }

    #[test]
    fn update_is_deterministic() {
        let mut a = History::new();
        let mut b = History::new();
        let m = mv(Square::D1, Square::D4);
        a.observe(Color::Black, m, true);
        b.observe(Color::Black, m, true);
        assert_eq!(a.raw(Color::Black, Square::D1, Square::D4), 2);
        assert_eq!(
            a.raw(Color::Black, Square::D1, Square::D4),
            b.raw(Color::Black, Square::D1, Square::D4)
        );
    }
}
