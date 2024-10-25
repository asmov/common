#[cfg(test)]
mod tests {
    use asmov_common_dataset_fieldenum::DatasetFieldEnum;

    #[test]
    fn test_dataset_field_enum() {
        #[derive(asmov_common_dataset_fieldenum_derive::DatasetFieldEnum)]
        enum MyEnum {
            Alpha,
            Bravo,
            Charlie
        }

        assert_eq!("Alpha", MyEnum::Alpha.name());
        assert_eq!("Bravo", MyEnum::Bravo.name());
        assert_eq!("Charlie", MyEnum::Charlie.name());

        assert_eq!(0, MyEnum::Alpha.ordinal());
        assert_eq!(1, MyEnum::Bravo.ordinal());
        assert_eq!(2, MyEnum::Charlie.ordinal());
    }
}
