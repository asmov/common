traitenum 
=========
A Rust library for using fieldless enums as schema definitions.

In short, a trait is declared with a configurable set of const properties,   using attributes for each method. An enum is then derived for that trait, with each variant filling in property values via attributes.

Relations (`Rel`) between traits / enums can be defined with a `nature` of:
- `OneToOne`  - Points to to a single variant of another enum
- `OneToMany` - Provides an iterator over another enum's variants
- `ManyToOne` -Points to a single variant of another enum

Supported types include:
- `Str` (static)
- `Num` (usize, i64, f32, etc.)
- `Enum`
- `Bool`

Default implementations for trait methods can be used to extend functionality.

Each method signature must properly correspond with its attribute. On the other hand, attributes can be elided from method signatures, either partially or completely. `Num`, for example, uses the method signature to determine what specific type of primitive to support (f64, u8, etc.).

Properties support defaults and presets.

Presets set a default value for a property in a pre-determined way:
- `Str` converts the variant name (snake case, kebab case, etc.)
- `Num` converts the ordinal of the variant (offset, increment, etc.)

Both default and preset values can be overridden by each enum variant.

Relationships require a method signature to return:
- `OneToOne` and `ManyToOne`
  + `-> Box<dyn OtherTrait>`
- `OneToMany`
  + `-> Box<dyn Iterator<Item = Box<dyn OtherTrait>>>`

Example
-------

```rust
// ... my-trait-crate/src/lib.rs ...

#[enumtrait(crate::MyParentTrait)]
pub trait MyParentTrait {
    #[enumtrait::Str()]
    pub fn alias(&self) -> &'static str;

    #[enumtrait::Num(preset(Serial), start(1), increment(100))]
    pub fn index(&self) -> usize;

    #[enumtrait::Rel(nature(OneToMany))]
    pub fn children(&self) -> Box<dyn Iterator<Item = Box<dyn MyChildTrait>>>;
}

#[enumtrait(crate::MyChildTrait)]
pub trait MyChildTrait {
    #[enumtrait::Str(preset(Kebab))]
    pub fn kebab_name(&self) -> &'static str;

    #[enumtrait::Num()]
    pub fn column(&self) -> i32;

    #[enumtrait::Rel(nature(ManyToOne))]
    pub fn parent(&self) -> Box<dyn MyParentTrait>;
}

// ... my-enum-crate/src/lib.rs ...

#[derive(MyParentTrait)]
pub enum MyParentEnum {
    #[traitenum(alias("Uno"), children(MyFirstChildEnum))]
    First,
    #[traitenum(alias("Dos"), children(MySecondChildEnum))]
    Second,
}

#[derive(MyChildTrait)]
#[traitenum(parent(MyParentEnum::First))]
pub enum MyFirstChildEnum {
    #[traitenum(column(1))]
    AlphaBravo,
    #[traitenum(column(3))]
    CharlieDelta,
}

#[derive(MyChildTrait)]
#[traitenum(parent(MyParentEnum::Second))]
pub enum MySecondChildEnum {
    #[traitenum(column(2))]
    EchoFoxtrot,
    #[traitenum(column(4))]
    GolfHotel 
}

// ... testing ...

assert_eq!(1, MyFirstChildEnum::AlphaBravo.column())
assert_eq!("echo-foxtrot", MySecondChildEnum::EchoFoxtrot.kebab_name())
assert_eq!("Uno", MyFirstChildEnum::CharlieDelta.parent().alias())
assert_eq!(2, MyParentEnum::Second.children().nth(0).unwrap().column())
```

Packages
--------
- [traitenum](./macro) : Macros used to define traitenum traits
- [traitenum-lib](./lib) : Exports used by traitenum macros
- [cargo-traitneum](./cargo) : Cargo addon that initializes traitenum workspaces and adds / removes traitenum traits

Documents
---------
- [Roadmap](./docs/Roadmap.md) : Planned fixes and enhancements
- [Design: Version 0](./docs/design/v0/README.md): Design notes for Version 0
- [Copying](./COPYING.txt) : The GPL3 licensing declaration as displayed below.
- [License](./LICENSE.txt) : The complete GPL3 license definition.

Structure
---------
- [Documents](./docs)
- [Export Tests](./tests/exporter/) : Compile testing of a traitenum workspace crate.
- [Import Tests](./tests/exporter/) : Compile testing of a traitenum end-user crate.


License (GPL 3)
---------------
traitenum: A Rust library for using fieldless enums as schema definitions.  
Copyright (C) 2023-2024 Asmov LLC

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a [copy](./LICENSE.txt) of the GNU General Public License
along with this program.  If not, see https://www.gnu.org/licenses/.
