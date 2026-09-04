//! Per-family marketing accent colors (product-owned; not upstream shell tokens).

/// Physics-family brand accents for marketing surfaces and navigation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PlatformFamilyBrand {
    Valence,
    Gluon,
    Nucleus,
    Chronon,
    Boson,
    Photon,
    Orbital,
    Spectra,
    Neutrino,
    Higgs,
    Phonon,
    Polaron,
    Magnon,
}

impl PlatformFamilyBrand {
    const fn seed_hex(self) -> &'static str {
        match self {
            Self::Valence => "#4f6bed",
            Self::Chronon => "#eaa300",
            Self::Boson => "#7160e8",
            Self::Photon => "#00b7c3",
            Self::Spectra => "#5c2e91",
            Self::Gluon => "#e3008c",
            Self::Nucleus => "#5b5fc7",
            Self::Neutrino => "#8764b8",
            Self::Higgs => "#0078d4",
            Self::Phonon => "#038387",
            Self::Polaron => "#ca5010",
            Self::Magnon => "#498205",
            Self::Orbital => "#4a89dc",
        }
    }

    /// Saturated accent for borders and titles.
    pub fn fg1(self) -> String {
        self.seed_hex().to_string()
    }

    /// Soft fill accent for tinted surfaces.
    pub fn bg2(self) -> String {
        self.seed_hex().to_string()
    }

    /// Border / rail accent (same seed as fg1 for marketing surfaces).
    pub fn stroke1(self) -> String {
        self.fg1()
    }

    /// Stable CSS hook for family-scoped styling.
    pub const fn as_class(self) -> &'static str {
        match self {
            Self::Valence => "ufs-family-valence",
            Self::Gluon => "ufs-family-gluon",
            Self::Nucleus => "ufs-family-nucleus",
            Self::Chronon => "ufs-family-chronon",
            Self::Boson => "ufs-family-boson",
            Self::Photon => "ufs-family-photon",
            Self::Orbital => "ufs-family-orbital",
            Self::Spectra => "ufs-family-spectra",
            Self::Neutrino => "ufs-family-neutrino",
            Self::Higgs => "ufs-family-higgs",
            Self::Phonon => "ufs-family-phonon",
            Self::Polaron => "ufs-family-polaron",
            Self::Magnon => "ufs-family-magnon",
        }
    }
}
