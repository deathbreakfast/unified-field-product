use leptos::prelude::*;
use leptos_router::hooks::{use_location, use_params_map};
use leptos_router::params::ParamsMap;
use uf_product::components::{Body1, Title3};

use super::collect_preview_registrations;

/// Resolve the preview slug from the splat param or the pathname under `/orbital/`.
pub fn preview_slug_from_route(params: &ParamsMap, pathname: &str) -> String {
    if let Some(slug) = params.get("slug") {
        let slug = normalize_preview_slug(&slug);
        if !slug.is_empty() {
            return slug;
        }
    }

    normalize_preview_slug(
        pathname
            .strip_prefix("/orbital/")
            .or_else(|| pathname.strip_prefix("/orbital"))
            .unwrap_or(pathname),
    )
}

/// Trim whitespace and leading/trailing slashes from a raw slug value.
pub fn normalize_preview_slug(raw: &str) -> String {
    raw.trim()
        .trim_start_matches('/')
        .trim_end_matches('/')
        .to_string()
}

/// Registry-driven catch-all preview page: renders the registration matching the
/// current route's slug, or a "not found" placeholder.
#[component]
pub fn PreviewSlugPage() -> impl IntoView {
    let params = use_params_map();
    let location = use_location();

    let slug = Memo::new(move |_| {
        let pathname = location.pathname.get();
        preview_slug_from_route(&params.get(), &pathname)
    });

    move || {
        let current = slug.get();
        let registration = collect_preview_registrations()
            .into_iter()
            .find(|item| item.slug == current);

        match registration {
            Some(item) => (item.render)().into_any(),
            None => view! {
                <div data-testid="preview-not-found">
                    <Title3>"Preview not found"</Title3>
                    <Body1>{format!("No preview registered for slug: {current}")}</Body1>
                </div>
            }
            .into_any(),
        }
    }
}
