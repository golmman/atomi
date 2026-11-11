//! `atomi` CLI: `prove` and `validate` subcommands plus `--help`/`--version`.

use atomi::cli;
use atomi::events::NullSink;
use atomi::search::{self, Limits};
use atomi::strategy;
use atomi::validate;
use atomic_movegen::board::{Board, FenPolicy};
use std::path::PathBuf;
use std::process::ExitCode;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const GIT_SHA: &str = env!("ATOMI_GIT_SHA");

fn print_help() {
    println!(
        "atomi {VERSION} ({GIT_SHA})\n\
         \n\
         USAGE:\n\
         \x20   atomi prove \"<FEN>\" [--seconds S] [--memory MB] [--evals N]\n\
         \x20                       [--proof-out PATH.jsonl] [--seed N]\n\
         \x20                       [--guide none|hand] [--guide-off LIST]\n\
         \x20                       [--guide-on LIST] [--guide-params PATH]\n\
         \x20                       [--guide-scale N] [--guide-ef-weight N]\n\
         \x20                       [--guide-max-sim N] [--guide-max-settle N]\n\
         \x20                       [--guide-aux-budget N] [--guide-sim-plies N]\n\
         \x20                       [--guide-group] [--proof-share PERCENT]\n\
         \x20                       [--stats] [--max-depth N]\n\
         \x20                       [--guide-fresh-dn] [--guide-replay-exact]\n\
         \x20   atomi validate PROOF.jsonl...\n\
         \x20   atomi moves \"<FEN>\"\n\
         \x20   atomi classify \"<FEN>\" [--guide none|hand] [--guide-off LIST]\n\
         \x20                   [--guide-on LIST] [--guide-params PATH]\n\
         \x20                   [--guide-scale N] [--guide-ef-weight N]\n\
         \x20                   [--guide-group] [--guide-fresh-dn]\n\
         \x20                   [--guide-replay-exact] [--seed N]\n\
         \x20                   [--attacker white|black]\n\
         \x20   atomi --help | --version"
    );
}

fn error(message: &str) -> ExitCode {
    println!("{{\"error\":\"{}\"}}", cli::json_escape(message));
    ExitCode::from(2)
}

fn prove(args: &[String]) -> ExitCode {
    let opts = match cli::parse_prove(args) {
        Ok(o) => o,
        Err(e) => return error(&e),
    };
    let root = Board::from_fen_with(&opts.fen, FenPolicy::Strict).unwrap();
    let limits = Limits {
        seconds: Some(opts.seconds),
        evals: opts.evals,
        memory_mb: opts.memory_mb,
        filter: true,
        guide: opts.guide,
        guide_params: opts.guide_params,
        proof_share_percent: opts.proof_share_percent,
        seed: opts.seed,
        stats: opts.stats,
        max_depth: opts.max_depth,
    };
    let output = search::prove(&root, &limits, NullSink);
    let result = output.result;
    let (proof, positions, proof_length, proof_written) = match &opts.proof_out {
        None => ("not-requested", None, None, false),
        Some(path) if result.exit != search::Exit::Proved => ("not-found", None, None, false),
        Some(path) => match strategy::write_proof(&root, &output.proofs, &PathBuf::from(path)) {
            Ok(stats) => ("written", Some(stats.positions), Some(stats.length), true),
            Err(e) => (e.as_str(), None, None, false),
        },
    };
    let evals_limit = opts.evals.map_or("null".to_string(), |n| n.to_string());
    let length = result.length.map_or("null".to_string(), |n| n.to_string());
    let positions = positions.map_or("null".to_string(), |n| n.to_string());
    let proof_length = proof_length.map_or("null".to_string(), |n| n.to_string());
    let found = result.exit == search::Exit::Proved;
    let stats = if opts.stats {
        let depth: Vec<String> = result.depth_histogram.iter().map(u64::to_string).collect();
        let class: Vec<String> = result.selected_class.iter().map(u64::to_string).collect();
        format!(
            ",\"stats\":{{\"unique_keys\":{},\"unique_keys_capped\":{},\
             \"re_expansions\":{},\"entry_proof_hits\":{},\"entry_tt_hits\":{},\
             \"entry_would_return\":{},\
             \"attacker_expansions\":{},\"defender_expansions\":{},\
             \"depth_histogram\":[{}],\"selected_class\":[{}]}}",
            result.unique_keys,
            result.unique_keys_capped,
            result.re_expansions,
            result.entry_proof_hits,
            result.entry_tt_hits,
            result.entry_would_return,
            result.attacker_expansions,
            result.defender_expansions,
            depth.join(","),
            class.join(","),
        )
    } else {
        String::new()
    };
    println!(
        "{{\"found\":{},\"length\":{},\"exit\":\"{}\",\"child_evals\":{},\
         \"expansions\":{},\"seed\":{},\"sim_nodes\":{},\"settle_nodes\":{},\
         \"aux_nodes\":{},\"aux_skipped\":{},\
         \"ms\":{},\"seconds\":{},\"memory_mb\":{},\
         \"evals_limit\":{},\"proof\":\"{}\",\"proof_positions\":{},\
         \"proof_length\":{},\"proof_entries\":{},\"proof_full\":{}{}}}",
        found,
        length,
        result.exit.as_str(),
        result.child_evals,
        result.expansions,
        opts.seed,
        result.sim_nodes,
        result.settle_nodes,
        result.aux_nodes,
        result.aux_skipped,
        result.ms,
        opts.seconds,
        opts.memory_mb,
        evals_limit,
        proof,
        positions,
        proof_length,
        result.proof_entries,
        result.proof_full,
        stats,
    );
    if opts.proof_out.is_some() && found && !proof_written {
        return ExitCode::from(3);
    }
    ExitCode::SUCCESS
}

fn moves_cmd(args: &[String]) -> ExitCode {
    if args.len() != 1 || args[0].starts_with('-') {
        return error("moves needs exactly one FEN");
    }
    let b = match Board::from_fen_with(&args[0], FenPolicy::AllowTerminal) {
        Ok(b) => b,
        Err(_) => return error(&format!("bad FEN: {}", args[0])),
    };
    let mut ml = atomic_movegen::types::MoveList::new();
    atomic_movegen::movegen::generate_legal(&b, &mut ml);
    let ucis: Vec<String> = (&ml)
        .into_iter()
        .map(|m| serde_json::to_string(&m.to_uci()).unwrap())
        .collect();
    println!("{{\"moves\":[{}]}}", ucis.join(","));
    ExitCode::SUCCESS
}

fn classify_cmd(args: &[String]) -> ExitCode {
    let opts = match atomi::classify::parse(args) {
        Ok(o) => o,
        Err(e) => return error(&e),
    };
    match atomi::classify::run(&opts) {
        Ok(line) => {
            println!("{line}");
            ExitCode::SUCCESS
        }
        Err(e) => error(&e),
    }
}

fn validate_cmd(args: &[String]) -> ExitCode {
    if args.is_empty() {
        return error("validate needs at least one file");
    }
    let mut all_valid = true;
    for arg in args {
        if arg.starts_with('-') {
            return error(&format!("unknown option: {arg}"));
        }
        let report = validate::validate_file(std::path::Path::new(arg));
        if !report.valid {
            all_valid = false;
        }
        println!("{}", report.to_json());
    }
    if all_valid {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        None | Some("--help") | Some("-h") => {
            print_help();
            ExitCode::SUCCESS
        }
        Some("--version") | Some("-V") => {
            println!("atomi {VERSION} ({GIT_SHA})");
            ExitCode::SUCCESS
        }
        Some("prove") => prove(&args[1..]),
        Some("moves") => moves_cmd(&args[1..]),
        Some("classify") => classify_cmd(&args[1..]),
        Some("validate") => validate_cmd(&args[1..]),
        Some(other) => error(&format!("unknown subcommand or option: {other}")),
    }
}
