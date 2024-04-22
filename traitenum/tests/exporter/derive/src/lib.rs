traitenum_lib::gen_require!(traitenum_test_exporter, traitenum_test_exporter_derive);

traitenum_lib::gen_derive_macro!(SimpleTraitEnum, derive_traitenum_simple, traitlib::TRAITENUM_MODEL_BYTES_SIMPLE_TRAIT);
traitenum_lib::gen_derive_macro!(ChildTraitEnum, derive_traitenum_child, traitlib::TRAITENUM_MODEL_BYTES_CHILD_TRAIT);
traitenum_lib::gen_derive_macro!(ParentTraitEnum, derive_traitenum_parent, traitlib::TRAITENUM_MODEL_BYTES_PARENT_TRAIT);