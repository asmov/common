use quote::{self, ToTokens};
use syn::{self, spanned::Spanned};
use proc_macro2;
use convert_case::{self as case, Casing};

use crate::{
    model, macros, model::parse,
    synerr, mksynerr,
    TRAIT_ATTRIBUTE_HELPER_NAME, ENUM_ATTRIBUTE_HELPER_NAME };

const IDENT_BOX: &'static str = "Box";
const IDENT_ITERATOR: &'static str = "Iterator";
const IDENT_ITEM: &'static str = "Item";
const IDENT_SELF: &'static str = "Self";

#[derive(Debug)]
pub(crate) struct EnumTraitMacroOutput {
    pub(crate) tokens: proc_macro2::TokenStream,
    pub(crate) model: model::EnumTrait
}

pub fn enumtrait_macro(attr: proc_macro2::TokenStream, item: proc_macro2::TokenStream)
        -> Result<proc_macro2::TokenStream, syn::Error> {
    let EnumTraitMacroOutput {tokens, model} = parse_enumtrait_macro(attr, item)?;
    let model_name = syn::Ident::new(
        &format!("{}{}", macros::MODEL_BYTES_NAME, model.identifier().name().to_case(case::Case::ScreamingSnake)),
        proc_macro2::Span::call_site());

    let bytes = model.serialize().unwrap();
    let bytes_len = bytes.len();
    let bytes_literal = syn::LitByteStr::new(&bytes, tokens.span());

    let output = quote::quote!{
        pub const #model_name: &'static [u8; #bytes_len] = #bytes_literal;

        #tokens
    };

    Ok(output)
}

pub(crate) fn parse_enumtrait_macro(
    attr: proc_macro2::TokenStream,
    item: proc_macro2::TokenStream) -> syn::Result<EnumTraitMacroOutput>
{
    if !attr.is_empty() {
        synerr!(attr, "Top-level #[enumtrait] does not accept arguments");
    }

    let mut trait_input: syn::ItemTrait = syn::parse2(item)?;
    let identifier = model::Identifier::new(vec![], trait_input.ident.to_string());

    let mut methods: Vec<model::Method> = Vec::new(); 

    // We only support trait methods. Everything else is either ignored or denied
    for trait_item in &trait_input.items {
        match trait_item {
            // Build a model Method
            syn::TraitItem::Fn(func) => parse_trait_fn(&mut methods, func)?,
            syn::TraitItem::Type(t) => synerr!(t, "Associated types are not supported"),
            _ => ()
        }
    }


    // Remove all #[tratienum] attributes from the TokenStream now that we're done parsing them
    clean_helper_attributes(&mut trait_input)?;

    Ok(EnumTraitMacroOutput {
        tokens: trait_input.to_token_stream(),
        model: model::EnumTrait::new(identifier, methods)
    })
}

fn parse_trait_fn(methods: &mut Vec<model::Method>, func: &syn::TraitItemFn) -> syn::Result<()> {
    // ignore functions with default implementations
    if func.default.is_some() {
        return Ok(());
    }

    let method_name = func.sig.ident.to_string();
    let (return_type, return_type_identifier) = parse_trait_fn_return(func)?;

    // Throw an error if the the wrong helper (enum vs trait) was used
    // 1. search for the wrong name ...
    let attrib = func.attrs.iter().find(|attrib| {
        attrib.path().segments.first().is_some_and(|s| ENUM_ATTRIBUTE_HELPER_NAME == s.ident.to_string())
    });

    // 2. throw the error
    if attrib.is_some() {
        synerr!(attrib.unwrap(),
            "Wrong attribute helper was used for trait: `#[{}]`. Please use for `#[{}]` traits.",
            ENUM_ATTRIBUTE_HELPER_NAME, TRAIT_ATTRIBUTE_HELPER_NAME);
    }

    // We expect a helper attribute that defines each trait method.
    // E.g., #[traitenum::Str(preset(Variant))
    //     Where "::Str" refers to model::StrAttributeDefinition(def) and it's associated definition property "preset"
    // 1. match against the 'traitenum' path segment to get started.
    let attrib = func.attrs.iter().find(|attrib| {
        attrib.path().segments.first().is_some_and(|s| TRAIT_ATTRIBUTE_HELPER_NAME == s.ident.to_string())
    });

    // Parse the attribute definition that is found. If not found, attempt to build a default based on method signature.
    let attribute_def = if let Some(attrib) = attrib {
        parse::parse_definition(attrib, return_type, return_type_identifier)?
    } else {
        model::Definition::partial(None, return_type, return_type_identifier)
            .map_err(|e| {
                mksynerr!(&func.sig,
                    "Unable to parse definition from return signature for `{}` :: {}",
                    method_name, e)
            })?
    };

    // Now perform a validation pass on all attribute definitions to enforce each def's specific rules
    if let Err(errmsg) = attribute_def.validate() {
        synerr!(attrib, "{}", errmsg);
    }

    let method = model::Method::new(method_name, return_type, attribute_def);
    methods.push(method);

    Ok(())
}

fn parse_trait_fn_return(func: &syn::TraitItemFn) -> syn::Result<(model::ReturnType, Option<model::Identifier>)> {
    let mut return_type: Option<model::ReturnType> = None;
    let mut return_type_identifier: Option<model::Identifier> = None;

    match &func.sig.output {
        syn::ReturnType::Default => synerr!(&func.sig, "Default return types () are not supported"),
        syn::ReturnType::Type(_, ref returntype) => match **returntype {
            syn::Type::Path(ref path_type) => {
                if let Ok(ret_type) = model::ReturnType::try_from(&path_type.path) {
                    // This models primitive return types that ReturnType supports. E.g., usize, f32, bool, etc.
                    return_type = Some(ret_type);
                } else if let Some((ret_type, ret_type_id)) = try_parse_trait_fn_return_boxed(path_type)? {
                    // This models ReturnType::BoxedTrait. E.g.: Box<dyn MyTrait>
                    // The trait is modeled as a model::Identifier.
                    return_type = Some(ret_type);
                    return_type_identifier = Some(ret_type_id);
                } else if path_type.path.segments[0].ident == IDENT_SELF {
                    return_type = Some(model::ReturnType::AssociatedType);
                    return_type_identifier = match model::Identifier::try_from(&path_type.path) {
                        Ok(id) => Some(id),
                        Err(_) => {
                            synerr!(return_type,
                                "Unsupported return type: {}",
                                &path_type.path.to_token_stream().to_string())
                        }
                    }
                } else {
                    // Anything else is modeled as ReturnType::Type, including enums.
                    return_type = Some(model::ReturnType::Type);
                    return_type_identifier = match model::Identifier::try_from(&path_type.path) {
                        Ok(id) => Some(id),
                        Err(_) => {
                            synerr!(return_type,
                                "Unsupported return type: {}",
                                &path_type.path.to_token_stream().to_string())
                        }
                    }
                }
            },
            // As for reference return types, we only support &'static str at the moment.
            syn::Type::Reference(ref ref_type) => {
                // only elided and static lifetimes are supported
                let _has_static_lifetime = match &ref_type.lifetime {
                    Some(lifetime) => {
                        if "static" == lifetime.ident.to_string() {
                            true
                        } else {
                            synerr!(ref_type, "Only elided and static lifetimes are supported for return types")
                        }
                    },
                    None => false
                };

                // mutability is not supported
                if ref_type.mutability.is_some() {
                    synerr!(ref_type, "Mutable return types are not supported");
                }

                // basically just ensure that the ident is a "str"
                if let syn::Type::Path(ref path_type) = *ref_type.elem {
                    if let Some(ident) = path_type.path.get_ident() {
                        if "str" == ident.to_string() {
                            return_type = Some(model::ReturnType::StaticStr);
                        }
                    }
                }

                // ... the else statement for each nested if statement above
                if return_type.is_none() {
                    synerr!(ref_type, "Unsupported return reference type")
                }
            },
            _ => synerr!(return_type, "Unimplemented trait return type"),
        }
    }

    let return_type = return_type.expect("Unable to parse return type");
    Ok((return_type, return_type_identifier))
}

fn parse_box_trait_bound(path: &syn::Path) -> syn::Result<&syn::TraitBound> {
    if path.segments[0].ident != IDENT_BOX {
        synerr!(path, "Expected a Box<..>") 
    }

    // -> Box< dyn Trait >
    //       ^~~~~~~~~~~~^
    let bracket_args = match &path.segments[0].arguments {
        syn::PathArguments::AngleBracketed(v) => v,
        _ => synerr!(&path.segments[0], "Unsupported use of Box")
    };

    let arg_type = match &bracket_args.args[0] {
        syn::GenericArgument::Type(v) => v,
        _ => synerr!(&bracket_args.args[0], "Invalid use of Box")
    };

    // -> Box< dyn Trait >
    //         ^~~~~~~~^
    let trait_obj = match arg_type {
        syn::Type::TraitObject(v) => v,
        _ => synerr!(&arg_type, "Unsupported use of Box. Only <dyn EnumTrait> arguments allowed")
    };

    let trait_bound = match trait_obj.bounds[0] {
        syn::TypeParamBound::Trait(ref v) => v,
        _ => synerr!(&trait_obj.bounds[0], "Unsupported use of Box. Only <dyn EnumTrait> arguments allowed")
    };

    Ok(trait_bound)
}

fn try_parse_trait_fn_return_boxed(
    type_path: &syn::TypePath) -> syn::Result<Option<(model::ReturnType, model::Identifier)>>
{
    if type_path.path.segments[0].ident != IDENT_BOX {
        return Ok(None)
    }

    let trait_bound = parse_box_trait_bound(&type_path.path)?;

    if trait_bound.path.segments[0].ident == IDENT_ITERATOR {
        try_parse_trait_fn_return_boxed_trait_iterator(trait_bound)
    } else {
        let id = model::Identifier::try_from(&trait_bound.path)
            .map_err(|_| mksynerr!(&trait_bound.path, "Unable to parse Boxed trait identifier"))?;

        Ok(Some((model::ReturnType::BoxedTrait, id)))
    }
}

fn try_parse_trait_fn_return_boxed_trait_iterator(
    trait_bound: &syn::TraitBound) -> syn::Result<Option<(model::ReturnType, model::Identifier)>>
{
    // -> Box<dyn Iterator<Item = Box<dyn Trait>>>
    //                    ^~~~~~~~~~~~~~~~~~~~~~^ 
    let bracket_args = match &trait_bound.path.segments[0].arguments {
        syn::PathArguments::AngleBracketed(v) => v,
        _ => synerr!(&trait_bound.path.segments[0], "Invalid use of Box<dyn Iterator>")
    };

    let assoc_type = match &bracket_args.args[0] {
        syn::GenericArgument::AssocType(v) => v,
        _ => synerr!(&bracket_args.args[0], "Invalid use of Box<dyn Iterator>")
    };

    if assoc_type.ident != IDENT_ITEM {
        synerr!(assoc_type, "Invalid use of Box<dyn Iterator>");
    }

    // -> Box<dyn Iterator<Item = Box<dyn Trait>>>
    //                            ^~~~~~~~~~~~~^
    let item_trait_path = match assoc_type.ty {
        syn::Type::Path(ref v) => &v.path,
        _ => synerr!(&assoc_type.ty, "Invalid use of Box<dyn Iterator>")
    };

    let item_box_trait_bound = parse_box_trait_bound(item_trait_path)?;
    
    let id = model::Identifier::try_from(&item_box_trait_bound.path)
        .map_err(|_| {
            mksynerr!(&item_box_trait_bound.path,
                "Unable to parse Boxed trait iterator identifier")
        })?;

    Ok(Some((model::ReturnType::BoxedTraitIterator, id)))
}


fn clean_helper_attributes(trait_input: &mut syn::ItemTrait) -> syn::Result<()> {
    // strip out all #enumtrait helper attributes
    for trait_item in &mut trait_input.items {
        match trait_item {
            syn::TraitItem::Fn(func) => {
                let mut count = 0;  
                func.attrs.retain(|attrib| {
                    if attrib.path().segments.first()
                            .is_some_and(|s| TRAIT_ATTRIBUTE_HELPER_NAME == s.ident.to_string()) {
                        count += 1;
                        false
                    } else {
                        true
                    }
                });

                // we only process one attribute helper per method. curtail expectations with an error.
                if count > 1 {
                    synerr!(trait_input, "Only one #traitenum helper attribute per method is supported");
                }
            
            },
            _ => ()
        }
    }

    Ok(())
}

