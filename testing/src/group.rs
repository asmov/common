use std::{collections::HashMap, path::{PathBuf, Path}};
use anyhow::Context;

use crate::{Testable, Module, TestBuilder, Namepath };

pub struct Group<'module,'func> {
    pub(crate) module: &'module Module,
    pub(crate) namepath: Namepath,
    pub(crate) temp_dir: Option<PathBuf>,
    pub(crate) fixture_dir: Option<PathBuf>,
    pub(crate) imported_fixture_dirs: Option<HashMap<Namepath, PathBuf>>,
    pub(crate) teardown_func: Option<Box<dyn FnOnce(&mut Group) + Sync + Send + 'func>>,
}

impl<'module,'func> Group<'module,'func> {
    pub fn module(&self) -> &Module {
        &self.module
    }

    pub fn test(&self, name: &str) -> TestBuilder {
        TestBuilder::new(&self.module, Some(&self), name)
    }

    pub fn name(&self) -> &str {
        match &self.namepath {
            Namepath::Group(namepath) => &namepath.name(),
            _ => panic!("GroupNamepath")
        }
    }

    pub fn namepath(&self) -> &Namepath {
        &self.namepath
    }

    pub(crate) fn try_imported_fixture_dir(&self, namepath: &Namepath) -> anyhow::Result<&Path> {
        if let Some(imported_fixture_dirs) = self.imported_fixture_dirs.as_ref() {
            if let Some(dir) = imported_fixture_dirs.get(namepath) {
                return Ok(dir.as_path());
            }
        }

        self.module.try_imported_fixture_dir(namepath)
            .context("Group: `imported fixture dirs` is not configured")
    }

    fn teardown(&mut self) {
        if let Some(teardown_func) = self.teardown_func.take() {
            teardown_func(self);
        }

        if let Some(dir) = self.temp_dir.take() {
            if dir.exists() && std::fs::remove_dir_all(&dir).is_err() {
                eprintln!("Unable to delete temp dir: {}", dir.to_str().unwrap());
            }
        }
    }
}

impl<'module, 'func> Testable for Group<'module, 'func> {
    fn fixture_dir(&self) -> &Path {
        &self.fixture_dir.as_ref().context("Group `fixture dir` is not configured").unwrap()
    }
    
    fn imported_fixture_dir(&self, namepath: &Namepath) -> &Path {
        self.try_imported_fixture_dir(namepath).unwrap()
    }

    fn temp_dir(&self) -> &Path {
        self.temp_dir.as_ref().context("Group `temp dir` is not configured").unwrap()
    }
}

impl<'module,'func> Drop for Group<'module,'func> {
    fn drop(&mut self) {
        self.teardown();
    }
}

pub struct GroupBuilder<'module,'func> {
    pub(crate) is_static: bool,
    pub(crate) module: &'module Module,
    pub(crate) name: String,
    pub(crate) using_temp_dir: bool,
    pub(crate) inherit_temp_dir: bool,
    pub(crate) using_fixture_dir: bool,
    pub(crate) inherit_fixture_dir: bool,
    pub(crate) imported_fixture_dirs: Option<HashMap<Namepath, PathBuf>>,
    pub(crate) setup_func: Option<Box<dyn FnOnce(&mut Group) + 'func>>,
    pub(crate) teardown_func: Option<Box<dyn FnOnce(&mut Group) + Sync + Send + 'func>>,
    pub(crate) static_teardown_func: Option<Box<extern fn()>>,
}

impl<'module,'func> GroupBuilder<'module,'func> {
    pub(crate) fn new(module: &'module Module, name: &str, is_static: bool) -> Self {
        debug_assert!(!name.contains(':') && !name.contains('/') && !name.contains('.'),
            "Group name should be a single non-delimited token.");

        Self {
            is_static,
            module: module,
            name: String::from(name), 
            using_temp_dir: false,
            inherit_temp_dir: false,
            using_fixture_dir: false,
            inherit_fixture_dir: false,
            imported_fixture_dirs: None,
            setup_func: None,
            teardown_func: None,
            static_teardown_func: None,
        }
    }

    pub fn build(self) -> Group<'module,'func> {
        let namepath = Namepath::group(&self.module, self.name);

        let temp_dir = if self.using_temp_dir {
            Some(crate::build_temp_dir(&namepath, &self.module.base_temp_dir()))
        } else if self.inherit_temp_dir {
            Some(self.module.temp_dir().to_owned())
        } else {
            None
        };

        let fixture_dir = if self.using_fixture_dir {
            Some(crate::build_fixture_dir(&namepath, self.module.use_case))
        } else if self.inherit_fixture_dir {
            Some(self.module.fixture_dir().to_owned())
        } else {
            None
        };

        let imported_fixture_dirs = self.imported_fixture_dirs;

        let mut group = Group {
            module: self.module,
            namepath: namepath,
            temp_dir,
            fixture_dir,
            imported_fixture_dirs,
            teardown_func: self.teardown_func
        };

        if let Some(setup_func) = self.setup_func {
            setup_func(&mut group);
        }

        if let Some(teardown_fn) = self.static_teardown_func {
            shutdown_hooks::add_shutdown_hook(*teardown_fn);
        }

        group
    }

    pub fn using_temp_dir(mut self) -> Self {
        assert!(!self.inherit_temp_dir);
        if self.module.temp_dir.is_none() {
            panic!("Group cannot use a temporary directory unless its parent Module uses one");
        }

        self.using_temp_dir = true;
        
        self
    }

    pub fn inherit_temp_dir(mut self) -> Self {
        assert!(!self.using_temp_dir);
        if self.module.temp_dir.is_none() {
            panic!("Group cannot use a temporary directory unless its parent Module uses one");
        }

        self.inherit_temp_dir = true;
        self
    }

    pub fn using_fixture_dir(mut self) -> Self {
        assert!(!self.inherit_fixture_dir);
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

    pub fn inherit_fixture_dir(mut self) -> Self {
        assert!(!self.using_fixture_dir);
        self.inherit_fixture_dir = true;
        self
    }

    pub fn setup(mut self, func: impl FnOnce(&mut Group) + 'func) -> Self {
        self.setup_func = Some(Box::new(func));
        self
    }

    pub fn teardown(mut self, func: impl FnOnce(&mut Group) + Sync + Send + 'func) -> Self {
        assert!(!self.is_static, "Static Group must use `teardown_static`");
        self.teardown_func = Some(Box::new(func));
        self
    }

    pub fn teardown_static(mut self, func: extern fn()) -> Self {
        assert!(self.is_static, "Only static Group should use `teardown_static`");
        self.static_teardown_func = Some(Box::new(func));
        self
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use crate::{self as testing, prelude::*, UseCase, NamepathTrait, Namepath, Group};

    static MODULE_BASIC: testing::StaticModule = testing::module(|| {
        testing::unit(module_path!())
            .build()
    });

    static MODULE_WITH_DIRS: testing::StaticModule = testing::module(|| {
        testing::unit(module_path!())
            .using_fixture_dir()
            .using_temp_dir()
            .build()
    });

    #[test] #[named]
    fn test_module() {
        let testgroup = MODULE_BASIC.local_group(function_name!()).build();
        assert_eq!(&*MODULE_BASIC, testgroup.module(),
            "Parent module should be retrievable");
    }

    #[test]
    #[named]
    fn test_name() {
        let testgroup = MODULE_BASIC.local_group(function_name!()).build();
        assert_eq!(function_name!(), testgroup.name(),
            "Name should be awesome");
    }

    // Group name should not contain namepath separator tokens: "::", '/', '.'
    #[test] #[should_panic]
    fn test_name_invalid() {
        MODULE_BASIC.test("foo/bar").build();  // should panic
    }

    // Group namepath should reflect: `Group.module().namepath()` / `Group.name()`
    #[test] #[named]
    fn test_namepath() {
        let expected_namepath = concat!(module_path!(), "::", function_name!());
        let testgroup = MODULE_BASIC.local_group(function_name!()).build();

        assert_eq!(expected_namepath, testgroup.namepath().path(),
            "Group namepath should reflect: `Group.module().namepath()` / `Group.name()`");
    }

    // Group not configured with a temp dir should panic when attempting to access it 
    #[test] #[should_panic] #[named]
    fn test_temp_dir_unconfigured_access() {
        let testgroup = MODULE_BASIC.local_group(function_name!()).build();
        testgroup.temp_dir();  // should panic
    }

    // Group should not allow configuration with `using_temp_dir()` if its parent Module is not using a temp dir.
    #[test] #[should_panic] #[named]
    fn test_temp_dir_using_unconfigured_module() {
        MODULE_BASIC.local_group(function_name!())
            .using_temp_dir()  // should panic
            .build();
    }

    // Group should not allow configuration with `inherit_temp_dir()` if its parent Module is not using a temp dir.
    #[test] #[should_panic] #[named]
    fn test_temp_dir_inherited_unconfigured_module() {
        MODULE_BASIC.local_group(function_name!())
            .inherit_temp_dir()  // should panic
            .build();
    }

    // Group configured with `using_tmp_dir()` should have a temp path of: `Module.tmp_dir()` + `Group.name()`
    // Group configured with `using_temp_dir()` should create the directory on construction if it does not exist.
    #[test] #[named]
    fn test_temp_dir_using() {
        let testgroup = MODULE_WITH_DIRS.local_group(function_name!())
            .using_temp_dir()
            .build();

        assert_eq!(MODULE_WITH_DIRS.temp_dir().join(function_name!()), testgroup.temp_dir(),
            "Group configured with `using_tmp_dir()` should have a temp path of: `Module.tmp_dir()` + `Group.name()`");

        assert!(testgroup.temp_dir().exists(), 
            "Group configured with `using_temp_dir()` should create the directory on construction if it does not exist.");
    }

    // Group configured to `inherit_temp_dir()` should have the same temp path as its parent.
    #[test] #[named]
    fn test_temp_dir_inherited() {
        let testgroup = MODULE_WITH_DIRS.local_group(function_name!())
            .inherit_temp_dir()
            .build();

        assert_eq!(MODULE_WITH_DIRS.temp_dir(), testgroup.temp_dir(),
            "Group configured to `inherit_temp_dir()` should have the same temp path as its parent.");
    }

    // Group not configured with a fixture dir should panic when attempting to access it 
    #[test] #[should_panic] #[named]
    fn test_fixture_dir_unconfigured_access() {
        let testgroup = MODULE_WITH_DIRS.local_group(function_name!()).build();
        testgroup.fixture_dir(); // should panic
    }

    // Group should not allow configuration with `using_fixture_dir()` if its parent Module is not using a fixture dir.
    #[test] #[should_panic] #[named]
    fn test_fixture_dir_using_unconfigured_module() {
        MODULE_BASIC.local_group(function_name!())
            .using_fixture_dir()  // should panic
            .build();
    }

    // Group should not allow configuration with `inherit_fixture_dir()` if its parent Module is not using a fixture dir.
    #[test] #[should_panic] #[named]
    fn test_fixture_dir_inherited_unconfigured_module() {
        MODULE_BASIC.local_group(function_name!())
            .inherit_fixture_dir()  // should panic
            .build();
    }


    // Group configured with `using_fixture_dir()` should have a path of: `Module.fixture_dir()` + `Group.name()`
    // Fixture path should exist for Group configured with `using_fixture_dir()`
     #[test] #[named]
    fn test_fixture_dir_using() {
        let testgroup = MODULE_WITH_DIRS.local_group(function_name!())
            .using_fixture_dir()
            .build();

        assert_eq!(MODULE_WITH_DIRS.fixture_dir().join(function_name!()), testgroup.fixture_dir(),
            "Group configured with `using_fixture_dir()` should have a path of: `Module.fixture_dir()` + `Group.name()`");

        assert!(testgroup.fixture_dir().exists(),
            "Fixture path should exist for Group configured with `using_fixture_dir()`");
    }

    // Group configured to `inherit_fixture_dir()` should have a fixture path that is the same as its Module.
    // Fixture path should exist for Group configured with `inherit_fixture_dir()`
    #[test] #[named]
    fn test_fixture_dir_inherited() {
        let testgroup = MODULE_WITH_DIRS.local_group(function_name!())
            .inherit_fixture_dir()
            .build();

        assert_eq!(MODULE_WITH_DIRS.fixture_dir(), testgroup.fixture_dir(),
            "Group configured to `inherit_fixture_dir()` should have a fixture path that is the same as its Module.");

        assert!(testgroup.fixture_dir().exists(),
            "Fixture path should exist for Group configured with `inherit_fixture_dir()`");
    }

    #[test] #[named]
    fn test_import_fixture_dir() {
        let testgroup = MODULE_BASIC.local_group(function_name!())
            .import_fixture_dir(&MODULE_WITH_DIRS.namepath())
            .build();

        assert_eq!(MODULE_WITH_DIRS.fixture_dir(), testgroup.imported_fixture_dir(MODULE_WITH_DIRS.namepath()),
            "Group should import external fixture dir");
    }

    #[test] #[named] #[should_panic]
    fn test_import_fixture_dir_fail() {
        let testgroup = MODULE_BASIC.local_group(function_name!())
            .build();

        testgroup.imported_fixture_dir(MODULE_WITH_DIRS.namepath()); // should panic
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
        let namepath = unit_module_namepath();
        let test_module = testing::unit(module_path!())
            .import_fixture_dir(&namepath)
            .nonstatic()
            .build();
        let test_group = test_module.local_group(function_name!())
            .build();

        assert_eq!(expected_unit_module_fixture_dir(), test_group.imported_fixture_dir(&namepath),
            "Group should lookup external fixture dir in parent module");
    }

     
    // unsafe: This can only be called once, by `test_setup_function()`. Not thread safe.
    static mut SETUP_FUNC_CALLED: bool = false;
    fn setup_func(_group: &mut Group) {
        unsafe {
            SETUP_FUNC_CALLED = true;
        }
    }

    // Group setup function should be ran on construction.
    #[test] #[named]
    fn test_setup_function() {
        let _testgroup = MODULE_BASIC.local_group(function_name!())
            .setup(setup_func)
            .build();

        unsafe {
            assert!(SETUP_FUNC_CALLED,
                "Group setup function should be ran on construction.");
        }
    }
 
    // Group setup closure should be ran on construction.
    #[test] #[named]
    fn test_setup_closure() {
        let mut setup_closure_called = false;
        MODULE_BASIC.local_group(function_name!())
            .setup(|_| {
                setup_closure_called = true;
            })
            .build();

        assert!(setup_closure_called,
            "Group setup closure should be ran on construction.");
    }
 
    // unsafe: This can only be called once, by `test_setup_function()`. Not thread safe.
    static mut TEARDOWN_FUNC_CALLED: bool = false;
    fn teardown_func(_group: &mut Group) {
        unsafe {
            TEARDOWN_FUNC_CALLED = true;
        }
    }

    // Group teardown function should be ran on destruction.
    #[test] #[named]
    fn test_teardown_function() {
        {
            MODULE_BASIC.local_group(function_name!())
            .teardown(teardown_func)
            .build();
        }

        unsafe {
            assert!(TEARDOWN_FUNC_CALLED,
                "Group teardown function should be ran on destruction.");
        }
    }
 
    // Group teardown closure should be ran on destruction.
    #[test] #[named]
    fn test_teardown_closure() {
        let mut teardown_closure_called = false;
        {
            MODULE_BASIC.local_group(function_name!())
                .teardown(|_| {
                    teardown_closure_called = true;
                })
                .build();
        }

        assert!(teardown_closure_called,
            "Group teardown closure should be ran on destruction.");

    }

    extern fn static_teardown_fn() {
        println!("STATIC_GROUP: {}::teardown_static() ran", STATIC_GROUP.namepath().path())
    }

    static STATIC_GROUP: testing::StaticGroup = testing::group(|| {
        MODULE_BASIC.group("group_basic")
            .teardown_static(static_teardown_fn)
            .build()
    });
   
    // Static Group teardown should be ran on process exist.
    //
    // NOTE: This will always pass and must be visually verified.
    //       The unit test should print the "STATIC GROUP:" line at the end of testing.
    //       An integration test is needed to automating testing of this.
    #[test]
    fn test_teardown_static() {
        STATIC_GROUP.namepath();
    }

    // Group constructed using `Module::local_group()` should not allow static teardown functions.
    #[test] #[named] #[should_panic]
    fn test_teardown_local_static_mismatch() {
        MODULE_BASIC.local_group(function_name!())
            .teardown_static(static_teardown_fn)  // should panic
            .build();
    }

    // Group constructed using `Module::group()` should not allow non-static teardown functions.
    #[test] #[named] #[should_panic]
    fn test_teardown_static_local_mismatch() {
        MODULE_BASIC.group(function_name!())
            .teardown(|_| {}) // should panic
            .build();
    }


}