use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::Parse;
use syn::{parse2, parse_quote, ItemFn, LitStr, Token};

struct ServerArgs {
    permission: Option<LitStr>,
    /// `None` = no step-up; `Some("")` / `Some("window")` = window; `Some("fresh")` = fresh.
    step_up: Option<LitStr>,
}

impl Parse for ServerArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(Self {
                permission: None,
                step_up: None,
            });
        }

        let mut permission = None;
        let mut step_up = None;

        while !input.is_empty() {
            let ident: syn::Ident = input.parse()?;
            if ident == "permission" {
                input.parse::<Token![=]>()?;
                permission = Some(input.parse::<LitStr>()?);
            } else if ident == "step_up" {
                if input.peek(Token![=]) {
                    input.parse::<Token![=]>()?;
                    step_up = Some(input.parse::<LitStr>()?);
                } else {
                    step_up = Some(LitStr::new("window", ident.span()));
                }
            } else {
                return Err(syn::Error::new_spanned(
                    ident,
                    "unsupported argument; expected `permission = \"...\"` or `step_up[=\"window\"|\"fresh\"]`",
                ));
            }
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(Self {
            permission,
            step_up,
        })
    }
}

/// Expand `#[uf_product_macros::server]` / `#[server]` attribute bodies.
///
/// Without `permission = "…"`, wraps the async body in
/// `uf_product::ssr::with_operation(fn_name, …)`. With a permission argument, uses
/// `higgs::server_runtime::with_operation` and a Gauge `actor_can` gate.
/// Optional `step_up` / `step_up = "fresh"` inserts
/// [`uf_product::permissions::require_step_up`] after the permission gate.
///
/// Prefer the public docs on [`crate::server`] for integrator examples.
pub fn expand_server(attr: TokenStream, input: TokenStream) -> TokenStream {
    let args = match parse2::<ServerArgs>(attr) {
        Ok(args) => args,
        Err(e) => return e.to_compile_error(),
    };
    let input_fn = match parse2::<ItemFn>(input) {
        Ok(item) => item,
        Err(e) => return e.to_compile_error(),
    };

    let fn_name = &input_fn.sig.ident;
    let fn_name_str = fn_name.to_string();

    if input_fn.sig.asyncness.is_none() {
        return syn::Error::new_spanned(
            &input_fn.sig,
            "#[uf_product::server] can only be used on async functions",
        )
        .to_compile_error();
    }

    let mut server_attrs = Vec::new();
    let mut other_attrs = Vec::new();

    for attr in &input_fn.attrs {
        if attr.path().is_ident("server") {
            server_attrs.push(attr.clone());
        } else {
            other_attrs.push(attr.clone());
        }
    }

    if server_attrs.is_empty() {
        server_attrs.push(parse_quote!(#[server]));
    }

    let body = &input_fn.block;
    let vis = &input_fn.vis;
    let sig = &input_fn.sig;
    let fn_name_str_lit = syn::LitStr::new(&fn_name_str, proc_macro2::Span::call_site());

    let has_permission_arg = args.permission.is_some();
    let permission_guard = args.permission.map_or_else(
        || quote! {},
        |permission| {
            quote! {
                #[cfg(feature = "ssr")]
                {
                    uf_product::permissions::require_permission(#permission).await?;
                }
            }
        },
    );

    let has_step_up = args.step_up.is_some();
    let step_up_guard = args.step_up.map_or_else(
        || quote! {},
        |mode| {
            let mode_lit = if mode.value().is_empty() {
                LitStr::new("window", mode.span())
            } else {
                mode
            };
            quote! {
                #[cfg(feature = "ssr")]
                {
                    uf_product::permissions::require_step_up(#mode_lit).await?;
                }
            }
        },
    );

    let use_higgs_runtime = has_permission_arg || has_step_up;
    let wrapped_body = if use_higgs_runtime {
        quote! {
            higgs::server_runtime::with_operation(#fn_name_str_lit, async move {
                #permission_guard
                #step_up_guard
                #body
            }).await
        }
    } else {
        quote! {
            uf_product::ssr::with_operation(#fn_name_str_lit, async move {
                #body
            }).await
        }
    };

    quote! {
        #(#other_attrs)*
        #(#server_attrs)*
        #vis #sig {
            #wrapped_body
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn server_wraps_async_fn_with_operation_happy_path() {
        let out = expand_server(
            TokenStream::new(),
            quote! {
                pub async fn get_apps() -> Result<(), ()> {
                    Ok(())
                }
            },
        )
        .to_string();
        assert!(
            out.contains("uf_product :: ssr :: with_operation"),
            "expected with_operation wrap: {out}"
        );
        assert!(
            out.contains("\"get_apps\""),
            "expected operation label: {out}"
        );
        assert!(out.contains("# [server]"), "expected #[server] attr: {out}");
    }

    #[test]
    fn server_permission_gate_uses_higgs_happy_path() {
        let out = expand_server(
            quote! { permission = "apps.view" },
            quote! {
                pub async fn gated() -> Result<(), ()> {
                    Ok(())
                }
            },
        )
        .to_string();
        assert!(
            out.contains("higgs :: server_runtime :: with_operation"),
            "expected higgs server_runtime with_operation: {out}"
        );
        assert!(
            out.contains("require_permission"),
            "expected permission gate: {out}"
        );
        assert!(
            out.contains("\"apps.view\""),
            "expected permission name: {out}"
        );
    }

    #[test]
    fn server_step_up_window_gate_happy_path() {
        let out = expand_server(
            quote! { permission = "GaugeAdmin", step_up },
            quote! {
                pub async fn add_user() -> Result<(), ()> {
                    Ok(())
                }
            },
        )
        .to_string();
        assert!(
            out.contains("require_permission"),
            "expected permission gate: {out}"
        );
        assert!(
            out.contains("require_step_up"),
            "expected step_up gate: {out}"
        );
        assert!(out.contains("\"window\""), "expected window mode: {out}");
    }

    #[test]
    fn server_step_up_fresh_gate_happy_path() {
        let out = expand_server(
            quote! { permission = "SecretsReveal", step_up = "fresh" },
            quote! {
                pub async fn reveal() -> Result<(), ()> {
                    Ok(())
                }
            },
        )
        .to_string();
        assert!(out.contains("\"fresh\""), "expected fresh mode: {out}");
        assert!(
            out.contains("require_step_up"),
            "expected step_up gate: {out}"
        );
    }

    #[test]
    fn server_rejects_sync_fn_sad() {
        let out = expand_server(
            TokenStream::new(),
            quote! {
                pub fn not_async() -> Result<(), ()> {
                    Ok(())
                }
            },
        )
        .to_string();
        assert!(
            out.contains("can only be used on async functions"),
            "expected async-only error: {out}"
        );
    }

    #[test]
    fn server_rejects_unknown_arg_sad() {
        let out = expand_server(
            quote! { foo = "bar" },
            quote! {
                pub async fn gated() -> Result<(), ()> {
                    Ok(())
                }
            },
        )
        .to_string();
        assert!(
            out.contains("unsupported argument"),
            "expected unsupported-arg error: {out}"
        );
    }
}
