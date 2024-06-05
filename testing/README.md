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

Documentation
-----------------------------------------------------

### Model

Tests are modeled within a heirarchy:
- `Module`
- `Group`
- `Test`

The `Module` model represents the Rust module that is being tested.

The `Group` model is an optional model that allows further sub-grouping of testing attributes and behaviors.

The `Test` model represents the actual test or benchmark that is being performed.

A `Module` is categorized between three use-cases:
1. Unit tests
2. Integration tests
3. Benchmarks

Each model uses a builder pattern for construction.

Each model holds a one-way reference to its parent in the heirarchy.

`Group` and `Test` models may inherit or override certain attributes from their parents in this heirarchy.

`Module` and `Group` models are typically created with a static builder pattern. Teardown is then handled by a process exit hook, as destructors are unavailable at the static scope.

`Test` models are typically constructed and dropped with the lifespan of the test.

Each model object is represented structurally within the project using a string `namepath`, based on the Rust module path scheme.

Where resources are represented externally, models are represented with the same heirarchy, described by the `namepath`.

In a filesystem, an example of this might be:
- `my-model / my-group / my-test`

### Namepathing

This crate uses a concept of a `namepath` which is an extended form of the Rust module pathing scheme.

Preceding the module path, a `/` path separator can be used to delimit a file-system-like heirarchy. This may represent some form of context for the project.

Following the module path, a `.` dot notation character can be used to delimit anything that can't be reached by the Rust module pathing scheme, real or conceptual.

Examples:
- `org-name/team-name/crate_name::module_name`
- `crate_name::module_name::type_name.concept_name`

### Project file structure

#### File fixture directories

File fixtures for testing purposes may be stored (by default) relative to the crate's project directory in `./testing/fixtures`.

The file structure within the base fixture directory reflects the test model's use-case and heirarchy:
```
./ testing / fixtures /
     [ unit | integration | benchmark ] /
       { module } /
         { group } /
           { test }
```

The default fixture path for a model mirrors its heirarchy and namepath.

It is an error to build a test model with a fixture path that does not exist.

Fixture dirs must be explicitly configured during construction:
- `using_fixture_dir()` uses a default calculated path.
- `inherit_fixture_dir()` inherits the same path as its parent in the model heirarchy.

The three aforementioned builder methods will verify that the configured path exists before construction.

After construction, a model's `fixture_dir()` can then be used retrieve the configured `Path`.

#### Temporary file directories

Temporary directories are created upon construction if requested in the builder. They are automatically deleted upon teardown.

Temporary directories follow the same parent heirarchy as the rest of this crate. The parent `Module` or `Group` will have its own randomly generated
directory, within which each child component will have a subdirectory.

The base path for temporary directories can be re-configured away from the operating system's default, if neededed.

Temporary directories must be explicitly configured during construction:
- `using_temp_dir()` uses a default calculated path.
- `inherit_temp_dir()` inherits the same path as its parent in the model heirarchy.

After construction, a model's `temp_dir()` can then be used retrieve the pre-created `Path`.


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