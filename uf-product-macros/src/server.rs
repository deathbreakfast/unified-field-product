use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::Parse;
use syn::{parse2, parse_quote, ItemFn, LitStr, Token};

struct ServerArgs {
    permission: Option<LitStr>,
}

impl Parse for ServerArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(Self { permission: None });
        }

        let ident: syn::Ident = input.parse()?;
        if ident != "permission" {
            return Err(syn::Error::new_spanned(
                ident,
                "unsupported argument; expected `permission = \"...\"`",
            ));
        }
        input.parse::<Token![=]>()?;
        let permission: LitStr = input.parse()?;

        if !input.is_empty() {
            return Err(input.error("unexpected trailing tokens in server macro arguments"));
        }

        Ok(Self {
            permission: Some(permission),
        })
    }
}

/// Expand `#[uf_product_macros::server]` / `#[server]` attribute bodies.
///
/// Without `permission = "…"`, wraps the async body in
/// `uf_product::ssr::with_operation(fn_name, …)`. With a permission argument, uses
/// `higgs::server_runtime::with_operation` and a Gauge `actor_can` gate.
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

    // Extract function name
    let fn_name = &input_fn.sig.ident;
    let fn_name_str = fn_name.to_string();

    // Check if function is async
    if input_fn.sig.asyncness.is_none() {
        return syn::Error::new_spanned(
            &input_fn.sig,
            "#[uf_product::server] can only be used on async functions",
        )
        .to_compile_error();
    }

    // Separate server attributes from other attributes
    let mut server_attrs = Vec::new();
    let mut other_attrs = Vec::new();

    for attr in &input_fn.attrs {
        if attr.path().is_ident("server") {
            server_attrs.push(attr.clone());
        } else {
            other_attrs.push(attr.clone());
        }
    }

    // If no #[server] attribute found, add a default one
    if server_attrs.is_empty() {
        server_attrs.push(parse_quote!(#[server]));
    }

    // Extract the function body
    let body = &input_fn.block;

    // Build the new function with wrapped body
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

    let wrapped_body = if has_permission_arg {
        quote! {
            // Operation tagging lives on `higgs::server_runtime::with_operation`;
            // payload helpers remain under the same module.
            higgs::server_runtime::with_operation(#fn_name_str_lit, async move {
                #permission_guard
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
