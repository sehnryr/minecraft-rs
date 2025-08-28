mod dec;
mod enc;

use proc_macro::TokenStream;
use syn::parse_macro_input;
use syn::spanned::Spanned as _;

#[proc_macro_derive(Decode, attributes(codec))]
pub fn derive_decode(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    let out = match &input.data {
        syn::Data::Struct(s) => dec::derive_struct(&input, s),
        syn::Data::Enum(e) => dec::derive_enum(&input, e.variants.iter().collect()),
        syn::Data::Union(u) => Err(syn::Error::new(
            u.union_token.span(),
            "unions not supported",
        )),
    };
    match out {
        Ok(ts) => ts,
        Err(e) => e.to_compile_error().into(),
    }
}

#[proc_macro_derive(Encode, attributes(codec))]
pub fn derive_encode(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    let out = match &input.data {
        syn::Data::Struct(s) => enc::derive_struct(&input, s),
        syn::Data::Enum(e) => enc::derive_enum(&input, e.variants.iter().collect()),
        syn::Data::Union(u) => Err(syn::Error::new(
            u.union_token.span(),
            "unions not supported",
        )),
    };
    match out {
        Ok(ts) => ts,
        Err(e) => e.to_compile_error().into(),
    }
}

enum FieldKind {
    Normal,
    VarInt,
    VarLong,
    PrefixedOption,
}

enum EnumKind {
    VarInt,
    VarLong,
}

fn field_kind(attrs: &[syn::Attribute]) -> syn::Result<FieldKind> {
    let mut kind = FieldKind::Normal;

    for a in attrs {
        if !a.path().is_ident("codec") {
            continue;
        }
        a.parse_nested_meta(|meta| {
            if meta.path.is_ident("varint") {
                kind = FieldKind::VarInt;
                return Ok(());
            }
            if meta.path.is_ident("varlong") {
                kind = FieldKind::VarLong;
                return Ok(());
            }
            if meta.path.is_ident("prefixed_option") {
                kind = FieldKind::PrefixedOption;
                return Ok(());
            }
            Err(meta.error(
                "unsupported #[codec(...)] argument; expected `varint`, `varlong` or \
                 `prefixed_option`",
            ))
        })?;
    }

    Ok(kind)
}

fn enum_kind(attrs: &[syn::Attribute]) -> syn::Result<EnumKind> {
    let mut kind: Option<EnumKind> = None;

    for a in attrs {
        if !a.path().is_ident("codec") {
            continue;
        }
        a.parse_nested_meta(|meta| {
            if meta.path.is_ident("varint") {
                kind = Some(EnumKind::VarInt);
                return Ok(());
            }
            if meta.path.is_ident("varlong") {
                kind = Some(EnumKind::VarLong);
                return Ok(());
            }
            Err(meta.error(
                "unsupported #[codec(...)] argument for enum; expected `varint` or `varlong`",
            ))
        })?;
    }

    match kind {
        Some(k) => Ok(k),
        None => Err(syn::Error::new(
            syn::spanned::Spanned::span(&attrs[0]),
            "enum must have a #[codec(varint)] or #[codec(varlong)] attribute",
        )),
    }
}
