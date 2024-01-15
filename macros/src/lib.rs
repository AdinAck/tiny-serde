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
            acc.push(quote! { <#t as Serialize>::Serialized::SIZE });
            Some(quote! { {#( #acc )+*} })
        })
        .collect();
    let cursors_b = cursors_a.clone();
    cursors_a.insert(0, quote! { 0 });

    quote! {
        impl Serialize for #ident {
            type Serialized = [u8; {#( <#types as Serialize>::Serialized::SIZE )+*}];
            type Error = ();

            fn serialize(self) -> [u8; {<#ident as Serialize>::Serialized::SIZE}] {
                let mut result = [0u8; {<#ident as Serialize>::Serialized::SIZE}];

                #(
                    let data = Serialize::serialize(self.#attrs);
                    result[#cursors_a..#cursors_b].copy_from_slice(&data);
                )*

                result
            }

            fn deserialize(data: Self::Serialized) -> Result<Self, Self::Error> {
                Ok(
                    Self {
                        #(
                            #attrs: Serialize::deserialize(data[#cursors_a..#cursors_b].try_into().unwrap())?
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
                        Some(quote! { result[<#repr as Serialize>::Serialized::SIZE..<#repr as Serialize>::Serialized::SIZE + <#ty as Serialize>::Serialized::SIZE].copy_from_slice(&Serialize::serialize(#ident)); })
                    } else {
                        Some(quote! { result[<#repr as Serialize>::Serialized::SIZE..<#repr as Serialize>::Serialized::SIZE + <#ty as Serialize>::Serialized::SIZE].copy_from_slice(&Serialize::serialize(value)); })
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

    let tag_consts: Vec<Ident> = variant_idents
        .iter()
        .map(|ident| {
            format_ident!(
                "{}_TAG",
                inflector::cases::screamingsnakecase::to_screaming_snake_case(&ident.to_string())
            )
        })
        .collect();

    let deser_types = associated_fields
        .iter()
        .cloned()
        .map(|maybe_field|
            maybe_field.and_then(
                |field| {
                    let ty = field.ty;
                    if let Some(ident) = field.ident {
                        Some(quote! { {#ident: Serialize::deserialize(data[<#repr as Serialize>::Serialized::SIZE..<#repr as Serialize>::Serialized::SIZE + <#ty as Serialize>::Serialized::SIZE].try_into().unwrap())?} })
                    } else {
                        Some(quote! { (Serialize::deserialize(data[<#repr as Serialize>::Serialized::SIZE..<#repr as Serialize>::Serialized::SIZE + <#ty as Serialize>::Serialized::SIZE].try_into().unwrap())?) })
                    }
                }
            )
        );

    let size = if filtered_types.len() > 0 {
        quote! {{
            let mut max = 0;

            #(
                if <#filtered_types as Serialize>::Serialized::SIZE > max {
                    max = <#filtered_types as Serialize>::Serialized::SIZE;
                }
            )*

            max + <#repr as Serialize>::Serialized::SIZE
        }}
    } else {
        quote! {
            <#repr as Serialize>::Serialized::SIZE
        }
    };

    quote! {
        impl Serialize for #ident {
            type Serialized = [u8; #size];
            type Error = ();

            fn serialize(self) -> Self::Serialized {
                let mut result = Self::Serialized::default();

                match self {
                    #(
                        Self::#variant_idents #type_captures => {
                            result[..<#repr as Serialize>::Serialized::SIZE].copy_from_slice(&Serialize::serialize(#tags as #repr));
                            #ser_types
                        }
                    )*
                }

                result
            }

            fn deserialize(data: Self::Serialized) -> Result<Self, Self::Error> {
                let tag = Serialize::deserialize(data[..<#repr as Serialize>::Serialized::SIZE].try_into().unwrap())?;

                #(
                    const #tag_consts: #repr = #tags;
                )*

                match tag {
                    #(
                        #tag_consts => Ok(Self::#variant_idents #deser_types),
                    )*
                    _ => Err(())
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

#[proc_macro_derive(Serialize)]
pub fn serialize(input: TokenStream) -> TokenStream {
    let body: DeriveInput = syn::parse2(input.into()).unwrap();

    match body.data {
        Data::Struct(s) => serialize_struct(body.ident, s),
        Data::Enum(e) => {
            let ty = get_repr(body.attrs);

            serialize_enum(body.ident, ty, e)
        }
        Data::Union(_) => panic!("#[derive(Serialize)] does not support union types."),
    }
    .into()
}
