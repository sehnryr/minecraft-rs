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
    let mut names = Vec::new();

    for f in fields {
        let ident = f
            .ident
            .as_ref()
            .ok_or_else(|| syn::Error::new(f.span(), "expected named field"))?;
        let ty: &syn::Type = &f.ty;

        let kind = field_kind(&f.attrs)?;
        let ctx = format!("Failed to decode {ident}");

        let stmt = match kind {
            FieldKind::Normal => {
                quote! {
                    let #ident = <#ty as ::codec::dec::Decode>::decode(reader)
                        .err_context(#ctx)?;
                }
            }
            FieldKind::VarInt => {
                quote! {
                    let #ident = ::codec::VarInt::decode(reader)
                        .err_context(#ctx)?
                        .value();
                }
            }
            FieldKind::VarLong => {
                quote! {
                    let #ident = ::codec::VarLong::decode(reader)
                        .err_context(#ctx)?
                        .value();
                }
            }
            FieldKind::PrefixedOption => {
                quote! {
                    let #ident: #ty = ::codec::PrefixedOption::decode(reader)
                        .err_context(#ctx)?
                        .into();
                }
            }
        };

        lets.push(stmt);
        names.push(ident);
    }

    Ok(quote! {
        impl ::codec::dec::Decode for #name {
            fn decode<R: ::std::io::Read>(
                reader: &mut R
            ) -> ::core::result::Result<Self, ::codec::dec::DecodeError> {
                use ::codec::dec::DecodeErrorContext as _;
                #(#lets)*
                Ok(Self { #(#names,)* })
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
        let lit = syn::LitInt::new(&disc.to_string(), v.span());
        arms.push(quote! { #lit => Ok(#name::#v_ident), });
    }

    let ctx = format!("Failed to decode {name}");

    let kind = enum_kind(&input.attrs)?;
    let decode_impl = match kind {
        EnumKind::VarInt => {
            quote! {
                let raw = ::codec::VarInt::decode(reader)
                    .err_context(#ctx)?
                    .value();
                match raw {
                    #(#arms)*
                    _ => Err(::codec::dec::DecodeError::Custom {
                        message: ::std::format!(
                            "Invalid {} discriminant: {}",
                            ::core::stringify!(#name), raw
                        ),
                    }),
                }
            }
        }
        EnumKind::VarLong => {
            quote! {
                let raw = ::codec::VarLong::decode(reader)
                    .err_context(#ctx)?
                    .value();
                match raw {
                    #(#arms)*
                    _ => Err(::codec::dec::DecodeError::Custom {
                        message: ::std::format!(
                            "Invalid {} discriminant: {}",
                            ::core::stringify!(#name), raw
                        ),
                    }),
                }
            }
        }
    };

    Ok(quote! {
        impl ::codec::dec::Decode for #name {
            fn decode<R: ::std::io::Read>(
                reader: &mut R
            ) -> ::core::result::Result<Self, ::codec::dec::DecodeError> {
                use ::codec::dec::DecodeErrorContext as _;
                #decode_impl
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
