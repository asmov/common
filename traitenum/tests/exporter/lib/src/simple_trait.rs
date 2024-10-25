use traitenum_prelude::*;

#[enumtrait]
pub trait SimpleTrait: EnumTrait {
    #[enumtrait::Str(default("spunko"))]
    fn name(&self) -> &'static str;
    fn column(&self) -> usize;

    fn default_impl(&self) -> String {
        format!("{} :: {}", self.name(), self.column())
    }
}
