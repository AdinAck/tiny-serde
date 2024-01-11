use core::panic;
use std::str::FromStr;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{Attribute, Data, DataEnum, DataStruct, DeriveInput, Field, Fields, Ident, Type};

fn serialize_struct(ident: Ident, s: DataStruct) -> TokenStream2 {
    let attrs: Vec<Ident> = s
        .fields
        .iter()
        .map(|field| field.ident.clone().unwrap())
        .collect();
    let types: Vec<Type> = s.fields.iter().map(|field| field.ty.clone()).collect();

    let mut cursors_a: Vec<TokenStream2> = types
        .iter()
        .scan(Vec::<TokenStream2>::new(), |acc, t| {
            let t = t.clone();
            acc.push(quote! { <#t as _TinySerSized>::SIZE });
            Some(quote! { {#( #acc )+*} })
        })
        .collect();
    let cursors_b = cursors_a.clone();
    cursors_a.insert(0, quote! { 0 });

    quote! {
        impl _TinySerSized for #ident {
            const SIZE: usize = {#( <#types as _TinySerSized>::SIZE )+*};
        }

        impl Serialize<{<#ident as _TinySerSized>::SIZE}> for #ident {
            fn serialize(self) -> [u8; {<#ident as _TinySerSized>::SIZE}] {
                let mut result = [0u8; {<#ident as _TinySerSized>::SIZE}];

                #(
                    let data = self.#attrs.serialize();
                    result[#cursors_a..#cursors_b].copy_from_slice(&data);
                )*

                result
            }
        }
    }
}

fn deserialize_struct(ident: Ident, s: DataStruct) -> TokenStream2 {
    let attrs: Vec<Ident> = s
        .fields
        .iter()
        .map(|field| field.ident.clone().unwrap())
        .collect();
    let types: Vec<Type> = s.fields.iter().map(|field| field.ty.clone()).collect();

    let mut cursors_a: Vec<TokenStream2> = types
        .iter()
        .scan(Vec::<TokenStream2>::new(), |acc, t| {
            let t = t.clone();
            acc.push(quote! { <#t as _TinyDeSized>::SIZE });
            Some(quote! { {#( #acc )+*} })
        })
        .collect();
    let cursors_b = cursors_a.clone();
    cursors_a.insert(0, quote! { 0 });

    quote! {
        impl _TinyDeSized for #ident {
            const SIZE: usize = {#( <#types as _TinyDeSized>::SIZE )+*};
        }

        impl Deserialize<{<#ident as _TinyDeSized>::SIZE}> for #ident {
            fn deserialize(data: [u8; {<#ident as _TinyDeSized>::SIZE}]) -> Option<Self> {
                Some(
                    Self {
                        #(
                            #attrs: #types::deserialize(data[#cursors_a..#cursors_b].try_into().unwrap())?
                        ),*
                    }
                )
            }
        }
    }
}

struct VariantTokenGroups(Vec<Ident>, Vec<TokenStream2>, Vec<Option<Field>>, Vec<Type>);

fn build_variant_token_groups(e: DataEnum) -> VariantTokenGroups {
    let mut variant_idents = Vec::new();
    let mut tags = Vec::new();
    let mut associated_fields = Vec::new();
    let mut i = 0; // count up by one starting at any known tag
    let mut last_anchor = quote! { 0 };

    for variant in e.variants.iter() {
        let ident = variant.ident.clone();

        if let Some((_, tag)) = &variant.discriminant {
            // a literal tag is provided, restart counter and update as last anchor
            let tokens = quote! { #tag };
            tags.push(tokens.clone());
            i = 0;
            last_anchor = tokens;
        } else {
            // a tag was not explicitly provided, we need to count up from last anchor
            let rendered_offset = TokenStream2::from_str(&i.to_string()).unwrap();
            tags.push(quote! { #last_anchor + #rendered_offset });
        }
        i += 1;

        match &variant.fields {
            Fields::Named(named) => {
                assert_eq!(
                    named.named.iter().len(),
                    1,
                    "Struct variants cannot hold more than one type yet."
                );

                let ty = named
                    .named
                    .iter()
                    .next()
                    .expect("Struct variant must contain at least one type.");

                associated_fields.push(Some(ty.clone()));
            }
            Fields::Unnamed(unnamed) => {
                assert_eq!(
                    unnamed.unnamed.iter().len(),
                    1,
                    "Tuple variants cannot hold more than one type yet."
                );

                let ty = unnamed
                    .unnamed
                    .iter()
                    .next()
                    .expect("Tuple variant must contain at least one type.");

                associated_fields.push(Some(ty.clone()));
            }
            Fields::Unit => {
                associated_fields.push(None);
            }
        }

        variant_idents.push(ident);
    }

    let filtered_types: Vec<Type> = associated_fields
        .iter()
        .cloned()
        .filter_map(|maybe_field| Some(maybe_field?.ty))
        .collect();

    VariantTokenGroups(variant_idents, tags, associated_fields, filtered_types)
}

fn serialize_enum(ident: Ident, repr: Type, e: DataEnum) -> TokenStream2 {
    let VariantTokenGroups(variant_idents, tags, associated_fields, filtered_types) =
        build_variant_token_groups(e);

    let ser_types = associated_fields
        .iter()
        .cloned()
        .map(|maybe_field|
            maybe_field.and_then(
                |field| {
                    let ty = field.ty;
                    if let Some(ident) = field.ident {
                        Some(quote! { result[<#repr as _TinySerSized>::SIZE..<#repr as _TinySerSized>::SIZE + <#ty as _TinySerSized>::SIZE].copy_from_slice(&#ident.serialize()); })
                    } else {
                        Some(quote! { result[<#repr as _TinySerSized>::SIZE..<#repr as _TinySerSized>::SIZE + <#ty as _TinySerSized>::SIZE].copy_from_slice(&value.serialize()); })
                    }
                }
            )
        );

    let type_captures = associated_fields.iter().cloned().map(|maybe_field| {
        maybe_field.and_then(|field| {
            if let Some(ident) = field.ident {
                Some(quote! { { #ident } })
            } else {
                Some(quote! { (value) })
            }
        })
    });

    quote! {
        impl _TinySerSized for #ident {
            const SIZE: usize = {
                let mut max = 0;

                #(
                    if <#filtered_types as _TinySerSized>::SIZE > max {
                        max = <#filtered_types as _TinySerSized>::SIZE;
                    }
                )*

                max + <#repr as _TinySerSized>::SIZE
            };
        }

        impl Serialize<{<#ident as _TinySerSized>::SIZE}> for #ident {
            fn serialize(self) -> [u8; <Self as _TinySerSized>::SIZE] {
                let mut result = [0u8; <Self as _TinySerSized>::SIZE];

                match self {
                    #(
                        Self::#variant_idents #type_captures => {
                            result[..<#repr as _TinySerSized>::SIZE].copy_from_slice(&((#tags) as #repr).serialize());
                            #ser_types
                        }
                    )*
                }

                result
            }
        }
    }
}

fn deserialize_enum(ident: Ident, repr: Type, e: DataEnum) -> TokenStream2 {
    let VariantTokenGroups(variant_idents, tags, associated_types, filtered_types) =
        build_variant_token_groups(e);

    let tag_consts: Vec<Ident> = variant_idents
        .iter()
        .map(|ident| {
            format_ident!(
                "{}_TAG",
                inflector::cases::screamingsnakecase::to_screaming_snake_case(&ident.to_string())
            )
        })
        .collect();

    let deser_types = associated_types
        .iter()
        .cloned()
        .map(|maybe_field|
            maybe_field.and_then(
                |field| {
                    let ty = field.ty;
                    if let Some(ident) = field.ident {
                        Some(quote! { {#ident: #ty::deserialize(data[<#repr as _TinyDeSized>::SIZE..<#repr as _TinyDeSized>::SIZE + <#ty as _TinyDeSized>::SIZE].try_into().unwrap())?} })
                    } else {
                        Some(quote! { (#ty::deserialize(data[<#repr as _TinyDeSized>::SIZE..<#repr as _TinyDeSized>::SIZE + <#ty as _TinyDeSized>::SIZE].try_into().unwrap())?) })
                    }
                }
            )
        );

    quote! {
        impl _TinyDeSized for #ident {
            const SIZE: usize = {
                let mut max = 0;

                #(
                    if <#filtered_types as _TinyDeSized>::SIZE > max {
                        max = <#filtered_types as _TinyDeSized>::SIZE;
                    }
                )*

                max + <#repr as _TinyDeSized>::SIZE
            };
        }

        impl Deserialize<{<#ident as _TinyDeSized>::SIZE}> for #ident {
            fn deserialize(data: [u8; <Self as _TinyDeSized>::SIZE]) -> Option<Self> {
                let tag = #repr::deserialize(data[..<#repr as _TinyDeSized>::SIZE].try_into().unwrap())?;

                #(
                    const #tag_consts: #repr = #tags;
                )*

                match tag {
                    #(
                        #tag_consts => Some(Self::#variant_idents #deser_types),
                    )*
                    _ => None
                }
            }
        }
    }
}

fn get_repr(attrs: Vec<Attribute>) -> Type {
    attrs
        .iter()
        .find(|&attr| attr.path().is_ident("repr"))
        .expect("Enum must have #[repr(...)] attribute.")
        .parse_args()
        .expect("#[repr(...) can only have one type.")
}

fn impl_serialize(body: DeriveInput) -> TokenStream2 {
    match body.data {
        Data::Struct(s) => serialize_struct(body.ident, s),
        Data::Enum(e) => {
            let ty = get_repr(body.attrs);

            serialize_enum(body.ident, ty, e)
        }
        Data::Union(_) => panic!("#[derive(Serialize)] does not support union types."),
    }
}

fn impl_deserialize(body: DeriveInput) -> TokenStream2 {
    match body.data {
        Data::Struct(s) => deserialize_struct(body.ident, s),
        Data::Enum(e) => {
            let ty = get_repr(body.attrs);

            deserialize_enum(body.ident, ty, e)
        }
        Data::Union(_) => panic!("#[derive(Serialize)] does not support union types."),
    }
}

#[proc_macro_derive(Serialize)]
pub fn serialize(input: TokenStream) -> TokenStream {
    impl_serialize(syn::parse2(input.into()).unwrap()).into()
}

#[proc_macro_derive(Deserialize)]
pub fn deserialize(input: TokenStream) -> TokenStream {
    impl_deserialize(syn::parse2(input.into()).unwrap()).into()
}
