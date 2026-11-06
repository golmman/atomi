//! Bakes the git HEAD sha into the binary as `ATOMI_GIT_SHA`.
//!
//! Emits rerun triggers for `.git/HEAD`, the ref file `HEAD` points at, and
//! `.git/packed-refs` (when present), so the sha is not stale after a commit.

use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let git_dir = manifest_dir.join(".git");

    println!("cargo:rerun-if-changed={}", git_dir.join("HEAD").display());

    if let Ok(head) = std::fs::read_to_string(git_dir.join("HEAD"))
        && let Some(reference) = head.trim().strip_prefix("ref: ")
    {
        let ref_file = git_dir.join(reference);
        println!("cargo:rerun-if-changed={}", ref_file.display());
    }
    let packed = git_dir.join("packed-refs");
    if packed.exists() {
        println!("cargo:rerun-if-changed={}", packed.display());
    }

    let sha = head_sha(&manifest_dir).unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=ATOMI_GIT_SHA={sha}");
}

fn head_sha(dir: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(dir)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let sha = String::from_utf8(output.stdout).ok()?.trim().to_string();
    (!sha.is_empty()).then_some(sha)
}
