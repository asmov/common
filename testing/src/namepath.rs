use std::{path::{PathBuf, Path}, hash::Hash};
use crate::{Module, Group, UseCase};

pub trait NamepathTrait {
    fn module_path(&self) -> &str;
    fn path(&self) -> &str;
    fn testing_path(&self) -> &str;

    fn components(&self) -> Vec<&str> {
        split(&self.path())
    }

    fn dir(&self) -> PathBuf {
        PathBuf::from_iter(split(&self.path()))
    }

    fn testing_dir(&self) -> PathBuf {
        PathBuf::from_iter(split(&self.testing_path()))
    }

    fn squash(&self) -> String {
        squash(self.path())
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub enum Namepath {
    Module(ModuleNamepath),
    Group(GroupNamepath),
    Test(TestNamepath),
}

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub struct ModuleNamepath {
    module_path: String,
    testing_path: String
}

impl NamepathTrait for ModuleNamepath {
    fn module_path(&self) -> &str {
        &self.module_path
    }

    fn path(&self) -> &str {
        &self.module_path
    }

    fn testing_path(&self) -> &str {
        &self.testing_path
    }
}

// Strips the crate name prefix and the test/tests suffix from a module_path!().
// If the path is from lib.rs, the crate name is returned. 
fn make_testing_path(use_case: UseCase, path: &str) -> Option<&str> {
    static REGEX_INTEGRATION: once_cell::sync::OnceCell<regex::Regex> = once_cell::sync::OnceCell::new();
    static REGEX_UNIT: once_cell::sync::OnceCell<regex::Regex> = once_cell::sync::OnceCell::new();
    let regex_integration = REGEX_INTEGRATION.get_or_init(|| {
        regex::Regex::new(r"^(\w+?)(?:::tests)?$").unwrap()
    });
    let regex_unit= REGEX_UNIT.get_or_init(|| {
        regex::Regex::new(r"^\w+::(.+?)(?:::tests)?$").unwrap()
    });

    let captures = match use_case {
        UseCase::Integration | UseCase::Benchmark => regex_integration.captures(path),
        UseCase::Unit => regex_unit.captures(path)
    };

    match captures {
        Some(captures) => Some(captures.get(1).unwrap().as_str()),
        None => None
    }
}

impl ModuleNamepath {
    pub fn new(use_case: UseCase, module_path: String) -> Self {
        Self {
            testing_path: String::from(make_testing_path(use_case, &module_path).unwrap()),
            module_path
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub struct GroupNamepath {
    module_path: String,
    name: String,
    path: String,
    testing_path: String
}

impl NamepathTrait for GroupNamepath {
    fn module_path(&self) -> &str {
        &self.module_path
    }

    fn path(&self) -> &str {
        &self.path
    }

    fn testing_path(&self) -> &str {
        &self.testing_path
    }
}

impl GroupNamepath {
    pub fn new(module: &Module, name: String) -> Self {
        let module_path = module.namepath().module_path().to_owned();
        Self {
            path: join(&module_path, &name),
            testing_path: join(make_testing_path(module.use_case, &module_path).unwrap(), &name),
            module_path,
            name
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    fn testing_path(&self) -> &str {
        &self.testing_path
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Hash)]
pub struct TestNamepath {
    module_path: String,
    group_name: Option<String>,
    name: String,
    path: String,
    testing_path: String
}

impl NamepathTrait for TestNamepath {
    fn module_path(&self) -> &str {
        &self.module_path
    }

    fn path(&self) -> &str {
        &self.path
    }

    fn testing_path(&self) -> &str {
        &self.testing_path
    }
}

impl TestNamepath {
    pub fn new(module: &Module, group: Option<&Group>, name: String) -> Self {
        let module_path = module.namepath().module_path().to_owned();
        let group_name;
        let path;
        let testing_path;
        
        match group {
            Some(group) => {
                let grp_name = group.name().to_owned();
                path = join_three(&module_path, &grp_name, &name);
                testing_path = join_three(make_testing_path(module.use_case, &module_path).unwrap(), &grp_name, &name);
                group_name = Some(grp_name);
            },
            None =>  {
                group_name = None;
                path = join(&module_path, &name);
                testing_path = join(make_testing_path(module.use_case, &module_path).unwrap(), &name);
            }
        }

        Self {
            path,
            module_path,
            group_name,
            name ,
            testing_path
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn group_name(&self) -> Option<&str>{
        self.group_name.as_deref()
    }
}

impl Namepath {
    pub fn module(use_case: UseCase, module_path: String) -> Self {
        Self::Module(ModuleNamepath::new(use_case, module_path))
    }

    pub fn group(module: &Module, name: String) -> Self {
        Self::Group(GroupNamepath::new(module, name))
    }

    pub fn test(module: &Module, group: Option<&Group>, name: String) -> Self {
        Self::Test(TestNamepath::new(module, group, name))
    }
}

impl NamepathTrait for Namepath {
    fn module_path(&self) -> &str {
        match self {
            Namepath::Module(module) => module.module_path(),
            Namepath::Group(group) => group.module_path(),
            Namepath::Test(test) => test.module_path(),
        }
    }

    fn path(&self) -> &str {
        match self {
            Namepath::Module(module) => module.path(),
            Namepath::Group(group) => group.path(),
            Namepath::Test(test) => test.path(),
        }
    }

    fn components(&self) -> Vec<&str> {
        match self {
            Namepath::Module(module) => module.components(),
            Namepath::Group(group) => group.components(),
            Namepath::Test(test) => test.components(),
        }
    }

    fn dir(&self) -> PathBuf {
        match self {
            Namepath::Module(module) => module.dir(),
            Namepath::Group(group) => group.dir(),
            Namepath::Test(test) => test.dir(),
        }
    }

    fn testing_path(&self) -> &str {
        match self {
            Namepath::Module(module) => module.testing_path(),
            Namepath::Group(group) => group.testing_path(),
            Namepath::Test(test) => test.testing_path(),
        }
    }
}

mod strings {
    pub const SEPARATOR: &str = "::";
    pub const SQUASH_SEPARATOR: &str = "_";
}

// Splits a namepath by its delimiters
pub fn split(path: &str) -> Vec<&str> {
    path.split(strings::SEPARATOR).into_iter().collect()
}

// Replaces all delimiters with with an underscore
pub fn squash(path: &str) -> String {
    path.replace(strings::SEPARATOR, strings::SQUASH_SEPARATOR)
}

// Creates a Path object representing a namepath as a directory heirarchy
pub fn dir(base_dir: &Path, path: &str) -> PathBuf {
    PathBuf::from(base_dir).join(PathBuf::from_iter(split(path)))
}
// Catencates a preceding namepath with another token: { base_namepath }::{ token }
pub fn join(left: &str, right: &str) -> String { 
    format!("{left}{}{right}", strings::SEPARATOR)
}

// Catencates a preceding namepath with two other tokens token: { base_namepath }::{ token }
pub fn join_three(first: &str, second: &str, third: &str) -> String { 
    format!("{first}{}{second}{}{third}", strings::SEPARATOR, strings::SEPARATOR)
}

pub fn join_all(items: &[&str]) -> String {
    items.join(strings::SEPARATOR)
}


#[cfg(test)]
mod tests {
    use super::*;

    // Should split strings by "::".
    #[test]
    fn test_split() {
        const INPUT: &str = "foo::bar::jar";
        const EXPECTED: [&str;3] = [
            "foo",
            "bar",
            "jar",
        ];

        assert_eq!(EXPECTED, split(INPUT).as_slice(), 
            "Should split strings by '::'.");
    }

    // Should convert "::" into underscores.
    #[test]
    fn test_squash() {
        const INPUT: &str = "foo::bar::jar";
        const EXPECTED: &str = "foo_bar_jar";

        assert_eq!(EXPECTED, squash(INPUT),
            "Should convert '::' into underscores.");

    }

    // Should convert a module path into a directory relative to the specified base.
    #[test]
    fn test_dir() {
        let input_base_dir = std::env::temp_dir();
        const INPUT_PATH: &str = "foo::bar::jar";
        let expected = PathBuf::from(&input_base_dir).join("foo").join("bar").join("jar");

        assert_eq!(expected, dir(&input_base_dir, INPUT_PATH),
            "Should convert a module path into a directory relative to the specified base.");
    }

    // Should join two string using "::".
    #[test]
    fn test_join() {
        const INPUT_LEFT: &str = "foo";
        const INPUT_RIGHT: &str = "bar";
        const EXPECTED: &str = "foo::bar";

        assert_eq!(EXPECTED, join(INPUT_LEFT, INPUT_RIGHT),
            "Should join two strings using '::'.");
    }

    // Should join multiple strings using "::".
    #[test]
    fn test_join_heirarchy() {
        const INPUT_FIRST: &str = "foo";
        const INPUT_SECOND: &str = "bar";
        const INPUT_THIRD: &str = "jar";
        const EXPECTED: &str = "foo::bar::jar";

        assert_eq!(EXPECTED, join_three(INPUT_FIRST, INPUT_SECOND, INPUT_THIRD),
            "Should join two strings using '::'.");
    }
}