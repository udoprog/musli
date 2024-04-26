#![allow(clippy::too_many_arguments)]
#![allow(clippy::needless_late_init)]

mod benchmarker;
mod test;

use proc_macro::TokenStream;

#[proc_macro_derive(Generate, attributes(generate))]
pub fn derive_generate(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    let mut cx = test::Ctxt::default();

    if let Ok(stream) = test::expand(&mut cx, input) {
        return stream.into();
    }

    let mut stream = proc_macro2::TokenStream::default();

    for error in cx.errors {
        stream.extend(error.to_compile_error());
    }

    stream.into()
}

#[proc_macro_attribute]
pub fn benchmarker(attr: TokenStream, input: TokenStream) -> TokenStream {
    let attrs = syn::parse_macro_input!(attr as benchmarker::Attributes);
    let input = syn::parse_macro_input!(input as benchmarker::Benchmarker);

    match input.expand(&attrs) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}
