//! `atomi prove` option parsing and output contract. Std-only parsing;
//! unknown options, missing values, bad numbers and bad FENs are errors
//! (`{"error":…}` on stdout, exit code 2).
//!
//! Output: exactly one JSON line in this field order:
//! `{"found","length","exit","child_evals","expansions","seed","sim_nodes",
//! "settle_nodes","ms","seconds","memory_mb","evals_limit","proof",
//! "proof_positions","proof_length","proof_entries","proof_full"}`.

use crate::guide::{GuideConfig, GuideParams};
use atomic_movegen::board::{Board, FenPolicy};

/// Parsed `prove` options.
pub struct ProveOptions {
    /// The root position (already `FenPolicy::Strict`-validated).
    pub fen: String,
    /// Wall-clock budget in seconds.
    pub seconds: u64,
    /// Table size label in MiB.
    pub memory_mb: usize,
    /// `child_evals` cap.
    pub evals: Option<u64>,
    /// Proof output path.
    pub proof_out: Option<String>,
    /// The hand-guide switches.
    pub guide: GuideConfig,
    /// The starting-number table.
    pub guide_params: GuideParams,
    /// Share of `--memory` given to the proof table, in percent (default 25).
    pub proof_share_percent: u8,
    /// Tie-break seed (0 = movegen order).
    pub seed: u64,
}

/// Parse `prove` arguments (everything after the `prove` subcommand).
pub fn parse_prove(args: &[String]) -> Result<ProveOptions, String> {
    let mut fen: Option<String> = None;
    let mut seconds = 10u64;
    let mut memory_mb = 192usize;
    let mut evals = None;
    let mut proof_out = None;
    let mut guide = GuideConfig::none();
    let mut guide_params = GuideParams::default();
    let mut proof_share_percent = 25u8;
    let mut seed = 0u64;

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg == "--guide-group" {
            // Item-15 kill-test switch: a value-less flag, default off.
            guide.group = true;
            i += 1;
            continue;
        }
        if arg.starts_with('-') {
            let value = args
                .get(i + 1)
                .ok_or_else(|| format!("missing value for {arg}"))?;
            match arg.as_str() {
                "--seconds" => seconds = parse_num(arg, value)?,
                "--memory" => memory_mb = parse_num(arg, value)?,
                "--evals" => evals = Some(parse_num(arg, value)?),
                "--proof-out" => proof_out = Some(value.clone()),
                "--seed" => {
                    seed = value
                        .parse()
                        .map_err(|_| format!("bad value for {arg}: {value}"))?
                }
                "--guide" => {
                    guide = match value.as_str() {
                        "none" => GuideConfig::none(),
                        "hand" => GuideConfig::hand(),
                        other => return Err(format!("bad value for {arg}: {other}")),
                    }
                }
                "--guide-off" => {
                    for name in value.split(',').filter(|s| !s.is_empty()) {
                        guide.set_off(name)?;
                    }
                }
                "--guide-on" => {
                    for name in value.split(',').filter(|s| !s.is_empty()) {
                        guide.set_on(name)?;
                    }
                }
                "--guide-params" => {
                    let text = std::fs::read_to_string(value)
                        .map_err(|e| format!("bad value for {arg}: {e}"))?;
                    guide_params = GuideParams::parse(&text)?;
                }
                "--guide-scale" => {
                    guide_params.global_scale = parse_num(arg, value)?;
                }
                "--guide-ef-weight" => {
                    let n: u32 = value
                        .parse()
                        .map_err(|_| format!("bad value for {arg}: {value}"))?;
                    if n > crate::search::value::OPEN_MAX {
                        return Err(format!("bad value for {arg}: {value}"));
                    }
                    guide_params.ef_weight = n;
                }
                "--guide-max-sim" => {
                    guide_params.max_sim = parse_zero_ok(arg, value)?;
                }
                "--guide-max-settle" => {
                    guide_params.max_settle = parse_zero_ok(arg, value)?;
                }
                "--proof-share" => {
                    let n: u8 = value
                        .parse()
                        .map_err(|_| format!("bad value for {arg}: {value}"))?;
                    if !(1..=90).contains(&n) {
                        return Err(format!("bad value for {arg}: {value} (1..=90)"));
                    }
                    proof_share_percent = n;
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
    Ok(ProveOptions {
        fen,
        seconds,
        memory_mb,
        evals,
        proof_out,
        guide,
        guide_params,
        proof_share_percent,
        seed,
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

/// Escape a string for inclusion in a JSON string literal.
pub fn json_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}
