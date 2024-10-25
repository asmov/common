use traitenum::enumtrait;
use traitenum_prelude::*;

#[enumtrait]
pub trait DatasetFieldEnum: EnumTrait {
    #[enumtrait::Str(preset(Kebab))]
    fn name(&self) -> &'static str;
    #[enumtrait::Num(preset(Ordinal))]
    fn ordinal(&self) -> usize;
}
