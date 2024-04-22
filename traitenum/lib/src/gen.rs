#[macro_export]
macro_rules! gen_require {
    ($traitlib_path:path, $macrolib_path:path) => {
        use proc_macro;
        use proc_macro2;
        use $traitlib_path as traitlib;
    };
}

#[macro_export]
macro_rules! gen_derive_macro {
    ($derive_name:ident, $derive_func:ident, $model_bytes_path:path) => {
        #[proc_macro_derive($derive_name, attributes(traitenum))]
        pub fn $derive_func(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
            match traitenum_lib::macros::traitenum_derive_macro(proc_macro2::TokenStream::from(item), $model_bytes_path) {
                Ok(token_stream) => proc_macro::TokenStream::from(token_stream),
                Err(err) => proc_macro::TokenStream::from(err.to_compile_error())
            }
        }        
    };
}