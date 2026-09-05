//! Welcome featured paths must not mid-request elevate to `Actor::System`.
#![cfg(feature = "ssr")]
#![allow(missing_docs)]

use std::fs;
use std::path::{Path, PathBuf};

fn crate_src_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
}

fn scan_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            scan_rs_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

fn forbidden_elevate(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.starts_with("//") {
        return false;
    }
    trimmed.contains("with_actor(Actor::System")
        || trimmed.contains("with_actor(valence::Actor::System")
        || trimmed.contains("unsafe_system_valence")
}

#[test]
fn welcome_src_must_not_elevate_to_system() {
    let root = crate_src_root();
    let mut files = Vec::new();
    scan_rs_files(&root, &mut files);
    let mut violations = Vec::new();
    for path in &files {
        let Ok(text) = fs::read_to_string(path) else {
            continue;
        };
        let rel = path
            .strip_prefix(&root)
            .map(|p| p.to_string_lossy().replace('\\', "/"))
            .unwrap_or_else(|_| path.display().to_string());
        for (i, line) in text.lines().enumerate() {
            if forbidden_elevate(line) {
                violations.push(format!("{rel}:{}: {}", i + 1, line.trim()));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "uf-welcome must use session Valence + WelcomeAdmin gate (no System elevate):\n{}",
        violations.join("\n")
    );
}
