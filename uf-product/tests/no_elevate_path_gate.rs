//! Interactive uf-product paths must not mid-request elevate to `Actor::System`
//! unless allowlisted (System-started backfill / bootstrap).
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
}

const fn elevate_allowlist() -> &'static [(&'static str, &'static str)] {
    &[(
        "workspace_search/demo/backfill_iter.rs",
        "IndexedDemo backfill should_run prefers job System; rebind only if not already System",
    )]
}

fn relative_src(path: &Path, root: &Path) -> String {
    path.strip_prefix(root).map_or_else(
        |_| path.display().to_string(),
        |p| p.to_string_lossy().replace('\\', "/"),
    )
}

#[test]
fn uf_product_interactive_paths_must_not_elevate_outside_allowlist() {
    let root = crate_src_root();
    let mut files = Vec::new();
    scan_rs_files(&root, &mut files);
    let allow: std::collections::HashSet<&str> =
        elevate_allowlist().iter().map(|(p, _)| *p).collect();

    let mut violations = Vec::new();
    for path in &files {
        let rel = relative_src(path, &root);
        if allow.contains(rel.as_str()) {
            continue;
        }
        let Ok(text) = fs::read_to_string(path) else {
            continue;
        };
        for (i, line) in text.lines().enumerate() {
            if forbidden_elevate(line) {
                violations.push(format!("{rel}:{}: {}", i + 1, line.trim()));
            }
        }
    }
    assert!(
        violations.is_empty(),
        "mid-request System elevates forbidden outside allowlist:\n{}",
        violations.join("\n")
    );
}
