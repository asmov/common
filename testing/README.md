# Asmov Testing
A toolkit for organized testing in Rust.

- Structures tests into heirarchies:
  - Module -> Group -> Test
- Allows granular setup and teardown callbacks at each level.
- Standardizes filepath helpers for temp and fixture directories.

## ... *Work in Progress* ...

This project is not ready for collaboration or release at the moment.

## Example Usage

```rust
// myproj::mycrate::mymod
#[cfg(test)]
mod tests {
    use asmov_testing as testing;

    static TESTING: testing::StaticModule = testing::module(|| {
        testing::integration(module_path!())
            // A one-time use temporary dir is created for this test. It is deleted on teardown.
            //     /tmp/myproj/{random string}/mycrate/mymod
            .using_temp_dir()
            // A file fixture directory for this test. It is gauranteed to exist.
            //     myproj/testing/integration/mycrate/mymod
            .using_fixture_dir()
            // A custom setup function is ran once at build().
            .setup(|testing| {
                println!("Fixture files located at: {}", testing.fixture_dir())
            })
            // A custom teardown function is ran when the program exits
            .teardown_static(teardown)
            .build()
    });

    // The custom teardown function
    fn teardown() {
        println!("Farewell, sweet test run")) ;
        TESTING.teardown();
    }

    #[test]
    fn test_things() {
        let test = TESTING.test("test_something")
	    // appends to parent path: myproj/testing/fixtures/.../test_something
            .using_fixture_dir()  
	    // deleted on drop. appends to parent path: /tmp/.../test_something
            .using_temp_dir()
            .build();

        let temp_file = test.temp_dir().join("hello_temp.txt");
        std::fs::write(temp_file, "Hello, World").unwrap();

        let fixture_file = test.fixture_dir().join("sample.txt");
        let _fixture_text = std::fs::read_to_string(fixture_file).unwrap();
    }
}
```

## License (GPL 3)
```
Asmov Testing - A toolkit for organized testing in Rust.
Copyright (C) 2023 Asmov LLC <devolopment.pub@asmov.software>

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
```
