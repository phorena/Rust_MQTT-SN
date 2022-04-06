/**
 * Copyright (c) 2020 Ayush Kumar Mishra
 *
 * This source code is licensed under the MIT License found in
 * the LICENSE file in the root directory of this source tree.
 */
extern crate proc_macro;
use proc_macro::{TokenStream, TokenTree};

/// This attribute will enable a function to access the caller's source location.
#[proc_macro_attribute]
pub fn trace(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut prefix: TokenStream = "

macro_rules! function {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        &name[..name.len() - 3]
    }}
}

    use trace_caller::backtrace::{Backtrace, BacktraceFrame};
    let (trace, curr_file, curr_line) = (Backtrace::new(), file!(), line!());
    let frames = trace.frames();
    let previous_symbol = frames
        .iter()
        .flat_map(BacktraceFrame::symbols)
        .skip_while(|s| {
            s.filename()
                .map(|p| !p.ends_with(curr_file))
                .unwrap_or(true)
                || s.lineno() != Some(curr_line)
        })
        .nth(1 as usize)
        .unwrap();

        println!(\"{:?} {:?}:{:?} called from {:?}:{:?}\",
            file!(), function!(), line!(),
            previous_symbol.filename().unwrap(),
            previous_symbol.lineno().unwrap());
    "
    .parse()
    .unwrap();

    item.into_iter()
        .map(|tt| match tt {
            TokenTree::Group(ref g) if g.delimiter() == proc_macro::Delimiter::Brace => {
                prefix.extend(g.stream());

                TokenTree::Group(proc_macro::Group::new(
                    proc_macro::Delimiter::Brace,
                    prefix.clone(),
                ))
            }
            other => other,
        })
        .collect()
}
