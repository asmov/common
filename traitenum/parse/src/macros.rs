const MODEL_BYTES_NAME: &'static str = "TRAITENUM_MODEL_BYTES_";

mod traitenum;
mod enumtrait;

pub use traitenum::traitenum_derive_macro;
pub use enumtrait::enumtrait_macro;

#[cfg(test)]
mod tests {
    use quote;
    use crate::{TRAIT_ATTRIBUTE_HELPER_NAME, model, macros::enumtrait, macros::traitenum};


    /// Asserts that the expected value has been defined for a given enum variant
    macro_rules! assert_traitenum_value {
        ($model:ident, $variant_name:literal, $attribute_name:literal, $value_type:ident, $expected:expr) => {
            assert!($model.variant($variant_name).is_some(), "Variant doesn't exist: {}", $variant_name);
            assert!($model.variant($variant_name).unwrap().value($attribute_name).is_some(),
                "Variant attribute doesn't exist: {} -> {}", $variant_name, $attribute_name);
            match $model.variant($variant_name).unwrap().value($attribute_name).unwrap().value() {
                model::Value::$value_type(ref val) => assert_eq!($expected, *val),
                _ => assert!(false, "Incorrect value type for attribute: {}", $attribute_name)
            }
        };
    }

    /// Asserts that the expected enum value has been defined for a given enum variant
    macro_rules! assert_traitenum_value_enum {
        ($model:ident, $variant_name:literal, $attribute_name:literal, $expected:literal) => {
            match $model.variant($variant_name).unwrap().value($attribute_name).unwrap().value() {
                model::Value::EnumVariant(ref val) => assert_eq!($expected, val.to_string()),
                _ => assert!(false, "Incorrect value type for attribute: {}", $attribute_name)
            }
        };
    }

    #[test]
    fn test_parse_enumtrait_primitives() {
        let attribute_src = quote::quote!{};

        let item_src = quote::quote!{
            pub trait MyTrait {
                // test Str default
                #[enumtrait::Str(default(":)"))]
                fn str_default(&self) -> &'static str;

                // test Num default
                #[enumtrait::Num(default(44))]
                fn num_default(&self) -> usize;

                // test Bool default
                #[enumtrait::Bool(default(true))]
                fn bool_default(&self) -> bool;

                // test Enum default
                #[enumtrait::Enum(default(RPS::Rock))]
                fn enum_default(&self) -> RPS;

                // test Str variant preset
                #[enumtrait::Str(preset(Variant))]
                fn str_preset_variant(&self) -> &'static str;

                // test Num serial preset w/start and increment 
                #[enumtrait::Num(preset(Serial), start(3), increment(2))]
                fn num_preset_serial_all(&self) -> u64;

                // test default implementation
                fn default_implementation(&self) {
                    todo!();
                }
            }
        };
        
        let model = enumtrait::parse_enumtrait_macro(attribute_src, item_src).unwrap().model;
        dbg!(&model);

        assert!(model.identifier().path().is_empty());
        assert_eq!("MyTrait", model.identifier().name());

        let item_src = quote::quote!{
            enum MyEnum {
                One,
                // test short-hand enum values
                #[traitenum(str_preset_variant("2"), enum_default(Paper))]
                Two,
                #[traitenum(bool_default(false))]
                Three,
                #[traitenum(enum_default(RPS::Scissors))]
                Four,
            }
        };

        let model_bytes = bincode::serialize(&model).unwrap();
        let traitenum::TraitEnumMacroOutput {model: enum_model, tokens: enum_tokens} = traitenum::parse_traitenum_macro(
            item_src, &model_bytes).unwrap();

        dbg!(&enum_model);
        dbg!(&enum_tokens.to_string());

        // test defaults
        assert_traitenum_value!(enum_model, "One", "str_default", StaticStr, ":)");
        assert_traitenum_value!(enum_model, "One", "bool_default", Bool, true);
        assert_traitenum_value!(enum_model, "One", "num_default", UnsignedSize, 44);
        assert_traitenum_value_enum!(enum_model, "One", "enum_default", "RPS::Rock");
        // test string preset(variant)
        assert_traitenum_value!(enum_model, "Two", "str_preset_variant", StaticStr, "2");
        // test u64 preset(serial) w/start(3), increment(2)
        assert_traitenum_value!(enum_model, "Three", "num_preset_serial_all", UnsignedInteger64, 7);
        // test short-hand enum value
        assert_traitenum_value_enum!(enum_model, "Two", "enum_default", "RPS::Paper");
        // test non-default bool
        assert_traitenum_value!(enum_model, "Three", "bool_default", Bool, false);
        // test non-default enum
        assert_traitenum_value_enum!(enum_model, "Four", "enum_default", "RPS::Scissors");
    }
    
    #[test]
    fn test_parse_enumtrait_boxed_trait_relations() {
        let attribute_src = quote::quote!{};

        let item_src = quote::quote!{
            pub trait MyTrait {
                // test Rel dynamic many-to-one
                #[enumtrait::Rel(nature(ManyToOne), dispatch(BoxedTrait))]
                fn many_to_one_dyn(&self) -> Box<dyn FirstOneTrait>;

                // test Rel many-to-one dynamic (elided)
                #[enumtrait::Rel(nature(ManyToOne))]
                fn many_to_one_dyn_elide(&self) -> Box<dyn SecondOneTrait>;

                // test Rel dynamic one-to-many
                #[enumtrait::Rel(nature(OneToMany), dispatch(BoxedTrait))]
                fn one_to_many_dyn(&self) -> Box<dyn Iterator<Item = Box<dyn FirstManyTrait>>>;

                // test elided Rel dynamic one-to-many
                fn one_to_many_elided_dyn(&self) -> Box<dyn Iterator<Item = Box<dyn SecondManyTrait>>>;
            }
        };
        
        let model = enumtrait::parse_enumtrait_macro(attribute_src, item_src).unwrap().model;
        dbg!(&model);

        let item_src = quote::quote!{
            #[traitenum(many_to_one_dyn(ManyToOneEnum::Dyn))]
            #[traitenum(many_to_one_dyn_elide(ManyToOneEnum::DynElide))]
            enum MyEnum {
                #[traitenum(one_to_many_dyn(OneToManyOneEnum), one_to_many_elided_dyn(OneToManyTwoEnum))]
                One,
                // test short-hand enum values
                #[traitenum(one_to_many_dyn(OneToManyThreeEnum), one_to_many_elided_dyn(OneToManyFourEnum))]
                Two,
            }
        };

        let model_bytes = bincode::serialize(&model).unwrap();
        let traitenum::TraitEnumMacroOutput {model: enum_model, tokens: enum_tokens} = traitenum::parse_traitenum_macro(
            item_src, &model_bytes).unwrap();

        dbg!(&enum_model);
        dbg!(&enum_tokens.to_string());
    }


    #[test]
    fn test_parse_enumtrait_errors() {
        let simple_attribute_src = quote::quote!{};

        let simple_item_src = quote::quote!{
            pub trait MyTrait {
                fn name(&self) -> &'static str;
            }
        };

        // test error: non-empty identifier
        let attribute_src = quote::quote!{ crate::MyTrait };
        assert!(enumtrait::parse_enumtrait_macro(attribute_src, simple_item_src.clone()).is_err(),
            "Non-empty #[{}(<pathspec>)] should throw an Error", TRAIT_ATTRIBUTE_HELPER_NAME);
        
        // test error: mismatched trait name with identifier
        let attribute_src = quote::quote!{ crate::tests::TheirTrait };
        assert!(enumtrait::parse_enumtrait_macro(attribute_src, simple_item_src.clone()).is_err(),
            "Mismatched trait name and #[{}(<pathspec>)] identifier should throw an Error", TRAIT_ATTRIBUTE_HELPER_NAME);

        let unimplemented_static_dispatch_src = quote::quote!{
            pub trait MyTrait {
                type ManyType: ManyTrait;

                #[traitenum::Rel(dispatch(Other))]
                fn many_to_one(&self) -> Self::ManyType;
            }
        };

        assert!(enumtrait::parse_enumtrait_macro(
            simple_attribute_src.clone(),
            unimplemented_static_dispatch_src).is_err(),
            "Dispatch::Other is permanently unimplemented and should throw an Error");

        let associated_types_src = quote::quote!{
            pub trait MyTrait {
                type ManyType: ManyTrait;

                fn many_to_one(&self) -> Self::ManyType;
            }
        };

        assert!(enumtrait::parse_enumtrait_macro(
            simple_attribute_src.clone(),
            associated_types_src).is_err(),
            "Associated types are not supported");
    }
}