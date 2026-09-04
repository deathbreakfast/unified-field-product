/// Build a preview page path from a registry slug.
pub fn preview_page(slug: &str) -> String {
    format!("/orbital/{}", slug.trim_start_matches('/'))
}
