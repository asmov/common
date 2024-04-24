Notes for Asmov Testing
=====================================================

Docs
=====================================================

Model
-----------------------------------------------------

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

Each model uses a builder pattern for const construction.

Each model holds a one-way reference to its parent in the heirarchy.

`Group` and `Test` models may inherit or override certain attributes from their parents in this heirarchy.

`Module` and `Group` models are typically created with a static builder pattern. Destruction is then handled by an process exit hook.

`Test` models are typically constructed and dropped within the lifespan of the test.

Each model object is represented structurally within the project using a string `namepath`, based on the Rust module path scheme.

Where resources are represented externally, models are represented with the same heirarchy, described by the `namepath`.

In a filesystem, an example of this might be:
- `my-model / my-group / my-test`

Namepathing
-----------------------------------------------------

This crate uses a concept of a `namepath` which is an extended form of the Rust module pathing scheme.

Preceding the module path, a `/` path separator can be used to delimit a file-system-like heirarchy. This may represent some form of context for the project.

Following the module path, a `.` dot notation character can be used to delimit anything that can't be reached by the Rust module pathing scheme, real or conceptual.

Examples:
- `org-name/team-name/crate_name::module_name`
- `crate_name::module_name::type_name.concept_name`

Project file structure
-----------------------------------------------------

# File fixture directories

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
- `fixture_dir()` specifies a custom path.

The three aforementioned builder methods will verify that the configured path exists before construction.

A model's `fixture_dir()` can then be used retrieve the configured `Path`.

.g# Temporary file directories







