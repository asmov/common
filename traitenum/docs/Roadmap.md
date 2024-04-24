Roadmap
=======

Version 1
---------

- Cargo addon should break each lib trait into it's own module and than pub import it.
  + Prevents generator from stomping on code documentation each time a trait is added via the cargo addon.
- Cargo addon
  + cargo traitenum trait remove
  + cargo traitenum verify 
  + cargo traitenum fix
  + integration tests
  + autogen an example for workspace init and workspace new, which will be checked into git to show differences in output
  + autogen an example of a trait added and a trait removed, same as above
- Research: Enums should be required to pass a #[traitenum::implements(TraitName)] to the enum declaration, after the #[derive{}]
  expression.
  1. This adheres to the "debuff magic" design of traitenum:
    + Exactly which custom trait is being implemented by the derive wrapper should be documented via code.
  2. This might allow the use of path aliasing (see below)
    + E.g., use mycrate::mypath::MyEnumTrait as Foo
  - No path aliasing: Accept only Ident as a value
    - AND Ensure the expected trait name is being imported in the derive_wrapper or compiler error.
  - Path aliasing: (if possible?) Accept Path as a value
    - AND Ensure trait equality using a magic trait method: Trait::model_bytes(&self) -> &'static [u8]
- Derive macro gen shouldn't require a function name input
- Relationships
  + OnetoOne
- Polishing: Document, Refactor, Test
  + lib, macro, cargo
  + Make errors more helpful to end-users. Include tips 


Version 2
---------

- Implement a common EnumTrait trait for each enum, including:
  + type Iterator: Iterator<Item = Self>
  + variant_ordinal(&self) -> usize :: Num(preset(Ordinal))
  + variant_iter() -> Self::Iterator
  + variant_name(&self) -> &'static str :: Str(preset(Variant))
  + from_variant_name(&str) -> Option<Self>
- Common Setting: optional(bool) => allows for Option<...> methods.
  + relationships: the one-side of relationships. the many-side can returns an empty iterator already
  + value on enum: e.g., #[traitenum(name(Some("foo")))] and #[traitenum(name(None))]


Version 3
---------

- Inline traits