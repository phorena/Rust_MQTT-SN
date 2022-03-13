use proc_macro2::TokenStream as TokenStream2;
use proc_macro2::{Ident, Span};
use proc_macro_error::{abort, ResultExt};
use syn::{self, ext::IdentExt, spanned::Spanned, Field, Lit, Meta, MetaNameValue, Visibility};

use self::GenMode::*;
use super::parse_attr;
use bytes::{BytesMut, BufMut};

#[derive(Debug)]
pub struct GenParams {
    pub mode: GenMode,
    pub global_attr: Option<Meta>,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum GenMode {
    Get,
    GetCopy,
    Set,
    GetMut,
}

impl GenMode {
    pub fn name(self) -> &'static str {
        match self {
            Get => "get",
            GetCopy => "get_copy",
            Set => "set",
            GetMut => "get_mut",
        }
    }

    pub fn prefix(self) -> &'static str {
        match self {
            Get | GetCopy | GetMut => "",
            Set => "set_",
        }
    }

    pub fn suffix(self) -> &'static str {
        match self {
            Get | GetCopy | Set => "",
            GetMut => "_mut",
        }
    }

    fn is_get(self) -> bool {
        match self {
            GenMode::Get | GenMode::GetCopy | GenMode::GetMut => true,
            GenMode::Set => false,
        }
    }
}

pub fn parse_visibility(attr: Option<&Meta>, meta_name: &str) -> Option<Visibility> {
    match attr {
        // `#[get = "pub"]` or `#[set = "pub"]`
        Some(Meta::NameValue(MetaNameValue {
            lit: Lit::Str(ref s),
            path,
            ..
        })) => {
            if path.is_ident(meta_name) {
                s.value().split(' ').find(|v| *v != "with_prefix").map(|v| {
                    syn::parse_str(v)
                        .map_err(|e| syn::Error::new(s.span(), e))
                        .expect_or_abort("invalid visibility found")
                })
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Some users want legacy/compatability.
/// (Getters are often prefixed with `get_`)
fn has_prefix_attr(f: &Field, params: &GenParams) -> bool {
    let inner = f
        .attrs
        .iter()
        .filter_map(|v| parse_attr(v, params.mode))
        .filter(|meta| {
            ["get", "get_copy"]
                .iter()
                .any(|ident| meta.path().is_ident(ident))
        })
        .last();

    // Check it the attr includes `with_prefix`
    let wants_prefix = |possible_meta: &Option<Meta>| -> bool {
        match possible_meta {
            Some(Meta::NameValue(meta)) => {
                if let Lit::Str(lit_str) = &meta.lit {
                    // Naive tokenization to avoid a possible visibility mod named `with_prefix`.
                    lit_str.value().split(' ').any(|v| v == "with_prefix")
                } else {
                    false
                }
            }
            _ => false,
        }
    };

    // `with_prefix` can either be on the local or global attr
    wants_prefix(&inner) || wants_prefix(&params.global_attr)
}

pub fn implement(name: &Ident, field: &Field, params: &GenParams) -> TokenStream2 {
   // dbg!((name, field, params));
    let field_name = field
        .clone()
        .ident
        .unwrap_or_else(|| abort!(field.span(), "Expected the field to have a name"));

    let fn_name = if !has_prefix_attr(field, params)
        && (params.mode.is_get())
        && params.mode.suffix().is_empty()
        && field_name.to_string().starts_with("r#")
    {
        field_name.clone()
    } else {
        Ident::new(
            &format!(
                "{}{}{}{}",
                if has_prefix_attr(field, params) && (params.mode.is_get()) {
                    "get_"
                } else {
                    ""
                },
                params.mode.prefix(),
                field_name.unraw(),
                params.mode.suffix()
            ),
            Span::call_site(),
        )
    };
    //dbg!(field.clone());
    //dbg!(field.ty.clone());
    let ty = field.ty.clone();

    let doc = field.attrs.iter().filter(|v| {
        v.parse_meta()
            .map(|meta| meta.path().is_ident("doc"))
            .unwrap_or(false)
    });

    let attr = field
        .attrs
        .iter()
        .filter_map(|v| parse_attr(v, params.mode))
        .last()
        .or_else(|| params.global_attr.clone());

    //dbg!(field_name.clone());
    //dbg!(fn_name.clone());
    let fn_constraint = Ident::new(
            &format!( "constraint_{}", field_name.unraw()),
            Span::call_site());
    //dbg!(fn_constraint.clone());
    let visibility = parse_visibility(attr.as_ref(), params.mode.name());
    match attr {
        Some(_) => match params.mode {
            GenMode::Get => {
                quote! {
                    #(#doc)*
                    #[inline(always)]
                    #visibility fn #fn_name(&self) -> &#ty {
                        &self.#field_name
                    }
                }
            }
            GenMode::GetCopy => {
                quote! {
                    #(#doc)*
                    #[inline(always)]
                    #visibility fn #fn_name(&self) -> &#ty {
                        &self.#field_name
                    }
                }
            }
            GenMode::GetMut => {
                quote! {
                    #(#doc)*
                    #[inline(always)]
                    #visibility fn #fn_name(&self) -> &#ty {
                        &self.#field_name
                    }
                }
            }
            // TODO constraint function check before setting
            GenMode::Set => {
                quote! {
                    #(#doc)*
                    #[inline(always)]
                    #visibility fn #fn_name(&mut self, val: #ty) -> &mut Self {
                        #name::#fn_constraint(&val);
                        self.#field_name = val.clone();
                        self
                    }
                }
            }
        }
        // Don't need to do anything.
        None => quote! {},
    }
}

/*
Use Result instead of Option

// assume length is checked before the call.
fn pop_u16(barray: &[u8], fr: usize) -> [u8; 2] {
        [barray[fr], barray[fr+1]]
}
fn pop_u32(barray: &[u8], fr: usize) -> [u8; 4] {
        [barray[fr], barray[fr+1], barray[fr+2], barray[fr+3]]
}

        let fn_name = Ident::new(
            &format!( "pop_{}", &p.path.segments[0].ident.to_string()[..] ),
            Span::call_site());


fn try_pop_u16(barray: &[u8], fr: usize) -> Option<[u8; 2]> {
// TODO check for correct len or assume length is checked before the call.
    if barray.len() < 2 { // this assume fr == 0.
        None
    } else {
        Some([barray[fr], barray[fr+1]])
    }
}
*/
pub fn read_functions(_name: &Ident, field: &Field, _params: &GenParams) -> TokenStream2 {
    let field_name = field
        .clone()
        .ident
        .unwrap_or_else(|| abort!(field.span(), "Expected the field to have a name"));

    // dbg!((_name, field, _params));
    // dbg!(field.ty.clone());
    let ty = field.ty.clone();
    let mut ret = quote!{};
    if let syn::Type::Path(p) = ty {
        // dbg!(p);
        // dbg!(&p.path.segments[0].ident);
        let ty = field.ty.clone();
        match &p.path.segments[0].ident.to_string()[..] {
            "u8"|"i8" => {
                ret = quote! {
                    let #field_name = bytes[__offset__];
                    // size_of u8 or i8 is 1
                    __offset__ += 1;
                };
            }
            "u16"|"u32"|"u64"|"u128"|"i16"|"i32"|"i64"|"i128" => {
                // dbg!(field_name.clone());
                ret = quote! {
                    // use array_ref! to convert slice into array, assume Big|Net Endian "_be_"
                    // TODO array_ref! is unsafe, replay it with pop_16(), pop_u32()

                    let #field_name = #ty::from_be_bytes(*array_ref![bytes, __offset__, mem::size_of::<#ty>()]);
                    __offset__ += mem::size_of::<#ty>();
                };
            }
            "String" => {
                ret = quote! {
                    let #field_name = match str::from_utf8(&bytes[__offset__ .. size]) {
                        Ok(v) => v.to_string(),
                        // TODO should not panic, return error
                        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
                    };
                };
            }
            _ => {
                ret = quote! {};
            }
        };
    }
    ret
}

pub fn read_fields(_name: &Ident, field: &Field, _params: &GenParams) -> TokenStream2 {
    let field_name = field
        .clone()
        .ident
        .unwrap_or_else(|| abort!(field.span(), "Expected the field to have a name"));

    // dbg!((name, field, params));
    // dbg!(field.ty.clone());
    let ty = field.ty.clone();
    let mut ret = quote!{};
    if let syn::Type::Path(p) = ty {
        // dbg!(p);
        // dbg!(&p.path.segments[0].ident);
        // use this expression to get the type string for comparision
        match &p.path.segments[0].ident.to_string()[..] {
            "String"|
            "u8"|"u16"|"u32"|"u64"|"u128"|"i8"|"i16"|"i32"|"i64"|"i128" => {
                // dbg!(field_name.clone());
                ret = quote! {
                    #field_name,
                };
            }
            _ => {
                ret = quote! {};
            }
        };
    }
    ret
}

pub fn write_functions(_name: &Ident, field: &Field, _params: &GenParams) -> TokenStream2 {
    let field_name = field
        .clone()
        .ident
        .unwrap_or_else(|| abort!(field.span(), "Expected the field to have a name"));

    // dbg!((_name, field, _params));
    // dbg!(field.ty.clone());
    let ty = field.ty.clone();
    let mut ret = quote!{};
    if let syn::Type::Path(p) = ty {
        // dbg!(p);
        // dbg!(&p.path.segments[0].ident);
        //
        let fn_name = Ident::new(
            &format!( "put_{}", &p.path.segments[0].ident.to_string()[..] ),
            Span::call_site());

        match &p.path.segments[0].ident.to_string()[..] {
            "u8"|"i8"|"u16"|"u32"|"u64"|"u128"|"i16"|"i32"|"i64"|"i128" => {
                ret = quote! {
                    bytes_buf.#fn_name(self.#field_name);
                };
            }
            "String" => {
                ret = quote! {
                    // convert String to bytes
                    bytes_buf.put(self.#field_name.as_bytes());
                };
            }
            _ => {
                ret = quote! {};
            }
        };
    }
    ret
}

