//! # Asmov Testing
//! 
//! Test away!
//! 
pub mod namepath;
pub mod extension;
pub mod test;
pub mod group;
pub mod module;


use std::path::{PathBuf, Path};

use anyhow::Context;
use once_cell::sync::Lazy;

pub use module::{Module, ModuleBuilder};
pub use group::{Group, GroupBuilder};
pub use test::{Test, TestBuilder};
pub use extension::{Extension, ExtensionBuilder};
pub use namepath::{Namepath, NamepathTrait};

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

#[derive(Debug, PartialEq, Eq)]
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

pub(crate) fn build_fixture_dir(namepath: &Namepath, use_case: &UseCase) -> PathBuf {
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