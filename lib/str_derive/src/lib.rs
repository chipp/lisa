use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(Str)]
pub fn str_macro_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let name = &ast.ident;
    let data = &ast.data;

    if let syn::Data::Enum(_) = data {
        let gen = quote! {
            impl std::fmt::Display for #name {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    self.serialize(f)
                }
            }

            impl std::str::FromStr for #name {
                type Err = serde::de::value::Error;

                fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
                    use serde::de::IntoDeserializer;

                    Self::deserialize(s.into_deserializer())
                }
            }
        };

        gen.into()
    } else {
        quote! {compile_error!("Str supports only enums")}.into()
    }
}
