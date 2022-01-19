use proc_macro_error::{abort, ResultExt};
use quote::{quote, ToTokens, TokenStreamExt};
use syn::{
    self,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    Ident, LitInt, Token,
};

struct QuoteOption<T>(Option<T>);

impl<T: ToTokens> ToTokens for QuoteOption<T> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.append_all(match self.0 {
            Some(ref t) => quote! { ::std::option::Option::Some(#t) },
            None => quote! { ::std::option::Option::None },
        });
    }
}

#[derive(Debug)]
pub enum Asn1Attr {
    Explicit(Ident),
    Implicit(Ident),
    Tag(Ident, i32),
}

impl Parse for Asn1Attr {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let name: Ident = input.parse()?;
        let name_str = name.to_string();

        if input.peek(Token![=]) {
            let assign_token = input.parse::<Token![=]>()?; // skip '='

            if input.peek(LitInt) {
                let lit: LitInt = input.parse()?;
                let lit_int = lit.base10_parse::<i32>()?;

                match &*name_str {
                    "tag" => Ok(Asn1Attr::Tag(name, lit_int)),
                    _ => abort!(name, "unexpected attribute: {}", name_str),
                }
            } else {
                abort!(
                    assign_token,
                    "expected `string literal` or `expression` after `=`"
                );
            }
        } else {
            match name_str.as_ref() {
                "explicit" => Ok(Asn1Attr::Explicit(name)),
                "implicit" => Ok(Asn1Attr::Implicit(name)),
                _ => abort!(name, "unexpected attribute: {}", name_str),
            }
        }
    }
}

fn parse_attributes(attrs: &[syn::Attribute]) -> Vec<Asn1Attr> {
    attrs
        .iter()
        .filter(|attr| attr.path.is_ident("asn1"))
        .flat_map(|attr| {
            attr.parse_args_with(Punctuated::<Asn1Attr, Token![,]>::parse_terminated)
                .unwrap_or_abort()
        })
        .collect()
}

pub fn derive_struct_impl(
    name: syn::Ident,
    generics: syn::Generics,
    container: syn::DataStruct,
) -> proc_macro2::TokenStream {
    let mut list = vec![];

    for (i, field) in container.fields.iter().enumerate() {
        let mut explicit = false;
        let mut tag = QuoteOption(None);

        for attr in parse_attributes(&field.attrs) {
            match attr {
                Asn1Attr::Explicit(_) => {
                    explicit = true;
                }
                Asn1Attr::Implicit(_) => {
                    explicit = false;
                }
                Asn1Attr::Tag(_, v) => {
                    tag = QuoteOption(Some(v));
                }
            }
        }

        let i = syn::Index::from(i);
        let field = field
            .ident
            .as_ref()
            .map(|name| quote!(#name))
            .unwrap_or_else(|| quote!(#i));

        list.push(proc_macro2::TokenStream::from(quote! {
            body.push(self.#field.marshal_with_params(&common::FieldParameters {
                optional: false,
                explicit: #explicit,
                application: false,
                private: false,
                default_value: None,
                tag: #tag,
                string_type: 0,
                time_type: 0,
                set: false,
                omit_empty: false,
            }));
        }));
    }

    let marshal_impl = quote! {
        let mut body = vec![];

        #(#list)*

        let tag_and_length = match params.tag {
            Some(tag) => {
                common::TagAndLength {
                    class: common::CLASS_CONTEXT_SPECIFIC,
                    tag: tag,
                    length: body.iter().fold(0, |total_length, item| total_length + item.len()),
                    is_compound: true,
                }
            },
            None => {
                common::TagAndLength {
                    class: 0,
                    tag: common::TAG_SEQUENCE,
                    length: body.iter().fold(0, |total_length, item| total_length + item.len()),
                    is_compound: true,
                }
            }
        };

        let mut v = tag_and_length.encode();
        body.iter_mut().for_each(|mut i| v.append(&mut i));
        v
    };

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    proc_macro2::TokenStream::from(quote! {
        impl #impl_generics  Marshaler for #name #ty_generics #where_clause {
            fn marshal_with_params(&self, params: &common::FieldParameters) -> Vec<u8> {
                #marshal_impl
            }
        }
    })
}

pub fn derive_enum_impl(
    name: syn::Ident,
    generics: syn::Generics,
    container: syn::DataEnum,
) -> proc_macro2::TokenStream {
    let variants = container.variants.iter().map(|v| {
        let ident = &v.ident;

        match &v.fields {
            syn::Fields::Unnamed(_) => {
                let mut _explicit = false;
                let mut tag = QuoteOption(None);
                for attr in parse_attributes(&v.attrs) {
                    match attr {
                        Asn1Attr::Explicit(_) => {
                            _explicit = true;
                        }
                        Asn1Attr::Implicit(_) => {
                            _explicit = false;
                        }
                        Asn1Attr::Tag(_, v) => {
                            tag = QuoteOption(Some(v));
                        }
                    }
                }

                quote! {
                    #name::#ident(value) => {
                        let bytes = Marshaler::marshal_with_params(
                            value,
                            &common::FieldParameters {
                                ..common::FieldParameters::default()
                            },
                        );

                        let rv = encoding_asn1::types::RawValue {
                            class: common::CLASS_CONTEXT_SPECIFIC,
                            tag: #tag.unwrap(),
                            is_compound: true,
                            bytes: bytes.to_vec(),
                            full_bytes: vec![],
                        };

                        Marshaler::marshal_with_params(
                            &rv,
                            &common::FieldParameters {
                                ..common::FieldParameters::default()
                            },
                        )
                     }
                }
            }
            _ => {
                todo!()
            }
        }
    });

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    proc_macro2::TokenStream::from(quote! {
        impl #impl_generics  Marshaler for #name #ty_generics #where_clause {
            fn marshal_with_params(&self, params: &common::FieldParameters) -> Vec<u8> {
                match self {
                    #(#variants),*
                }
            }
        }
    })
}
