//! Justification for >10 KB: an independent validator must stay a single
//! self-contained module (its independence test audits this file).

//! Independent JSONL proof validator.
//!
//! **Independence rule:** this module uses only `std`, `serde_json` and
//! `atomic_movegen` — never other `crate::` modules (enforced by a test in
//! `tests/validate.rs`). Helpers shared with the prover (the insufficient-
//! material rule, the walk shape) are re-implemented here, not imported.
//!
//! Semantics follow the external checker's JSONL reader: the header is
//! `{"root": FEN, "moves": L}` where `moves` is informational (a `provari`
//! field is an error). Every other line with a `move` maps `(position,
//! counter) -> UCI`; position identity is the first four fields of the
//! re-rendered legal-en-passant FEN. The walk then checks the strategy the
//! same way the checker does, and any cycle in the union of all lines rejects
//! the proof.

use atomic_movegen::board::{Board, FenPolicy, StateInfo};
use atomic_movegen::movegen::generate_legal;
use atomic_movegen::types::{Color, EnPassantMode, MoveList, Outcome};
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// The validation verdict for one file.
#[derive(Debug)]
pub struct Report {
    /// The file name.
    pub file: String,
    /// Whether the proof is valid.
    pub valid: bool,
    /// Won attacker (position, counter) pairs (the checker's `won_positions`).
    pub positions: Option<usize>,
    /// The computed proof length in attacker moves.
    pub length: Option<u16>,
    /// The header's `moves` field (informational).
    pub header_moves: Option<u64>,
    /// The failure reason when `!valid`.
    pub error: Option<String>,
}

impl Report {
    /// The JSON output line.
    pub fn to_json(&self) -> String {
        let num = |o: &Option<u64>| o.map_or("null".to_string(), |n| n.to_string());
        let pos = self.positions.map(|n| n as u64);
        let len = self.length.map(u64::from);
        let err = self
            .error
            .as_deref()
            .map_or("null".to_string(), |e| serde_json::to_string(e).unwrap());
        format!(
            "{{\"file\":{},\"valid\":{},\"positions\":{},\"length\":{},\
             \"header_moves\":{},\"error\":{}}}",
            serde_json::to_string(&self.file).unwrap(),
            self.valid,
            num(&pos),
            num(&len),
            num(&self.header_moves),
            err
        )
    }
}

/// Validate a proof file on disk.
pub fn validate_file(path: &Path) -> Report {
    let file = path.display().to_string();
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => return report(file, Err(format!("cannot read: {e}"))),
    };
    let mut r = validate_text(&text);
    r.file = file;
    r
}

/// Validate proof text (the walk runs on a 256 MiB-stack thread).
pub fn validate_text(text: &str) -> Report {
    let owned = text.to_string();
    std::thread::scope(|scope| {
        std::thread::Builder::new()
            .stack_size(256 << 20)
            .spawn_scoped(scope, || run(&owned))
            .expect("spawn validator thread")
            .join()
            .expect("join validator thread")
    })
}

fn report(file: String, result: Result<(usize, u16, Option<u64>), String>) -> Report {
    match result {
        Ok((positions, length, header_moves)) => Report {
            file,
            valid: true,
            positions: Some(positions),
            length: Some(length),
            header_moves,
            error: None,
        },
        Err(error) => Report {
            file,
            valid: false,
            positions: None,
            length: None,
            header_moves: None,
            error: Some(error),
        },
    }
}

type Err<T> = Result<T, String>;

/// A parsed move line: exact position identity + the counter.
type LineKey = (String, u16);

fn run(text: &str) -> Report {
    report(String::new(), check(text))
}

fn check(text: &str) -> Err<(usize, u16, Option<u64>)> {
    let mut lines = text.lines().filter(|l| !l.trim().is_empty());
    let header = lines.next().ok_or("empty file")?;
    let header: serde_json::Value =
        serde_json::from_str(header).map_err(|e| format!("bad header JSON: {e}"))?;
    let obj = header.as_object().ok_or("header is not an object")?;
    if obj.contains_key("provari") {
        return Err("provari field present".into());
    }
    let root_fen = obj
        .get("root")
        .and_then(|v| v.as_str())
        .ok_or("header has no string root")?
        .to_string();
    let header_moves = obj.get("moves").and_then(|v| v.as_u64());

    let mut moves: HashMap<LineKey, String> = HashMap::new();
    for line in lines {
        let v: serde_json::Value =
            serde_json::from_str(line).map_err(|e| format!("bad line JSON: {e}"))?;
        let Some(mv) = v.get("move").and_then(|m| m.as_str()) else {
            continue;
        };
        let fen = v
            .get("fen")
            .and_then(|f| f.as_str())
            .ok_or("line with move lacks fen")?;
        let b = parse_board(fen)?;
        let key = (identity(&b), b.rule50());
        match moves.get(&key) {
            Some(prev) if prev != mv => {
                return Err(format!("conflicting duplicate at {fen}"));
            }
            Some(_) => {}
            None => {
                moves.insert(key, mv.to_string());
            }
        }
    }

    let root = parse_board(&root_fen)?;
    if root.rule50() >= 100 {
        return Err("root at 50-move limit".into());
    }
    if !matches!(classify(&root, root.side_to_move()), Class::Open) {
        return Err("root is over".into());
    }
    let mut w = Walk {
        attacker: root.side_to_move(),
        moves,
        won: HashMap::new(),
        won_attackers: 0,
        edges: HashMap::new(),
        on_stack: HashSet::new(),
    };
    let root_id = identity(&root);
    let length = w.attacker(&root)?;
    if has_cycle(&root_id, &w.edges) {
        return Err("cycle".into());
    }
    Ok((w.won_attackers, length, header_moves))
}

fn parse_board(fen: &str) -> Err<Board> {
    Board::from_fen_with(fen, FenPolicy::AllowTerminal).map_err(|_| format!("bad fen: {fen}"))
}

/// Exact position identity: placement, side, castling, legal ep square.
fn identity(b: &Board) -> String {
    let fen = b.fen_with(EnPassantMode::Legal);
    fen.split(' ').take(4).collect::<Vec<_>>().join(" ")
}

struct Walk {
    attacker: Color,
    moves: HashMap<LineKey, String>,
    /// Won (identity, counter) pairs -> length (attackers and defenders).
    won: HashMap<LineKey, u16>,
    /// Count of won attacker (position, counter) pairs — the checker's
    /// `won_positions` (capture wins included).
    won_attackers: usize,
    /// Identity edges over the union of all lines.
    edges: HashMap<String, HashSet<String>>,
    /// (identity, counter) pairs on the current walk stack.
    on_stack: HashSet<LineKey>,
}

impl Walk {
    fn edge(&mut self, from: &Board, to: &Board) {
        self.edges
            .entry(identity(from))
            .or_default()
            .insert(identity(to));
    }

    fn attacker(&mut self, board: &Board) -> Err<u16> {
        let id = identity(board);
        let c = board.rule50();
        if let Some(&len) = self.won.get(&(id.clone(), c)) {
            return Ok(len);
        }
        if !self.on_stack.insert((id.clone(), c)) {
            return Err("cycle".into());
        }
        let r = self.attacker_inner(board, &id, c);
        self.on_stack.remove(&(id, c));
        r
    }

    fn attacker_inner(&mut self, board: &Board, id: &str, c: u16) -> Err<u16> {
        let id = id.to_string();
        if let Some(uci) = self.moves.get(&(id.clone(), c)).cloned() {
            let m = board
                .move_from_uci(&uci)
                .ok_or_else(|| format!("illegal move {uci} at {id}"))?;
            let mut next = board.clone();
            next.do_move(m, &mut StateInfo::new());
            self.edge(board, &next);
            let contrib = match classify(&next, self.attacker) {
                Class::AttackerWon => 0,
                Class::Failed => {
                    return Err(format!("move {uci} at {id} leads to a failed position"));
                }
                Class::Open => self.defender(&next)?,
            };
            let len = 1 + contrib;
            self.won.insert((id, c), len);
            self.won_attackers += 1;
            return Ok(len);
        }
        // No stored move: a capture after which the attacker has won.
        let mut legal = MoveList::new();
        generate_legal(board, &mut legal);
        for m in &legal {
            if !board.is_capture(m) {
                continue;
            }
            let mut next = board.clone();
            next.do_move(m, &mut StateInfo::new());
            if matches!(classify(&next, self.attacker), Class::AttackerWon) {
                self.won.insert((id, c), 1);
                self.won_attackers += 1;
                return Ok(1);
            }
        }
        Err(format!("missing move at {id}"))
    }

    fn defender(&mut self, board: &Board) -> Err<u16> {
        let id = identity(board);
        let c = board.rule50();
        if let Some(&len) = self.won.get(&(id.clone(), c)) {
            return Ok(len);
        }
        if board.rule50() >= 100 {
            return Err(format!("50-move limit at {id}"));
        }
        let mut legal = MoveList::new();
        generate_legal(board, &mut legal);
        let mut len = 0u16;
        for m in &legal {
            let mut next = board.clone();
            next.do_move(m, &mut StateInfo::new());
            match classify(&next, self.attacker) {
                Class::AttackerWon => {}
                Class::Failed => {
                    return Err(format!("defender reply leads to a failed position at {id}"));
                }
                Class::Open => {
                    self.edge(board, &next);
                    len = len.max(self.attacker(&next)?);
                }
            }
        }
        self.won.insert((id, c), len);
        Ok(len)
    }
}

/// The validator's own end-of-game classification (same precedence as the
/// rules layer, re-implemented — never imported).
enum Class {
    /// The attacker wins: the defender's commoner is gone, or the defender
    /// is mated.
    AttackerWon,
    /// A draw or a defender win: never an attacker win.
    Failed,
    /// Undecided.
    Open,
}

fn classify(b: &Board, attacker: Color) -> Class {
    let stm = b.side_to_move();
    match b.outcome() {
        Some(Outcome::Loss) => {
            if stm == attacker {
                Class::Failed
            } else {
                Class::AttackerWon
            }
        }
        Some(Outcome::Win) => {
            if stm.flip() == attacker {
                Class::Failed
            } else {
                Class::AttackerWon
            }
        }
        Some(_) => Class::Failed,
        None if material_draw(b) => Class::Failed,
        None => Class::Open,
    }
}

/// The insufficient-material rule, restated for the validator (same rule as
/// the rules layer's copy, re-typed — see the module docs).
fn material_draw(board: &Board) -> bool {
    insufficient_material(board, Color::White) && insufficient_material(board, Color::Black)
}

fn insufficient_material(board: &Board, s: Color) -> bool {
    use atomic_movegen::types::{Bitboard, PieceType};
    let o = s.flip();
    let s_commoner = board.commoners(s);
    let o_commoner = board.commoners(o);
    if o_commoner.is_empty() {
        return false;
    }
    let s_others = board.pieces_color(s) & !s_commoner;
    if s_others.is_empty() {
        return true;
    }
    let o_others = board.pieces_color(o) & !o_commoner;
    let noncommoner = board.occupied() & !board.pieces_pt(PieceType::Commoner);
    if !o_others.is_empty() {
        let bishops = board.pieces_pt(PieceType::Bishop);
        if noncommoner != bishops {
            return false;
        }
        let s_b = bishops & board.pieces_color(s);
        let o_b = bishops & board.pieces_color(o);
        let mut light = 0u64;
        for sq in 0..64u64 {
            if (sq % 8 + sq / 8) % 2 == 1 {
                light |= 1 << sq;
            }
        }
        let light = Bitboard(light);
        let dark = !light;
        return (s_b & dark).is_empty() && (o_b & light).is_empty()
            || (s_b & light).is_empty() && (o_b & dark).is_empty();
    }
    if !(board.pieces_pt(PieceType::Queen) | board.pieces_pt(PieceType::Pawn)).is_empty() {
        return false;
    }
    let count = noncommoner.count();
    if count == 1 {
        let pt = board.piece_on(noncommoner.lsb()).type_of();
        return matches!(
            pt,
            Some(PieceType::Knight | PieceType::Bishop | PieceType::Rook)
        );
    }
    noncommoner == board.pieces_pt(PieceType::Knight) && count <= 2
}

/// Iterative DFS over the identity edge graph: any cycle rejects.
fn has_cycle(root: &str, edges: &HashMap<String, HashSet<String>>) -> bool {
    #[derive(Clone, Copy, PartialEq)]
    enum Mark {
        Grey,
        Black,
    }
    let mut marks: HashMap<String, Mark> = HashMap::new();
    let mut stack: Vec<(String, bool)> = vec![(root.to_string(), false)];
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
        marks.insert(node.clone(), Mark::Grey);
        stack.push((node.clone(), true));
        if let Some(tos) = edges.get(&node) {
            let mut tos: Vec<String> = tos.iter().cloned().collect();
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
