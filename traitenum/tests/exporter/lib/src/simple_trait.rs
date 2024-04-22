use traitenum::enumtrait;

#[enumtrait]
pub trait SimpleTrait {
    #[enumtrait::Str(default("spunko"))]
    fn name(&self) -> &'static str;
    fn column(&self) -> usize;

    fn default_impl(&self) -> String {
        format!("{} :: {}", self.name(), self.column())
    }
}
