todo!()
======================================================

In Progress
------------------------------------------------------


Task Pool
------------------------------------------------------

# Replace Module::local_group() with GroupBuilder::nonstatic()

# Complete coverage of unit testing for Module, Group, Test

Everything should match.

# Integration test cleanup
Ick.car

# Unit test namepath.rs

# Safe destruction of temp directories

No stepping on each others' toes.

# Clippy!

- Formatting standardization.
- Documentation standardization.
  + Licensing

# Attribute macro for the crate, applied to test functions

Specifying `#[testing]` should include:
- `#[test]`
- `#[named]`

## Extend attribute macro with variables

Something like `#[testing(module = {variable}, group = {variable}, ...]` would create a Test const named "TEST"local to that function.

Depending on what is possible, the `module` attribute could be autofilled if a variable named "TESTING" exists or if there is only one static `module` model in that test's module.

Attributes could include:
- `using_` and `inherit_` for temporary and fixture directories

# A testing `Component` model/interface to handle library types of setup/teardown

Actual name TBD. For use as a library for the Module/Group/Test suite that sets up, tears down.

They have their own builders.

Anything beyond trait bound use should be accessed through an inspected method that casts.

This is for use with common test requirements across test modules or across crates.

An example might be:
- The ability to setup / teardown a connection to a SQL database. A library.
- Setup / teardown a connection to a specific SQL database. Common configuration use said library.

