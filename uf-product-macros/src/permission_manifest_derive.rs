use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse2, spanned::Spanned, Data, DeriveInput, Expr, ExprLit, Fields, Lit, Meta, MetaNameValue,
};

fn lit_str(expr: &Expr) -> Option<String> {
    if let Expr::Lit(ExprLit {
        lit: Lit::Str(s), ..
    }) = expr
    {
        Some(s.value())
    } else {
        None
    }
}

fn parse_manifest_attr(input: &DeriveInput) -> syn::Result<(String, String, String)> {
    let mut domain_key = None;
    let mut domain_name = None;
    let mut domain_description = None;

    for attr in &input.attrs {
        if !attr.path().is_ident("permission_manifest") {
            continue;
        }

        let metas = attr.parse_args_with(
            syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated,
        )?;
        for meta in metas {
            let Meta::NameValue(MetaNameValue { path, value, .. }) = meta else {
                return Err(syn::Error::new(
                    attr.span(),
                    "permission_manifest entries must be key = \"value\"",
                ));
            };

            if path.is_ident("domain_key") {
                domain_key = lit_str(&value);
            } else if path.is_ident("domain_name") {
                domain_name = lit_str(&value);
            } else if path.is_ident("domain_description") {
                domain_description = lit_str(&value);
            } else {
                return Err(syn::Error::new_spanned(
                    path,
                    "unsupported permission_manifest key",
                ));
            }
        }
    }

    match (domain_key, domain_name, domain_description) {
        (Some(key), Some(name), Some(description)) => Ok((key, name, description)),
        _ => Err(syn::Error::new(
            input.span(),
            "missing #[permission_manifest(domain_key = \"...\", domain_name = \"...\", domain_description = \"...\")]",
        )),
    }
}

fn parse_permission_description(variant: &syn::Variant) -> syn::Result<String> {
    for attr in &variant.attrs {
        if !attr.path().is_ident("permission") {
            continue;
        }

        let metas = attr.parse_args_with(
            syn::punctuated::Punctuated::<Meta, syn::Token![,]>::parse_terminated,
        )?;
        for meta in metas {
            let Meta::NameValue(MetaNameValue { path, value, .. }) = meta else {
                return Err(syn::Error::new(
                    attr.span(),
                    "permission entries must be key = \"value\"",
                ));
            };
            if path.is_ident("description") {
                if let Some(desc) = lit_str(&value) {
                    return Ok(desc);
                }
                return Err(syn::Error::new_spanned(
                    value,
                    "description must be a string",
                ));
            }
        }
    }

    Err(syn::Error::new(
        variant.span(),
        "missing #[permission(description = \"...\")] on enum variant",
    ))
}

pub fn expand_derive_permission_manifest(input: TokenStream) -> TokenStream {
    let input = match parse2::<DeriveInput>(input) {
        Ok(input) => input,
        Err(e) => return e.to_compile_error(),
    };
    let enum_ident = input.ident.clone();
    let (domain_key, domain_name, domain_description) = match parse_manifest_attr(&input) {
        Ok(v) => v,
        Err(e) => return e.to_compile_error(),
    };

    let Data::Enum(data_enum) = &input.data else {
        return syn::Error::new(
            input.span(),
            "UfPermissionManifest can only be derived for enums",
        )
        .to_compile_error();
    };

    let mut variants = Vec::new();
    let mut descriptions = Vec::new();
    for variant in &data_enum.variants {
        if !matches!(variant.fields, Fields::Unit) {
            return syn::Error::new_spanned(
                &variant.fields,
                "UfPermissionManifest only supports fieldless enum variants",
            )
            .to_compile_error();
        }
        variants.push(variant.ident.clone());
        match parse_permission_description(variant) {
            Ok(desc) => descriptions.push(desc),
            Err(e) => return e.to_compile_error(),
        }
    }

    let all_variants_const = format_ident!("__{}_ALL", enum_ident.to_string().to_uppercase());
    let permission_specs_const = format_ident!(
        "__{}_PERMISSION_SPECS",
        enum_ident.to_string().to_uppercase()
    );
    let domain_specs_const =
        format_ident!("__{}_DOMAIN_SPECS", enum_ident.to_string().to_uppercase());
    let manifest_static = format_ident!(
        "__{}_APP_PERMISSION_MANIFEST",
        enum_ident.to_string().to_uppercase()
    );

    let expanded = quote! {
        impl #enum_ident {
            pub fn as_str(self) -> &'static str {
                match self {
                    #(Self::#variants => stringify!(#variants),)*
                }
            }
        }

        impl ::core::marker::Copy for #enum_ident {}
        impl ::core::clone::Clone for #enum_ident {
            fn clone(&self) -> Self {
                *self
            }
        }

        const #all_variants_const: &[#enum_ident] = &[
            #(#enum_ident::#variants,)*
        ];

        const #permission_specs_const: &[::uf_product::PermissionSpec] = &[
            #(::uf_product::PermissionSpec {
                name: stringify!(#variants),
                description: #descriptions,
            },)*
        ];

        const #domain_specs_const: &[::uf_product::PermissionDomainSpec] = &[
            ::uf_product::PermissionDomainSpec {
                key: #domain_key,
                name: #domain_name,
                description: #domain_description,
                permissions: #permission_specs_const,
            }
        ];

        static #manifest_static: ::uf_product::AppPermissionManifest = ::uf_product::AppPermissionManifest {
            app_id: #domain_key,
            domains: #domain_specs_const,
        };

        impl ::uf_product::PermissionEnum for #enum_ident {
            fn as_str(self) -> &'static str {
                self.as_str()
            }

            fn all() -> &'static [Self] {
                #all_variants_const
            }
        }

        impl ::uf_product::AppPermissionManifestProvider for #enum_ident {
            fn manifest() -> &'static ::uf_product::AppPermissionManifest {
                &#manifest_static
            }
        }
    };

    expanded
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn permission_manifest_derive_expands_provider_happy_path() {
        let out = expand_derive_permission_manifest(quote! {
            #[permission_manifest(
                domain_key = "apps",
                domain_name = "Apps",
                domain_description = "Apps directory access"
            )]
            enum AppsPermission {
                #[permission(description = "View apps directory")]
                View,
                #[permission(description = "Manage apps directory")]
                Manage,
            }
        })
        .to_string();
        assert!(
            out.contains("AppPermissionManifestProvider"),
            "expected provider impl: {out}"
        );
        assert!(
            out.contains("PermissionEnum"),
            "expected PermissionEnum impl: {out}"
        );
        assert!(out.contains("\"apps\""), "expected domain_key: {out}");
        assert!(out.contains("View"), "expected View variant: {out}");
    }

    #[test]
    fn permission_manifest_missing_attr_sad() {
        let out = expand_derive_permission_manifest(quote! {
            enum AppsPermission {
                #[permission(description = "View apps directory")]
                View,
            }
        })
        .to_string();
        assert!(
            out.contains("missing #[permission_manifest"),
            "expected missing-attr error: {out}"
        );
    }

    #[test]
    fn permission_manifest_rejects_struct_sad() {
        let out = expand_derive_permission_manifest(quote! {
            #[permission_manifest(
                domain_key = "apps",
                domain_name = "Apps",
                domain_description = "Apps directory access"
            )]
            struct NotAnEnum;
        })
        .to_string();
        assert!(
            out.contains("can only be derived for enums"),
            "expected enum-only error: {out}"
        );
    }

    #[test]
    fn permission_manifest_missing_variant_description_sad() {
        let out = expand_derive_permission_manifest(quote! {
            #[permission_manifest(
                domain_key = "apps",
                domain_name = "Apps",
                domain_description = "Apps directory access"
            )]
            enum AppsPermission {
                View,
            }
        })
        .to_string();
        assert!(
            out.contains("missing #[permission(description"),
            "expected missing description error: {out}"
        );
    }
}
