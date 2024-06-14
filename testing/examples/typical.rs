fn main() {}

#[cfg(test)]
mod tests {
    use std::fs;
    use asmov_testing::{self as testing, prelude::*};

    static TESTING: testing::StaticModule = testing::module(|| {
        testing::integration(module_path!())
            .using_temp_dir()
            .using_fixture_dir()
            .setup(|module| {
                let tmp_file = module.temp_dir()
                    .join("hello.txt");
                fs::write(&tmp_file,
                    "Hello, Temp").unwrap();
            })
            .teardown_static(teardown)
            .build()
    });

    extern fn teardown() {
        println!("Farewell, sweet test run");
    }

    #[named]
    #[test]
    fn test_things() {
        let test = TESTING.test(function_name!())
            .using_fixture_dir()  
            .inherit_temp_dir()
            .build();

        let temp_file = test.temp_dir()
            .join("hello.txt");
        let temp_text = fs::read_to_string(temp_file)
            .unwrap();
        assert_eq!("Hello, Temp", temp_text);

        let fixture_file = test.fixture_dir()
            .join("sample.txt");
        let _fixture_text = fs::read_to_string(fixture_file)
            .unwrap();
        assert_eq!("Hello, Fixture", _fixture_text);
    }
}
