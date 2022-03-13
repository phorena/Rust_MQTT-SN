//! DSL implementation for defining finite state machines for `rust-fsm`. See
//! more in the `rust-fsm` crate documentation.

#![recursion_limit = "128"]
extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use std::collections::HashSet;
use syn::{parse_macro_input, Ident};

mod parser;

/// The full information about a state transition. Used to unify the
/// represantion of the simple and the compact forms.
struct Transition<'a> {
    initial_state: &'a Ident,
    input_value: &'a Ident,
    trans_fn: & 'a Ident,
    final_state: &'a Ident,
    output: &'a Option<Ident>,
    err_state: &'a Ident,
    err_output: &'a Option<Ident>,
}

#[proc_macro]
pub fn state_machine(tokens: TokenStream) -> TokenStream {
    let input = parse_macro_input!(tokens as parser::StateMachineDef);
    // eprintln!("{:?}", input);

    let derives = if let Some(derives) = input.derives {
        quote! { #[derive(#(#derives,)*)] }
    } else {
        quote! {}
    };

    if input.transitions.is_empty() {
        let output = quote! {
            compile_error!("rust-fsm: at least one state transition must be provided");
        };
        return output.into();
    }

    let struct_name = input.name;
    let visibility = input.visibility;

    let transitions: Vec<_> = input
        .transitions
        .iter()
        .flat_map(|def| {
            def.transitions.iter().map(move |transition| Transition {
                initial_state: &def.initial_state,
                input_value: &transition.input_value,
                trans_fn: &transition.trans_fn,
                final_state: &transition.final_state,
                output: &transition.output,
                err_state: &transition.err_state,
                err_output: &transition.err_output,
            })
        })
        .collect();

    let mut states = HashSet::new();
    let mut inputs = HashSet::new();
    let mut outputs = HashSet::new();
    let mut trans_fns = HashSet::new();

    states.insert(&input.initial_state);

    for transition in transitions.iter() {
        states.insert(&transition.initial_state);
        states.insert(&transition.final_state);
        states.insert(&transition.err_state);
        inputs.insert(&transition.input_value);
        trans_fns.insert(&transition.trans_fn);
        if let Some(ref output) = transition.output {
            outputs.insert(output);
        }
        if let Some(ref err_output) = transition.err_output {
            outputs.insert(err_output);
        }
        // println!("trans_fn ----------{:?}", &transition.trans_fn);
    }

    let initial_state_name = &input.initial_state;
    // XXX add this and modify the transition_cases if necessary
    let input_enum_name = input.input_enum;
    let state_enum_name = input.state_enum;
    let output_enum_name = input.output_enum;
    let transfer_struct_name = input.transfer_struct;
    dbg!(transfer_struct_name.clone());

    let mut transition_cases = vec![];
    let mut transition_cases_true = vec![];
    let mut transition_cases_false = vec![];
    for transition in transitions.iter() {
        let initial_state = &transition.initial_state;
        let input_value = &transition.input_value;
        let trans_fn = &transition.trans_fn;
        let final_state = &transition.final_state;
        let output = &transition.output;
        let err_state = &transition.err_state;
        let err_output = &transition.err_output;
        transition_cases.push(quote! {
            (#state_enum_name::#initial_state, #input_enum_name::#input_value) => {
                if #trans_fn (#state_enum_name::#initial_state,
                              #input_enum_name::#input_value,
                              transfer,
                              buf,
                              size,
                              ) {
                    Some((#state_enum_name::#final_state, #output_enum_name::#output))
                } else {
                    Some((#state_enum_name::#err_state, #output_enum_name::#err_output))
                }
            }
        });
        transition_cases_true.push(quote! {
            (#state_enum_name::#initial_state, #input_enum_name::#input_value) => {
                    Some((#state_enum_name::#final_state, #output_enum_name::#output))
            }
        });
        transition_cases_false.push(quote! {
            (#state_enum_name::#initial_state, #input_enum_name::#input_value) => {
                    Some((#state_enum_name::#err_state, #output_enum_name::#err_output))
            }
        });
    }
    let output = quote! {
        #derives
        #visibility struct #struct_name;

        impl rust_fsm::StateMachineImpl for #struct_name {
            type Input = #input_enum_name; // XXX add this line to add input enum
            type State = #state_enum_name;
            type Output = #output_enum_name;
            type Transfer = #transfer_struct_name;
            const INITIAL_STATE: Self::State = #state_enum_name::#initial_state_name;

            fn transition(state: &Self::State,
                          input: &Self::Input,
                          transfer: &mut Self::Transfer,
                          buf: &[u8],
                          size: usize,
                          )
                -> Option<(Self::State, Self::Output)> {
                match (state, input) {
                    #(#transition_cases)*
                    _ => None,
                }
            }
            fn transition_true(state: &Self::State, input: &Self::Input) -> Option<(Self::State, Self::Output)> {
                match (state, input) {
                    #(#transition_cases_true)*
                    _ => None,
                }
            }
            fn transition_false(state: &Self::State, input: &Self::Input) -> Option<(Self::State, Self::Output)> {
                match (state, input) {
                    #(#transition_cases_false)*
                    _ => None,
                }
            }
        }
    };

    output.into()
}
