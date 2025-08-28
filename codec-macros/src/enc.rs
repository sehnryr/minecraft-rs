use proc_macro::TokenStream;
use quote::quote;
use syn::spanned::Spanned as _;

use crate::{
    EnumKind,
    FieldKind,
    enum_kind,
    field_kind,
};

pub fn derive_struct(
    input: &syn::DeriveInput,
    s: &syn::DataStruct,
) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let fields = match &s.fields {
        syn::Fields::Named(n) => &n.named,
        _ => {
            return Err(syn::Error::new(
                s.fields.span(),
                "only named-field structs supported",
            ));
        }
    };

    let mut lets = Vec::new();

    for f in fields {
        let ident = f
            .ident
            .as_ref()
            .ok_or_else(|| syn::Error::new(f.span(), "expected named field"))?;

        let kind = field_kind(&f.attrs)?;
        let ctx = format!("Failed to encode {ident}");

        let stmt = match kind {
            FieldKind::Normal => {
                quote! {
                    self.#ident
                        .encode(writer)
                        .err_context(#ctx)?;
                }
            }
            FieldKind::VarInt => {
                quote! {
                    ::codec::VarInt::new(self.#ident)
                        .encode(writer)
                        .err_context(#ctx)?;
                }
            }
            FieldKind::VarLong => {
                quote! {
                    ::codec::VarLong::new(self.#ident)
                        .encode(writer)
                        .err_context(#ctx)?;
                }
            }
            FieldKind::PrefixedOption => {
                quote! {
                    ::codec::PrefixedOption::from(self.#ident.as_ref())
                        .encode(writer)
                        .err_context(#ctx)?;
                }
            }
        };

        lets.push(stmt);
    }

    Ok(quote! {
        impl ::codec::enc::Encode for #name {
            fn encode<W: ::std::io::Write>(
                &self,
                writer: &mut W,
            ) -> ::core::result::Result<usize, ::codec::enc::EncodeError> {
                use ::codec::enc::EncodeErrorContext as _;
                let mut written_bytes = 0;
                #(written_bytes += #lets)*
                Ok(written_bytes)
            }
        }
    }
    .into())
}

pub fn derive_enum(
    input: &syn::DeriveInput,
    variants: Vec<&syn::Variant>,
) -> syn::Result<TokenStream> {
    let name = &input.ident;

    // unit variants with explicit integer discriminants
    let mut arms = Vec::new();
    for v in variants {
        if !matches!(v.fields, syn::Fields::Unit) {
            return Err(syn::Error::new(
                v.span(),
                "enum variants with fields are not supported",
            ));
        }
        let disc = match &v.discriminant {
            Some((_, expr)) => int_lit(expr)?,
            _ => {
                return Err(syn::Error::new(
                    v.span(),
                    "enum variants must have an explicit integer discriminant, e.g. `Variant = 1`",
                ));
            }
        };

        let v_ident = &v.ident;
        arms.push(quote! { #name::#v_ident => #disc, });
    }

    let ctx = format!("Failed to encode {name}");

    let kind = enum_kind(&input.attrs)?;
    let encode_impl = match kind {
        EnumKind::VarInt => {
            quote! {
                ::codec::VarInt::new(*self as i32)
                    .encode(writer)
                    .err_context(#ctx)
            }
        }
        EnumKind::VarLong => {
            quote! {
                ::codec::VarLong::new(*self as i64)
                    .encode(writer)
                    .err_context(#ctx)
            }
        }
    };

    Ok(quote! {
        impl ::codec::enc::Encode for #name {
            fn encode<W: ::std::io::Write>(
                &self,
                writer: &mut W,
            ) -> ::core::result::Result<usize, ::codec::enc::EncodeError> {
                use ::codec::enc::EncodeErrorContext as _;
                #encode_impl
            }
        }
    }
    .into())
}

fn int_lit(expr: &syn::Expr) -> syn::Result<i64> {
    if let syn::Expr::Lit(syn::ExprLit {
        lit: syn::Lit::Int(li),
        ..
    }) = expr
    {
        Ok(li.base10_parse::<i64>()?)
    } else {
        Err(syn::Error::new(
            expr.span(),
            "discriminant must be an integer literal",
        ))
    }
}
