//! Extensions:
//! - Provide additional resources and functionality.
//! - Attach to Groups.
//! - Have extendable setup and teardown phases.
//! - May have dependencies to other Extensions.
//!   + They do not have `inherit_` functionality.
//! - May have temporary and fixture directories.
//! - May have TOML configuration.
//!   + A crate-wide config and a group-specific config. Complete-replace.
//!     * `testing/config/adapters/{Extension publisher}--{Extension name}`
//!     * `testing/config/{Group testing namepath}/{Extension publisher}--{Extension name}
//! - Have event handling for test setup and teardown.

use std::path::{PathBuf, Path};
use anyhow::{Context};

use crate::{Group, Namepath};

/// Models an extension, which implements the `ExtensionTrait`.
pub struct Extension<'module, 'group,'grpfunc,'func> {
    pub(crate) group: &'group Group<'module,'grpfunc>,
    pub(crate) namepath: Namepath,
    pub(crate) temp_dir: Option<PathBuf>,
    pub(crate) fixture_dir: Option<PathBuf>,
    pub(crate) teardown_func: Option<Box<dyn FnOnce(&mut Extension) + 'func>>,
}

impl<'module,'group,'grpfunc,'func> Extension<'module,'group,'grpfunc,'func> {
    pub fn namepath(&self) -> &Namepath {
        &self.namepath
    }

    pub fn name(&self) -> &str {
        match &self.namepath {
            Namepath::Extension(namepath) => namepath.name(),
            _ => panic!("Namepath::Extension")
        }
    }

    pub fn group(&self) -> &'module Group {
        self.group
    }

    pub fn temp_dir(&self) -> &Path {
        &self.temp_dir.as_ref().context("Extension `temp dir` is not configured").unwrap()
    }

    pub fn fixture_dir(&self) -> &Path {
        &self.fixture_dir.as_ref().context("Extension `fixture dir` is not configured").unwrap()
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

impl<'module,'group,'grpfunc,'func> Drop for Extension<'module,'group,'grpfunc,'func> {
    fn drop(&mut self) {
        self.teardown();
    }
}

pub struct ExtensionBuilder<'module,'group,'grpfunc,'func> {
    pub(crate) publisher: String,
    pub(crate) name: String,
    pub(crate) group: &'group Group<'module,'grpfunc>,
    pub(crate) using_temp_dir: bool,
    pub(crate) using_fixture_dir: bool,
    pub(crate) setup_func: Option<Box<dyn FnOnce(&mut Extension) + 'func>>,
    pub(crate) teardown_func: Option<Box<dyn FnOnce(&mut Extension) + 'func>>,
}

impl<'module,'group,'grpfunc,'func>
ExtensionBuilder<'module,'group,'grpfunc,'func> {
    pub(crate) fn new(group: &'group Group<'module,'grpfunc>, publisher: String, name: String) -> Self{
        debug_assert!(!name.contains(':') && !name.contains('/') && !name.contains('.'),
            "Extension name should be a single non-delimited token.");

        Self {
            publisher,
            name,
            group,
            using_temp_dir: false,
            using_fixture_dir: false,
            setup_func: None,
            teardown_func: None,
        }
    }

    pub fn build(self) -> Extension<'module,'group,'grpfunc,'func> {
        let namepath = Namepath::extension(&self.group, self.publisher.to_owned(), self.name.to_owned());

        let temp_dir = if self.using_temp_dir {
            Some( crate::build_temp_dir(&namepath, &self.group.temp_dir()) )
        } else {
            None
        };

        let fixture_dir = if self.using_fixture_dir {
            Some( crate::build_fixture_dir(&namepath, &self.group.module().use_case()) )
        } else {
            None
        };

        let mut extension = Extension {
            group: self.group,
            namepath,
            temp_dir,
            fixture_dir,
            teardown_func: self.teardown_func,
        };

        if let Some(setup_fn) = self.setup_func {
            setup_fn(&mut extension);
        }

        extension
    }

    pub fn using_fixture_dir(mut self) -> Self {
        self.using_fixture_dir = true;
        self
    }

    pub fn using_temp_dir(mut self) -> Self {
        assert!(self.group.temp_dir.is_some(),
            "Extension cannot use a temporary directory unless its parent Group uses one");

        self.using_temp_dir = true;
        self
    }

    pub fn setup(mut self, func: impl FnOnce(&mut Extension) + 'func) -> Self {
        self.setup_func = Some(Box::new(func));
        self
    }

    pub fn teardown(mut self, func: impl FnOnce(&mut Extension) + 'func) -> Self {
        self.teardown_func = Some(Box::new(func));
        self
    }
}

pub trait ExtensionTrait: Send + Sync {
    fn extension_path() -> &'static str where Self: Sized;
    fn extension_path_self(&self) -> &'static str;
}

#[cfg(test)]
mod tests {
    use crate::{self as testing, NamepathTrait, Extension, extension::ExtensionTrait};
    use function_name::named;

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

    // Extension parent Module should be bound.
    #[test] #[named]
    fn test_module() {
        let test = MODULE_BASIC.test(function_name!()).build();
        assert_eq!(&*MODULE_BASIC, test.module(),
            "Extension parent Module should be bound.");
    }

    // Extension parent Group should be bound.
    #[test] #[named]
    fn test_group() {
        let test = GROUP_BASIC.test(function_name!()).build();
        assert_eq!(GROUP_BASIC.namepath(), test.group().unwrap().namepath(),
            "Extension parent Group should be bound.");
    }

    // Extension name should be set.
    #[test] #[named]
    fn test_name() {
        let test = MODULE_BASIC.test(function_name!()).build();
        assert_eq!(function_name!(), test.name(),
            "Extension name should be set.");
    }
    
    // Extension name should not contain namepath separator tokens: "::", '/', '.'
    #[test] #[should_panic]
    fn test_name_invalid() {
        MODULE_BASIC.test("foo.bar").build();  // should panic
    }

    // Extension with only a parent Module should have a namepath of: `Extension::module().namepath()` / `Extension::name()`
    // Extension with a parent Group should have a namepath of: `Extension::group().namepath()` / `Extension::name()`
    #[test] #[named]
    fn test_namepath() {
        let expected_namepath_module = concat!(module_path!(), "::", function_name!());
        let test = MODULE_BASIC.test(function_name!()).build();

        assert_eq!(expected_namepath_module, test.namepath().path(),
            "Extension with only a parent Module should have a namepath of: `Extension::module().namepath()` / `Extension::name()`");

        let expected_namepath_group = concat!(module_path!(), "::", "group_basic", "::", function_name!());
        let test = GROUP_BASIC.test(function_name!()).build();

        assert_eq!(expected_namepath_group, test.namepath().path(),
            "Extension with a parent Group should have a namepath of: `Extension::group().namepath()` / `Extension::name()`");
    }

    // Extension not configured with a temp dir should panic when attempting to access it 
    #[test] #[should_panic] #[named]
    fn test_temp_dir_unconfigured_access() {
        MODULE_BASIC.test(function_name!())
            .build()
            .temp_dir();  // should panic
    }

    // Extension should not allow configuration with `using_temp_dir()` if its parent Module is not using a temp dir.
    #[test] #[should_panic] #[named]
    fn test_temp_dir_using_unconfigured_module() {
        MODULE_BASIC.test(function_name!())
            .using_temp_dir()  // should panic
            .build();
    }

    // Extension should not allow configuration with `using_temp_dir()` if its parent Group is not using a temp dir.
    #[test] #[should_panic] #[named]
    fn test_temp_dir_using_unconfigured_group() {
        GROUP_BASIC.test(function_name!())
            .using_temp_dir()  // should panic
            .build();
    }

    // Extension should not allow configuration with `inherit_temp_dir()` if its parent Module is not using a temp dir.
    #[test] #[should_panic] #[named]
    fn test_temp_dir_inherited_unconfigured_module() {
        MODULE_BASIC.test(function_name!())
            .inherit_temp_dir()  // should panic
            .build();
    }

    // Extension should not allow configuration with `inherit_temp_dir()` if its parent Group is not using a temp dir.
    #[test] #[should_panic] #[named]
    fn test_temp_dir_inherited_unconfigured_group() {
        GROUP_BASIC.test(function_name!())
            .inherit_temp_dir()  // should panic
            .build();
    }

    // Extension configured with `using_tmp_dir()` should have a temp path of: `Module.tmp_dir()` + `Extension.name()`
    // Extension configured with `using_temp_dir()` should create the directory on construction if it does not exist.
    #[test] #[named]
    fn test_temp_dir_using() {
        let test = MODULE_WITH_DIRS.test(function_name!())
            .using_temp_dir()
            .build();

        assert_eq!(MODULE_WITH_DIRS.temp_dir().join(function_name!()), test.temp_dir(),
            "Extension configured with `using_tmp_dir()` should have a temp path of: `Module.tmp_dir()` + `Extension.name()`");

        assert!(test.temp_dir().exists(), 
            "Extension configured with `using_temp_dir()` should create the directory on construction if it does not exist.");
    }

    // Extension configured to `inherit_temp_dir()` should have the same temp path as its parent.
    #[test] #[named]
    fn test_temp_dir_inherited() {
        let test = MODULE_WITH_DIRS.test(function_name!())
            .inherit_temp_dir()
            .build();

        assert_eq!(MODULE_WITH_DIRS.temp_dir(), test.temp_dir(),
            "Extension configured to `inherit_temp_dir()` should have the same temp path as its parent.");
    }

    // Extension not configured with a fixture dir should panic when attempting to access it 
    #[test] #[should_panic] #[named]
    fn test_fixture_dir_unconfigured_access() {
        MODULE_WITH_DIRS.test(function_name!())
            .build()
            .fixture_dir(); // should panic
    }

    // Extension should not allow configuration with `using_fixture_dir()` if its parent Module is not using a fixture dir.
    #[test] #[should_panic] #[named]
    fn test_fixture_dir_using_unconfigured_module() {
        MODULE_BASIC.test(function_name!())
            .using_fixture_dir()  // should panic
            .build();
    }

    // Extension should not allow configuration with `using_fixture_dir()` if its parent Group is not using a fixture dir.
    #[test] #[should_panic] #[named]
    fn test_fixture_dir_using_unconfigured_group() {
        GROUP_BASIC.test(function_name!())
            .using_fixture_dir()  // should panic
            .build();
    }

    // Extension should not allow configuration with `inherit_fixture_dir()` if its parent Module is not using a fixture dir.
    #[test] #[should_panic] #[named]
    fn test_fixture_dir_inherited_unconfigured_module() {
        MODULE_BASIC.test(function_name!())
            .inherit_fixture_dir()  // should panic
            .build();
    }

    // Extension should not allow configuration with `inherit_fixture_dir()` if its parent Group is not using a fixture dir.
    #[test] #[should_panic] #[named]
    fn test_fixture_dir_inherited_unconfigured_group() {
        GROUP_BASIC.test(function_name!())
            .inherit_fixture_dir()  // should panic
            .build();
    }


    // Extension configured with `using_fixture_dir()` should have a path of: `Module::fixture_dir()` + `Extension::name()`
    // Fixture path should exist for Extension configured as `using_fixture_dir()` with a parent Module.
    // Extension configured with `using_fixture_dir()` should have a path of: `Group::fixture_dir()` + `Extension::name()`
    // Fixture path should exist for Extension configured as `using_fixture_dir()` with a parent Module.
     #[test] #[named]
    fn test_fixture_dir_using() {
        let test = MODULE_WITH_DIRS.test(function_name!())
            .using_fixture_dir()
            .build();

        assert_eq!(MODULE_WITH_DIRS.fixture_dir().join(function_name!()), test.fixture_dir(),
            "Extension configured with `using_fixture_dir()` should have a path of: `Module::fixture_dir()` + `Extension::name()`");
        assert!(test.fixture_dir().exists(),
            "Fixture path should exist for Extension configured as `using_fixture_dir()`");

        let test = GROUP_WITH_DIRS.test(function_name!())
            .using_fixture_dir()
            .build();

        assert_eq!(GROUP_WITH_DIRS.fixture_dir().join(function_name!()), test.fixture_dir(),
            "Extension configured with `using_fixture_dir()` should have a path of: `Group::fixture_dir()` + `Extension::name()`");
        assert!(test.fixture_dir().exists(),
            "Fixture path should exist for Extension configured as `using_fixture_dir()`");
 
    }

    // Extension configured to `inherit_fixture_dir()` should have a fixture path that is the same as its Module.
    // Fixture path should exist for Extension configured to `inherit_fixture_dir()` from Module
    // Extension configured to `inherit_fixture_dir()` should have a fixture path that is the same as its Group.
    // Fixture path should exist for Extension configured to `inherit_fixture_dir()` from Group
    #[test] #[named]
    fn test_fixture_dir_inherited() {
        let test = MODULE_WITH_DIRS.test(function_name!())
            .inherit_fixture_dir()
            .build();

        assert_eq!(MODULE_WITH_DIRS.fixture_dir(), test.fixture_dir(),
            "Extension configured to `inherit_fixture_dir()` should have a fixture path that is the same as its Module.");
        assert!(test.fixture_dir().exists(),
            "Fixture path should exist for Extension configured to `inherit_fixture_dir()` from Module");

        let test = GROUP_WITH_DIRS.test(function_name!())
            .inherit_fixture_dir()
            .build();

        assert_eq!(GROUP_WITH_DIRS.fixture_dir(), test.fixture_dir(),
            "Extension configured to `inherit_fixture_dir()` should have a fixture path that is the same as its Module.");
        assert!(test.fixture_dir().exists(),
            "Fixture path should exist for Extension configured to `inherit_fixture_dir()` from Module");
    }

    // Extension `parent()` should return its Module if configured without a Group.
    // Extension `parent()` should return its Group if configured with one. 
    #[test] #[named]
    fn test_parent() {
        let test = MODULE_BASIC.test(function_name!()).build();

        assert!(test.parent().is_module(),
            "Extension `parent()` should return its Module if configured without a Group.");
        assert_eq!(MODULE_BASIC.namepath(), test.parent().namepath(),
            "Extension `parent()` should return its Module if configured without a Group.");

        let test = GROUP_BASIC.test(function_name!()).build();

        assert!(test.parent().is_group(),
            "Extension `parent()` should return its Group if configured with one.");
         assert_eq!(GROUP_BASIC.namepath(), test.parent().namepath(),
            "Extension `parent()` should return its Group if configured with one.");
    }

    // unsafe: This can only be called once, by `test_setup_function()`. Not thread safe.
    static mut SETUP_FUNC_CALLED: bool = false;
    fn setup_func(_test: &mut Extension) {
        unsafe {
            SETUP_FUNC_CALLED = true;
        }
    }
/* 
    // Extension setup function should be ran on construction.
    #[test] #[named]
    fn test_setup_function() {
        let _testgroup = GROUP_BASIC.test(function_name!())
            .setup(setup_func)
            .build();

        unsafe {
            assert!(SETUP_FUNC_CALLED,
                "Extension setup function should be ran on construction.");
        }
    }
 
    // Extension setup closure should be ran on construction.
    #[test] #[named]
    fn test_setup_closure() {
        let mut setup_closure_called = false;
        GROUP_BASIC.test(function_name!())
            .setup(|_| {
                setup_closure_called = true;
            })
            .build();

        assert!(setup_closure_called,
            "Extension setup closure should be ran on construction.");
    }
 
    // unsafe: This can only be called once, by `test_setup_function()`. Not thread safe.
    static mut TEARDOWN_FUNC_CALLED: bool = false;
    fn teardown_func(_group: &mut Extension) {
        unsafe {
            TEARDOWN_FUNC_CALLED = true;
        }
    }

    // Extension teardown function should be ran on destruction.
    #[test] #[named]
    fn test_teardown_function() {
        {
            GROUP_BASIC.test(function_name!())
            .teardown(teardown_func)
            .build();
        }

        unsafe {
            assert!(TEARDOWN_FUNC_CALLED,
                "Extension teardown function should be ran on destruction.");
        }
    }
 
    // Extension teardown closure should be ran on destruction.
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
            "Extension teardown closure should be ran on destruction.");
    }
*/
    #[test] #[named]
    fn test_extend() {
        const PUBLISHER_NAME: &str = concat!("publisher_{}", function_name!());
        const EXTENSION_NAME: &str = concat!("extension_{}", function_name!());

        struct MyExtension {}

        impl MyExtension {
            pub fn new() -> Self {
                MyExtension {}
            }
        }

        impl ExtensionTrait for MyExtension {
            fn extension_path() -> &'static str {
                module_path!()
            }

            fn extension_path_self(&self) -> &'static str {
                module_path!()
            }
        }

        let test_group = MODULE_BASIC.group(concat!(function_name!(), "_group"))
            .extension(MyExtension::new())
            .build();

        let test = test_group.test(concat!(function_name!(), "_test")).build();

        let my_extension = test.extension(MyExtension::extension_path());
    }
}
 

