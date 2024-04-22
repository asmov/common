use quote::{self, ToTokens};
use syn;
use proc_macro2;

use crate::{
    model, model::parse,
    synerr, mksynerr, error::{span, span_site},
    ENUM_ATTRIBUTE_HELPER_NAME};

#[derive(Debug)]
pub(crate) struct TraitEnumMacroOutput {
    pub(crate) tokens: proc_macro2::TokenStream,
    pub(crate) model: model::TraitEnum
}

pub fn traitenum_derive_macro(
    item: proc_macro2::TokenStream,
    model_bytes: &[u8]) -> Result<proc_macro2::TokenStream, syn::Error>
{
    let TraitEnumMacroOutput { tokens, model: _model } = parse_traitenum_macro(item, model_bytes)?;
    Ok(tokens)
}
 
pub(crate) fn parse_traitenum_macro(
    item: proc_macro2::TokenStream,
    enumtrait_model_bytes: &[u8]
) -> Result<TraitEnumMacroOutput, syn::Error> {
    let enumtrait = model::EnumTrait::deserialize(enumtrait_model_bytes).unwrap();
    let input: syn::DeriveInput = syn::parse2(item)?;
    let trait_ident = syn::Ident::new(enumtrait.identifier().name(), span(&input));

    // the actual parsing is done with this call, the rest is building a tokenstream
    let traitenum = parse_traitenum_model(&input, &enumtrait)?;

    let data_enum = data_enum(&input)?;
    // write a method for each one defined by the enum trait, which returns the value defined by each enum variant
    let method_outputs = enumtrait.methods().iter().map(|method| {
        let method_name = method.name();
        let func: syn::Ident = syn::Ident::new(method_name, span(&input));
        let return_type = method.return_type_tokens();

        match method.attribute_definition() {
            model::Definition::Relation(reldef) => {
                let rel_id = traitenum.relation_enum_identifier(method_name).unwrap();
                let relation_path: syn::Path = rel_id.into();
                let dispatch = reldef.dispatch().unwrap();
                
                match reldef.nature.unwrap() {
                    model::RelationNature::OneToMany => {
                        match dispatch { 
                            model::Dispatch::BoxedTrait => {
                                let iterator_ident = syn::Ident::new(
                                    &format!("{}{}", rel_id.name(), IDENT_BOXED_ITERATOR), span(&input));
                                
                                return quote::quote!{
                                    fn #func(&self) -> #return_type {
                                        ::std::boxed::Box::new(#iterator_ident::new())
                                    }
                                }
                            },
                            model::Dispatch::Other => unimplemented!("Dispatch::Other is permanently unimplemented")
                        }
                    },
                    model::RelationNature::ManyToOne | model::RelationNature::OneToOne => {
                        match dispatch { 
                            model::Dispatch::BoxedTrait => return quote::quote!{
                                fn #func(&self) -> #return_type {
                                    ::std::boxed::Box::new(#relation_path)
                                }
                            },
                            model::Dispatch::Other => unimplemented!("Dispatch::Other is permanently unimplemented")
                        }
                    }
                }
            },
            _ => {}
        }

        // create the match{} body of the method, mapping variants to their return value
        let variant_outputs = data_enum.variants.iter().map(|variant_data| {
            let variant_ident = &variant_data.ident;
            let variant_name = variant_ident.to_string();
            let value = traitenum
                .variant(&variant_name).unwrap()
                .value(method_name).unwrap()            
                .to_token_stream();

            quote::quote!{
                Self::#variant_ident => #value,
            }
        });

        // the final method signature and body
        let output = quote::quote!{
            fn #func(&self) -> #return_type {
                match self {
                    #(#variant_outputs)*
                }
            }
        };

        output
    });

    let boxed_trait_relation_iterators_outputs = build_boxed_trait_relation_iterators(&enumtrait, &traitenum)?;

    let input_ident = &input.ident;

    let output = quote::quote!{
        impl #trait_ident for #input_ident {
            #(#method_outputs)*
        }

        #(#boxed_trait_relation_iterators_outputs)*
    };

    Ok(TraitEnumMacroOutput {
        tokens: output,
        model: traitenum
    })
}

fn data_enum(input: &syn::DeriveInput) -> Result<&syn::DataEnum, syn::Error> {
    match input.data {
        syn::Data::Enum(ref data_enum) => Ok(data_enum),
        _ => synerr!(input, "Only enums are supported for #[{}]", ENUM_ATTRIBUTE_HELPER_NAME)
    }
}

fn parse_traitenum_model(input: &syn::DeriveInput, enumtrait: &model::EnumTrait)
        -> Result<model::TraitEnum, syn::Error> {
    let mut traitenum_build = model::TraitEnumBuilder::new();
    traitenum_build.identifier(model::Identifier::from(&input.ident));

    //parse top-level attributes (item.attr) as relations -> #[traitenum(<relation name>(<trait path>))]
    for attr in &input.attrs {
        attr.parse_nested_meta(|meta| {
            // this will be the method and relation name as well
            let attr_name = meta.path.get_ident()
                .ok_or_else(|| {
                    mksynerr!(attr, "Invalid traitenum attribute")
                })?
                .to_string();

            // prevent duplicates
            if traitenum_build.has_relation_enum(&attr_name) {
                synerr!(attr, "Duplicate traitenum attribute for enum: {}", attr_name);
            }

            // find the matching trait method by name
            let attribute_definition = enumtrait.methods().iter()
                .find(|m| { m.name() == attr_name })
                .ok_or_else(|| mksynerr!(attr, "No matching trait method for enum attribute: {}", attr_name))?
                .attribute_definition();

            // ensure that we're using a relation attribute definition for this method
            match attribute_definition {
                model::Definition::Relation(_) => (),
                _ => synerr!(attr, "Trait method definition is not a Relation as expected for enum attribute: {}", attr_name)
            }

            let content;
            syn::parenthesized!(content in meta.input);
            let relation_path: syn::Path = content.parse()?;
            traitenum_build.relation_enum(attr_name, relation_path.try_into().unwrap());

            Ok(())
        })?;
    }


    // parse enum attribute values, if provided
    let data_enum = data_enum(input)?;
    let mut ordinal: usize = 0;
    for variant in &data_enum.variants {
        let variant_name = variant.ident.to_string();
        // find the #[traitenum] attribute or continue
        let attribute = variant.attrs.iter()
            .find(|a| a.path().segments.first()
                .is_some_and(|s| ENUM_ATTRIBUTE_HELPER_NAME == s.ident.to_string()));

        let mut variant_build = if let Some(attribute) = attribute {
            parse::parse_variant(&variant_name, attribute, &enumtrait)?
        } else {
            let mut build = model::VariantBuilder::new();
            build.name(variant_name.to_owned());
            build
        };

        // set attribute value defaults. throw errors where values are required, but not provided
        for method in enumtrait.methods() {
            let method_name = method.name();
            let definition = method.attribute_definition();
            if variant_build.has_value(method_name) {
                continue;
            } else if !definition.needs_value() {
                continue;
            } else if !definition.has_default_or_preset() {
                synerr!(variant, "Missing value for attribute `{}`: {}", method_name, variant_name);
            } else {
                let value = definition.default_or_preset(&variant_name, ordinal).unwrap();
                variant_build.value(method_name.to_string(), model::AttributeValue::new(value));
            }
        }

        // if this was a Rel attribute that needs a value, we create a relation_enum for it, as it wasn't
        // processed at the top of the enum (it's a one-to-many)
        for method in enumtrait.methods() {
            match method.attribute_definition() {
                model::Definition::Relation(reldef) => {
                    match reldef.nature {
                        Some(model::RelationNature::OneToMany) => {
                            let method_name = method.name();
                            let attr_value = variant_build.get_value(&method_name).unwrap();
                            if let model::Value::Relation(id) = attr_value.value() {
                                traitenum_build.relation_enum(method_name.to_owned(), id.to_owned());
                            } else {
                                unreachable!();
                            }
                        },
                        Some(_) => (),
                        None => unreachable!(),
                    }
                },
                _ => ()
            } 
        }

        traitenum_build.variant(variant_build.build());
        ordinal += 1;
    }

    Ok(traitenum_build.build())
}

const IDENT_BOXED_ITERATOR: &'static str = "BoxedIterator";

// Creates iterator structs and implementations for dynamically dispatched many-to-many relations
fn build_boxed_trait_relation_iterators(
    enumtrait: &model::EnumTrait,
    traitenum: &model::TraitEnum) -> syn::Result<Vec<proc_macro2::TokenStream>>
{
    let structs = enumtrait.relation_methods().iter()
        .filter(|(_, rel)| rel.dispatch().unwrap() == model::Dispatch::BoxedTrait)
        .filter(|(_, rel)| rel.nature().unwrap() == model::RelationNature::ManyToOne)
        .map(|(_method, _relation_def)| {
            // The name of the iterator struct. E.g., MyEnumBoxedIterator
            let iterator_ident = syn::Ident::new(
                &format!("{}{}", traitenum.identifier().name(), IDENT_BOXED_ITERATOR), span_site());
                
            let item_path: syn::Path = traitenum.identifier().try_into().unwrap();
            let item_trait_path: syn::Path = enumtrait.identifier().try_into().unwrap();

            // Build the match body for the Iterator's next(). This simply maps a traitenum variant by its ordinal.
            let mut ordinal: usize = 0;
            let next_ordinal_match_body = traitenum.variants().iter().map(|variant| {
                let variant_ident = syn::Ident::new(variant.name(), span_site());
                let output = quote::quote!{
                    #ordinal => ::std::option::Option::Some(Box::new(#item_path::#variant_ident)),
                };

                ordinal += 1;
                output
            });

            // Build the Iterator struct, it's new function, and it's Iterator implementation for the traitenum.
            quote::quote!{
                struct #iterator_ident {
                    next_ordinal: usize
                }

                impl #iterator_ident {
                    fn new() -> Self {
                        Self {
                            next_ordinal: 0
                        }
                    }
                }

                impl ::std::iter::Iterator for #iterator_ident {
                    type Item = ::std::boxed::Box<dyn #item_trait_path>;

                    fn next(&mut self) -> std::option::Option<Self::Item> {
                        let ordinal = self.next_ordinal;
                        self.next_ordinal += 1;
                        match ordinal {
                            #(#next_ordinal_match_body)*
                            _ => ::std::option::Option::None
                        }
                    }
                }
            }
        })
        .collect();

    Ok(structs)
}