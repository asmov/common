use std::str::FromStr;
use quote::{self, TokenStreamExt, ToTokens};
use syn::{self, parse, meta::ParseNestedMeta};

use crate::{model, error::Errors, synerr, mksynerr, error::span_site, TRAIT_ATTRIBUTE_HELPER_NAME};

use super::{BoolDefinition, FieldlessEnumDefinition, NumberDefinition};

impl parse::Parse for model::Identifier {
    fn parse(input: parse::ParseStream) -> syn::Result<Self> {
        let path: syn::Path = input.parse()?;
        Self::try_from(&path)
            .map_err(|_| {
                mksynerr!(&path, "Unable to parse Identifier from Path: {}",
                    path.to_token_stream().to_string())
            })
    }
}

impl From<&syn::Ident> for model::Identifier{
    fn from(ident: &syn::Ident) -> Self {
        model::Identifier::new(Vec::new(), ident.to_string())
    }
}

impl TryFrom<&syn::Path> for model::Identifier {
    type Error = &'static str;

    fn try_from(path: &syn::Path) -> Result<Self, Self::Error> {
        Self::try_from(path.clone())
    }
}

impl TryFrom<syn::Path> for model::Identifier{
    type Error = &'static str;

    fn try_from(mut path: syn::Path) -> Result<Self, Self::Error> {
        let name = path.segments.pop().unwrap()
            .value().ident.to_string();
        let path = path.segments.pairs()
            .map(|pair| {
                if !pair.value().arguments.is_empty() {
                    Err("Path contains arguments")
                } else {
                    Ok(pair.value().ident.to_string())
                }
            })
            .collect::<Result<Vec<String>, Self::Error>>()?;

        Ok(Self::new(path, name))
    }
}


impl TryFrom<&syn::Path> for model::ReturnType {
    type Error = ();

    fn try_from(path: &syn::Path) -> Result<Self, Self::Error> {
        match path.get_ident() {
            Some(v) => model::ReturnType::from_str(&v.to_string()),
            None => Err(()) 
        }
    }
}

pub(crate) fn parse_definition(
        attr: &syn::Attribute,
        return_type: model::ReturnType,
        return_type_id: Option<model::Identifier>
    ) -> Result<model::Definition, syn::Error> {
    if attr.path().segments.len() != 2 {
        synerr!(attr.path(), "Unable to parse helper attribute: `{}`. Format: {}::DefinitionName",
            TRAIT_ATTRIBUTE_HELPER_NAME,
            attr.path().to_token_stream().to_string())
    }

    let definition_type_name = attr.path().segments.last()
        .ok_or_else(|| {
            mksynerr!(attr,
                "Empty helper attribute definition name. Format: {}::DefinitionName",
                TRAIT_ATTRIBUTE_HELPER_NAME)
        })?
        .ident
        .to_string();

    let mut def = model::Definition::partial(Some(&definition_type_name), return_type, return_type_id)
        .map_err(|_| mksynerr!(attr, "Unable to parse return type for definition"))?;

    attr.parse_nested_meta(|meta| {
        let content;
        syn::parenthesized!(content in meta.input);

        match definition_type_name.as_str() {
            BoolDefinitionParser::NAME => BoolDefinitionParser::parse_definition(&mut def, &meta, content, return_type)?,
            StrDefinitionParser::NAME => StrDefinitionParser::parse_definition(&mut def, &meta, content, return_type)?,
            NumDefinitionParser::NAME => NumDefinitionParser::parse_definition(&mut def, &meta, content, return_type)?,
            EnumDefinitionParser::NAME => EnumDefinitionParser::parse_definition(&mut def, &meta, content, return_type)?,
            RelDefinitionParser::NAME => RelDefinitionParser::parse_definition(&mut def, &meta, content, return_type)?,
             _ => synerr!(meta.path, "Unknown definition type: {}", definition_type_name)
        };

        Ok(())
    })?;

    Ok(def)
}

trait DefinitionParser {
    const DEFINITION_DEFAULT: &'static str = "default";
    const DEFINITION_PRESET: &'static str = "preset";
    const NAME: &'static str;

    fn parse_definition(
        def: &mut model::Definition,
        meta: &ParseNestedMeta,
        content: syn::parse::ParseBuffer,
        return_type: model::ReturnType
    ) -> syn::Result<()>;

    fn parse_setting_name(meta: &ParseNestedMeta) -> syn::Result<String> {
        Ok(meta.path.get_ident()
            .ok_or_else(|| {
                mksynerr!(&meta.path,
                    "Expected {} definition setting name",
                    Self::NAME)
            })?
            .to_string())
    }

    fn err_unknown_setting<T>(meta_path: impl quote::ToTokens, setting_name: String) -> syn::Result<T> {
        Errors::UnknownDefinitionSetting {
            definition: Self::NAME.to_owned(),
            setting: setting_name
        }.to_syn_err(meta_path)
    }
}

macro_rules! bind_def {
    ($variant:path, $def:ident, $setting_name:ident) => {
        match $def {
            $variant(d) => d,
            _ => unreachable!("Mismatch $variant definition for setting `{}`", $setting_name)
        }
    };
}

struct BoolDefinitionParser{}

impl DefinitionParser for BoolDefinitionParser {
    const NAME: &'static str = BoolDefinition::TYPE_NAME;

    fn parse_definition(
        def: &mut model::Definition,
        meta: &ParseNestedMeta,
        content: syn::parse::ParseBuffer,
        _return_type: model::ReturnType
    ) -> syn::Result<()> {
        let setting_name = Self::parse_setting_name(meta)?;
        let booldef = bind_def!(model::Definition::Bool, def, setting_name);

        match setting_name.as_str() {
            Self::DEFINITION_DEFAULT => {
                    booldef.default = Some(content.parse::<syn::LitBool>()?.value())
            },
            _ => return Self::err_unknown_setting(&meta.path, setting_name)
        }

        Ok(())
    }
}

struct EnumDefinitionParser{}

impl DefinitionParser for EnumDefinitionParser {
    const NAME: &'static str = FieldlessEnumDefinition::TYPE_NAME;

    fn parse_definition(
        def: &mut model::Definition,
        meta: &ParseNestedMeta,
        content: syn::parse::ParseBuffer,
        _return_type: model::ReturnType
    ) -> syn::Result<()> {
        let setting_name = Self::parse_setting_name(meta)?;
        let enumdef = bind_def!(model::Definition::FieldlessEnum, def, setting_name);

        match setting_name.as_str() {
            Self::DEFINITION_DEFAULT => {
                let id: model::Identifier = content.parse()?;
                enumdef.default = Some(id)
            },
            _ => return Self::err_unknown_setting(&meta.path, setting_name) 
        }

        Ok(())
    }
}

struct StrDefinitionParser{}

impl DefinitionParser for StrDefinitionParser {
    const NAME: &'static str = model::StaticStrDefinition::DEFINITION_NAME;

    fn parse_definition(
        def: &mut model::Definition,
        meta: &ParseNestedMeta,
        content: syn::parse::ParseBuffer,
        _return_type: model::ReturnType
    ) -> syn::Result<()> {
        let setting_name = Self::parse_setting_name(meta)?;
        let strdef = bind_def!(model::Definition::StaticStr, def, setting_name);

        match setting_name.as_str() {
            Self::DEFINITION_DEFAULT => {
                    strdef.default = Some(content.parse::<syn::LitStr>()?.value())
            },
            Self::DEFINITION_PRESET => {
                    let variant_ident = content.parse::<syn::Ident>()?;
                    let variant_name = variant_ident.to_string();
                    let preset = model::StringPreset::from_str(&variant_name)
                        .map_err(|_| {
                            mksynerr!(&meta.path, "Unknown String preset: {}", variant_name)
                        })?;
                    strdef.preset = Some(preset);
            },
            _ => return Self::err_unknown_setting(&meta.path, setting_name)
        }

        Ok(())
    }
}
struct RelDefinitionParser{}

impl RelDefinitionParser {
    const DEFINITION_NATURE: &'static str = "nature";
    const DEFINITION_DISPATCH: &'static str = "dispatch";
}

impl DefinitionParser for RelDefinitionParser {
    const NAME: &'static str = model::RelationDefinition::TYPE_NAME;

    fn parse_definition(
        def: &mut model::Definition,
        meta: &ParseNestedMeta,
        content: syn::parse::ParseBuffer,
        _return_type: model::ReturnType
    ) -> syn::Result<()> {
        let setting_name = Self::parse_setting_name(meta)?;
        let reldef = bind_def!(model::Definition::Relation, def, setting_name);

        match setting_name.as_str() {
            Self::DEFINITION_NATURE => {
                let variant_ident = content.parse::<syn::Ident>()?;
                let variant_name = variant_ident.to_string();
                let relationship = model::RelationNature::from_str(&variant_name)
                    .map_err(|_| mksynerr!(variant_ident, "Unknown nature: {}", variant_name) )?;
                reldef.nature = Some(relationship);
            },
            Self::DEFINITION_DISPATCH => {
                let variant_ident = content.parse::<syn::Ident>()?;
                let variant_name = variant_ident.to_string();
                let dispatch = model::Dispatch::from_str(&variant_name)
                    .map_err(|_| mksynerr!(variant_ident, "Unknown dispatch: {}", variant_name) )?;
                reldef.dispatch = Some(dispatch);
            },
            _ => return Self::err_unknown_setting(&meta.path, setting_name)
        }

        Ok(())
    }
}

struct NumDefinitionParser{}

impl NumDefinitionParser {
    const DEFINITION_START: &'static str = "start";
    const DEFINITION_INCREMENT: &'static str = "increment";

    fn parse_number_definition<N>(
            def: &mut model::NumberDefinition<N>,
            meta: &ParseNestedMeta,
            setting_name: &str,
            content: syn::parse::ParseBuffer,
            _return_type: model::ReturnType,
            is_float: bool) -> Result<(), syn::Error>
    where
        N: FromStr,
        N::Err: std::fmt::Display
    {
        macro_rules! parsenum {
            () => {
                if is_float {
                        content.parse::<syn::LitFloat>()?.base10_parse()?
                } else {
                        content.parse::<syn::LitInt>()?.base10_parse()?
                } 
            };
        }

        match setting_name {
            Self::DEFINITION_DEFAULT => {
                    let n: N = parsenum!();
                    def.default = Some(n)
            },
            Self::DEFINITION_PRESET => {
                    let variant_ident = content.parse::<syn::Ident>()?;
                    let variant_name = variant_ident.to_string();
                    let preset = model::NumberPreset::from_str(&variant_name)
                        .map_err(|_| {
                            mksynerr!(variant_ident, "Unknown definition preset for Num: {}", variant_name)
                        })?;
                    def.preset = Some(preset);
            },
            Self::DEFINITION_START => {
                    let n: N = parsenum!();
                    def.start = Some(n)
            },
            Self::DEFINITION_INCREMENT => {
                    let n: N = parsenum!();
                    def.increment = Some(n)
            },
            _ => return Self::err_unknown_setting(&meta.path, setting_name.to_owned())
        }

        Ok(())
    }
}

impl DefinitionParser for NumDefinitionParser {
    const NAME: &'static str = NumberDefinition::<usize>::DEFINITION_NAME;

    fn parse_definition(
        def: &mut model::Definition,
        meta: &ParseNestedMeta,
        content: syn::parse::ParseBuffer,
        return_type: model::ReturnType
    ) -> syn::Result<()> {
        let setting_name = Self::parse_setting_name(meta)?;
        match def {
            model::Definition::UnsignedSize(def) => Self::parse_number_definition(def, meta, &setting_name, content, return_type, false),
            model::Definition::UnsignedInteger64(def) => Self::parse_number_definition(def, meta, &setting_name, content, return_type, false),
            model::Definition::Integer64(def) => Self::parse_number_definition(def, meta, &setting_name, content, return_type, false),
            model::Definition::Float64(def) => Self::parse_number_definition(def, meta, &setting_name, content, return_type, true),
            model::Definition::UnsignedInteger32(def) => Self::parse_number_definition(def, meta, &setting_name, content, return_type, false),
            model::Definition::Integer32(def) => Self::parse_number_definition(def, meta, &setting_name, content, return_type, true),
            model::Definition::Float32(def) => Self::parse_number_definition(def, meta, &setting_name, content, return_type, false),
            _ => unreachable!("Unexpected Num definition associated data for setting: {}", setting_name)
        }
    }
}


pub(crate) fn parse_variant(variant_name: &str, attr: &syn::Attribute, model: &model::EnumTrait)
        -> Result<model::VariantBuilder, syn::Error> {
    let mut variant_build = model::VariantBuilder::new();
    variant_build.name(variant_name.to_owned());
    attr.parse_nested_meta(|meta| {
        let attr_name = meta.path.get_ident()
            .ok_or_else(|| {
                mksynerr!(&meta.path, "Invalid enum attribute: `{}`", meta.path.to_token_stream().to_string())
            })?
            .to_string();

        if variant_build.has_value(&attr_name) {
            synerr!(&meta.path, "Duplicate enum attribute value for: {}", attr_name);
        }

        let method = model.methods().iter().find(|m| m.name() == attr_name)
            .ok_or_else(|| {
                mksynerr!(&meta.path, "Unknown enum attribute: {}", attr_name)
            })?;

        let attribute_def = &method.attribute_definition();

        let content;
        syn::parenthesized!(content in meta.input);

        let value = match attribute_def {
            model::Definition::Bool(_) => model::Value::Bool(
                content.parse::<syn::LitBool>()?.value()),
            model::Definition::StaticStr(_) => model::Value::StaticStr(
                content.parse::<syn::LitStr>()?.value()),
            model::Definition::UnsignedSize(_) => model::Value::UnsignedSize(
                content.parse::<syn::LitInt>()?.base10_parse()?),
            model::Definition::UnsignedInteger64(_) => model::Value::UnsignedInteger64(
                content.parse::<syn::LitInt>()?.base10_parse()?),
            model::Definition::Integer64(_) => model::Value::Integer64(
                content.parse::<syn::LitInt>()?.base10_parse()?),
            model::Definition::Float64(_) => model::Value::Float64(
                content.parse::<syn::LitFloat>()?.base10_parse()?),
            model::Definition::UnsignedInteger32(_) => model::Value::UnsignedInteger32(
                content.parse::<syn::LitInt>()?.base10_parse()?),
            model::Definition::Integer32(_) => model::Value::Integer32(
                content.parse::<syn::LitInt>()?.base10_parse()?),
            model::Definition::Float32(_) => model::Value::Float32(
                content.parse::<syn::LitFloat>()?.base10_parse()?),
            model::Definition::Byte(_) => model::Value::Byte(
                content.parse::<syn::LitByte>()?.value()),
            model::Definition::FieldlessEnum(enumdef) => {
                let mut id = content.parse::<model::Identifier>()?;
                // users are allowed to drop the enum type in short-hand (Foo instead of MyEnum::Foo)
                // fill in the path if they do this
                if id.path().is_empty() {
                    id = enumdef.identifier.append(id)
                }

                model::Value::EnumVariant(id)
            },
            model::Definition::Relation(_) => model::Value::Relation(
                content.parse::<model::Identifier>()?),
            model::Definition::Type(_) => model::Value::Type(
                content.parse::<model::Identifier>()?),
        };

        let attribute_value = model::AttributeValue::new(value);
        variant_build.value(attr_name, attribute_value);

        Ok(())
    })?;

    Ok(variant_build)
}

impl quote::ToTokens for model::AttributeValue {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.append_all(
            match &self.value() {
                model::Value::Bool(b) => quote::quote!(#b),
                model::Value::StaticStr(s) => quote::quote!(#s),
                model::Value::UnsignedSize(n) => quote::quote!(#n),
                model::Value::UnsignedInteger64(n) => quote::quote!(#n),
                model::Value::Integer64(n) => quote::quote!(#n),
                model::Value::Float64(n) => quote::quote!(#n),
                model::Value::UnsignedInteger32(n) => quote::quote!(#n),
                model::Value::Integer32(n) => quote::quote!(#n),
                model::Value::Float32(n) => quote::quote!(#n),
                model::Value::Byte(n) => quote::quote!(#n),
                model::Value::EnumVariant(id) => id.to_token_stream(),
                model::Value::Relation(id) => id.to_token_stream(),
                model::Value::Type(id) => id.to_token_stream(),
            }
        );
    }
}

impl From<model::Identifier> for syn::Path {
    fn from(value: model::Identifier) -> Self {
        Self::from(&value)
    }
}

impl From<&model::Identifier> for syn::Path {
    fn from(value: &model::Identifier) -> Self {
        let mut path = syn::Path {
            leading_colon: None,
            segments: syn::punctuated::Punctuated::new()
        };

        value.path.iter().for_each(|s| {
                let ident = syn::Ident::new(s, span_site());
                let segment = syn::PathSegment::from(ident);
                path.segments.push(segment)
            }
        );

        let ident = syn::Ident::new(value.name(), span_site());
        let segment = syn::PathSegment::from(ident);
        path.segments.push(segment);
 
        path

    }
}

/// Using this with the following return types will panic!():
///   - ReturnType::BoxedTrait
///   - ReturnType::BoxedTraitIterator
///   - ReturnType::AssociatedType
///   - ReturnType::Type
impl quote::ToTokens for model::ReturnType{
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.append_all(
            match &self {
                model::ReturnType::Bool => quote::quote!{ bool },
                model::ReturnType::StaticStr => quote::quote!{ &'static str },
                model::ReturnType::UnsignedSize => quote::quote!{ usize },
                model::ReturnType::UnsignedInteger64 => quote::quote!{ u64 },
                model::ReturnType::Integer64 => quote::quote!{ i64 },
                model::ReturnType::Float64 => quote::quote!{ f64 },
                model::ReturnType::UnsignedInteger32 => quote::quote!{ u32 },
                model::ReturnType::Integer32 => quote::quote!{ i32 },
                model::ReturnType::Float32 => quote::quote!{ f32 },
                model::ReturnType::Byte => quote::quote!{ u8 },
                // this has to be handled conditionally
                model::ReturnType::BoxedTrait => unreachable!("ReturnType::BoxedTrait cannot directly produce a TokenStream"),
                model::ReturnType::BoxedTraitIterator => unreachable!("ReturnType::BoxedTraitIterator cannot directly produce a TokenStream"),
                model::ReturnType::AssociatedType => unreachable!("ReturnType::AssociatedType cannot directly produce a TokenStream"),
                model::ReturnType::Enum => unreachable!("ReturnType::Enum cannot directly produce a TokenStream"),
                model::ReturnType::Type => unreachable!("ReturnType::Type cannot directly produce a TokenStream")
            }
        );
    }
}

impl quote::ToTokens for model::Identifier {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.append_all(syn::Path::from(self).to_token_stream())
    }
}

impl model::Method {
    pub fn return_type_tokens(&self) -> proc_macro2::TokenStream {
        match self.return_type {
            model::ReturnType::BoxedTrait => {
                let ident = self.attribute_definition()
                    .get_relation_definition()
                    .identifier()
                    .to_token_stream();

                quote::quote!{
                    ::std::boxed::Box<dyn #ident>
                }
            },
            model::ReturnType::BoxedTraitIterator => {
                let ident = self.attribute_definition()
                    .get_relation_definition()
                    .identifier()
                    .to_token_stream();

                quote::quote!{
                    ::std::boxed::Box<dyn ::std::iter::Iterator<Item = ::std::boxed::Box<dyn #ident>>>
                }
            },
            model::ReturnType::Type => {
                match self.attribute_definition() {
                    model::Definition::FieldlessEnum(enumdef) => enumdef.identifier.to_token_stream(),
                    // statically dispatched relations
                    model::Definition::Relation(reldef) => reldef.identifier.to_token_stream(),
                    _ => unreachable!("Invalid attribute definition for ReturnType::Type")
                }
            },
            _ => self.return_type.to_token_stream()
        }
    }
}