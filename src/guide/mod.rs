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
pub mod null;
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

/// The mechanism names `--guide-on` / `--guide-off` accept (D33: parsing
/// defers them, so the list lives here next to `GuideConfig::set_switch`.
pub const MECHANISMS: [&str; 23] = [
    "classes",
    "evasions",
    "quiet",
    "efpn",
    "closures",
    "priors",
    "history",
    "sim",
    "replay",
    "group",
    "fresh-dn",
    "replay-exact",
    "sim-exact",
    "null-exact",
    "null-screen",
    "fresh-class",
    "efpn-banded",
    "evasions-count",
    "threat-count",
    "hanging-quiet",
    "unlikely-exact",
    "closure-bits",
    "numbers",
];

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
    /// default off; the kill-test flag). Plan-24 GROUP-V1 (score-free
    /// members, default on) failed its gate: `deep-win` 0/3 at 120 s
    /// (report24), so the group stays off and is never a product arm.
    pub group: bool,
    /// Plan-13 conformance probe: fresh disproof numbers are 1
    /// (`--guide-fresh-dn`, default off).
    pub fresh_dn: bool,
    /// Plan-13 conformance probe: exact sibling proof replay
    /// (`--guide-replay-exact`, default off).
    pub replay_exact: bool,
    /// Plan-18 exact sibling simulation: replace the greedy playout with
    /// the exact sibling proof replay (`--guide-sim-exact`, default off).
    pub sim_exact: bool,
    /// Plan-18 exact null-threat replay: replace the signature prior with
    /// verified replay of a null-position proof (`--guide-null-exact`,
    /// default off).
    pub null_exact: bool,
    /// Plan-25 NULL-S: run the exact null replay's pass 2 only when the null
    /// position has an immediate explosion threat (`--guide-null-screen`,
    /// default off; the heuristic arm).
    pub null_screen: bool,
    /// Plan-19 R1 `fresh-class`: a fresh child's deciding number is its
    /// class and the other number is 1 (`--guide-on fresh-class`).
    pub fresh_class: bool,
    /// Plan-19 R2 `efpn-banded`: EF-PN as a three-step band from the
    /// parent's eval (`--guide-on efpn-banded`).
    pub efpn_banded: bool,
    /// Plan-19 R3 `evasions-count`: the check number is the surviving
    /// evasions, capped (`--guide-on evasions-count`).
    pub evasions_count: bool,
    /// Plan-19 R4 `threat-count`: the OR number comes from the child's
    /// explosion-target count (`--guide-on threat-count`).
    pub threat_count: bool,
    /// Plan-19 R5 `hanging-quiet`: hanging means a quiet move whose piece
    /// the defender blasts cheaply (`--guide-on hanging-quiet`).
    pub hanging_quiet: bool,
    /// Plan-19 R6 `unlikely-exact`: the `UNLIKELY` prior sets the pair
    /// outright (`--guide-on unlikely-exact`).
    pub unlikely_exact: bool,
    /// Plan-20 R11 `closure-bits`: the one-ply explosion settle and its
    /// screen read bitboards instead of building a child and generating its
    /// legal moves (`--guide-on closure-bits`, default off).
    pub closure_bits: bool,
}

impl GuideConfig {
    /// The bare skeleton (`--guide none`).
    pub fn none() -> GuideConfig {
        GuideConfig::default()
    }

    /// The shipped hand guide (`--guide hand`): the product stack plus the
    /// plan-19 starting-number algebra R1, R3 and R6 (report19 §5) and the
    /// plan-20 AUX-V1 stack (report20): the exact sibling simulation, the
    /// exact null-threat replay and the board-free one-ply closures. R2
    /// (`efpn_banded`) and R4/R5 (`threat_count`, `hanging_quiet`) stay
    /// opt-in. The greedy playout and the signature prior are inert while
    /// their exact replacements are on; `--guide-off sim-exact,null-exact,
    /// closure-bits` restores the plan-19 default.
    pub fn hand() -> GuideConfig {
        let mut c = GuideConfig::product();
        c.r1_r3();
        c.unlikely_exact = true;
        c.sim_exact = true;
        c.null_exact = true;
        c.closure_bits = true;
        c
    }

    /// The hand guide before plan19 (the ablations' control). `--guide hand
    /// --guide-off fresh-class,evasions-count,unlikely-exact` spells this
    /// configuration on the shipped binary.
    pub fn product() -> GuideConfig {
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
            sim_exact: false,
            null_exact: false,
            null_screen: false,
            fresh_class: false,
            efpn_banded: false,
            evasions_count: false,
            threat_count: false,
            hanging_quiet: false,
            unlikely_exact: false,
            closure_bits: false,
        }
    }

    /// The plan-19 NUMBERS-V1 stack (R1-R6) on top of the bare skeleton.
    pub fn numbers() -> GuideConfig {
        let mut c = GuideConfig::none();
        c.set_on("numbers").expect("numbers is a mechanism");
        c
    }

    /// Plan-19's adopted pair mechanism (R1+R3) without R6, used by
    /// [`GuideConfig::hand`].
    fn r1_r3(&mut self) {
        self.fresh_class = true;
        self.evasions_count = true;
        self.enabled = true;
    }

    /// Turn the six NUMBERS-V1 mechanisms on or off together.
    fn numbers_switches(&mut self, on: bool) {
        self.fresh_class = on;
        self.efpn_banded = on;
        self.evasions_count = on;
        self.threat_count = on;
        self.hanging_quiet = on;
        self.unlikely_exact = on;
    }

    /// Switch one named mechanism off (`--guide-off`).
    pub fn set_off(&mut self, name: &str) -> Result<(), String> {
        self.set_switch(name, false)
    }

    /// Switch one named mechanism on (`--guide-on`), enabling the guide.
    pub fn set_on(&mut self, name: &str) -> Result<(), String> {
        self.set_switch(name, true)?;
        self.enabled = true;
        Ok(())
    }

    /// Switch one named mechanism without changing `enabled`: a value-less
    /// switch such as `--guide-group` on a `--guide none` base stays inert.
    pub(crate) fn set_switch(&mut self, name: &str, on: bool) -> Result<(), String> {
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
            "sim-exact" => self.sim_exact = on,
            "null-exact" => self.null_exact = on,
            "null-screen" => self.null_screen = on,
            "fresh-class" => self.fresh_class = on,
            "efpn-banded" => self.efpn_banded = on,
            "evasions-count" => self.evasions_count = on,
            "threat-count" => self.threat_count = on,
            "hanging-quiet" => self.hanging_quiet = on,
            "unlikely-exact" => self.unlikely_exact = on,
            "closure-bits" => self.closure_bits = on,
            "numbers" => self.numbers_switches(on),
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
    /// Plan-19 R2: the EF-PN band edge in `hand_eval` units (`[-256, 256]`);
    /// `0` puts every parent in the middle step.
    pub ef_band: u32,
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
            ef_band: 128,
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
                "ef_weight" | "ef_band" | "max_sim" | "max_settle" | "aux_budget" | "sim_plies"
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
                "ef_band" => p.ef_band = v32,
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
            "global_scale={}\nef_weight={}\nef_band={}\nmax_sim={}\nmax_settle={}\n\
             aux_budget={}\nsim_plies={}\n",
            self.global_scale,
            self.ef_weight,
            self.ef_band,
            self.max_sim,
            self.max_settle,
            self.aux_budget,
            self.sim_plies
        ));
        out
    }
}

/// The per-child inputs of a fresh pair (plan-19 R1/R3/R4/R6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StartInput {
    /// The move's class.
    pub class: MoveClass,
    /// R4/R5's exact OR number, when they decide one.
    pub or_number: Option<u32>,
    /// The AND class: `2` when the defence is unlikely to refute (R1's
    /// class and R6's previous pair), else `1`.
    pub and_number: u32,
    /// The surviving evasions (R3), else the parent's legal move count.
    pub evasions: u32,
    /// The child's eval, attacker-oriented (legacy EF-PN only).
    pub child_eval: i32,
    /// The sibling simulation's score for this child (0 = none).
    pub sim: i32,
    /// The legacy signature prior applies to this defence (pre-plan-19
    /// `UNLIKELY`; off once R6 sets the pair outright).
    pub unlikely_prior: bool,
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
    ///
    /// Plan 25 DEFENDER-EVAL: `hand_eval` is computed only where it is read —
    /// at attacker (OR) nodes, under `efpn_banded` (both node kinds), or when
    /// the opt-in symmetric EF-PN mode `Both` needs the defender side.
    pub fn context(
        &self,
        board: &Board,
        state: &StateInfo,
        legal: u32,
        attacker_node: bool,
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
        let eval = if self.config.efpn_banded
            || (self.config.ef_pn != EfPnMode::Off
                && (attacker_node || self.config.ef_pn == EfPnMode::Both))
        {
            eval::hand_eval(board)
        } else {
            0
        };
        let null_sig = if need_null
            && self.config.null_replay
            && !self.config.null_exact
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
    ///
    /// Order: the base pair (R1's class-only pair, or the product table),
    /// `global_scale`, the exact `UNLIKELY` pair (R6), the history prior, the
    /// parent-eval band (R2), the legacy per-child EF-PN and the sibling
    /// simulation. Every step only reorders `Open` numbers; none decides a
    /// value.
    pub fn start_numbers(
        &self,
        ctx: &NodeContext,
        input: &StartInput,
        attacker_node: bool,
        side: Color,
        mv: Move,
        history: &History,
    ) -> (u32, u32) {
        if !self.config.enabled {
            return (1, 1);
        }
        let c = &self.config;
        let (mut pn, mut dn) = self.base_pair(input, attacker_node);
        // Calibration (plan4): the fresh base magnitude only.
        let scale = u64::from(self.params.global_scale);
        if scale != 1 {
            pn = (u64::from(pn) * scale).min(u64::from(OPEN_MAX)) as u32;
            dn = (u64::from(dn) * scale).min(u64::from(OPEN_MAX)) as u32;
        }
        // R6: a defence unlikely to refute starts as `(1, 2)` outright.
        if c.unlikely_exact && !attacker_node && input.and_number == 2 {
            (pn, dn) = (1, 2);
        }
        // Defence/attacker history: good for the mover is good for the
        // mover's orientation, so at AND nodes the attacker numbers invert.
        if c.defender_priors {
            let h = i32::from(history.score(side, mv));
            if h > 0 {
                let e = if attacker_node { h } else { -h };
                (pn, dn) = eval::scale(pn, dn, e);
            }
        }
        // R2: the three-step band from the parent's eval, both node kinds.
        if c.efpn_banded {
            let oriented = if attacker_node { ctx.eval } else { -ctx.eval };
            let step = eval::band_step(oriented, self.params.ef_band);
            (pn, dn) = eval::scale_banded(pn, dn, step, attacker_node);
        }
        if c.ef_pn != EfPnMode::Off && (attacker_node || c.ef_pn == EfPnMode::Both) {
            (pn, dn) = eval::scale_weighted(pn, dn, input.child_eval, self.params.ef_weight);
        }
        if c.sibling_sim && input.sim != 0 {
            (pn, dn) = self.sim_adjust(pn, dn, input.sim);
        }
        // The product `UNLIKELY` prior (the null-signature replay), kept for
        // the pre-plan-19 configurations.
        if input.unlikely_prior {
            (pn, dn) = self.unlikely(pn, dn);
        }
        (pn.clamp(1, OPEN_MAX), dn.clamp(1, OPEN_MAX))
    }

    /// The base pair of a fresh child, attacker-oriented: before
    /// `global_scale`, the priors, EF-PN and the simulation.
    ///
    /// R1 (`fresh-class`): the deciding number is the class and the other
    /// number is 1 — `(class, 1)` at an OR node, `(1, class)` at an AND node
    /// with the AND class in `{1, 2}` (2 for a defence unlikely to refute).
    /// Otherwise the product table, with the R3 evasion count as the check
    /// base and the plan-13 `fresh-dn` patch.
    pub fn base_pair(&self, input: &StartInput, attacker_node: bool) -> (u32, u32) {
        let c = &self.config;
        if c.fresh_class {
            return if attacker_node {
                (self.or_number(input), 1)
            } else {
                (1, input.and_number.clamp(1, 2))
            };
        }
        let base = if input.class == MoveClass::Evasion && c.check_evasions {
            Pair {
                pn: input.evasions.max(1),
                dn: input.evasions.max(1),
            }
        } else if input.class == MoveClass::Quiet {
            if c.quiet_grouping {
                self.params.quiet
            } else {
                Pair { pn: 1, dn: 1 }
            }
        } else if c.move_classes {
            self.params.pair(input.class)
        } else {
            Pair { pn: 1, dn: 1 }
        };
        let (pn, dn) = if attacker_node {
            (base.pn, base.dn)
        } else {
            (base.dn, base.pn)
        };
        // Plan-13 conformance probe: a fresh disproof number is 1.
        (pn, if c.fresh_dn { 1 } else { dn })
    }

    /// The OR deciding number of a fresh child under R1: R4's exact count
    /// when it decides, else the surviving evasion count, else the class
    /// table.
    fn or_number(&self, input: &StartInput) -> u32 {
        if let Some(pn) = input.or_number {
            return pn.clamp(1, OPEN_MAX);
        }
        if input.class == MoveClass::Evasion && self.config.check_evasions {
            return input.evasions.clamp(1, OPEN_MAX);
        }
        self.params.pair(input.class).pn.clamp(1, OPEN_MAX)
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

    /// A plain fresh-pair input: no exact number, AND class 1, the context's
    /// legal count, no child eval and no simulation.
    fn si(class: MoveClass) -> StartInput {
        StartInput {
            class,
            or_number: None,
            and_number: 1,
            evasions: 12,
            child_eval: 0,
            sim: 0,
            unlikely_prior: false,
        }
    }

    fn start(g: &Guide, c: &NodeContext, input: &StartInput, attacker_node: bool) -> (u32, u32) {
        let h = History::new();
        g.start_numbers(c, input, attacker_node, Color::White, mv(), &h)
    }

    #[test]
    fn disabled_guide_is_identity() {
        let g = Guide::new(GuideConfig::none());
        for attacker_node in [true, false] {
            for class in [MoveClass::Threat, MoveClass::Evasion, MoveClass::Quiet] {
                assert_eq!(start(&g, &ctx(false), &si(class), attacker_node), (1, 1));
            }
        }
    }

    #[test]
    fn class_pairs_are_oriented_and_positive() {
        let g = Guide::new(GuideConfig::product());
        let c = ctx(false);
        assert_eq!(start(&g, &c, &si(MoveClass::Threat), true), (4, 16));
        assert_eq!(start(&g, &c, &si(MoveClass::Threat), false), (16, 4));
        for class in [
            MoveClass::Threat,
            MoveClass::WeakThreat,
            MoveClass::Quiet,
            MoveClass::Hanging,
            MoveClass::Evasion,
        ] {
            for attacker_node in [true, false] {
                let (pn, dn) = start(&g, &c, &si(class), attacker_node);
                assert!(pn >= 1 && dn >= 1 && pn <= OPEN_MAX && dn <= OPEN_MAX);
            }
        }
    }

    #[test]
    fn evasion_base_is_the_legal_move_count() {
        let g = Guide::new(GuideConfig::product());
        let c = ctx(true);
        let legal = StartInput {
            evasions: 7,
            ..si(MoveClass::Evasion)
        };
        assert_eq!(start(&g, &c, &legal, true), (7, 7));
        // R3 hands the surviving count in place of the legal count.
        let g3 = Guide::new(GuideConfig {
            evasions_count: true,
            ..GuideConfig::product()
        });
        let input = StartInput {
            evasions: 3,
            ..si(MoveClass::Evasion)
        };
        assert_eq!(start(&g3, &c, &input, true), (3, 3));
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
                &si(MoveClass::Quiet),
                attacker_node,
                Color::White,
                mv(),
                &h,
            );
            assert!(pn >= 1 && dn >= 1 && pn < crate::search::value::INFINITY);
            assert!(dn < crate::search::value::INFINITY);
        }
    }

    /// R1: a fresh child's deciding number is its class and the other number
    /// is 1 — `(class, 1)` at an OR node, `(1, class)` at an AND node with
    /// the AND class in `{1, 2}`. `--guide none` stays `(1, 1)`.
    #[test]
    fn fresh_class_pair_on_both_node_kinds() {
        let g = Guide::new(GuideConfig {
            enabled: true,
            fresh_class: true,
            ..GuideConfig::none()
        });
        let c = ctx(false);
        assert_eq!(start(&g, &c, &si(MoveClass::Quiet), true), (9, 1));
        assert_eq!(start(&g, &c, &si(MoveClass::Threat), true), (4, 1));
        assert_eq!(start(&g, &c, &si(MoveClass::Quiet), false), (1, 1));
        let unlikely = StartInput {
            and_number: 2,
            ..si(MoveClass::Quiet)
        };
        assert_eq!(start(&g, &c, &unlikely, false), (1, 2));
        // R4's exact number wins over the table at an OR node.
        let exact = StartInput {
            or_number: Some(1),
            ..si(MoveClass::Threat)
        };
        assert_eq!(start(&g, &c, &exact, true), (1, 1));
        // The guide off stays the identity for both node kinds.
        let off = Guide::new(GuideConfig::none());
        assert_eq!(start(&off, &c, &si(MoveClass::Threat), true), (1, 1));
        assert_eq!(start(&off, &c, &unlikely, false), (1, 1));
        // The aggregate switch turns R1 on (the R2 band scales the pair
        // afterwards, so the base is what this pins).
        let n = Guide::new(GuideConfig::numbers());
        assert_eq!(n.base_pair(&si(MoveClass::Quiet), true), (9, 1));
        assert_eq!(n.base_pair(&unlikely, false), (1, 2));
    }

    /// Plan-19 §7: the shipped `--guide hand` is the product stack plus R1,
    /// R3 and R6; R2 and R4/R5 stay opt-in.
    #[test]
    fn hand_is_the_adopted_numbers_arm() {
        let hand = GuideConfig::hand();
        let product = GuideConfig::product();
        assert!(hand.fresh_class && hand.evasions_count && hand.unlikely_exact);
        assert!(!hand.efpn_banded && !hand.threat_count && !hand.hanging_quiet);
        // Plan-20 AUX-V1: the exact replays and the board-free closures.
        assert!(hand.sim_exact && hand.null_exact && hand.closure_bits);
        // Every other product switch is untouched.
        assert_eq!(
            GuideConfig {
                fresh_class: false,
                evasions_count: false,
                unlikely_exact: false,
                sim_exact: false,
                null_exact: false,
                closure_bits: false,
                ..hand
            },
            product
        );
        // The explicit spelling on the CLI resolves to the same config.
        let args: Vec<String> = [
            "4k3/8/8/8/8/8/4P3/4K3 w - - 0 1",
            "--guide",
            "hand",
            "--guide-on",
            "numbers,sim-exact,null-exact,closure-bits",
            "--guide-off",
            "efpn-banded,threat-count,hanging-quiet",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect();
        assert_eq!(crate::cli::parse_prove(&args).unwrap().guide, hand);
    }

    /// R6: a defence unlikely to refute starts as `(1, 2)` outright; the
    /// product prior's `pn/2, dn*2` no longer applies.
    #[test]
    fn unlikely_exact_sets_the_pair_outright() {
        let g = Guide::new(GuideConfig {
            unlikely_exact: true,
            ..GuideConfig::product()
        });
        let c = ctx(false);
        let unlikely = StartInput {
            and_number: 2,
            ..si(MoveClass::Threat)
        };
        assert_eq!(start(&g, &c, &unlikely, false), (1, 2));
        let likely = StartInput {
            and_number: 1,
            ..si(MoveClass::Threat)
        };
        assert_eq!(start(&g, &c, &likely, false), (16, 4));
        // OR children are untouched.
        assert_eq!(start(&g, &c, &unlikely, true), (4, 16));
    }

    /// R2: the band is three steps from the parent's eval, applied at both
    /// node kinds: the deciding number takes the larger factor swing.
    #[test]
    fn efpn_banded_steps_both_node_kinds() {
        let g = Guide::new(GuideConfig {
            enabled: true,
            fresh_class: true,
            efpn_banded: true,
            ..GuideConfig::none()
        });
        let input = StartInput {
            or_number: Some(9),
            ..si(MoveClass::Quiet)
        };
        // `ahead` is the attacker's eval: at an AND node the parent's eval is
        // read from the defender's side, hence the sign flip.
        let or = |ahead: i32| {
            start(
                &g,
                &NodeContext {
                    eval: ahead,
                    ..ctx(false)
                },
                &input,
                true,
            )
        };
        let and = |ahead: i32| {
            start(
                &g,
                &NodeContext {
                    eval: -ahead,
                    ..ctx(false)
                },
                &input,
                false,
            )
        };
        // OR: the proof number is the deciding one.
        assert_eq!(or(128), (9, 21));
        assert_eq!(or(0), (54, 11));
        assert_eq!(or(-128), (99, 1));
        // AND: the disproof number is the deciding one.
        assert_eq!(and(128), (1, 11));
        assert_eq!(and(0), (11, 6));
        assert_eq!(and(-128), (21, 1));
        // The eval is read at the parent, oriented to the attacker.
        let tight = Guide::new(GuideConfig {
            enabled: true,
            fresh_class: true,
            efpn_banded: true,
            ..GuideConfig::none()
        })
        .with_params(GuideParams {
            ef_band: 64,
            ..GuideParams::default()
        });
        let (pn, _) = start(
            &tight,
            &NodeContext {
                eval: 128,
                ..ctx(false)
            },
            &input,
            true,
        );
        assert_eq!(pn, 9, "eval 128 is beyond a 64 edge");
        // Edges: step 2 at exactly the edge, step 1 just inside it.
        assert_eq!(eval::band_step(64, 64), 2);
        assert_eq!(eval::band_step(63, 64), 1);
        assert_eq!(eval::band_step(-64, 64), 0);
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
            ("sim-exact", |c| c.sim_exact),
            ("null-exact", |c| c.null_exact),
            ("null-screen", |c| c.null_screen),
            ("fresh-class", |c| c.fresh_class),
            ("efpn-banded", |c| c.efpn_banded),
            ("evasions-count", |c| c.evasions_count),
            ("threat-count", |c| c.threat_count),
            ("hanging-quiet", |c| c.hanging_quiet),
            ("unlikely-exact", |c| c.unlikely_exact),
            ("closure-bits", |c| c.closure_bits),
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
        p.ef_band = 64;
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
        assert_eq!(GuideParams::parse("ef_band=0").unwrap().ef_band, 0);
        assert_eq!(GuideParams::parse("ef_band=64").unwrap().ef_band, 64);
    }

    #[test]
    fn calibration_scales_only_fresh_numbers() {
        let h = History::new();
        let c = ctx(false);
        let mut p = GuideParams::default();
        let g = Guide::new(GuideConfig::product()).with_params(p);
        assert_eq!(
            g.start_numbers(&c, &si(MoveClass::Threat), true, Color::White, mv(), &h),
            (4, 16)
        );
        p.global_scale = 4;
        let g = Guide::new(GuideConfig::product()).with_params(p);
        assert_eq!(
            g.start_numbers(&c, &si(MoveClass::Threat), true, Color::White, mv(), &h),
            (16, 64)
        );
        // Weight 16 is the plan3 identity, weight 0 neutralises the eval.
        p.global_scale = 1;
        p.ef_weight = 0;
        let g = Guide::new(GuideConfig::product()).with_params(p);
        let scored = StartInput {
            child_eval: 200,
            ..si(MoveClass::Threat)
        };
        assert_eq!(
            g.start_numbers(
                &NodeContext { eval: 200, ..c },
                &scored,
                true,
                Color::White,
                mv(),
                &h
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
        // The signature prior is built only while the exact replay is off.
        let g = Guide::new(GuideConfig {
            null_exact: false,
            ..GuideConfig::hand()
        });
        let c = g.context(&b, &st, 30, false, true);
        assert!(c.null_sig.is_some());
        assert_eq!(c.null_sig, Some(threat::signature_for(&b, Color::Black)));
        // With the exact replay on (the shipped default) no signature is
        // computed at all.
        let exact = Guide::new(GuideConfig::hand()).context(&b, &st, 30, false, true);
        assert!(exact.null_sig.is_none());
    }
}
