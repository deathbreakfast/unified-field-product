//! Gate: uf-product platform crates must not hard-depend on lepton-shell.

use std::fs;
use std::path::PathBuf;

fn product_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..")
}

fn assert_no_lepton_shell(rel: &str) {
    let text = fs::read_to_string(product_root().join(rel)).unwrap_or_else(|e| {
        panic!("read {rel}: {e}");
    });
    assert!(
        !text
            .lines()
            .any(|l| l.trim_start().starts_with("lepton-shell")),
        "{rel} must not declare a lepton-shell dependency"
    );
}

#[test]
fn uf_product_has_no_hard_lepton_shell_dep_happy_path() {
    assert_no_lepton_shell("Cargo.toml");
    for pkg in ["uf-welcome", "uf-apps", "uf-integrations", "uf-product"] {
        assert_no_lepton_shell(&format!("{pkg}/Cargo.toml"));
    }
}
