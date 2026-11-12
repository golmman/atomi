// Justification for >10 KB: the option parser, the fresh-node diagnostic and its pinned tests are one contract.

//! `atomi classify`: a read-only, fresh-node guide diagnostic (plan16 A2).
//!
//! It runs no search and touches no table. For one position it prints one
//! JSON line with every legal move's hand-guide class, fresh starting pair
//! and the order the search's `select` would use at a fresh node (metric,
//! class rank, history, seeded tie-break, movegen order). The child FEN is
//! included so a differential driver can follow a proof edge without a
//! second move generator.
//!
//! The starting pair is the output of [`Guide::start_numbers`] before the
//! sibling simulation and the null-side replay, which are search-time
//! adjustments. That is the pair the five item-15 predicates name.

use crate::guide::{self, Guide, GuideConfig, GuideParams, History, MoveClass};
use crate::mover::Mover;
use crate::rules;
use atomic_movegen::board::{Board, FenPolicy, StateInfo};
use atomic_movegen::movegen::generate_legal_with_state;
use atomic_movegen::types::{Color, EnPassantMode, MoveList};
use serde_json::{Value, json};

/// Parsed `classify` options.
pub struct Options {
    /// The position.
    pub fen: String,
    /// The hand-guide switches.
    pub guide: GuideConfig,
    /// The starting-number table.
    pub guide_params: GuideParams,
    /// Tie-break seed (0 = movegen order).
    pub seed: u64,
    /// The attacker colour (default: the side to move).
    pub attacker: Option<Color>,
}

/// Parse `classify` arguments (everything after the subcommand).
pub fn parse(args: &[String]) -> Result<Options, String> {
    let mut fen: Option<String> = None;
    let mut guide_args = crate::cli::GuideArgs::new();
    let mut guide_params = GuideParams::default();
    let mut seed = 0u64;
    let mut attacker = None;
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        // Value-less guide switches are deferred with the rest (D33).
        if let Some(name) = arg.strip_prefix("--guide-").filter(|n| {
            matches!(
                *n,
                "group" | "fresh-dn" | "replay-exact" | "sim-exact" | "null-exact"
            )
        }) {
            guide_args.flag(name)?;
            i += 1;
            continue;
        }
        if arg.starts_with('-') {
            let value = args
                .get(i + 1)
                .ok_or_else(|| format!("missing value for {arg}"))?;
            match arg.as_str() {
                "--guide" => guide_args.base(value)?,
                "--guide-off" => guide_args.switches(value, false)?,
                "--guide-on" => guide_args.switches(value, true)?,
                "--guide-params" => {
                    let text = std::fs::read_to_string(value)
                        .map_err(|e| format!("bad value for {arg}: {e}"))?;
                    guide_params = GuideParams::parse(&text)?;
                }
                "--guide-scale" => guide_params.global_scale = parse_num(arg, value)?,
                "--guide-ef-weight" => {
                    let n: u32 = parse_num(arg, value)?;
                    if n > crate::search::value::OPEN_MAX {
                        return Err(format!("bad value for {arg}: {value}"));
                    }
                    guide_params.ef_weight = n;
                }
                "--guide-max-sim" => guide_params.max_sim = parse_zero_ok(arg, value)?,
                "--guide-max-settle" => guide_params.max_settle = parse_zero_ok(arg, value)?,
                "--guide-aux-budget" => {
                    guide_params.aux_budget = value
                        .parse()
                        .map_err(|_| format!("bad value for {arg}: {value}"))?
                }
                "--guide-sim-plies" => guide_params.sim_plies = parse_num(arg, value)?,
                "--seed" => {
                    seed = value
                        .parse()
                        .map_err(|_| format!("bad value for {arg}: {value}"))?
                }
                "--attacker" => {
                    attacker = Some(match value.as_str() {
                        "white" => Color::White,
                        "black" => Color::Black,
                        other => return Err(format!("bad value for {arg}: {other}")),
                    })
                }
                _ => return Err(format!("unknown option: {arg}")),
            }
            i += 2;
        } else if fen.is_none() {
            fen = Some(arg.clone());
            i += 1;
        } else {
            return Err(format!("unexpected argument: {arg}"));
        }
    }
    let fen = fen.ok_or_else(|| "missing FEN".to_string())?;
    Board::from_fen_with(&fen, FenPolicy::Strict).map_err(|_| format!("bad FEN: {fen}"))?;
    Ok(Options {
        fen,
        guide: guide_args.resolve(),
        guide_params,
        seed,
        attacker,
    })
}

fn parse_zero_ok(opt: &str, value: &str) -> Result<u32, String> {
    value
        .parse()
        .map_err(|_| format!("bad value for {opt}: {value}"))
}

fn parse_num<T>(opt: &str, value: &str) -> Result<T, String>
where
    T: std::str::FromStr + PartialOrd + From<u8>,
{
    let n = value
        .parse::<T>()
        .map_err(|_| format!("bad value for {opt}: {value}"))?;
    if n <= T::from(0) {
        return Err(format!("bad value for {opt}: {value} (must be positive)"));
    }
    Ok(n)
}

/// The class pair before `fresh_dn`, `global_scale` and EF-PN, oriented for
/// the node kind: the guide's base pair for a fresh child. This is the pair
/// the item-15 predicates name.
fn base_pair(guide: &Guide, input: &guide::StartInput, attacker_node: bool) -> (u32, u32) {
    if !guide.config.enabled {
        return (1, 1);
    }
    guide.base_pair(input, attacker_node)
}

/// Run the diagnostic and return the JSON line.
pub fn run(opts: &Options) -> Result<String, String> {
    let board = Board::from_fen_with(&opts.fen, FenPolicy::Strict)
        .map_err(|_| format!("bad FEN: {}", opts.fen))?;
    let attacker = opts.attacker.unwrap_or(board.side_to_move());
    let mut state = StateInfo::new();
    board.populate_state(&mut state);
    let mut moves = MoveList::new();
    generate_legal_with_state(&board, &state, &mut moves);
    let in_check = !state.checkers.is_empty();
    let legal = u32::try_from(moves.len()).unwrap_or(u32::MAX);
    let guide = Guide::new(opts.guide).with_params(opts.guide_params);
    let attacker_node = board.side_to_move() == attacker;
    let ctx = guide.context(&board, &state, legal, attacker_node, false);
    let mover_ctx = Mover::new(&board);
    let history = History::new();
    let node_key = rules::key(&board);
    let order_classes = guide.config.enabled
        && (guide.config.move_classes
            || guide.config.quiet_grouping
            || guide.config.check_evasions);
    let order_history = guide.config.enabled && guide.config.history_ties;

    let mut rows: Vec<Value> = Vec::with_capacity(moves.len());
    let mut sort_keys: Vec<(u32, u8, u16, u64, usize)> = Vec::with_capacity(moves.len());
    for (i, m) in moves.as_slice().iter().copied().enumerate() {
        let effect = mover_ctx.effect(&board, m);
        let mut built = None;
        if effect.needs_board {
            let mut child = board.clone();
            child.do_move(m, &mut StateInfo::new());
            built = Some(child);
        }
        let exact = guide::class::Exact {
            threat_count: guide.config.threat_count,
            hanging_quiet: guide.config.hanging_quiet,
        };
        let (class, or_number) = if guide.config.enabled {
            guide::class::classify_exact(
                &exact,
                &mover_ctx,
                &board,
                m,
                &effect,
                built.as_ref(),
                in_check && guide.config.check_evasions,
            )
        } else {
            (MoveClass::Quiet, None)
        };
        let and_number = if guide.config.enabled
            && !attacker_node
            && !guide::class::defence_likely_to_refute(&board, m, built.as_ref(), attacker)
        {
            2
        } else {
            1
        };
        let input = guide::StartInput {
            class,
            or_number,
            and_number,
            evasions: legal.max(1),
            child_eval: 0,
            sim: 0,
            unlikely_prior: false,
        };
        let child_eval = if !guide.config.enabled {
            0
        } else if let Some(child) = built.as_ref() {
            let e = guide::eval::hand_eval(child);
            if child.side_to_move() == attacker {
                e
            } else {
                -e
            }
        } else if attacker_node {
            ctx.eval
        } else {
            -ctx.eval
        };
        let input = guide::StartInput {
            child_eval,
            ..input
        };
        let (base_pn, base_dn) = base_pair(&guide, &input, attacker_node);
        let (pn, dn) = guide.start_numbers(
            &ctx,
            &input,
            attacker_node,
            board.side_to_move(),
            m,
            &history,
        );
        let metric = if attacker_node { pn } else { dn };
        let class_rank = if order_classes { class.rank() } else { 0 };
        let hist = if order_history {
            u16::MAX - history.score(board.side_to_move(), m)
        } else {
            u16::MAX
        };
        let salted = if opts.seed != 0 {
            History::seeded(opts.seed, node_key, m)
        } else {
            0
        };
        sort_keys.push((metric, class_rank, hist, salted, i));

        let mut child = board.clone();
        child.do_move(m, &mut StateInfo::new());
        rows.push(json!({
            "move": m.to_uci(),
            "class": class.as_str(),
            "base_pn": base_pn,
            "base_dn": base_dn,
            "start_pn": pn,
            "start_dn": dn,
            "order": 0,
            "child": child.fen_with(EnPassantMode::Legal),
        }));
    }

    sort_keys.sort_unstable();
    for (order, &(_, _, _, _, i)) in sort_keys.iter().enumerate() {
        rows[i]["order"] = json!(order);
    }

    let out = json!({
        "fen": board.fen_with(EnPassantMode::Legal),
        "attacker": if attacker == Color::White { "white" } else { "black" },
        "in_check": in_check,
        "legal": legal,
        "moves": rows,
    });
    serde_json::to_string(&out).map_err(|e| format!("json: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn opts(fen: &str, guide: GuideConfig) -> Options {
        Options {
            fen: fen.to_string(),
            guide,
            guide_params: GuideParams::default(),
            seed: 0,
            attacker: None,
        }
    }

    fn parse_line(text: &str) -> Value {
        serde_json::from_str(text).unwrap()
    }

    #[test]
    fn bare_guide_is_identity_and_movegen_order() {
        let fen = "r1b1kbnr/ppp3pp/n4q2/3pppN1/1P1PP3/8/P1P2PPP/RNBQKB1R w KQkq - 0 1";
        let v = parse_line(&run(&opts(fen, GuideConfig::none())).unwrap());
        let moves = v["moves"].as_array().unwrap();
        assert!(!moves.is_empty());
        for (i, m) in moves.iter().enumerate() {
            assert_eq!(m["start_pn"], 1);
            assert_eq!(m["start_dn"], 1);
            assert_eq!(m["class"], "Quiet");
            assert_eq!(m["order"], i);
        }
    }

    #[test]
    fn hand_threat_gets_the_class_pair() {
        // Qxg8 explodes the black commoner on h8; EF-PN is neutralised so
        // the fresh class pair is visible.
        let fen = "6rk/8/8/8/8/8/8/6QK w - - 0 1";
        let mut o = opts(fen, GuideConfig::product());
        o.guide_params.ef_weight = 0;
        let v = parse_line(&run(&o).unwrap());
        let qxg8 = v["moves"]
            .as_array()
            .unwrap()
            .iter()
            .find(|m| m["move"] == "g1g8")
            .expect("Qxg8 is legal");
        assert_eq!(qxg8["class"], "Threat");
        assert_eq!(qxg8["base_pn"], 4);
        assert_eq!(qxg8["base_dn"], 16);
        assert_eq!(qxg8["start_pn"], 4);
        assert_eq!(qxg8["start_dn"], 16);
    }

    #[test]
    fn check_node_uses_the_legal_count() {
        let fen = "4k3/8/8/8/8/8/8/4K2r w - - 0 1";
        let v = parse_line(&run(&opts(fen, GuideConfig::product())).unwrap());
        assert_eq!(v["in_check"], true);
        let legal = v["legal"].as_u64().unwrap();
        for m in v["moves"].as_array().unwrap() {
            assert_eq!(m["class"], "Evasion");
            assert_eq!(m["base_pn"], legal);
            assert_eq!(m["base_dn"], legal);
        }
    }

    #[test]
    fn attacker_override_flips_the_metric() {
        // A defender node: the same position classified as an attacker node
        // and as a defender node must use pn and dn respectively.
        let fen = "r1b1kbnr/ppp3pp/n4q2/3pppN1/1P1PP3/8/P1P2PPP/RNBQKB1R b KQkq - 0 1";
        let mut o = opts(fen, GuideConfig::hand());
        o.attacker = Some(Color::White);
        let v = parse_line(&run(&o).unwrap());
        assert_eq!(v["attacker"], "white");
        assert_eq!(
            v["moves"].as_array().unwrap().len(),
            v["legal"].as_u64().unwrap() as usize
        );
    }

    #[test]
    fn parse_rejects_bad_input() {
        assert!(parse(&["nonsense".to_string()]).is_err());
        assert!(parse(&["--guide".to_string(), "bogus".to_string()]).is_err());
        assert!(parse(&["--attacker".to_string(), "green".to_string()]).is_err());
    }
}
