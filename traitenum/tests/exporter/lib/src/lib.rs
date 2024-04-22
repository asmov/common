use traitenum::{self, enumtrait};

pub mod simple_trait;
pub use simple_trait::{SimpleTrait, TRAITENUM_MODEL_BYTES_SIMPLE_TRAIT};

#[enumtrait]
pub trait ParentTrait {
    #[enumtrait::Str(preset(Variant))]
    fn name(&self) -> &'static str;

    #[enumtrait::Rel(nature(OneToMany))]
    fn children(&self) -> Box<dyn Iterator<Item = Box<dyn ChildTrait>>>;
}

#[enumtrait]
pub trait ChildTrait {
    #[enumtrait::Str(preset(Variant))]
    fn topic(&self) -> &'static str;

    #[enumtrait::Num(preset(Ordinal))]
    fn ordinal(&self) -> usize;

    #[enumtrait::Rel(nature(ManyToOne))]
    fn parent(&self) -> Box<dyn ParentTrait>;
}

#[cfg(test)]
mod tests {
    use traitenum_lib;
    use bincode;

    #[test]
    fn test_load_model() {
        let bytes = crate::simple_trait::TRAITENUM_MODEL_BYTES_SIMPLE_TRAIT;
        let _model: traitenum_lib::model::EnumTrait = bincode::deserialize(bytes).unwrap();
    }
}
