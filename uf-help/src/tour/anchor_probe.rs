//! DOM check for whether a spotlight anchor is on screen.

/// `true` when an element with `id` exists, has a non-zero box, and is not
/// `visibility: hidden`. Below-the-fold elements count as visible because the
/// spotlight scrolls its anchor into view.
#[cfg(target_arch = "wasm32")]
pub(crate) fn anchor_visible(id: &str) -> bool {
    let Some(window) = web_sys::window() else {
        return false;
    };
    let Some(element) = window.document().and_then(|doc| doc.get_element_by_id(id)) else {
        return false;
    };
    let rect = element.get_bounding_client_rect();
    if rect.width() <= 0.0 || rect.height() <= 0.0 {
        return false;
    }
    match window.get_computed_style(&element) {
        Ok(Some(style)) => style
            .get_property_value("visibility")
            .map_or(true, |value| value != "hidden"),
        _ => true,
    }
}

/// Server render has no DOM, so no anchor is visible.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn anchor_visible(_id: &str) -> bool {
    false
}
