//! Top-level pages: the dev shell layout and the introduction page, plus
//! re-exports of preview components for convenience.

/// The dev shell layout and its `ComponentPreview` outlet page.
pub mod dev;
/// The Orbital design-language introduction page.
pub mod introduction;

// Re-export page-level components
pub use dev::{ComponentPreview, OrbitalDevLayout};
pub use introduction::IntroductionPage;

// Re-export preview components from their new locations
pub use crate::components::components::AutoGridPreview;
pub use crate::components::components::EmptyStatePreview;
pub use crate::components::components::InfiniteScrollPreview;
pub use crate::components::components::PaginatorPreview;
pub use crate::components::components::ScrollAreaPreview;
pub use crate::components::components::SearchSourcePickerPreview;
pub use crate::components::components::StepperPreview;
pub use crate::components::components::TextPreview;
pub use crate::components::components::UnifiedFieldIconsPreview;
pub use crate::components::patterns::{
    FeatureSectionPreview, HeroSectionPreview, IdentityCardPreview,
};
