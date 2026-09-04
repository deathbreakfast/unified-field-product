//! Product [`BrandTone`] — upstream shell tones plus per-family marketing accents.

use super::platform_family_brand::PlatformFamilyBrand;

/// Cross-page accent selection mapped to Orbital brand CSS variables.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BrandTone {
    Brand,
    Neutral,
    Subtle,
    Accent,
    Family(PlatformFamilyBrand),
}

impl BrandTone {
    pub const fn as_class(self) -> &'static str {
        match self {
            Self::Brand => "orbital-token-tone-brand",
            Self::Neutral => "orbital-token-tone-neutral",
            Self::Subtle => "orbital-token-tone-subtle",
            Self::Accent => "orbital-token-tone-accent",
            Self::Family(_) => "orbital-token-tone-family",
        }
    }

    pub const fn accent_token(self) -> &'static str {
        match self {
            Self::Brand => "var(--orb-color-brand-bg)",
            Self::Neutral => "var(--orb-color-surface-subtle)",
            Self::Subtle => "var(--orb-color-surface-overlay)",
            Self::Accent | Self::Family(_) => "var(--orb-color-brand-bg-subtle)",
        }
    }
}
