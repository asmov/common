//! # Model
//! 
//! Tests are modeled within a heirarchy:
//! - `Module`
//! - `Group`
//! - `Test`
//! 
//! The `Module` model represents the Rust module that is being tested.
//! 
//! The `Group` model is an optional model that allows further sub-grouping of testing attributes and behaviors.
//! 
//! The `Test` model represents the actual test or benchmark that is being performed.
//! 
//! A `Module` is categorized between three use-cases:
//! 1. Unit tests
//! 2. Integration tests
//! 3. Benchmarks
//! 
//! Each model uses a builder pattern for construction.
//! 
//! Each model holds a one-way reference to its parent in the heirarchy.
//! 
//! `Group` and `Test` models may inherit or override certain attributes from their parents in this heirarchy.
//! 
//! `Module` and `Group` models are typically created with a static builder pattern. Teardown is then handled by a process exit hook, as destructors are unavailable at the static scope.
//! 
//! `Test` models are typically constructed and dropped with the lifespan of the test.
//! 
//! Each model object is represented structurally within the project using a string `namepath`, based on the Rust module path scheme.
//! 
//! Where resources are represented externally, models are represented with the same heirarchy, described by the `namepath`.
//! 
//! In a filesystem, an example of this might be:
//! - `my-model / my-group / my-test`
//! 
//! # Namepathing
//! 
//! This crate uses a concept of a `namepath` which is an extended form of the Rust module pathing scheme.
//! 
//! Preceding the module path, a `/` path separator can be used to delimit a file-system-like heirarchy. This may represent some form of context for the project.
//! 
//! Following the module path, a `.` dot notation character can be used to delimit anything that can't be reached by the Rust module pathing scheme, real or conceptual.
//! 
//! Examples:
//! - `org-name/team-name/crate_name::module_name`
//! - `crate_name::module_name::type_name.concept_name`
//! 
//! # Project file structure
//! 
//! ## File fixture directories
//! 
//! File fixtures for testing purposes may be stored (by default) relative to the crate's project directory in `./testing/fixtures`.
//! 
//! The file structure within the base fixture directory reflects the test model's use-case and heirarchy:
//! ```bash
//! ./ testing / fixtures /
//!      [ unit | integration | benchmark ] /
//!        { module } /
//!          { group } /
//!            { test }
//! ```
//! 
//! The default fixture path f/or a model mirrors its heirarchy and namepath.
//! 
//! It is an error to build a test model with a fixture path that does not exist.
//! 
//! Fixture dirs must be explicitly configured during construction:
//! - `using_fixture_dir()` uses a default calculated path.
//! - `inherit_fixture_dir()` inherits the same path as its parent in the model heirarchy.
//! 
//! The three aforementioned builder methods will verify that the configured path exists before construction.
//! 
//! After construction, a model's `fixture_dir()` can then be used retrieve the configured `Path`.
//! 
//! ## Temporary file directories
//! 
//! Temporary directories are created upon construction if requested in the builder. They are automatically deleted upon teardown.
//! 
//! Temporary directories follow the same parent heirarchy as the rest of this crate. The parent `Module` or `Group` will have its own randomly generated
//! directory, within which each child component will have a subdirectory.
//! 
//! The base path for temporary directories can be re-configured away from the operating system's default, if neededed.
//! 
//! Temporary directories must be explicitly configured during construction:
//! - `using_temp_dir()` uses a default calculated path.
//! - `inherit_temp_dir()` inherits the same path as its parent in the model heirarchy.
//! 
//! After construction, a model's `temp_dir()` can then be used retrieve the pre-created `Path`.

pub mod namepath;
pub mod test;
pub mod group;
pub mod module;

use std::path::{PathBuf, Path};
use anyhow::Context;
use once_cell::sync::Lazy;

pub use module::{Module, ModuleBuilder};
pub use group::{Group, GroupBuilder};
pub use test::{Test, TestBuilder};
pub use namepath::{Namepath, NamepathTrait};
pub use function_name::named;

/// A static reference to a [Module] instance.
pub type StaticModule = Lazy<Module>;
pub type StaticGroup<'module,'func> = Lazy<Group<'module,'func>>;

pub const fn module(func: fn() -> Module) -> StaticModule {
    StaticModule::new(func)
}

pub const fn group<'module,'func>(func: fn() -> Group<'module,'func>) -> StaticGroup<'module,'func> {
    StaticGroup::new(func)
}

pub fn unit(module_path: &str) -> ModuleBuilder {
    ModuleBuilder::new(module_path, UseCase::Unit)
}

pub fn integration(module_path: &str) -> ModuleBuilder {
    ModuleBuilder::new(module_path, UseCase::Integration)
}

pub fn benchmark(module_path: &str) -> ModuleBuilder {
    ModuleBuilder::new(module_path, UseCase::Benchmark)
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum UseCase {
    Unit,
    Integration,
    Benchmark
}

impl UseCase {
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Unit => "unit",
            Self::Integration => "integration",
            Self::Benchmark => "benchmark"
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Model {
    Module,
    Group,
    Test 
}

impl Model {
    pub fn to_str(&self) -> &'static str {
        match self {
            Self::Module => "module",
            Self::Group => "group",
            Self::Test => "test"
        }
    }
}

// Helper function for test models configuring their temp_dir during `build()`.
pub(crate) fn build_temp_dir(namepath: &Namepath, base_temp_dir: &Path) -> PathBuf {
    let temp_dir = base_temp_dir.join(PathBuf::from_iter(namepath.components().iter()));

    if !temp_dir.exists() {
        std::fs::create_dir_all(&temp_dir)
            .context(format!("Unable to create temporary directory: {}", &temp_dir.to_str().unwrap()))
            .unwrap();
    }

    temp_dir.canonicalize().unwrap()
}

pub(crate) fn build_fixture_dir(namepath: &Namepath, use_case: UseCase) -> PathBuf {
    // path: ./ testing / fixtures / [ unit | integration | benchmark ] / { module } / { group ... } / { test } 
    let fixture_dir = PathBuf::from(strings::TESTING)
        .join(strings::FIXTURES)
        .join(use_case.to_str())
        .join(namepath.testing_dir());
    let fixture_dir = fixture_dir.canonicalize()
        .context(format!("Module `fixture directory` does not exist: {}", fixture_dir.to_str().unwrap()))
        .unwrap();

    fixture_dir
}

pub(crate) mod strings {
    pub(crate) const TESTING: &'static str = "testing";
    pub(crate) const FIXTURES: &'static str = "fixtures";
}


#[cfg(test)]
pub(crate) mod tests {
}