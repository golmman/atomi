// Justification for >10 KB: the config/params contract, the orientation rules and the unit tests that pin them are one unit.

//! The hand guide (D21/D22): move classes, check evasions, quiet grouping,
//! EF-PN scaling, defender priors, history, simulation and the null-side
//! replay.
//!
//! The guide writes only fresh `Open` numbers and tie-breaks. It never edits
//! `Proved`/`Refuted` values, never searches the null position and never
//! touches the tables. The one exact closure it adds is the verified
//! one-ply explosion settle in `src/search/frame.rs`.

pub mod class;
pub mod eval;
pub mod history;
pub mod replay;
pub mod simulate;
pub mod threat;

pub use class::MoveClass;
pub use history::History;
pub use threat::ThreatSignature;

use crate::search::value::OPEN_MAX;
use atomic_movegen::board::{Board, StateInfo};
use atomic_movegen::types::{Color, Move};

/// EF-PN application mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EfPnMode {
    /// No EF-PN scaling.
    #[default]
    Off,
    /// Scale fresh numbers at attacker (OR) nodes only (the default).
    OrOnly,
    /// Scale at both node kinds.
    Both,
}

impl EfPnMode {
    /// Parse `off|or_only|both`.
    pub fn parse(s: &str) -> Option<EfPnMode> {
        match s {
            "off" => Some(EfPnMode::Off),
            "or_only" => Some(EfPnMode::OrOnly),
            "both" => Some(EfPnMode::Both),
            _ => None,
        }
    }
}

/// One runtime switch per mechanism, so the ablations are flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GuideConfig {
    /// `--guide hand` (false = `--guide none`).
    pub enabled: bool,
    /// Move-class starting numbers.
    pub move_classes: bool,
    /// Check = number of evasions.
    pub check_evasions: bool,
    /// Quiet grouping (the quiet class pair).
    pub quiet_grouping: bool,
    /// EF-PN hand-eval scaling.
    pub ef_pn: EfPnMode,
    /// Per-child static closures (the one-ply settle).
    pub static_closures: bool,
    /// `UNLIKELY` + defence history scaling.
    pub defender_priors: bool,
    /// History tie-breaks.
    pub history_ties: bool,
    /// Sibling simulation.
    pub sibling_sim: bool,
    /// Null-side threat replay.
    pub null_replay: bool,
    /// The item-15 attacker plain-move `GROUP` child (`--guide-group`,
    /// default off; the kill-test flag).
    pub group: bool,
    /// Plan-13 conformance probe: fresh disproof numbers are 1
    /// (`--guide-fresh-dn`, default off).
    pub fresh_dn: bool,
    /// Plan-13 conformance probe: exact sibling proof replay
    /// (`--guide-replay-exact`, default off).
    pub replay_exact: bool,
}

impl GuideConfig {
    /// The bare skeleton (`--guide none`).
    pub fn none() -> GuideConfig {
        GuideConfig::default()
    }

    /// The full hand-guide stack (`--guide hand`).
    pub fn hand() -> GuideConfig {
        GuideConfig {
            enabled: true,
            move_classes: true,
            check_evasions: true,
            quiet_grouping: true,
            ef_pn: EfPnMode::OrOnly,
            static_closures: true,
            defender_priors: true,
            history_ties: true,
            sibling_sim: true,
            null_replay: true,
            group: false,
            fresh_dn: false,
            replay_exact: false,
        }
    }

    /// Switch one named mechanism off (`--guide-off`).
    pub fn set_off(&mut self, name: &str) -> Result<(), String> {
        self.set(name, false)
    }

    /// Switch one named mechanism on (`--guide-on`).
    pub fn set_on(&mut self, name: &str) -> Result<(), String> {
        self.set(name, true)?;
        self.enabled = true;
        Ok(())
    }

    fn set(&mut self, name: &str, on: bool) -> Result<(), String> {
        match name {
            "classes" => self.move_classes = on,
            "evasions" => self.check_evasions = on,
            "quiet" => self.quiet_grouping = on,
            "efpn" => self.ef_pn = if on { EfPnMode::OrOnly } else { EfPnMode::Off },
            "closures" => self.static_closures = on,
            "priors" => self.defender_priors = on,
            "history" => self.history_ties = on,
            "sim" => self.sibling_sim = on,
            "replay" => self.null_replay = on,
            "group" => self.group = on,
            "fresh-dn" => self.fresh_dn = on,
            "replay-exact" => self.replay_exact = on,
            _ => return Err(format!("unknown guide mechanism: {name}")),
        }
        Ok(())
    }

    /// Whether any tie-break beyond the metric is active.
    pub fn any_order(&self) -> bool {
        self.enabled
            && (self.move_classes
                || self.quiet_grouping
                || self.check_evasions
                || self.history_ties)
    }
}

/// A mover-perspective `(pn, dn)` base pair.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pair {
    /// Proof-number base.
    pub pn: u32,
    /// Disproof-number base.
    pub dn: u32,
}

/// The tunable starting-number table (one `key=value` dump for the external
/// optimizer contract; no optimizer lives here).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GuideParams {
    /// Threat pair.
    pub threat: Pair,
    /// Weak-threat pair.
    pub weak: Pair,
    /// Quiet pair.
    pub quiet: Pair,
    /// Hanging pair.
    pub hanging: Pair,
    /// Evasion pair (the base is the legal-move count; this pair is the
    /// fallback when the count is zero).
    pub evasion: Pair,
    /// Global multiplier on fresh start numbers (`1` = plan3 identity).
    pub global_scale: u32,
    /// EF-PN weight (`16` = plan3 identity; `0` neutralises the eval).
    pub ef_weight: u32,
    /// Per-frame settle budget in settle work units (`0` = unlimited).
    pub max_settle: u32,
    /// Per-frame simulation budget in playout moves (`0` = unlimited).
    pub max_sim: u32,
    /// Global auxiliary-work budget (settle movegens + playout moves,
    /// `0` = unlimited). Deterministic, never wall-clock.
    pub aux_budget: u64,
    /// Maximum plies of one simulation playout (`6` = plan3 identity).
    pub sim_plies: u32,
}

impl Default for GuideParams {
    fn default() -> Self {
        GuideParams {
            threat: Pair { pn: 4, dn: 16 },
            weak: Pair { pn: 8, dn: 12 },
            quiet: Pair { pn: 9, dn: 9 },
            hanging: Pair { pn: 15, dn: 9 },
            evasion: Pair { pn: 1, dn: 1 },
            global_scale: 1,
            ef_weight: 16,
            max_settle: 0,
            max_sim: 0,
            aux_budget: 0,
            sim_plies: u32::from(simulate::SIM_PLIES),
        }
    }
}

impl GuideParams {
    /// The base pair for `class`.
    pub fn pair(&self, class: MoveClass) -> Pair {
        match class {
            MoveClass::Threat => self.threat,
            MoveClass::WeakThreat => self.weak,
            MoveClass::Quiet => self.quiet,
            MoveClass::Hanging => self.hanging,
            MoveClass::Evasion => self.evasion,
        }
    }

    /// Parse a `key=value` file (absent file = built-in table). Unknown keys
    /// and bad values are errors.
    pub fn parse(text: &str) -> Result<GuideParams, String> {
        let mut p = GuideParams::default();
        for (index, raw) in text.lines().enumerate() {
            let line = raw.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            let (key, value) = line
                .split_once('=')
                .ok_or_else(|| format!("params line {}: expected key=value", index + 1))?;
            let v: u64 = value
                .trim()
                .parse()
                .map_err(|_| format!("params line {}: bad value {value}", index + 1))?;
            let zero_ok = matches!(
                key.trim(),
                "ef_weight" | "max_sim" | "max_settle" | "aux_budget" | "sim_plies"
            );
            if (v == 0 && !zero_ok) || v > u64::from(OPEN_MAX) {
                return Err(format!("params line {}: {v} out of range", index + 1));
            }
            let v32 = v as u32;
            match key.trim() {
                "threat_pn" => p.threat.pn = v32,
                "threat_dn" => p.threat.dn = v32,
                "weak_pn" => p.weak.pn = v32,
                "weak_dn" => p.weak.dn = v32,
                "quiet_pn" => p.quiet.pn = v32,
                "quiet_dn" => p.quiet.dn = v32,
                "hanging_pn" => p.hanging.pn = v32,
                "hanging_dn" => p.hanging.dn = v32,
                "evasion_pn" => p.evasion.pn = v32,
                "evasion_dn" => p.evasion.dn = v32,
                "global_scale" => p.global_scale = v32,
                "ef_weight" => p.ef_weight = v32,
                "max_sim" => p.max_sim = v32,
                "max_settle" => p.max_settle = v32,
                "aux_budget" => p.aux_budget = v,
                "sim_plies" => p.sim_plies = v32,
                other => return Err(format!("params: unknown key {other}")),
            }
        }
        Ok(p)
    }

    /// A stable `key=value` dump (the optimizer contract).
    pub fn dump(&self) -> String {
        let rows = [
            ("threat", self.threat),
            ("weak", self.weak),
            ("quiet", self.quiet),
            ("hanging", self.hanging),
            ("evasion", self.evasion),
        ];
        let mut out = String::new();
        for (name, p) in rows {
            out.push_str(&format!("{name}_pn={}\n{name}_dn={}\n", p.pn, p.dn));
        }
        out.push_str(&format!(
            "global_scale={}\nef_weight={}\nmax_sim={}\nmax_settle={}\n\
             aux_budget={}\nsim_plies={}\n",
            self.global_scale,
            self.ef_weight,
            self.max_sim,
            self.max_settle,
            self.aux_budget,
            self.sim_plies
        ));
        out
    }
}

/// Per-node guide context (computed once per frame).
#[derive(Debug, Clone, Copy)]
pub struct NodeContext {
    /// The side to move is in check.
    pub in_check: bool,
    /// Legal moves at this node.
    pub legal: u32,
    /// Hand eval from the side to move's perspective.
    pub eval: i32,
    /// The other side's explosion threats (the replay baseline).
    pub null_sig: Option<ThreatSignature>,
}

/// The guide facade: config + parameters.
pub struct Guide {
    /// The runtime switches.
    pub config: GuideConfig,
    /// The starting-number table.
    pub params: GuideParams,
}

impl Guide {
    /// Build a guide from a config (built-in params).
    pub fn new(config: GuideConfig) -> Guide {
        Guide {
            config,
            params: GuideParams::default(),
        }
    }

    /// Replace the parameters (the `--guide-params` file).
    pub fn with_params(mut self, params: GuideParams) -> Guide {
        self.params = params;
        self
    }

    /// Build the node context. No movegen beyond the caller's own.
    pub fn context(
        &self,
        board: &Board,
        state: &StateInfo,
        legal: u32,
        need_null: bool,
    ) -> NodeContext {
        let in_check = !state.checkers.is_empty();
        if !self.config.enabled {
            return NodeContext {
                in_check,
                legal,
                eval: 0,
                null_sig: None,
            };
        }
        let eval = if self.config.ef_pn != EfPnMode::Off {
            eval::hand_eval(board)
        } else {
            0
        };
        let null_sig = if need_null
            && self.config.null_replay
            && !in_check
            && !board.commoners(board.side_to_move()).is_empty()
        {
            Some(threat::signature_for(board, board.side_to_move().flip()))
        } else {
            None
        };
        NodeContext {
            in_check,
            legal,
            eval,
            null_sig,
        }
    }

    /// The starting `(pn, dn)` pair for a fresh child, attacker-oriented.
    #[allow(clippy::too_many_arguments)]
    pub fn start_numbers(
        &self,
        ctx: &NodeContext,
        class: MoveClass,
        attacker_node: bool,
        side: Color,
        mv: Move,
        history: &History,
        child_eval: i32,
    ) -> (u32, u32) {
        if !self.config.enabled {
            return (1, 1);
        }
        let base = if class == MoveClass::Evasion && self.config.check_evasions {
            Pair {
                pn: ctx.legal.max(1),
                dn: ctx.legal.max(1),
            }
        } else if class == MoveClass::Quiet {
            if self.config.quiet_grouping {
                self.params.quiet
            } else {
                Pair { pn: 1, dn: 1 }
            }
        } else if self.config.move_classes {
            self.params.pair(class)
        } else {
            Pair { pn: 1, dn: 1 }
        };
        let (mut pn, mut dn) = if attacker_node {
            (base.pn, base.dn)
        } else {
            (base.dn, base.pn)
        };
        // Plan-13 conformance probe: a fresh disproof number is 1, not the
        // invented complementary base.
        if self.config.fresh_dn {
            dn = 1;
        }
        // Calibration (plan4): the fresh base magnitude only. Decided values
        // never pass through here.
        let scale = u64::from(self.params.global_scale);
        if scale != 1 {
            pn = (u64::from(pn) * scale).min(u64::from(OPEN_MAX)) as u32;
            dn = (u64::from(dn) * scale).min(u64::from(OPEN_MAX)) as u32;
        }
        // Defence/attacker history: good for the mover is good for the
        // mover's orientation, so at AND nodes the attacker numbers invert.
        if self.config.defender_priors {
            let h = i32::from(history.score(side, mv));
            if h > 0 {
                let e = if attacker_node { h } else { -h };
                (pn, dn) = eval::scale(pn, dn, e);
            }
        }
        if self.config.ef_pn != EfPnMode::Off
            && (attacker_node || self.config.ef_pn == EfPnMode::Both)
        {
            (pn, dn) = eval::scale_weighted(pn, dn, child_eval, self.params.ef_weight);
        }
        (pn.clamp(1, OPEN_MAX), dn.clamp(1, OPEN_MAX))
    }

    /// The simulation adjustment: `score` is from the attacker's perspective
    /// (`+` = good for the attacker), applied to attacker-oriented numbers.
    pub fn sim_adjust(&self, pn: u32, dn: u32, score: i32) -> (u32, u32) {
        if !self.config.sibling_sim || score == 0 {
            return (pn, dn);
        }
        let (pn, dn) = eval::scale(pn, dn, score / 8);
        (pn.clamp(1, OPEN_MAX), dn.clamp(1, OPEN_MAX))
    }

    /// The `UNLIKELY` defender prior on an attacker-oriented AND-child pair.
    pub fn unlikely(&self, pn: u32, dn: u32) -> (u32, u32) {
        (
            (pn / 2).clamp(1, OPEN_MAX),
            (u64::from(dn) * 2).min(u64::from(OPEN_MAX)) as u32,
        )
    }

    /// The replay comparison at an AND node: `(parried, created)`.
    pub fn replay(
        &self,
        ctx: &NodeContext,
        child: &Board,
        attacker: Color,
    ) -> Option<(bool, bool)> {
        let null = ctx.null_sig?;
        if !self.config.null_replay {
            return None;
        }
        let after = threat::signature_for(child, attacker);
        let parried = null.removed_from(after) != 0;
        let created = !threat::signature_for(child, attacker.flip()).is_empty();
        Some((parried, created))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use atomic_movegen::board::StateInfo;
    use atomic_movegen::types::Square;

    fn ctx(in_check: bool) -> NodeContext {
        NodeContext {
            in_check,
            legal: 12,
            eval: 0,
            null_sig: None,
        }
    }

    fn mv() -> Move {
        Move::make_move(Square::E2, Square::E4)
    }

    #[test]
    fn disabled_guide_is_identity() {
        let g = Guide::new(GuideConfig::none());
        let h = History::new();
        assert_eq!(
            g.start_numbers(
                &ctx(false),
                MoveClass::Threat,
                true,
                Color::White,
                mv(),
                &h,
                0
            ),
            (1, 1)
        );
        assert_eq!(
            g.start_numbers(
                &ctx(true),
                MoveClass::Evasion,
                false,
                Color::White,
                mv(),
                &h,
                0
            ),
            (1, 1)
        );
    }

    #[test]
    fn class_pairs_are_oriented_and_positive() {
        let g = Guide::new(GuideConfig::hand());
        let h = History::new();
        let c = NodeContext {
            eval: 0,
            ..ctx(false)
        };
        let (pn, dn) = g.start_numbers(&c, MoveClass::Threat, true, Color::White, mv(), &h, 0);
        assert_eq!((pn, dn), (4, 16));
        let (pn, dn) = g.start_numbers(&c, MoveClass::Threat, false, Color::White, mv(), &h, 0);
        assert_eq!((pn, dn), (16, 4));
        for class in [
            MoveClass::Threat,
            MoveClass::WeakThreat,
            MoveClass::Quiet,
            MoveClass::Hanging,
            MoveClass::Evasion,
        ] {
            for attacker_node in [true, false] {
                let (pn, dn) = g.start_numbers(&c, class, attacker_node, Color::White, mv(), &h, 0);
                assert!(pn >= 1 && dn >= 1 && pn <= OPEN_MAX && dn <= OPEN_MAX);
            }
        }
    }

    #[test]
    fn evasion_base_is_the_legal_move_count() {
        let g = Guide::new(GuideConfig::hand());
        let h = History::new();
        let c = NodeContext {
            legal: 7,
            ..ctx(true)
        };
        assert_eq!(
            g.start_numbers(&c, MoveClass::Evasion, true, Color::White, mv(), &h, 0),
            (7, 7)
        );
    }

    #[test]
    fn history_and_eval_stay_in_range() {
        let g = Guide::new(GuideConfig::hand());
        let mut h = History::new();
        for _ in 0..100 {
            h.observe(Color::White, mv(), true);
        }
        let c = NodeContext {
            eval: 256,
            ..ctx(false)
        };
        for attacker_node in [true, false] {
            let (pn, dn) = g.start_numbers(
                &c,
                MoveClass::Quiet,
                attacker_node,
                Color::White,
                mv(),
                &h,
                0,
            );
            assert!(pn >= 1 && dn >= 1 && pn < crate::search::value::INFINITY);
            assert!(dn < crate::search::value::INFINITY);
        }
    }

    #[test]
    fn unlikely_orientation() {
        let g = Guide::new(GuideConfig::hand());
        assert_eq!(g.unlikely(10, 10), (5, 20));
        assert_eq!(g.unlikely(1, OPEN_MAX), (1, OPEN_MAX));
    }

    #[test]
    fn config_names_toggle_exactly_one_mechanism() {
        for (name, check) in [
            (
                "classes",
                (|c: &GuideConfig| c.move_classes) as fn(&GuideConfig) -> bool,
            ),
            ("evasions", |c| c.check_evasions),
            ("quiet", |c| c.quiet_grouping),
            ("efpn", |c| c.ef_pn != EfPnMode::Off),
            ("closures", |c| c.static_closures),
            ("priors", |c| c.defender_priors),
            ("history", |c| c.history_ties),
            ("sim", |c| c.sibling_sim),
            ("replay", |c| c.null_replay),
            ("group", |c| c.group),
            ("fresh-dn", |c| c.fresh_dn),
            ("replay-exact", |c| c.replay_exact),
        ] {
            let base = GuideConfig::hand();
            let mut off = base;
            off.set_off(name).unwrap();
            let mut on = GuideConfig::none();
            on.set_on(name).unwrap();
            assert!(!check(&off), "{name} off");
            assert!(check(&on), "{name} on");
        }
    }

    #[test]
    fn params_parse_and_dump_round_trip() {
        let mut p = GuideParams::default();
        p.threat.pn = 3;
        p.global_scale = 64;
        p.ef_weight = 0;
        p.max_sim = 7;
        p.max_settle = 0;
        p.aux_budget = 123_456_789;
        p.sim_plies = 2;
        let text = p.dump();
        assert_eq!(GuideParams::parse(&text).unwrap(), p);
        assert!(GuideParams::parse("threat_pn=0").is_err());
        assert!(GuideParams::parse("global_scale=0").is_err());
        assert!(GuideParams::parse("bogus=1").is_err());
        // The new zero-allowed keys accept zero.
        assert_eq!(GuideParams::parse("ef_weight=0").unwrap().ef_weight, 0);
        assert_eq!(GuideParams::parse("max_sim=0").unwrap().max_sim, 0);
        assert_eq!(
            GuideParams::parse("aux_budget=123456789")
                .unwrap()
                .aux_budget,
            123_456_789
        );
        assert_eq!(GuideParams::parse("sim_plies=2").unwrap().sim_plies, 2);
    }

    #[test]
    fn calibration_scales_only_fresh_numbers() {
        let h = History::new();
        let c = ctx(false);
        let mut p = GuideParams::default();
        let g = Guide::new(GuideConfig::hand()).with_params(p);
        assert_eq!(
            g.start_numbers(&c, MoveClass::Threat, true, Color::White, mv(), &h, 0),
            (4, 16)
        );
        p.global_scale = 4;
        let g = Guide::new(GuideConfig::hand()).with_params(p);
        assert_eq!(
            g.start_numbers(&c, MoveClass::Threat, true, Color::White, mv(), &h, 0),
            (16, 64)
        );
        // Weight 16 is the plan3 identity, weight 0 neutralises the eval.
        p.global_scale = 1;
        p.ef_weight = 0;
        let g = Guide::new(GuideConfig::hand()).with_params(p);
        assert_eq!(
            g.start_numbers(
                &NodeContext { eval: 200, ..c },
                MoveClass::Threat,
                true,
                Color::White,
                mv(),
                &h,
                200,
            ),
            (4, 16)
        );
    }

    #[test]
    fn null_signature_never_keys_or_searches() {
        // The null side is a pure function of the board: same board, same
        // signature, no state.
        let b =
            Board::from_fen("r1b1kbnr/ppp3pp/n4q2/3pppN1/1P1PP3/8/P1P2PPP/RNBQKB1R w KQkq - 0 1")
                .unwrap();
        let mut st = StateInfo::new();
        b.populate_state(&mut st);
        let g = Guide::new(GuideConfig::hand());
        let c = g.context(&b, &st, 30, true);
        assert!(c.null_sig.is_some());
        assert_eq!(c.null_sig, Some(threat::signature_for(&b, Color::Black)));
    }
}
