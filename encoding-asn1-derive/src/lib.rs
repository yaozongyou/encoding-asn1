extern crate proc_macro;

mod marshal;
mod unmarshal;

use proc_macro_error::proc_macro_error;

#[proc_macro_derive(Marshal, attributes(asn1))]
#[proc_macro_error]
pub fn marshal_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: syn::DeriveInput = syn::parse(input).unwrap();

    match input.data {
        syn::Data::Struct(v) => marshal::derive_struct_impl(input.ident, input.generics, v),
        syn::Data::Enum(v) => marshal::derive_enum_impl(input.ident, input.generics, v),
        _ => todo!(),
    }
    .into()
}

#[proc_macro_derive(Unmarshal, attributes(asn1))]
#[proc_macro_error]
pub fn unmarshal_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input: syn::DeriveInput = syn::parse(input).unwrap();

    match input.data {
        syn::Data::Struct(v) => unmarshal::derive_struct_impl(input.ident, input.generics, v),
        syn::Data::Enum(v) => unmarshal::derive_enum_impl(input.ident, input.generics, v),
        _ => todo!(),
    }
    .into()
}
