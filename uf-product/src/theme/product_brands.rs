//! Canonical brand seed colors per registered Orbital product.

/// Default brand seed when no product-specific entry exists.
pub const SHELL_BRAND_SEED: &str = "#1a6f94";

/// Platform shell / utility apps share the Orbital blue accent.
pub const UF_SHELL_BRAND_SEED: &str = "#4a89dc";

/// `(app_id, brand_seed_hex)` — aligned with physics-family anchors in `orb_aliases.rs`.
const PRODUCT_BRAND_SEEDS: &[(&str, &str)] = &[
    ("valence", "#4f6bed"),
    ("chronon", "#eaa300"),
    ("boson", "#7160e8"),
    ("photon", "#00b7c3"),
    ("spectra", "#5c2e91"),
    ("gluon", "#e3008c"),
    ("database", "#5b5fc7"),
    ("permission", "#5b5fc7"),
    ("secrets", "#0b6a0b"),
    ("tag", UF_SHELL_BRAND_SEED),
    ("counter", "#1a6f94"),
    ("lepton-app", UF_SHELL_BRAND_SEED),
    ("apps", UF_SHELL_BRAND_SEED),
    ("auth", UF_SHELL_BRAND_SEED),
    ("notifications", UF_SHELL_BRAND_SEED),
    ("welcome", UF_SHELL_BRAND_SEED),
    ("orbital", UF_SHELL_BRAND_SEED),
    ("setup-wizard", UF_SHELL_BRAND_SEED),
    ("marketing", UF_SHELL_BRAND_SEED),
    ("shell", SHELL_BRAND_SEED),
];

/// Single-letter glyph for product app bar avatars (physics-family shorthand).
pub fn product_avatar_letter(app_id: &str) -> char {
    match app_id {
        "database" | "secrets" => 'N',
        _ => app_id
            .chars()
            .next()
            .map_or('?', |c| c.to_ascii_uppercase()),
    }
}

/// Resolve the default brand seed hex for a registered app id.
pub fn brand_seed_for_app_id(app_id: &str) -> &'static str {
    PRODUCT_BRAND_SEEDS
        .iter()
        .find(|(id, _)| *id == app_id)
        .map_or(SHELL_BRAND_SEED, |(_, seed)| *seed)
}

/// Product preset chips for the Appearance settings page (display name, seed).
pub const PRODUCT_BRAND_PRESETS: &[(&str, &str)] = &[
    ("Valence", "#4f6bed"),
    ("Chronon", "#eaa300"),
    ("Boson", "#7160e8"),
    ("Photon", "#00b7c3"),
    ("Spectra", "#5c2e91"),
    ("Gluon", "#e3008c"),
    ("Nucleus", "#5b5fc7"),
    ("Neutrino", "#0b6a0b"),
    ("Counter", "#1a6f94"),
    ("Orbital", UF_SHELL_BRAND_SEED),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_products_resolve() {
        assert_eq!(brand_seed_for_app_id("chronon"), "#eaa300");
        assert_eq!(brand_seed_for_app_id("valence"), "#4f6bed");
        assert_eq!(brand_seed_for_app_id("unknown-app"), SHELL_BRAND_SEED);
    }

    #[test]
    fn avatar_letters_use_physics_shorthand() {
        assert_eq!(product_avatar_letter("valence"), 'V');
        assert_eq!(product_avatar_letter("chronon"), 'C');
        assert_eq!(product_avatar_letter("database"), 'N');
        assert_eq!(product_avatar_letter("secrets"), 'N');
    }
}
