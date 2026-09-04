//! Orbital design-language introduction page (`/orbital`).

mod layout_chapter;
mod page;
mod principles;
mod surfaces;
mod type_motion;

/// Chapter components composed by [`page::IntroductionPage`].
pub(super) mod chapters {
    pub(super) use super::layout_chapter::LayoutChapter;
    pub(super) use super::principles::PrinciplesChapter;
    pub(super) use super::surfaces::{ColorChapter, ElevationChapter, MaterialChapter};
    pub(super) use super::type_motion::{FurtherReadingFooter, MotionChapter, TypographyChapter};
}

pub use page::IntroductionPage;
