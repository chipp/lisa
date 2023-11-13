use proc_macro::TokenStream;

#[proc_macro_derive(Str)]
pub fn str_macro_derive(input: TokenStream) -> TokenStream {
    input
}
