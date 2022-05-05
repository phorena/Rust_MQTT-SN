extern crate proc_macro;
extern crate syn;
#[macro_use]
extern crate quote;
extern crate proc_macro2;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use proc_macro_error::{abort, abort_call_site, proc_macro_error, ResultExt};
use syn::{spanned::Spanned, DataStruct, DeriveInput, Meta};
//use byte::*;
//use byte::ctx::*;

mod generate;
use crate::generate::{GenMode, GenParams};
// use bytes::{BufMut, BytesMut};

#[proc_macro_derive(Getters, attributes(get, with_prefix, getset))]
#[proc_macro_error]
pub fn getters(input: TokenStream) -> TokenStream {
    // Parse the string representation
    let ast: DeriveInput = syn::parse(input).expect_or_abort("Couldn't parse for getters");
    let params = GenParams {
        mode: GenMode::Get,
        global_attr: parse_global_attr(&ast.attrs, GenMode::Get),
    };

    // Build the impl
    let gen = produce(&ast, &params);

    // Return the generated impl
    gen.into()
}

#[proc_macro_derive(CopyGetters, attributes(get_copy, with_prefix, getset))]
#[proc_macro_error]
pub fn copy_getters(input: TokenStream) -> TokenStream {
    // Parse the string representation
    let ast: DeriveInput = syn::parse(input).expect_or_abort("Couldn't parse for getters");
    let params = GenParams {
        mode: GenMode::GetCopy,
        global_attr: parse_global_attr(&ast.attrs, GenMode::GetCopy),
    };

    // Build the impl
    let gen = produce(&ast, &params);

    // Return the generated impl
    gen.into()
}

#[proc_macro_derive(MutGetters, attributes(get_mut, getset))]
#[proc_macro_error]
pub fn mut_getters(input: TokenStream) -> TokenStream {
    // Parse the string representation
    let ast: DeriveInput = syn::parse(input).expect_or_abort("Couldn't parse for getters");
    let params = GenParams {
        mode: GenMode::GetMut,
        global_attr: parse_global_attr(&ast.attrs, GenMode::GetMut),
    };

    // Build the impl
    let gen = produce2(&ast, &params);
    // Return the generated impl
    gen.into()
}

#[proc_macro_derive(Setters, attributes(set, getset))]
#[proc_macro_error]
pub fn setters(input: TokenStream) -> TokenStream {
    // Parse the string representation
    let ast: DeriveInput = syn::parse(input).expect_or_abort("Couldn't parse for setters");
    let params = GenParams {
        mode: GenMode::Set,
        global_attr: parse_global_attr(&ast.attrs, GenMode::Set),
    };
    // dbg!(&ast);
    // dbg!(&params);

    // Build the impl
    let gen = produce(&ast, &params);

    // Return the generated impl
    gen.into()
}

fn parse_global_attr(attrs: &[syn::Attribute], mode: GenMode) -> Option<Meta> {
    attrs
        .iter()
        .filter_map(|v| parse_attr(v, mode)) // non "meta" attributes are not our concern
        .last()
}

fn parse_attr(attr: &syn::Attribute, mode: GenMode) -> Option<Meta> {
    use syn::{punctuated::Punctuated, Token};

    if attr.path.is_ident("getset") {
        attr.parse_args_with(Punctuated::<Meta, Token![,]>::parse_terminated)
            .unwrap_or_abort()
            .into_iter()
            .inspect(|meta| {
                if !(meta.path().is_ident("get")
                    || meta.path().is_ident("get_copy")
                    || meta.path().is_ident("get_mut")
                    || meta.path().is_ident("set"))
                {
                    abort!(meta.path().span(), "unknown setter or getter")
                }
            })
            .filter(|meta| meta.path().is_ident(mode.name()))
            .last()
    } else {
        attr.parse_meta()
            .ok()
            .filter(|meta| meta.path().is_ident(mode.name()))
    }
}

fn produce(ast: &DeriveInput, params: &GenParams) -> TokenStream2 {
    let name = &ast.ident;
    // dbg!(name);
    let generics = &ast.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Is it a struct?
    if let syn::Data::Struct(DataStruct { ref fields, .. }) = ast.data {
        let generated = fields.iter().map(|f| generate::implement(name, f, params));

        quote! {
            impl #impl_generics #name #ty_generics #where_clause {
                #(#generated)*
            }
        }
    } else {
        // Nope. This is an Enum. We cannot handle these!
        abort_call_site!("#[derive(Getters)] is only defined for structs, not for enums!");
    }
}

/*
 Implement try_read() and try_write() for the byte lib with a macro.
 See byte crate for more example.
 - Assume length is the length of the entire struct, (***MUST BE "len" for now***).
 - Variable length field is the LAST field in the struct, and it's a String.
 - generate::read_functions(name, f, params)
    - "let len = bytes.read_with::<u8>(__offset__, endian)? as u8;" for scalar types.
    - ...
    - "let client_id = bytes.read_with::<&str>(__offset__, Str::Len(len as usize - *__offset__))?;
        let client_id = client_id.to_string();" for String type.

  - generate::read_fields(name, f, params);
    -       "len,
            msg_type,
            flags,
            protocol_id,
            duration,
            client_id,"

  - generate::write_functions(name, f, params);
        "bytes.write_with::<u8>(offset, self.len, endian)?;", for scalar types.
        ...
        bytes.write::<&str>(offset, &self.client_id)?;", for String type.

use byte::*;
#[derive (Debug)]
#[repr(C)]
struct Connect {
    len: u8,
    msg_type: u8,
    #[debug(format = "0b{:08b}")]
    flags: u8,
    protocol_id: u8,
    duration: u16,
    client_id: String,
}

impl Connect {
    fn try_read(bytes: &[u8], endian: Endian) -> Result<(Self, usize)> {
        let __offset__ = &mut 0;

        let len = bytes.read_with::<u8>(__offset__, endian)? as u8;
        let msg_type = bytes.read_with::<u8>(__offset__, endian)? as u8;
        let flags = bytes.read_with::<u8>(__offset__, endian)? as u8;
        let protocol_id = bytes.read_with::<u8>(__offset__, endian)? as u8;
        let duration = bytes.read_with::<u16>(__offset__, endian)? as u16;
        let client_id = bytes.read_with::<&str>(__offset__, Str::Len(len as usize - *__offset__))?;
        let client_id = client_id.to_string();

        let connect = Connect {
            len,
            msg_type,
            flags,
            protocol_id,
            duration,
            client_id,
        };

        Ok((connect, *__offset__))
    }

    fn try_write(self, bytes: &mut [u8], endian: Endian) -> Result<usize> {
        let offset = &mut 0;

        bytes.write_with::<u8>(offset, self.len, endian)?;
        bytes.write_with::<u8>(offset, self.msg_type, endian)?;
        bytes.write_with::<u8>(offset, self.flags, endian)?;
        bytes.write_with::<u8>(offset, self.protocol_id, endian)?;
        bytes.write_with::<u16>(offset, self.duration, endian)?;
        bytes.write::<&str>(offset, &self.client_id)?;

        Ok(*offset)
    }
}
*/
// Assuming data coming from the network (Big Endian)
// TODO attribute: struct len vs field len
// struct len: len in connect struct
fn produce2(ast: &DeriveInput, params: &GenParams) -> TokenStream2 {
    let name = &ast.ident; // struct name
                           // dbg!(name);
    let generics = &ast.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    // dbg!(impl_generics.clone());
    // dbg!(ty_generics.clone());
    // dbg!(where_clause.clone());

    // Is it a struct?
    if let syn::Data::Struct(DataStruct { ref fields, .. }) = ast.data {
        let generated_writes = fields
            .iter()
            .map(|f| generate::write_functions(name, f, params));
        // let generated_writes = fields.iter().map(|f| generate::read_functions2(name, f, params));
        let generated_reads = fields
            .iter()
            .map(|f| generate::read_functions(name, f, params));
        let generated_fields = fields
            .iter()
            .map(|f| generate::read_fields(name, f, params));

        quote! {
        //            use bytes::Buf;

                    impl #impl_generics #name #ty_generics #where_clause {
                        // fn try_read(bytes: &[u8], size: usize, msg_type: u8) -> Result<(Self, usize), Error> {
                        // pub fn try_read(bytes: &[u8], size: usize, msg_type: u8) -> Option<(Self, usize)> {
                        pub fn try_read(bytes: &[u8], size: usize) -> Option<(Self, usize)> {
                            // TODO check size vs len of struct
                            let mut __offset__ = 0usize;
                            // let __len__ = 0;
                            #(#generated_reads)*
                            let __name__ = #name {
                                #(#generated_fields)*
                            };
                            Some((__name__, __offset__))
                        }
                        // TODO return buf or param &mut? Use
                        // let mut bytes_buf = BytesMut::with_capacity(1500);
                        // fn try_write(self, bytes_buf: &mut BytesMut) -> Result<usize, Error> {
                        pub fn try_write(self, bytes_buf: &mut BytesMut) -> Option<usize> {
                            let mut __offset__ = 0usize;
                            #(#generated_writes)*
                            Some(__offset__)
                        }
                    }
                }
    } else {
        // Nope. This is an Enum. We cannot handle these!
        abort_call_site!("#[derive(Getters)] is only defined for structs, not for enums!");
    }
}
