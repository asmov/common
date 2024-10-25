use quote;
use thiserror;
use syn;

pub(crate) const ERROR_PREFIX: &'static str = "[traitenum] ";

pub fn span(source: impl quote::ToTokens) -> ::proc_macro2::Span {
    syn::spanned::Spanned::span(&source.to_token_stream()) 
}

pub fn span_site() -> ::proc_macro2::Span {
    ::proc_macro2::Span::call_site()
}

/// Creates an Err(syn::Error) object. The error message is built using format!() and supports variable arguments.
/// 
/// Requires that ERROR_PREFIX be in scope. E.g., const ERROR_PREFIX: &'static str = "traitenum: ";
/// 
/// Use `synerr!()` to force a `return` from the current block with an Err() of this value.
#[macro_export]
macro_rules! mksynerr {
    ($source:expr, $message:literal) => {
        ::syn::Error::new(crate::error::span($source), ::std::format!("{}{}", crate::error::ERROR_PREFIX, $message))
    };
    ($source:expr, $message:literal, $($v:expr),+) => {
        ::syn::Error::new(crate::error::span($source), ::std::format!("{}{}", crate::error::ERROR_PREFIX,
            ::std::format!($message $( , $v)+)
        ))
    };
}

/// Forces a return from the current block with an Err(syn::Error). The error message is built using format!() and
/// supports variable arguments.
/// 
/// Requires that ERROR_PREFIX be in scope. E.g., const ERROR_PREFIX: &'static str = "traitenum: ";
/// 
/// Use `mksynerr!()` to simply generate a syn::Error.
#[macro_export]
macro_rules! synerr {
    ($source:expr, $message:expr) => {
        return Err(crate::mksynerr!($source, $message))
    };
    ($source:expr, $message:literal, $($v:expr),+) => {
        return Err(::syn::Error::new(crate::error::span($source), ::std::format!("{}{}", crate::error::ERROR_PREFIX,
            ::std::format!($message $( , $v)+)
        )))
    };
}


#[derive(Debug, thiserror::Error)]
pub enum Errors {
    #[error("Unknown {definition} definition setting: {setting}")]
    UnknownDefinitionSetting { definition: String, setting: String }
}

impl Errors {
    pub fn to_syn_error(&self, source: impl quote::ToTokens) -> syn::Error {
        syn::Error::new(
            span(source),
            format!("{}{}",
                ERROR_PREFIX,
                self.to_string())
        )
    }

    pub fn to_syn_err<T>(&self, source: impl quote::ToTokens) -> syn::Result<T> {
        Err(self.to_syn_error(source))
    }
}