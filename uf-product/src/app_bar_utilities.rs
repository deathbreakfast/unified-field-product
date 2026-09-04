//! Inventory contributions for default product app-bar utilities.
//!
//! Optional offerings (`uf-help`, `uf-apps`, `uf-appearance`) submit controls here.
//! `uf-integrations` `DefaultAppBarUtilities` collects and renders them when the
//! host omits an `AppBarUtilities` slot.

use leptos::prelude::AnyView;

/// One default app-bar utility contributed by an optional product offering.
pub struct AppBarUtilityContribution {
    /// Sort key (lower first). Help=10, Apps=20, Appearance=30.
    pub order: u8,
    /// Stable id for tests and docs (`help`, `apps`, `appearance`).
    pub id: &'static str,
    /// Render the control as an [`AnyView`].
    pub render: fn() -> AnyView,
}

impl AppBarUtilityContribution {
    /// Construct a contribution for inventory registration.
    pub const fn new(order: u8, id: &'static str, render: fn() -> AnyView) -> Self {
        Self { order, id, render }
    }
}

inventory::collect!(AppBarUtilityContribution);

/// No-op touch point so offering crates can force-link inventory into the binary.
pub fn register_app_bar_utility() {}

/// Collect offering contributions sorted by [`AppBarUtilityContribution::order`].
pub fn collect_app_bar_utilities() -> Vec<&'static AppBarUtilityContribution> {
    let mut items: Vec<&'static AppBarUtilityContribution> =
        inventory::iter::<AppBarUtilityContribution>
            .into_iter()
            .collect();
    items.sort_by_key(|c| c.order);
    items
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn collect_app_bar_utilities_is_sorted_happy_path() {
        let items = collect_app_bar_utilities();
        let mut prev = 0u8;
        for item in &items {
            assert!(item.order >= prev, "utilities must sort by order");
            prev = item.order;
            assert!(!item.id.is_empty());
        }
    }
}
