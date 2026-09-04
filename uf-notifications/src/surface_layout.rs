//! Shared width constraints for notification bell dropdown and inbox page surfaces.

/// Minimum bell dropdown panel width — empty/loading copy must not shrink below this.
pub const BELL_DROPDOWN_MIN_WIDTH: &str = "360px";

/// Maximum bell dropdown panel width — long titles/messages truncate inside this cap.
pub const BELL_DROPDOWN_MAX_WIDTH: &str = "400px";

/// Minimum inbox page content width — matches dropdown floor for consistent chrome.
pub const INBOX_MIN_WIDTH: &str = "360px";
