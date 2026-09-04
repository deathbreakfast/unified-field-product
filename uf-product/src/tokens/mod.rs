//! Product design tokens — upstream shell tokens plus marketing family accents.

mod brand_tone;
mod platform_family_brand;

pub use brand_tone::BrandTone;
pub use platform_family_brand::PlatformFamilyBrand;

pub use orbital_shell::tokens::{
    CornerRadius, Elevation, GlowIntensity, InteractionState, Material, MotionDuration,
    MotionEasing, MotionPreset, Shape, StrokeWidth, SurfaceTag,
};
