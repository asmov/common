use std::{collections::HashMap, path::{PathBuf, Path}};
use anyhow::Context;
use crate::{Group, Module, Namepath, Testable};

pub enum Parent<'module,'group,'grpfunc> {
    Module(&'module Module),
    Group(&'group Group<'module,'grpfunc>)
}

impl<'module,'group,'grpfunc> Parent<'module,'group,'grpfunc> {
    pub fn is_module(&self) -> bool {
        match *self {
            Self::Module(_) => true,
            _ => false
        }
    }

    pub fn is_group(&self) -> bool {
        match *self {
            Self::Group(_) => true,
            _ => false
        }
    } 

    pub fn namepath(&self) -> &Namepath {
        match *self {
            Self::Module(module) => module.namepath(), 
            Self::Group(group) => group.namepath()
        }
    } 
}

impl<'module,'group,'grpfnc> Testable for Parent<'group,'module,'grpfnc> {
    fn fixture_dir(&self) -> &Path {
        match *self {
            Self::Module(module) => module.fixture_dir(),
            Self::Group(group) => group.fixture_dir()
        }
    }

    fn imported_fixture_dir(&self, namepath: &Namepath) -> &Path {
        match *self {
            Self::Module(module) => module.imported_fixture_dir(namepath),
            Self::Group(group) => group.imported_fixture_dir(namepath)
        }
    }
    
    fn temp_dir(&self) -> &Path {
        match *self {
            Self::Module(module) => module.temp_dir(),
            Self::Group(group) => group.temp_dir()
        }
    }
}

/// A single test that can be ran.
/// It has a parent [Module] and possibly a parent [Group], from which it may inherit settings from.
pub struct Test<'module,'group,'grpfunc,'func> {
    pub(crate) module: &'module Module,
    pub(crate) namepath: Namepath,
    pub(crate) group: Option<&'group Group<'module,'grpfunc>>,
    pub(crate) temp_dir: Option<PathBuf>,
    pub(crate) fixture_dir: Option<PathBuf>,
    pub(crate) imported_fixture_dirs: Option<HashMap<Namepath, PathBuf>>,
    pub(crate) teardown_func: Option<Box<dyn FnOnce(&mut Test) + 'func>>,
}

impl<'module,'group,'grpfunc,'func> Test<'module,'group,'grpfunc,'func> {
    /// The full namepath of this test, including its parent Module and Group.
    pub fn namepath(&self) -> &Namepath {
        &self.namepath
    }

    /// The name of this test. Usually this is the function name that created it.
    pub fn name(&self) -> &str {
        match &self.namepath {
            Namepath::Test(namepath) => namepath.name(),
            _ => panic!("Namepath::Test")
        }
    }

    /// The parent Module of this test.
    pub fn module(&self) -> &'module Module {
        &self.module
    }

    /// The parent Group of this test, if it has one.
    pub fn group(&self) -> Option<&'module Group> {
        self.group.as_deref()
    }

    /// The most direct parent of this test: [Group], if it has one, otherwise [Module].
    pub fn parent(&self) -> Parent {
        match self.group {
            Some(group) => Parent::Group(&group),
            None => Parent::Module(&self.module)
        }
    }

    pub(crate) fn try_imported_fixture_dir(&self, namepath: &Namepath) -> anyhow::Result<&Path> {
        if let Some(imported_fixture_dirs) = self.imported_fixture_dirs.as_ref() {
            if let Some(dir) = imported_fixture_dirs.get(namepath) {
                return Ok(dir.as_path());
            }
        }

        let parent_result = match self.group {
            Some(group) => group.try_imported_fixture_dir(namepath),
            None => self.module.try_imported_fixture_dir(namepath)
        };

        parent_result.context("Test: `imported fixture dirs` is not configured")
    }

    fn teardown(&mut self) {
        if let Some(teardown_fn) = self.teardown_func.take() {
            teardown_fn(self);
        }

        if let Some(dir) = self.temp_dir.take() {
            if dir.exists() && std::fs::remove_dir_all(&dir).is_err() {
                eprintln!("Unable to delete temp dir: {}", dir.to_str().unwrap());
            }
        }

    }
}

impl<'module,'group,'grpfunc,'func> Testable for Test<'module,'group,'grpfunc,'func> {
    fn fixture_dir(&self) -> &Path {
        &self.fixture_dir.as_ref().context("Test `fixture dir` is not configured").unwrap()
    }

    fn imported_fixture_dir(&self, namepath: &Namepath) -> &Path {
        self.try_imported_fixture_dir(namepath).unwrap()
    }

    fn temp_dir(&self) -> &Path {
        self.temp_dir.as_ref().context("Test `temp dir` is not configured").unwrap()
    }
}

impl<'module,'group,'grpfunc,'func> Drop for Test<'module,'group,'grpfunc,'func> {
    fn drop(&mut self) {
        self.teardown();
    }
}

/// Builder for a Test.
pub struct TestBuilder<'module,'group,'grpfunc,'func> {
    pub(crate) name: String,
    pub(crate) module: &'module Module,
    pub(crate) group: Option<&'group Group<'module,'grpfunc>>,
    pub(crate) using_temp_dir: bool,
    pub(crate) inherit_temp_dir: bool,
    pub(crate) using_fixture_dir: bool,
    pub(crate) inherit_fixture_dir: bool,
    pub(crate) imported_fixture_dirs: Option<HashMap<Namepath, PathBuf>>,
    pub(crate) setup_func: Option<Box<dyn FnOnce(&mut Test) + 'func>>,
    pub(crate) teardown_func: Option<Box<dyn FnOnce(&mut Test) + 'func>>,
}

impl<'module,'group,'grpfunc,'func>
TestBuilder<'module,'group,'grpfunc,'func> {
    pub(crate) fn new(module: &'module Module, group: Option<&'group Group<'module,'grpfunc>>, name: &str) -> Self{
        debug_assert!(!name.contains("::") && !name.contains('/') && !name.contains('.'),
            "Test name should be a single non-delimited token.");

        Self {
            name: name.to_owned(),
            module, 
            group,
            using_temp_dir: false,
            inherit_temp_dir: false,
            using_fixture_dir: false,
            inherit_fixture_dir: false,
            imported_fixture_dirs: None,
            setup_func: None,
            teardown_func: None,
        }
    }

    /// Builds the test and initializes it.
    pub fn build(self) -> Test<'module,'group,'grpfunc,'func> {
        let namepath = Namepath::test(&self.module, self.group, self.name);

        let temp_dir = if self.using_temp_dir {
            Some(crate::build_temp_dir(&namepath, &self.module.base_temp_dir()))
        } else if self.inherit_temp_dir {
            Some(match self.group {
                Some(group) => group.temp_dir().to_owned(),
                None => self.module.temp_dir().to_owned() })
        } else {
            None
        };

        let fixture_dir = if self.using_fixture_dir {
            Some(crate::build_fixture_dir(&namepath, self.module.use_case))
        } else if self.inherit_fixture_dir {
            Some( match self.group {
                Some(group) => group.fixture_dir().to_owned(),
                None => self.module.fixture_dir().to_owned() })
        } else {
            None
        };

        let imported_fixture_dirs = self.imported_fixture_dirs;

        let mut test = Test {
            module: self.module,
            namepath,
            group: self.group,
            temp_dir,
            fixture_dir,
            imported_fixture_dirs,
            teardown_func: self.teardown_func,
        };

        if let Some(setup_fn) = self.setup_func {
            setup_fn(&mut test);
        }

        test
    }

    /// Configures this test to use an existing fixture directory.
    /// The base path is defined by the parent Module or Group, with an existing subdirectory expected to be the name of this test.
    pub fn using_fixture_dir(mut self) -> Self {
        assert!(!self.inherit_fixture_dir, "Configuring both `inherit` and `using` for `fixture_dir` is ambiguous");
        self.using_fixture_dir = true;
        self
    }

    pub fn import_fixture_dir(mut self, namepath: &Namepath) -> Self {
        let dir = crate::build_fixture_dir(&namepath, self.module.use_case);
        let dir = dir.canonicalize()
            .context(format!("Imported fixture dir does not exist: {}", &dir.to_str().unwrap()))
            .unwrap();

        if self.imported_fixture_dirs.is_none() {
            self.imported_fixture_dirs = Some(HashMap::new());
        }

        self.imported_fixture_dirs.as_mut().expect("Option should exist")
            .insert(namepath.to_owned(), dir);
        
        self
    }

    /// Configures the test to use a temporary directory.  
    /// The base path is defined by the parent Module or Group, with a subdirectory created just for this test (by its name).  
    pub fn using_temp_dir(mut self) -> Self {
        assert!(!self.inherit_temp_dir);
        if self.module.temp_dir.is_none() {
            panic!("Test cannot use a temporary directory unless its parent Module uses one");
        } else if let Some(group) = self.group {
            if group.temp_dir.is_none() {
                panic!("Test cannot use a temporary directory unless its parent Group uses one");
            }
        }

        self.using_temp_dir = true;
        self
    }

    /// Configures the test to use the exact same temporary directory as its parent Module or Group.
    /// A separate subdirectory will not be created for this test.
    pub fn inherit_temp_dir(mut self) -> Self {
        assert!(!self.using_temp_dir);
        if self.module.temp_dir.is_none() {
            panic!("Test cannot use a temporary directory unless its parent Module uses one");
        } else if let Some(group) = self.group {
            if group.temp_dir.is_none() {
                panic!("Test cannot inherit a temporary directory unless its parent Group uses one");
            }
        }

        self.inherit_temp_dir = true;
        self
    }

    /// Configures the test to use the exact same fixture directory as its parent Module or Group.
    /// A separate subdirectory for this test is not expected to exist.
    pub fn inherit_fixture_dir(mut self) -> Self {
        assert!(!self.using_fixture_dir);
        self.inherit_fixture_dir = true;
        self
    }

    /// Calls the provided function once on construction of the test.
    pub fn setup(mut self, func: impl FnOnce(&mut Test) + 'func) -> Self {
        self.setup_func = Some(Box::new(func));
        self
    }

    /// Calls the provided function once on destruction of the test.
    pub fn teardown(mut self, func: impl FnOnce(&mut Test) + 'func) -> Self {
        self.teardown_func = Some(Box::new(func));
        self
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use crate::{self as testing, prelude::*, NamepathTrait, Test, Namepath, UseCase};

    static MODULE_BASIC: testing::StaticModule = testing::module(|| {
        testing::unit(module_path!())
            .build()
    });

    static GROUP_BASIC: testing::StaticGroup = testing::group(|| {
        MODULE_WITH_DIRS.group("group_basic")
            .build()
    });

    static MODULE_WITH_DIRS: testing::StaticModule = testing::module(|| {
        testing::unit(module_path!())
            .using_fixture_dir()
            .using_temp_dir()
            .build()
    });

    static GROUP_WITH_DIRS: testing::StaticGroup = testing::group(|| {
        MODULE_WITH_DIRS.group("group_with_dirs")
            .using_fixture_dir()
            .using_temp_dir()
            .build()
    });

    // Test parent Module should be bound.
    #[test] #[named]
    fn test_module() {
        let test = MODULE_BASIC.test(function_name!()).build();
        assert_eq!(&*MODULE_BASIC, test.module(),
            "Test parent Module should be bound.");
    }

    // Test parent Group should be bound.
    #[test] #[named]
    fn test_group() {
        let test = GROUP_BASIC.test(function_name!()).build();
        assert_eq!(GROUP_BASIC.namepath(), test.group().unwrap().namepath(),
            "Test parent Group should be bound.");
    }

    // Test name should be set.
    #[test] #[named]
    fn test_name() {
        let test = MODULE_BASIC.test(function_name!()).build();
        assert_eq!(function_name!(), test.name(),
            "Test name should be set.");
    }
    
    // Test name should not contain namepath separator tokens: "::", '/', '.'
    #[test] #[should_panic]
    fn test_name_invalid() {
        MODULE_BASIC.test("foo.bar").build();  // should panic
    }

    // Test with only a parent Module should have a namepath of: `Test::module().namepath()` / `Test::name()`
    // Test with a parent Group should have a namepath of: `Test::group().namepath()` / `Test::name()`
    #[test] #[named]
    fn test_namepath() {
        let expected_namepath_module = concat!(module_path!(), "::", function_name!());
        let test = MODULE_BASIC.test(function_name!()).build();

        assert_eq!(expected_namepath_module, test.namepath().path(),
            "Test with only a parent Module should have a namepath of: `Test::module().namepath()` / `Test::name()`");

        let expected_namepath_group = concat!(module_path!(), "::", "group_basic", "::", function_name!());
        let test = GROUP_BASIC.test(function_name!()).build();

        assert_eq!(expected_namepath_group, test.namepath().path(),
            "Test with a parent Group should have a namepath of: `Test::group().namepath()` / `Test::name()`");
    }

    // Test not configured with a temp dir should panic when attempting to access it 
    #[test] #[should_panic] #[named]
    fn test_temp_dir_unconfigured_access() {
        MODULE_BASIC.test(function_name!())
            .build()
            .temp_dir();  // should panic
    }

    // Test should not allow configuration with `using_temp_dir()` if its parent Module is not using a temp dir.
    #[test] #[should_panic] #[named]
    fn test_temp_dir_using_unconfigured_module() {
        MODULE_BASIC.test(function_name!())
            .using_temp_dir()  // should panic
            .build();
    }

    // Test should not allow configuration with `using_temp_dir()` if its parent Group is not using a temp dir.
    #[test] #[should_panic] #[named]
    fn test_temp_dir_using_unconfigured_group() {
        GROUP_BASIC.test(function_name!())
            .using_temp_dir()  // should panic
            .build();
    }

    // Test should not allow configuration with `inherit_temp_dir()` if its parent Module is not using a temp dir.
    #[test] #[should_panic] #[named]
    fn test_temp_dir_inherited_unconfigured_module() {
        MODULE_BASIC.test(function_name!())
            .inherit_temp_dir()  // should panic
            .build();
    }

    // Test should not allow configuration with `inherit_temp_dir()` if its parent Group is not using a temp dir.
    #[test] #[should_panic] #[named]
    fn test_temp_dir_inherited_unconfigured_group() {
        GROUP_BASIC.test(function_name!())
            .inherit_temp_dir()  // should panic
            .build();
    }

    // Test configured with `using_tmp_dir()` should have a temp path of: `Module.tmp_dir()` + `Test.name()`
    // Test configured with `using_temp_dir()` should create the directory on construction if it does not exist.
    #[test] #[named]
    fn test_temp_dir_using() {
        let test = MODULE_WITH_DIRS.test(function_name!())
            .using_temp_dir()
            .build();

        assert_eq!(MODULE_WITH_DIRS.temp_dir().join(function_name!()), test.temp_dir(),
            "Test configured with `using_tmp_dir()` should have a temp path of: `Module.tmp_dir()` + `Test.name()`");

        assert!(test.temp_dir().exists(), 
            "Test configured with `using_temp_dir()` should create the directory on construction if it does not exist.");
    }

    // Test configured to `inherit_temp_dir()` should have the same temp path as its parent.
    #[test] #[named]
    fn test_temp_dir_inherited() {
        let test = MODULE_WITH_DIRS.test(function_name!())
            .inherit_temp_dir()
            .build();

        assert_eq!(MODULE_WITH_DIRS.temp_dir(), test.temp_dir(),
            "Test configured to `inherit_temp_dir()` should have the same temp path as its parent.");
    }

    // Test not configured with a fixture dir should panic when attempting to access it 
    #[test] #[should_panic] #[named]
    fn test_fixture_dir_unconfigured_access() {
        MODULE_WITH_DIRS.test(function_name!())
            .build()
            .fixture_dir(); // should panic
    }

    // Test should not allow configuration with `using_fixture_dir()` if its parent Module is not using a fixture dir.
    #[test] #[should_panic] #[named]
    fn test_fixture_dir_using_unconfigured_module() {
        MODULE_BASIC.test(function_name!())
            .using_fixture_dir()  // should panic
            .build();
    }

    // Test should not allow configuration with `using_fixture_dir()` if its parent Group is not using a fixture dir.
    #[test] #[should_panic] #[named]
    fn test_fixture_dir_using_unconfigured_group() {
        GROUP_BASIC.test(function_name!())
            .using_fixture_dir()  // should panic
            .build();
    }

    // Test should not allow configuration with `inherit_fixture_dir()` if its parent Module is not using a fixture dir.
    #[test] #[should_panic] #[named]
    fn test_fixture_dir_inherited_unconfigured_module() {
        MODULE_BASIC.test(function_name!())
            .inherit_fixture_dir()  // should panic
            .build();
    }

    // Test should not allow configuration with `inherit_fixture_dir()` if its parent Group is not using a fixture dir.
    #[test] #[should_panic] #[named]
    fn test_fixture_dir_inherited_unconfigured_group() {
        GROUP_BASIC.test(function_name!())
            .inherit_fixture_dir()  // should panic
            .build();
    }


    // Test configured with `using_fixture_dir()` should have a path of: `Module::fixture_dir()` + `Test::name()`
    // Fixture path should exist for Test configured as `using_fixture_dir()` with a parent Module.
    // Test configured with `using_fixture_dir()` should have a path of: `Group::fixture_dir()` + `Test::name()`
    // Fixture path should exist for Test configured as `using_fixture_dir()` with a parent Module.
     #[test] #[named]
    fn test_fixture_dir_using() {
        let test = MODULE_WITH_DIRS.test(function_name!())
            .using_fixture_dir()
            .build();

        assert_eq!(MODULE_WITH_DIRS.fixture_dir().join(function_name!()), test.fixture_dir(),
            "Test configured with `using_fixture_dir()` should have a path of: `Module::fixture_dir()` + `Test::name()`");
        assert!(test.fixture_dir().exists(),
            "Fixture path should exist for Test configured as `using_fixture_dir()`");

        let test = GROUP_WITH_DIRS.test(function_name!())
            .using_fixture_dir()
            .build();

        assert_eq!(GROUP_WITH_DIRS.fixture_dir().join(function_name!()), test.fixture_dir(),
            "Test configured with `using_fixture_dir()` should have a path of: `Group::fixture_dir()` + `Test::name()`");
        assert!(test.fixture_dir().exists(),
            "Fixture path should exist for Test configured as `using_fixture_dir()`");
 
    }

    // Test configured to `inherit_fixture_dir()` should have a fixture path that is the same as its Module.
    // Fixture path should exist for Test configured to `inherit_fixture_dir()` from Module
    // Test configured to `inherit_fixture_dir()` should have a fixture path that is the same as its Group.
    // Fixture path should exist for Test configured to `inherit_fixture_dir()` from Group
    #[test] #[named]
    fn test_fixture_dir_inherited() {
        let test = MODULE_WITH_DIRS.test(function_name!())
            .inherit_fixture_dir()
            .build();

        assert_eq!(MODULE_WITH_DIRS.fixture_dir(), test.fixture_dir(),
            "Test configured to `inherit_fixture_dir()` should have a fixture path that is the same as its Module.");
        assert!(test.fixture_dir().exists(),
            "Fixture path should exist for Test configured to `inherit_fixture_dir()` from Module");

        let test = GROUP_WITH_DIRS.test(function_name!())
            .inherit_fixture_dir()
            .build();

        assert_eq!(GROUP_WITH_DIRS.fixture_dir(), test.fixture_dir(),
            "Test configured to `inherit_fixture_dir()` should have a fixture path that is the same as its Module.");
        assert!(test.fixture_dir().exists(),
            "Fixture path should exist for Test configured to `inherit_fixture_dir()` from Module");
    }

    #[test] #[named]
    fn test_import_fixture_dir() {
        let test = MODULE_BASIC.test(function_name!())
            .import_fixture_dir(&MODULE_WITH_DIRS.namepath())
            .build();

        assert_eq!(MODULE_WITH_DIRS.fixture_dir(), test.imported_fixture_dir(MODULE_WITH_DIRS.namepath()),
            "Test should import external fixture dir");
    }

    #[test] #[named] #[should_panic]
    fn test_import_fixture_dir_fail() {
        let test = MODULE_BASIC.test(function_name!())
            .build();

        test.imported_fixture_dir(MODULE_WITH_DIRS.namepath()); // should panic
    }

    fn unit_module_namepath() -> Namepath {
        Namepath::module(UseCase::Unit, "asmov_testing::module".to_string())
    }

    fn expected_unit_module_fixture_dir() -> PathBuf {
        PathBuf::from(crate::strings::TESTING).join(crate::strings::FIXTURES)
            .join(UseCase::Unit.to_str())
            .join("module")
            .canonicalize()
            .unwrap()
    }

    #[test] #[named]
    fn test_module_lookup_imported_fixture_dir() {
        let expected_unit_module_fixture_dir = expected_unit_module_fixture_dir();
        let namepath = unit_module_namepath();
        let test_module = testing::unit(module_path!())
            .import_fixture_dir(&namepath)
            .nonstatic()
            .build();
        let test_group = test_module.local_group(function_name!()).build();
        let test = test_group.test(function_name!()).build();

        assert_eq!(expected_unit_module_fixture_dir, test.imported_fixture_dir(&namepath),
            "Test should lookup external fixture dir in parent module");
    }

    #[test] #[named]
    fn test_group_lookup_imported_fixture_dir() {
        let expected_unit_module_fixture_dir = expected_unit_module_fixture_dir();
        let namepath = unit_module_namepath();
        let test_module = testing::unit(module_path!())
            .nonstatic()
            .build();
        let test_group = test_module.local_group(function_name!())
            .import_fixture_dir(&namepath)
            .build();
        let test = test_group.test(function_name!()).build();

        assert_eq!(expected_unit_module_fixture_dir, test.imported_fixture_dir(&namepath),
            "Test should lookup external fixture dir in parent group");
    }


    // Test `parent()` should return its Module if configured without a Group.
    // Test `parent()` should return its Group if configured with one. 
    #[test] #[named]
    fn test_parent() {
        let test = MODULE_BASIC.test(function_name!()).build();

        assert!(test.parent().is_module(),
            "Test `parent()` should return its Module if configured without a Group.");
        assert_eq!(MODULE_BASIC.namepath(), test.parent().namepath(),
            "Test `parent()` should return its Module if configured without a Group.");

        let test = GROUP_BASIC.test(function_name!()).build();

        assert!(test.parent().is_group(),
            "Test `parent()` should return its Group if configured with one.");
         assert_eq!(GROUP_BASIC.namepath(), test.parent().namepath(),
            "Test `parent()` should return its Group if configured with one.");
    }

    // unsafe: This can only be called once, by `test_setup_function()`. Not thread safe.
    static mut SETUP_FUNC_CALLED: bool = false;
    fn setup_func(_test: &mut Test) {
        unsafe {
            SETUP_FUNC_CALLED = true;
        }
    }

    // Test setup function should be ran on construction.
    #[test] #[named]
    fn test_setup_function() {
        let _testgroup = GROUP_BASIC.test(function_name!())
            .setup(setup_func)
            .build();

        unsafe {
            assert!(SETUP_FUNC_CALLED,
                "Test setup function should be ran on construction.");
        }
    }
 
    // Test setup closure should be ran on construction.
    #[test] #[named]
    fn test_setup_closure() {
        let mut setup_closure_called = false;
        GROUP_BASIC.test(function_name!())
            .setup(|_| {
                setup_closure_called = true;
            })
            .build();

        assert!(setup_closure_called,
            "Test setup closure should be ran on construction.");
    }
 
    // unsafe: This can only be called once, by `test_setup_function()`. Not thread safe.
    static mut TEARDOWN_FUNC_CALLED: bool = false;
    fn teardown_func(_group: &mut Test) {
        unsafe {
            TEARDOWN_FUNC_CALLED = true;
        }
    }

    // Test teardown function should be ran on destruction.
    #[test] #[named]
    fn test_teardown_function() {
        {
            GROUP_BASIC.test(function_name!())
            .teardown(teardown_func)
            .build();
        }

        unsafe {
            assert!(TEARDOWN_FUNC_CALLED,
                "Test teardown function should be ran on destruction.");
        }
    }
 
    // Test teardown closure should be ran on destruction.
    #[test] #[named]
    fn test_teardown_closure() {
        let mut teardown_closure_called = false;
        {
            GROUP_BASIC.test(function_name!())
                .teardown(|_| {
                    teardown_closure_called = true;
                })
                .build();
        }

        assert!(teardown_closure_called,
            "Test teardown closure should be ran on destruction.");
    }
}
 
