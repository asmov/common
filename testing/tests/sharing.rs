mod shared;

#[cfg(test)]
mod tests {
    use asmov_testing::{self as testing, prelude::*};
    use super::*;

    #[test]
    fn test_shared_imported_fixture_dir() {
        let test_module = testing::integration(module_path!())
            .import_fixture_dir(&shared::NAMEPATH)
            .build();

        assert_eq!(&*shared::FIXTURE_DIR, test_module.imported_fixture_dir(&*shared::NAMEPATH));
    }

    #[test] #[should_panic]
    fn test_shared_imported_fixture_dir_fail() {
        let test_module = testing::integration(module_path!())
            .build();

        assert_eq!(&*shared::FIXTURE_DIR, test_module.imported_fixture_dir(&*shared::NAMEPATH));
    }


}