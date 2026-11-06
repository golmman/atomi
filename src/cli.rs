//! `atomi prove` option parsing and output contract. Std-only parsing;
//! unknown options, missing values, bad numbers and bad FENs are errors
//! (`{"error":…}` on stdout, exit code 2).
//!
//! Output: exactly one JSON line in this field order:
//! `{"found","length","exit","child_evals","expansions","ms","seconds",
//! "memory_mb","evals_limit","proof","proof_positions","proof_length"}`.

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
    /// Proof output path (Phase D).
    pub proof_out: Option<String>,
    /// Proof store byte cap in MiB (Phase D).
    pub proof_memory: usize,
}

/// Parse `prove` arguments (everything after the `prove` subcommand).
pub fn parse_prove(args: &[String]) -> Result<ProveOptions, String> {
    let mut fen: Option<String> = None;
    let mut seconds = 10u64;
    let mut memory_mb = 192usize;
    let mut evals = None;
    let mut proof_out = None;
    let mut proof_memory = 1024usize;

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg.starts_with('-') {
            let value = args
                .get(i + 1)
                .ok_or_else(|| format!("missing value for {arg}"))?;
            match arg.as_str() {
                "--seconds" => seconds = parse_num(arg, value)?,
                "--memory" => memory_mb = parse_num(arg, value)?,
                "--evals" => evals = Some(parse_num(arg, value)?),
                "--proof-out" => proof_out = Some(value.clone()),
                "--proof-memory" => proof_memory = parse_num(arg, value)?,
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
        proof_memory,
    })
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
