use syn::{
    braced, bracketed, parenthesized,
    parse::{Error, Parse, ParseStream, Result},
    // token::{Bracket, Paren},
    token::{Bracket},
    Ident, Token, Visibility,
};

/*
To add a new entry like input_enum
 1. add custom_keyword!(input_enum);
 2. add Ident to StateMachineDef
pub struct StateMachineDef {
    /// The visibility modifier (applies to all generated items)
    pub visibility: Visibility,
    pub name: Ident,
    pub initial_state: Ident,
    pub transitions: Vec<TransitionDef>,
    pub derives: Option<Vec<Ident>>,
    pub input_enum: Ident, // XXX Add this!
    pub state_enum: Ident,
    pub output_enum: Ident,
}

 3. add parsing code to parse for StateMachineDef
impl Parse for StateMachineDef {
    fn parse(input: ParseStream) -> Result<Self> {
        // parse input_num(input_enum_name)
        // Add below XXX
        let input_enum;
        let _kw = input.parse::<kw::input_enum>()?;
        let content;
        parenthesized!(content in input);
        input_enum = content.parse()?;
        println!("{:?}", input_enum);

 4. add entry to return
        Ok(Self {
            visibility,
            name,
            initial_state,
            transitions,
            derives,
            input_enum, // XXX add this
            state_enum,
            output_enum,
        })
 5. modify the lib.rs file.

 6.
// XXX might have to add type in the other lib.rs
pub trait StateMachineImpl {
    /// The input alphabet.
    type Input;
    /// The set of possible states.
    type State;
    /// The output alphabet.
    type Output;
    /// The initial state of the machine.
    // allow since there is usually no interior mutability because states are enums
*/

mod kw {
    syn::custom_keyword!(derive);
    syn::custom_keyword!(input_enum);
    syn::custom_keyword!(state_enum);
    syn::custom_keyword!(output_enum);
    syn::custom_keyword!(transfer_struct);
}

/// The output of a state transition
pub struct Output(Option<Ident>);

impl Parse for Output {
    fn parse(input: ParseStream) -> Result<Self> {
        if input.lookahead1().peek(Bracket) {
            let output_content;
            bracketed!(output_content in input);
            Ok(Self(Some(output_content.parse()?)))
        } else {
            Ok(Self(None))
        }
    }
}

impl Into<Option<Ident>> for Output {
    fn into(self) -> Option<Ident> {
        self.0
    }
}

/// Represents a part of state transition without the initial state. The `Parse`
/// trait is implemented for the compact form.
pub struct TransitionEntry {
    pub input_value: Ident,
    pub trans_fn: Ident,
    pub final_state: Ident,
    pub output: Option<Ident>,
    pub err_state: Ident,
    pub err_output: Option<Ident>,
}

impl Parse for TransitionEntry {
    fn parse(input: ParseStream) -> Result<Self> {
        let input_value = input.parse()?;
        input.parse::<Token![=>]>()?;
        let trans_fn = input.parse()?;
        input.parse::<Token![?]>()?;
        let final_state = input.parse()?;
        let output = input.parse::<Output>()?.into();
        input.parse::<Token![:]>()?;
        let err_state = input.parse()?;
        let err_output = input.parse::<Output>()?.into();
        Ok(Self {
            input_value,
            trans_fn,
            final_state,
            output,
            err_state,
            err_output,
        })
    }
}

/// Parses the transition in any of the possible formats.
pub struct TransitionDef {
    pub initial_state: Ident,
    pub transitions: Vec<TransitionEntry>,
}

// Parse the transition in the compact format
// InitialState => {
//     Input1 => TransitionFn ? PassState[Output] : ErrState[ErrOutPut],
//     Input1 => TransitionFn2 ? PassState2[Output2] : ErrState2[ErrOutPut2],
// }
impl Parse for TransitionDef {
    fn parse(input: ParseStream) -> Result<Self> {
        let initial_state = input.parse()?;
        let transitions = {
            input.parse::<Token![=>]>()?;
            let entries_content;
            braced!(entries_content in input);

            let entries: Vec<_> = entries_content
                .parse_terminated::<_, Token![,]>(TransitionEntry::parse)?
                .into_iter()
                .collect();
            if entries.is_empty() {
                return Err(Error::new_spanned(
                    initial_state,
                    "No transitions provided for a compact representation",
                ));
            }
            entries
        };
        Ok(Self {
            initial_state,
            transitions,
        })
    }
}

struct Derives {
    derives: Option<Vec<Ident>>,
}

impl Parse for Derives {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::derive) {
            let kw_derive = input.parse::<kw::derive>()?;
            let entries_content;
            parenthesized!(entries_content in input);
            let entries: Vec<_> = entries_content
                .parse_terminated::<_, Token![,]>(Ident::parse)?
                .into_iter()
                .collect();
            if entries.is_empty() {
                return Err(Error::new_spanned(kw_derive, "Derive list cannot be empty"));
            }
            return Ok(Derives {
                derives: Some(entries),
            });
        }
        Ok(Derives { derives: None })
    }
}

/// Parses the whole state machine definition in the following form (example):
///
/// ```rust,ignore
/// state_machine! {
///     CircuitBreaker(Closed)
///
///     Closed(Unsuccessful) => Open [SetupTimer],
///     Open(TimerTriggered) => HalfOpen,
///     HalfOpen => {
///         Successful => Closed,
///         Unsuccessful => Open [SetupTimer]
///     }
/// }
/// ```
pub struct StateMachineDef {
    /// The visibility modifier (applies to all generated items)
    pub visibility: Visibility,
    pub name: Ident,
    pub initial_state: Ident,
    pub transitions: Vec<TransitionDef>,
    pub derives: Option<Vec<Ident>>,
    pub input_enum: Ident,
    pub state_enum: Ident,
    pub output_enum: Ident,
    pub transfer_struct: Ident,
}

impl Parse for StateMachineDef {
    fn parse(input: ParseStream) -> Result<Self> {
        // parse input_num(input_enum_name)
        let input_enum;
        let _kw = input.parse::<kw::input_enum>()?;
        let content;
        parenthesized!(content in input);
        input_enum = content.parse()?;

        // parse state_num(state_enum_name)
        let state_enum;
        let _kw = input.parse::<kw::state_enum>()?;
        let content;
        parenthesized!(content in input);
        state_enum = content.parse()?;

        // parse output_enum(output_enum_name)
        let output_enum;
        let _kw = input.parse::<kw::output_enum>()?;
        let content;
        parenthesized!(content in input);
        output_enum = content.parse()?;

        // parse transfer_struct(transfer_struct_name)
        let transfer_struct;
        let _kw = input.parse::<kw::transfer_struct>()?;
        let content;
        parenthesized!(content in input);
        transfer_struct = content.parse()?;

        let Derives { derives } = input.parse()?;
        let visibility = input.parse()?;
        let name = input.parse()?;

        let initial_state_content;
        parenthesized!(initial_state_content in input);
        let initial_state = initial_state_content.parse()?;

        let transitions = input
            .parse_terminated::<_, Token![,]>(TransitionDef::parse)?
            .into_iter()
            .collect();

        Ok(Self {
            visibility,
            name,
            initial_state,
            transitions,
            derives,
            input_enum, // XXX add this
            state_enum,
            output_enum,
            transfer_struct,
        })
    }
}
