Design Notes for Version 3
==========================

Planning
--------

### Whiteboards
- [Whiteboard: Inline enums 2023-12-26](./whiteboard-inline-enums_2023-12-26.jpeg)


### Feature: Inline Traits for Enums

#### Structural approach

```rust
#[derive(EnumTrait)]
#[traitenum::Trait(TraitName, {
    property_name: DefinitionType {
        setting_name: setting_value,
        setting_name: setting_value,
    },
    property_name: DefinitionType {
        setting_name: setting_value,
        setting_name: setting_value
    }
})]
enum MyEnum {
    #[traitenum(property_name(value), property_name(value))]
    VariantName,
    #[traitenum(property_name(value), property_name(value))]
    VariantName,
}
```

Pros / Cons:
+ The structural approach looks cleaner.
- Needs completely new parsing code.
- Completely different than anything that exists.


#### Psuedo-code approach

```rust
#[derive(EnumTrait)]
#[traitenum::Trait({
    pub trait TraitName {
        #[traitenum::DefinitionType(setting_name(setting_value), setting_name(setting_value))]
        fn property_name(&self) -> ReturnType;

        #[traitenum::DefinitionType(setting_name(setting_value), setting_name(setting_value))]
        fn property_name(&self) -> ReturnType;

        fn default_implementation(&self) -> AnyReturnType {
            todo!()
        }
    }
})]
enum MyEnum {
    #[traitenum(property_name(value), property_name(value))]
    VariantName,
    #[traitenum(property_name(value), property_name(value))]
    VariantName,
}
```

Pros / Cons:
+ Reuses even more of the existing parsing code.
+ Looks exactly the same as the heavy approach to the end-user.
+ Easy to simply the copy the trait and expand to the heavy format if needed.
+ Adheres to the self-documenting code design principles better.
- Not as clean looking at the structural approach.