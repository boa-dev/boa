use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitStr};

/// Construct a utf-16 array literal from a utf-8 [`str`] literal.
#[proc_macro]
pub fn utf16(input: TokenStream) -> TokenStream {
    let literal = parse_macro_input!(input as LitStr);
    let utf8 = literal.value();
    let utf16 = utf8.encode_utf16().collect::<Vec<_>>();
    quote! {
        [#(#utf16),*].as_slice()
    }
    .into()
}
