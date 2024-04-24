use traitenum_lib as lib;

#[proc_macro_attribute]
pub fn enumtrait(attr: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match lib::macros::enumtrait_macro(proc_macro2::TokenStream::from(attr), proc_macro2::TokenStream::from(item)) {
        Ok(token_stream) => proc_macro::TokenStream::from(token_stream),
        Err(err) => proc_macro::TokenStream::from(err.to_compile_error())
    }
}