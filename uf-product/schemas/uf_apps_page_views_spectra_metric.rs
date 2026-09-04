#![allow(clippy::too_many_arguments)]

use spectra::spectra_metric;

spectra_metric! {
    UfAppsPageViews {
        store: "uf",
        name: "uf_apps_page_views",
        version: "0.1.0",
        description: "Main-shell page navigations. Labels: app_id, outcome.",
    }
}
