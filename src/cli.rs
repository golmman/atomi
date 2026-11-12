// Justification for >10 KB: the option contract, the D33 deferred guide
// switches and their order tests are one unit.

//! `atomi prove` option parsing and output contract. Std-only parsing;
//! unknown options, missing values, bad numbers and bad FENs are errors
//! (`{"error":…}` on stdout, exit code 2).
//!
//! Output: exactly one JSON line in this field order:
//! `{"found","length","exit","child_evals","expansions","seed","sim_nodes",
//! "settle_nodes","aux_nodes","aux_skipped","ms","seconds","memory_mb",
//! "evals_limit","proof","proof_positions","proof_length","proof_entries",
//! "proof_full","proof_attacker_entries","proof_defender_entries",
//! "proof_archive_refused"}`, plus an optional `"stats"` object (the plan-12
//! attribution counters) when `--stats` is given.

use crate::guide::{GuideConfig, GuideParams, MECHANISMS};
use atomic_movegen::board::{Board, FenPolicy};

/// Deferred guide switches (D33).
///
/// `--guide hand|none` replaces the whole [`GuideConfig`], so applying it
/// where it appears in argv would silently drop every guide switch given
/// before it. The parser collects the base and the switches here instead and
/// applies the **base first, then the switches in argv order** ([`resolve`]),
/// which makes the resolved configuration independent of the order.
///
/// [`resolve`]: GuideArgs::resolve
#[derive(Default)]
pub struct GuideArgs {
    base: Option<GuideConfig>,
    /// `(mechanism, on, enables the guide)`.
    ops: Vec<(&'static str, bool, bool)>,
}

impl GuideArgs {
    /// An empty accumulator (the base stays `--guide none`).
    pub fn new() -> GuideArgs {
        GuideArgs::default()
    }
    /// Record `--guide none|hand` (the last one given wins).
    pub fn base(&mut self, name: &str) -> Result<(), String> {
        self.base = Some(match name {
            "none" => GuideConfig::none(),
            "hand" => GuideConfig::hand(),
            other => return Err(format!("bad value for --guide: {other}")),
        });
        Ok(())
    }

    /// Record a `--guide-on` (`on = true`) or `--guide-off` list.
    pub fn switches(&mut self, names: &str, on: bool) -> Result<(), String> {
        for name in names.split(',').filter(|s| !s.is_empty()) {
            let known = MECHANISMS
                .into_iter()
                .find(|&k| k == name)
                .ok_or_else(|| format!("unknown guide mechanism: {name}"))?;
            self.ops.push((known, on, on));
        }
        Ok(())
    }

    /// Record a value-less switch (`--guide-group`, `--guide-sim-exact`, …):
    /// it never enables the guide on its own.
    pub fn flag(&mut self, name: &str) -> Result<(), String> {
        self.switches(name, true)?;
        if let Some(op) = self.ops.last_mut() {
            op.2 = false;
        }
        Ok(())
    }

    /// The resolved configuration: base, then every switch in argv order.
    pub fn resolve(self) -> GuideConfig {
        let mut guide = self.base.unwrap_or_else(GuideConfig::none);
        for (name, on, enable) in self.ops {
            guide.set_switch(name, on).expect("name validated above");
            guide.enabled |= enable;
        }
        guide
    }
}

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
    /// Collect the plan-12 attribution counters.
    pub stats: bool,
    /// Opt-in ply cap (`--max-depth`, default off; measurement only).
    pub max_depth: Option<u16>,
}

/// Parse `prove` arguments (everything after the `prove` subcommand).
pub fn parse_prove(args: &[String]) -> Result<ProveOptions, String> {
    let mut fen: Option<String> = None;
    let mut seconds = 10u64;
    let mut memory_mb = 192usize;
    let mut evals = None;
    let mut proof_out = None;
    let mut guide_args = GuideArgs::new();
    let mut guide_params = GuideParams::default();
    let mut proof_share_percent = 25u8;
    let mut seed = 0u64;
    let mut stats = false;
    let mut max_depth = None;

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        // Value-less guide switches are deferred like every other guide
        // switch (D33), so `--guide hand` later in argv cannot drop them.
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
        if arg == "--stats" {
            stats = true;
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
                "--guide" => guide_args.base(value)?,
                "--guide-off" => guide_args.switches(value, false)?,
                "--guide-on" => guide_args.switches(value, true)?,
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
                "--guide-aux-budget" => {
                    guide_params.aux_budget = value
                        .parse()
                        .map_err(|_| format!("bad value for {arg}: {value}"))?;
                }
                "--guide-sim-plies" => {
                    guide_params.sim_plies = parse_num(arg, value)?;
                }
                "--max-depth" => {
                    let n: u16 = value
                        .parse()
                        .map_err(|_| format!("bad value for {arg}: {value}"))?;
                    if n == 0 {
                        return Err(format!("bad value for {arg}: {value}"));
                    }
                    max_depth = Some(n);
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
    let guide = guide_args.resolve();
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
        stats,
        max_depth,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::guide::EfPnMode;

    const FEN: &str = "4k3/8/8/8/8/8/4P3/4K3 w - - 0 1";

    fn parse(list: &[&str]) -> ProveOptions {
        let mut args = vec![FEN.to_string()];
        args.extend(list.iter().map(|s| s.to_string()));
        parse_prove(&args).expect("parse")
    }

    /// D33: `--guide hand` replaces the config it meets, so a switch given
    /// before it used to vanish silently. Base first, switches second, in
    /// either argv order.
    #[test]
    fn guide_switch_before_the_base_survives() {
        let before = parse(&["--guide-off", "efpn", "--guide", "hand"]);
        let after = parse(&["--guide", "hand", "--guide-off", "efpn"]);
        assert_eq!(before.guide, after.guide);
        assert_eq!(before.guide.ef_pn, EfPnMode::Off);
        assert!(before.guide.enabled && before.guide.move_classes);
        // A value-less switch is deferred the same way.
        let flag = parse(&["--guide-group", "--guide", "hand"]);
        assert!(flag.guide.group);
        assert!(parse(&["--guide", "hand"]).guide == GuideConfig::hand());
        assert!(!parse(&["--guide", "hand"]).guide.group);
    }

    /// The last base wins and every switch applies after it, in argv order.
    #[test]
    fn guide_base_then_switches_in_order() {
        let p = parse(&[
            "--guide-on",
            "classes",
            "--guide",
            "none",
            "--guide",
            "hand",
            "--guide-off",
            "sim,null-exact",
            "--guide-on",
            "sim",
        ]);
        assert!(p.guide.enabled && p.guide.sibling_sim);
        assert!(!p.guide.null_exact);
        assert!(p.guide.move_classes);
    }

    /// `--guide none` with no switch is the bare skeleton, exactly.
    #[test]
    fn guide_none_is_the_identity() {
        assert_eq!(parse(&[]).guide, GuideConfig::none());
        assert_eq!(parse(&["--guide", "none"]).guide, GuideConfig::none());
        // Switches still apply to a bare base (they never enable anything on
        // their own except through `--guide-on`).
        let off = parse(&["--guide", "none", "--guide-off", "classes"]);
        assert_eq!(off.guide, GuideConfig::none());
    }

    /// The plan-19 NUMBERS-V1 aggregate switch is accepted and resolves to
    /// the six mechanisms over the base it meets.
    #[test]
    fn numbers_switch_resolves_over_the_base() {
        let p = parse(&["--guide", "hand", "--guide-on", "numbers,sim-exact"]);
        assert!(p.guide.fresh_class && p.guide.efpn_banded && p.guide.evasions_count);
        assert!(p.guide.threat_count && p.guide.hanging_quiet && p.guide.unlikely_exact);
        assert!(p.guide.sim_exact && p.guide.sibling_sim);
        // Unknown mechanisms are parse errors whatever the order.
        let mut args = vec![FEN.to_string()];
        args.extend([
            "--guide".into(),
            "hand".into(),
            "--guide-on".into(),
            "nope".into(),
        ]);
        assert!(parse_prove(&args).is_err());
    }
}
