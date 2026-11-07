//! Justification for >10 KB: the walk, the cycle detector and the file
//! writer are one export contract; the unit tests exercise them together.

//! JSONL proof writer (R2): walks real positions from the root and needs no
//! search state beyond the store. Runs on a spawned 256 MiB-stack thread.
//!
//! Termination: the chosen entry at `(X, c)` was formed from reply entries of
//! strictly smaller length, so lengths fall strictly along every line and the
//! walk is finite; the residual cross-line cycle is rejected by the post-walk
//! DFS. Output is byte-deterministic and written via `<path>.tmp` + rename;
//! a failure writes nothing.

use crate::rules;
use crate::strategy::ProofSource;
use crate::strategy::store::StoreEntry;
use atomic_movegen::board::{Board, StateInfo};
use atomic_movegen::movegen::generate_legal;
use atomic_movegen::types::{Color, EnPassantMode, Move, MoveList};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// The written proof's stats.
#[derive(Debug)]
pub struct WriteStats {
    /// Lines written (attacker `(position, counter)` pairs).
    pub positions: usize,
    /// The walked proof's length in attacker moves.
    pub length: u16,
    /// Keys written with more than one distinct move across counters.
    pub multi_move_positions: usize,
}

/// Why an export failed (the CLI `proof` field values).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportError {
    /// The store hit its byte cap.
    StoreFull,
    /// The walked position graph has a cycle.
    Cycle,
    /// No usable entry, or a stored move that is not legal.
    Missing,
    /// A non-terminal position at counter >= 100.
    Rule50Limit,
    /// An I/O error.
    Io,
}

impl ExportError {
    /// The CLI string.
    pub fn as_str(self) -> &'static str {
        match self {
            ExportError::StoreFull => "store-full",
            ExportError::Cycle => "cycle",
            ExportError::Missing => "missing",
            ExportError::Rule50Limit => "rule50-limit",
            ExportError::Io => "io",
        }
    }
}

/// Walk the store into a JSONL proof file. `root`'s side to move is the
/// attacker. Fails closed on any error.
pub fn write_proof<S: ProofSource + Sync + ?Sized>(
    root: &Board,
    store: &S,
    path: &Path,
) -> Result<WriteStats, ExportError> {
    if store.is_full() {
        return Err(ExportError::StoreFull);
    }
    let walk = std::thread::scope(|scope| {
        std::thread::Builder::new()
            .stack_size(256 << 20)
            .spawn_scoped(scope, || {
                let mut w = Walker {
                    attacker: root.side_to_move(),
                    store,
                    lines: Vec::new(),
                    edges: HashMap::new(),
                    attacker_memo: HashMap::new(),
                    defender_memo: HashMap::new(),
                    chosen: HashMap::new(),
                };
                let root_length = w.attacker(root, root.rule50(), u16::MAX)?;
                let root_key = rules::key(root);
                if has_cycle(root_key, &w.edges) {
                    return Err(ExportError::Cycle);
                }
                let multi_move_positions = w.chosen.values().filter(|m| m.len() > 1).count();
                Ok(WalkResult {
                    stats: WriteStats {
                        positions: w.lines.len(),
                        length: root_length,
                        multi_move_positions,
                    },
                    lines: w.lines,
                    root_length,
                })
            })
            .expect("spawn writer thread")
            .join()
            .expect("join writer thread")
    })?;
    write_file(root, path, walk)
}

struct Walker<'a, S: ProofSource + Sync + ?Sized> {
    attacker: Color,
    store: &'a S,
    /// Recorded `(fen, uci)` lines in preorder.
    lines: Vec<(String, String)>,
    /// Deduplicated position edges (keys, counters ignored).
    edges: HashMap<u64, HashSet<u64>>,
    attacker_memo: HashMap<(u64, u16), u16>,
    defender_memo: HashMap<(u64, u16), u16>,
    /// Moves chosen so far, per position key (across counters).
    chosen: HashMap<u64, Vec<Move>>,
}

struct WalkResult {
    stats: WriteStats,
    lines: Vec<(String, String)>,
    root_length: u16,
}

impl<S: ProofSource + Sync + ?Sized> Walker<'_, S> {
    fn edge(&mut self, from: u64, to: u64) {
        self.edges.entry(from).or_default().insert(to);
    }

    /// Attacker node `board` at counter `c`; chosen entries must be strictly
    /// shorter than `bound`.
    fn attacker(&mut self, board: &Board, c: u16, bound: u16) -> Result<u16, ExportError> {
        let key = rules::key(board);
        if let Some(&len) = self.attacker_memo.get(&(key, c)) {
            return Ok(len);
        }
        let entry = self.choose(key, c, bound).ok_or(ExportError::Missing)?;
        if !board.is_legal(entry.mv) {
            return Err(ExportError::Missing);
        }
        // Preorder line.
        let fen = board.fen_with(EnPassantMode::Legal);
        self.lines.push((fen, entry.mv.to_uci()));
        let moves = self.chosen.entry(key).or_default();
        if !moves.contains(&entry.mv) {
            moves.push(entry.mv);
        }
        let mut next = board.clone();
        next.do_move(entry.mv, &mut StateInfo::new());
        self.edge(key, rules::key(&next));
        let contrib = match rules::status(&next, self.attacker) {
            rules::Status::Won => 0,
            rules::Status::Failed { .. } => {
                return Err(if next.rule50() >= 100 {
                    ExportError::Rule50Limit
                } else {
                    ExportError::Missing
                });
            }
            rules::Status::Open => self.defender(&next, entry.length)?,
        };
        let len = 1 + contrib;
        self.attacker_memo.insert((key, c), len);
        Ok(len)
    }

    /// Defender node `board`; every legal reply must be an attacker position
    /// or an immediate attacker win.
    fn defender(&mut self, board: &Board, bound: u16) -> Result<u16, ExportError> {
        let key = rules::key(board);
        let c = board.rule50();
        if let Some(&len) = self.defender_memo.get(&(key, c)) {
            return Ok(len);
        }
        let mut moves = MoveList::new();
        generate_legal(board, &mut moves);
        let mut len = 0u16;
        for m in &moves {
            let mut next = board.clone();
            next.do_move(m, &mut StateInfo::new());
            match rules::status(&next, self.attacker) {
                rules::Status::Won => {
                    // A defender reply cannot explode its own commoner (the
                    // rules generate no such move); if it ever did, the reply
                    // is an attacker win and contributes nothing.
                    debug_assert!(false, "defender reply produced a Won child");
                }
                rules::Status::Failed { .. } => {
                    return Err(if next.rule50() >= 100 {
                        ExportError::Rule50Limit
                    } else {
                        ExportError::Missing
                    });
                }
                rules::Status::Open => {
                    self.edge(key, rules::key(&next));
                    len = len.max(self.attacker(&next, next.rule50(), bound)?);
                }
            }
        }
        self.defender_memo.insert((key, c), len);
        Ok(len)
    }

    /// The shortest entry with `limit >= c` and `length < bound`. Ties: a
    /// move already chosen for this key at another counter, then the higher
    /// limit, then the lower `Move::to_bits`.
    fn choose(&self, key: u64, c: u16, bound: u16) -> Option<StoreEntry> {
        let chosen = self.chosen.get(&key).map(Vec::as_slice).unwrap_or(&[]);
        let mut best: Option<StoreEntry> = None;
        let mut entries = Vec::new();
        self.store.entries(key, &mut entries);
        for &e in &entries {
            if u16::from(e.limit) < c || e.length >= bound {
                continue;
            }
            let better = match best {
                None => true,
                Some(b) if e.length != b.length => e.length < b.length,
                Some(b) if chosen.contains(&e.mv) != chosen.contains(&b.mv) => {
                    chosen.contains(&e.mv)
                }
                Some(b) if e.limit != b.limit => e.limit > b.limit,
                Some(b) => e.mv.to_bits() < b.mv.to_bits(),
            };
            if better {
                best = Some(e);
            }
        }
        best
    }
}

/// Grey/black DFS for a cycle over the deduplicated position edges
/// (iterative). Returns `true` when a back edge exists.
pub fn has_cycle(root: u64, edges: &HashMap<u64, HashSet<u64>>) -> bool {
    #[derive(Clone, Copy, PartialEq)]
    enum Mark {
        Grey,
        Black,
    }
    let mut marks: HashMap<u64, Mark> = HashMap::new();
    let mut stack: Vec<(u64, bool)> = vec![(root, false)];
    while let Some((node, expanded)) = stack.pop() {
        if expanded {
            marks.insert(node, Mark::Black);
            continue;
        }
        if matches!(marks.get(&node), Some(Mark::Grey)) {
            return true;
        }
        if matches!(marks.get(&node), Some(Mark::Black)) {
            continue;
        }
        marks.insert(node, Mark::Grey);
        stack.push((node, true));
        if let Some(tos) = edges.get(&node) {
            let mut tos: Vec<u64> = tos.iter().copied().collect();
            tos.sort_unstable();
            for to in tos {
                match marks.get(&to) {
                    Some(Mark::Grey) => return true,
                    Some(Mark::Black) => {}
                    None => stack.push((to, false)),
                }
            }
        }
    }
    false
}

fn write_file(root: &Board, path: &Path, walk: WalkResult) -> Result<WriteStats, ExportError> {
    let tmp = tmp_path(path);
    let attempt = (|| -> std::io::Result<()> {
        let mut buf = String::new();
        let root_fen = serde_json::to_string(&root.fen_with(EnPassantMode::Legal)).unwrap();
        buf.push_str(&format!(
            "{{\"root\":{root_fen},\"moves\":{}}}\n",
            walk.root_length
        ));
        for (fen, mv) in &walk.lines {
            buf.push_str(&format!(
                "{{\"fen\":{},\"move\":{}}}\n",
                serde_json::to_string(fen).unwrap(),
                serde_json::to_string(mv).unwrap()
            ));
        }
        std::fs::write(&tmp, buf)?;
        std::fs::rename(&tmp, path)
    })();
    match attempt {
        Ok(()) => Ok(walk.stats),
        Err(_) => {
            let _ = std::fs::remove_file(&tmp);
            Err(ExportError::Io)
        }
    }
}

fn tmp_path(path: &Path) -> PathBuf {
    let mut s = path.as_os_str().to_os_string();
    s.push(".tmp");
    PathBuf::from(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{ProofEvent, ProofSink};
    use crate::strategy::store::ProofStore;

    fn insert(store: &mut ProofStore, key: u64, from: &str, length: u16, limit: u8) {
        let mv = Move::from_uci(from).unwrap();
        store.proven(ProofEvent::AttackerProven {
            key,
            mv,
            length,
            limit,
        });
    }

    #[test]
    fn cycle_detector() {
        // a -> b -> c -> b is a cross-line cycle; a -> b, a -> c, c -> b not.
        let mut edges: HashMap<u64, HashSet<u64>> = HashMap::new();
        let add = |e: &mut HashMap<u64, HashSet<u64>>, f: u64, t: u64| {
            e.entry(f).or_default().insert(t);
        };
        add(&mut edges, 1, 2);
        add(&mut edges, 2, 3);
        add(&mut edges, 3, 4);
        assert!(!has_cycle(1, &edges));
        add(&mut edges, 4, 2);
        assert!(has_cycle(1, &edges));
        let mut edges2: HashMap<u64, HashSet<u64>> = HashMap::new();
        add(&mut edges2, 1, 2);
        add(&mut edges2, 1, 3);
        add(&mut edges2, 3, 2);
        add(&mut edges2, 2, 4);
        assert!(!has_cycle(1, &edges2));
        // Diamond back to the root is a cycle.
        add(&mut edges2, 4, 1);
        assert!(has_cycle(1, &edges2));
    }

    #[test]
    fn store_full_fails_before_writing() {
        let root = Board::from_fen("k7/8/1Q6/8/8/8/8/K6R w - - 0 1").unwrap();
        let mut store = ProofStore::with_cap_bytes(0);
        store.proven(ProofEvent::AttackerProven {
            key: 1,
            mv: Move::from_uci("a2a3").unwrap(),
            length: 1,
            limit: 99,
        });
        assert!(store.is_full());
        let dir = std::env::temp_dir();
        let path = dir.join(format!("atomi-test-{}-full.jsonl", std::process::id()));
        assert!(matches!(
            write_proof(&root, &store, &path),
            Err(ExportError::StoreFull)
        ));
        assert!(!path.exists());
    }

    #[test]
    fn missing_fails_without_file() {
        // A proved root whose store is empty: the chosen entry is missing.
        let root = Board::from_fen("k7/8/1Q6/8/8/8/8/K6R w - - 0 1").unwrap();
        let store = ProofStore::with_cap_bytes(1 << 20);
        let path =
            std::env::temp_dir().join(format!("atomi-test-{}-missing.jsonl", std::process::id()));
        assert!(matches!(
            write_proof(&root, &store, &path),
            Err(ExportError::Missing)
        ));
        assert!(!path.exists());
        assert!(!path.with_extension("jsonl.tmp").exists());
    }

    #[test]
    fn uci_roundtrip() {
        // Castling both sides and wings, all four promotions, en passant.
        let cases = [
            ("r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1", vec!["e1g1", "e1c1"]),
            ("r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 0 1", vec!["e8g8", "e8c8"]),
            (
                "4k3/1P6/8/8/8/8/8/4K3 w - - 0 1",
                vec!["b7b8q", "b7b8r", "b7b8b", "b7b8n"],
            ),
            ("4k3/8/8/2pP4/8/8/8/4K3 w - c6 0 2", vec!["d5c6"]),
        ];
        for (fen, ucis) in cases {
            let b = Board::from_fen(fen).unwrap();
            let mut moves = MoveList::new();
            generate_legal(&b, &mut moves);
            for uci in ucis {
                let m = b
                    .move_from_uci(uci)
                    .unwrap_or_else(|| panic!("{fen}: {uci}"));
                assert!(b.is_legal(m), "{fen}: {uci} not legal");
                assert_eq!(m.to_uci(), uci);
            }
        }
    }

    #[test]
    fn choose_prefers_reused_move() {
        // Two counters, one position: the shortest entry does not hold at the
        // higher counter, so a second (reused) move is written.
        let mut store = ProofStore::with_cap_bytes(1 << 20);
        let key = 7u64;
        // Entry A: length 2, limit 5 — best at c=5 only if B absent... craft:
        // At c=0 choose length-2 entry (mv a2a3). At c=8 only the length-3
        // entry (limit 20) is usable — but its move differs, so two distinct
        // moves are recorded for the same key.
        insert(&mut store, key, "a2a3", 2, 5);
        insert(&mut store, key, "b2b3", 3, 20);
        let mut w = Walker {
            attacker: Color::White,
            store: &store,
            lines: Vec::new(),
            edges: HashMap::new(),
            attacker_memo: HashMap::new(),
            defender_memo: HashMap::new(),
            chosen: HashMap::new(),
        };
        let e0 = w.choose(key, 0, u16::MAX).unwrap();
        assert_eq!((e0.length, e0.mv.to_uci().as_str()), (2, "a2a3"));
        // Record the choice as made.
        w.chosen.entry(key).or_default().push(e0.mv);
        let e8 = w.choose(key, 8, u16::MAX).unwrap();
        assert_eq!((e8.length, e8.mv.to_uci().as_str()), (3, "b2b3"));
        // At c=8 the length-2 entry no longer holds → different move written.
        assert_ne!(e0.mv, e8.mv);
        // And when the same short entry would still hold, the chosen move is
        // reused: give the short entry a higher limit.
        let mut store2 = ProofStore::with_cap_bytes(1 << 20);
        insert(&mut store2, key, "a2a3", 2, 20);
        insert(&mut store2, key, "b2b3", 3, 20);
        let mut w2 = Walker {
            attacker: Color::White,
            store: &store2,
            lines: Vec::new(),
            edges: HashMap::new(),
            attacker_memo: HashMap::new(),
            defender_memo: HashMap::new(),
            chosen: HashMap::new(),
        };
        let f0 = w2.choose(key, 0, u16::MAX).unwrap();
        w2.chosen.entry(key).or_default().push(f0.mv);
        let f8 = w2.choose(key, 8, u16::MAX).unwrap();
        // Both hold at c=8: the shorter (a2a3) is still chosen anyway.
        assert_eq!(f8.mv.to_uci().as_str(), "a2a3");
    }
}
