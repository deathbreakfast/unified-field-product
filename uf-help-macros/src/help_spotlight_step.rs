use darling::FromMeta;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, ItemFn, LitStr};

#[derive(Debug, FromMeta)]
struct HelpSpotlightStepArgs {
    route: LitStr,
    feature_highlight: LitStr,
    /// Human-readable header shown in the spotlight panel.
    /// Defaults to `feature_highlight` when omitted.
    #[darling(default)]
    title: Option<LitStr>,
    #[darling(default)]
    spotlight: Option<LitStr>,
    #[darling(default)]
    order: Option<u16>,
    /// Popover placement relative to the spotlight anchor (`top`, `bottom`, `left`, `right`, …).
    #[darling(default)]
    position: Option<LitStr>,
}

fn position_variant(lit: &LitStr) -> Result<syn::Ident, syn::Error> {
    let name = match lit.value().to_ascii_lowercase().as_str() {
        "top" => "Top",
        "bottom" => "Bottom",
        "left" => "Left",
        "right" => "Right",
        "topstart" | "top-start" => "TopStart",
        "topend" | "top-end" => "TopEnd",
        "leftstart" | "left-start" => "LeftStart",
        "leftend" | "left-end" => "LeftEnd",
        "rightstart" | "right-start" => "RightStart",
        "rightend" | "right-end" => "RightEnd",
        "bottomstart" | "bottom-start" => "BottomStart",
        "bottomend" | "bottom-end" => "BottomEnd",
        other => {
            return Err(syn::Error::new_spanned(
                lit,
                format!(
                    "unknown help spotlight position `{other}`; expected top, bottom, left, right, or *-start / *-end variants"
                ),
            ));
        }
    };
    Ok(syn::Ident::new(name, lit.span()))
}

pub fn expand(attr: TokenStream, input: TokenStream) -> TokenStream {
    let args = match HelpSpotlightStepArgs::from_list(
        &match darling::ast::NestedMeta::parse_meta_list(attr) {
            Ok(v) => v,
            Err(e) => return e.to_compile_error(),
        },
    ) {
        Ok(v) => v,
        Err(e) => return e.write_errors(),
    };

    let item = match parse2::<ItemFn>(input) {
        Ok(v) => v,
        Err(e) => return e.to_compile_error(),
    };

    let fn_name = &item.sig.ident;
    let route = &args.route;
    let feature_highlight = &args.feature_highlight;
    let title = args.title.as_ref().unwrap_or(feature_highlight);
    let order = args.order.unwrap_or(0);
    let spotlight_expr = match &args.spotlight {
        Some(s) => quote! { ::core::option::Option::Some(#s) },
        None => quote! { ::core::option::Option::None },
    };
    let position_expr = match &args.position {
        Some(p) => match position_variant(p) {
            Ok(variant) => {
                quote! {
                    ::core::option::Option::Some(
                        ::uf_product::primitives::PopoverPosition::#variant
                    )
                }
            }
            Err(e) => return e.to_compile_error(),
        },
        None => quote! { ::core::option::Option::None },
    };

    quote! {
        #item

        ::uf_help::inventory::submit! {
            ::uf_help::HelpStepDescriptor {
                route: #route,
                feature_highlight: #feature_highlight,
                title: #title,
                spotlight: #spotlight_expr,
                order: #order,
                position: #position_expr,
                render: || #fn_name().into_any(),
            }
        }
    }
}
