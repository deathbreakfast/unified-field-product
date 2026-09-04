use std::path::Path;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{parse_file, File, Item, ItemMacro, Token};

use crate::AppRouteInfo;

/// Parse a Rust source file for `uf_app`! macro invocations
pub fn parse_file_for_app_routes(
    file_path: &Path,
    package_name: &str,
) -> anyhow::Result<Vec<AppRouteInfo>> {
    let content = std::fs::read_to_string(file_path)
        .map_err(|e| anyhow::anyhow!("Failed to read file {}: {}", file_path.display(), e))?;

    // Parse the file as a syn::File
    let file: File = parse_file(&content)?;

    let mut app_routes = Vec::new();

    // Walk through items looking for uf_app! macro invocations
    for item in &file.items {
        if let Item::Macro(item_macro) = item {
            if is_uf_app_macro(item_macro) {
                if let Some(route_info) = parse_uf_app_macro(item_macro, package_name) {
                    app_routes.push(route_info);
                }
            }
        }
    }

    Ok(app_routes)
}

/// Check if a macro invocation is `uf_app!`.
fn is_uf_app_macro(item_macro: &ItemMacro) -> bool {
    let matches = |ident: &syn::Ident| ident == "uf_app";
    item_macro.mac.path.get_ident().map_or_else(
        || {
            item_macro
                .mac
                .path
                .segments
                .last()
                .is_some_and(|segment| matches(&segment.ident))
        },
        matches,
    )
}

/// Parse a `uf_app!` macro invocation to extract route information
fn parse_uf_app_macro(item_macro: &ItemMacro, package_name: &str) -> Option<AppRouteInfo> {
    // The macro body should contain fields like:
    // { name: "...", id: "...", routes: SomeRoutesComponent, route_path: "/path", ... }

    // Parse the macro tokens as a struct-like syntax
    let tokens = &item_macro.mac.tokens;

    // Try parsing using a custom parser for the macro body
    // The format is: { field1: value1, field2: value2, ... }
    let parsed_fields: syn::Result<AppFields> = syn::parse2(tokens.clone());

    if let Ok(fields) = parsed_fields {
        // If we have both app_id and routes_component, we found a valid registration
        if let (Some(id), Some(routes)) = (fields.id, fields.routes) {
            // Default route_path to "/{id}" if not provided
            let path = fields.route_path.unwrap_or_else(|| format!("/{id}"));
            let name = fields.name.unwrap_or_else(|| id.clone());

            return Some(AppRouteInfo {
                app_id: id,
                app_name: name,
                package_name: package_name.to_string(),
                routes_component: routes,
                route_path: path,
                brand_seed: fields.brand_seed,
            });
        }
    }

    // Fallback: try string-based parsing for simple cases
    let token_string = tokens.to_string();

    let app_id = extract_field_string_value(&token_string, "id");
    let app_name = extract_field_string_value(&token_string, "name");
    // Extract routes field (identifier)
    let routes_component = extract_field_ident_value(&token_string, "routes");
    // Extract route_path field
    let route_path = extract_field_string_value(&token_string, "route_path");
    let brand_seed = extract_field_string_value(&token_string, "brand_seed");

    // If we have both app_id and routes_component, we found a valid registration
    if let (Some(id), Some(routes)) = (app_id, routes_component) {
        let path = route_path.unwrap_or_else(|| format!("/{id}"));
        let name = app_name.unwrap_or_else(|| id.clone());

        Some(AppRouteInfo {
            app_id: id,
            app_name: name,
            package_name: package_name.to_string(),
            routes_component: routes,
            route_path: path,
            brand_seed,
        })
    } else {
        // No routes field means this app doesn't participate in registered routes
        None
    }
}

/// Helper to extract a string literal value from a field
fn extract_field_string_value(token_string: &str, field_name: &str) -> Option<String> {
    let pattern = format!("{field_name} :");
    let start = token_string.find(&pattern)?;
    let after_field = &token_string[start + pattern.len()..];
    let after_ws = after_field.trim_start();
    // Look for string literal
    let after_quote = after_ws.strip_prefix('"')?;
    let end = after_quote.find('"')?;
    Some(after_quote[..end].to_string())
}

/// Helper to extract an identifier value from a field
fn extract_field_ident_value(token_string: &str, field_name: &str) -> Option<String> {
    let pattern = format!("{field_name} :");
    let start = token_string.find(&pattern)?;
    let after_field = &token_string[start + pattern.len()..];
    let after_ws = after_field.trim_start();
    // Extract identifier (starts with letter/underscore, continues with alphanumeric/underscore)
    let mut ident_end = 0;
    for (i, ch) in after_ws.char_indices() {
        if i == 0 {
            if !(ch.is_alphabetic() || ch == '_') {
                break;
            }
        } else if !(ch.is_alphanumeric() || ch == '_') {
            break;
        }
        ident_end = i + ch.len_utf8();
    }
    if ident_end > 0 {
        Some(after_ws[..ident_end].to_string())
    } else {
        None
    }
}

/// Struct to parse `uf_app`! macro fields
struct AppFields {
    id: Option<String>,
    name: Option<String>,
    routes: Option<String>,
    route_path: Option<String>,
    brand_seed: Option<String>,
}

impl Parse for AppFields {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let fields: Punctuated<AppField, Token![,]> = Punctuated::parse_terminated(input)?;

        let mut id = None;
        let mut name = None;
        let mut routes = None;
        let mut route_path = None;
        let mut brand_seed = None;

        for field in fields {
            match field.name.as_str() {
                "id" => {
                    if let syn::Expr::Lit(lit) = field.value {
                        if let syn::Lit::Str(s) = lit.lit {
                            id = Some(s.value());
                        }
                    }
                }
                "name" => {
                    if let syn::Expr::Lit(lit) = field.value {
                        if let syn::Lit::Str(s) = lit.lit {
                            name = Some(s.value());
                        }
                    }
                }
                "routes" => {
                    if let syn::Expr::Path(path) = field.value {
                        if let Some(segment) = path.path.segments.last() {
                            routes = Some(segment.ident.to_string());
                        }
                    }
                }
                "route_path" => {
                    if let syn::Expr::Lit(lit) = field.value {
                        if let syn::Lit::Str(s) = lit.lit {
                            route_path = Some(s.value());
                        }
                    }
                }
                "brand_seed" => {
                    if let syn::Expr::Lit(lit) = field.value {
                        if let syn::Lit::Str(s) = lit.lit {
                            brand_seed = Some(s.value());
                        }
                    }
                }
                _ => {}
            }
        }

        Ok(Self {
            id,
            name,
            routes,
            route_path,
            brand_seed,
        })
    }
}

struct AppField {
    name: String,
    value: syn::Expr,
}

impl Parse for AppField {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name_ident: syn::Ident = input.parse()?;
        input.parse::<Token![:]>()?;
        let value: syn::Expr = input.parse()?;
        Ok(Self {
            name: name_ident.to_string(),
            value,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_temp(name: &str, contents: &str) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(name);
        std::fs::write(&path, contents).unwrap();
        path
    }

    #[test]
    fn parse_uf_app_with_uf_apps_routes_happy_path() {
        let test_file = write_temp(
            "uf_codegen_uf_apps_routes_happy.rs",
            r#"
uf_app! {
    name: "Apps",
    id: "apps",
    description: "Apps directory",
    icon: "📱",
    version: "0.1.0",
    routes: UfAppsRoutes,
    route_path: "/apps",
}
"#,
        );

        let result = parse_file_for_app_routes(&test_file, "uf-apps").unwrap();
        assert_eq!(result.len(), 1);
        let route_info = &result[0];
        assert_eq!(route_info.app_id, "apps");
        assert_eq!(route_info.app_name, "Apps");
        assert_eq!(route_info.routes_component, "UfAppsRoutes");
        assert_eq!(route_info.route_path, "/apps");
        assert_eq!(route_info.package_name, "uf-apps");

        std::fs::remove_file(&test_file).ok();
    }

    #[test]
    fn parse_uf_app_defaults_route_path_happy_path() {
        let test_file = write_temp(
            "uf_codegen_default_route_path_happy.rs",
            r#"
uf_app! {
    name: "Welcome",
    id: "welcome",
    routes: WelcomeRoutes,
}
"#,
        );

        let result = parse_file_for_app_routes(&test_file, "uf-welcome").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].route_path, "/welcome");

        std::fs::remove_file(&test_file).ok();
    }

    #[test]
    fn parse_uf_app_with_brand_seed_happy_path() {
        let test_file = write_temp(
            "uf_codegen_brand_seed_happy.rs",
            r##"
uf_app! {
    name: "UfApp",
    id: "uf-app",
    routes: UfRoutes,
    route_path: "/uf",
    brand_seed: "#112233",
}
"##,
        );

        let result = parse_file_for_app_routes(&test_file, "uf-package").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].brand_seed.as_deref(), Some("#112233"));

        std::fs::remove_file(&test_file).ok();
    }

    #[test]
    fn parse_path_qualified_uf_app_happy_path() {
        let test_file = write_temp(
            "uf_codegen_path_qualified_happy.rs",
            r#"
uf_product_macros::uf_app! {
    name: "Qualified",
    id: "qualified",
    routes: QualifiedRoutes,
    route_path: "/qualified",
}
"#,
        );

        let result = parse_file_for_app_routes(&test_file, "pkg").unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].app_id, "qualified");
        assert_eq!(result[0].routes_component, "QualifiedRoutes");

        std::fs::remove_file(&test_file).ok();
    }

    #[test]
    fn parse_uf_app_without_routes_skipped_sad() {
        let test_file = write_temp(
            "uf_codegen_no_routes_sad.rs",
            r#"
uf_app! {
    name: "TestApp",
    id: "test-app",
    description: "Test application",
    icon: "🧪",
    version: "0.1.0",
}
"#,
        );

        let result = parse_file_for_app_routes(&test_file, "test-package").unwrap();
        assert!(
            result.is_empty(),
            "apps without routes must not enter the registered route table"
        );

        std::fs::remove_file(&test_file).ok();
    }

    #[test]
    fn parse_uf_app_missing_id_skipped_sad() {
        let test_file = write_temp(
            "uf_codegen_missing_id_sad.rs",
            r#"
uf_app! {
    name: "Broken",
    routes: BrokenRoutes,
    route_path: "/broken",
}
"#,
        );

        let result = parse_file_for_app_routes(&test_file, "broken").unwrap();
        assert!(
            result.is_empty(),
            "missing id cannot produce AppRouteInfo, got {result:?}"
        );

        std::fs::remove_file(&test_file).ok();
    }
}
