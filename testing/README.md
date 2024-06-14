Asmov Testing
===============================================================================
[![Latest Version]][crates.io]

[Latest Version]: https://img.shields.io/crates/v/asmov-testing.svg
[crates.io]: https://crates.io/crates/asmov-testing

A toolkit for organized testing in Rust.

## Features
- Structures tests into heirarchies:
  - Module
  - Group
  - Test
- Allows granular setup and teardown callbacks at each level.
- Standardizes filepath helpers for temp and fixture directories.

## Example Usage

```rust
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
```


Documentation
-------------

Refer to [docs.rs/asmov-testing](https://docs.rs/asmov-testing/latest/asmov_testing)


License (GPL 3)
-------------------------------------------------------------------------------
Asmov Testing: A toolkit for organized testing in Rust  
Copyright (C) 2023 Asmov LLC

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a [copy](./COPYING.txt) of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.

Third-Party Licenses
-------------------------------------------------------------------------------
## crate: [function_name](https://crates.io/crates/function_name)

>Our library publically exports the **named** macro from [Daniel Henry-Mantilla](https://github.com/danielhenrymantilla)'s crate: [function_name](https://github.com/danielhenrymantilla/rust-function_name). It is available for use from our crate as `asmov_testing::named`.

**License (MIT):**  
[Copyright (c) 2019 Daniel Henry-Mantilla](./docs/licenses/danielhenrymantilla/function_name/LICENSE.txt)