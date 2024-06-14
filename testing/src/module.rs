//! # Module
//! Testing for a module
//! This is testing

use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{PathBuf, Path};
use once_cell::sync::Lazy;
use std::sync::Mutex;
use anyhow::{self, bail, Context};
use rand::{self, Rng};

use crate::{UseCase, NamepathTrait};
use crate::GroupBuilder;
use crate::TestBuilder;
use crate::namepath::Namepath;

const MAX_RAND_DIR_RETRIES: i32 = 64;
const MAX_RAND_DIR_CHARS: i32 = 8;

#[derive(PartialEq, Eq, Debug)]
pub struct Module {
    pub(crate) namepath: Namepath,
    pub(crate) use_case: UseCase,
    pub(crate) base_temp_dir: Option<PathBuf>,
    pub(crate) temp_dir: Option<PathBuf>,
    pub(crate) fixture_dir: Option<PathBuf>,
    pub(crate) imported_fixture_dirs: Option<HashMap<Namepath, PathBuf>>
}

impl Module {
    pub fn namepath(&self) -> &Namepath {
        &self.namepath
    }

    pub fn use_case(&self) -> &UseCase {
        &self.use_case
    }

    pub fn base_temp_dir(&self) -> &Path {
        &self.base_temp_dir.as_ref().context("Module `base temp dir` is not configured").unwrap()
    }

    pub fn temp_dir(&self) -> &Path {
        &self.temp_dir.as_ref().context("Module `temp dir` is not configured").unwrap()
    }

    pub fn fixture_dir(&self) -> &Path {
        &self.fixture_dir.as_ref().context("Module `fixture dir` is not configured").unwrap()
    }

    pub fn imported_fixture_dir(&self, namepath: &Namepath) -> &Path {
        self.imported_fixture_dirs.as_ref()
            .context("Module `shared fixture dirs` is not configured").unwrap()
            .get(namepath)
            .context(format!("Imported fixture dir not found for namepath: {}", namepath.path()))
            .unwrap()
            .as_path()
    }

    // Creates a GroupBuilder configured as static. This is the expected usage.
    pub fn group(&self, name: &str) -> GroupBuilder {
        GroupBuilder::new(self, name, true) 
    }

    // Creates a GroupBuilder configured as non-static. This is expected to be used in testing of this crate.
    pub fn local_group(&self, name: &str) -> GroupBuilder {
        GroupBuilder::new(self, name, false) 
    }

    /// Creates a [TestBuilder].
    pub fn test(&self, name: &str) -> TestBuilder {
        TestBuilder::new(&self, None, name)
    }

    fn teardown(&mut self) {
        let mut teardown = Teardown {
            base_temp_dir: self.base_temp_dir.take()
        };

        teardown.destroy();
    }
}

struct Teardown {
    base_temp_dir: Option<PathBuf>
}

impl Teardown {
    pub(crate) fn destroy(&mut self) {
        if let Some(dir) = &self.base_temp_dir {
            if dir.exists() && std::fs::remove_dir_all(&dir).is_err() {
                eprintln!("Unable to delete base temp dir: {}", dir.to_str().unwrap());
            }
        }
    }
}

impl Drop for Module {
    fn drop(&mut self) {
        self.teardown();
    }
}

pub struct ModuleBuilder<'func> {
    pub(crate) use_case: UseCase,
    pub(crate) module_path: String,
    pub(crate) base_temp_dir: PathBuf,
    pub(crate) using_temp_dir: bool,
    pub(crate) using_fixture_dir: bool,
    pub(crate) imported_fixture_dirs: Option<HashMap<Namepath, PathBuf>>,
    pub(crate) setup_func: Option<Box<dyn FnOnce(&mut Module) + 'func>>,
    pub(crate) static_teardown_func: Option<Box<extern fn()>>,
    pub(crate) is_static: bool 
}

impl<'func> ModuleBuilder<'func> {
    pub(crate) fn new(module_path: &str, use_case: crate::UseCase) -> Self {
        ModuleBuilder {
            use_case,
            module_path: String::from(module_path),
            base_temp_dir: std::env::temp_dir(),
            using_temp_dir: false,
            using_fixture_dir: false,
            imported_fixture_dirs: None,
            setup_func: None,
            static_teardown_func: None,
            is_static: true,
        }
    }

    fn create_random_subdir(base_dir: &Path, prefix: &str) -> anyhow::Result<PathBuf> {
        let mut randgen = rand::thread_rng();
        let mut random_dir;

        for _ in 0..MAX_RAND_DIR_RETRIES {
            let rand_chars: String = (0..MAX_RAND_DIR_CHARS)
                .map(|_| randgen.sample(rand::distributions::Alphanumeric) as char)
                .collect();

            let name = format!("{prefix}.{rand_chars}");
            random_dir = base_dir.join(name);
            if random_dir.exists() {
                continue;
            }

            if std::fs::create_dir_all(&random_dir).is_ok() {
                return Ok(random_dir.canonicalize()?);
            }
        }

        bail!("Unable to create temporary directory in: {}", base_dir.to_str().unwrap())
    }

    pub fn build(self) -> Module {
        let namepath = Namepath::module(self.use_case, self.module_path);

        let base_temp_dir;
        let temp_dir = if self.using_temp_dir {
            base_temp_dir = Some( Self::create_random_subdir(&self.base_temp_dir, &namepath.squash()) // todo: use squashed prefix
                .context(format!("Unable to create temporary directory in base: {}", &self.base_temp_dir.to_str().unwrap()))
                .unwrap() );

            Some( crate::build_temp_dir(&namepath, &base_temp_dir.as_ref().unwrap()) )
        } else {
            base_temp_dir = None;
            None
        };

        let fixture_dir = if self.using_fixture_dir {
            Some( crate::build_fixture_dir(&namepath, self.use_case) )
        } else {
            None
        };

        let imported_fixture_dirs = self.imported_fixture_dirs;

        let mut module = Module {
            namepath,
            use_case: self.use_case,
            base_temp_dir,
            temp_dir,
            fixture_dir,
            imported_fixture_dirs
        };

        if let Some(setup_fn) = self.setup_func {
            setup_fn(&mut module);
        }

        if let Some(static_teardown_func) = self.static_teardown_func {
            shutdown_hooks::add_shutdown_hook(*static_teardown_func);
        }

        if self.is_static {
            let mut teardown_list = STATIC_TEARDOWN_QUEUE.lock().unwrap();
            teardown_list.push(Teardown {
                base_temp_dir: module.base_temp_dir.clone()
            });

            if teardown_list.len() == 1 {
                shutdown_hooks::add_shutdown_hook(default_static_func);
            }
        }

        module
    }

    pub fn base_temp_dir<P>(mut self, dir: &P) -> Self
    where
        P: ?Sized + AsRef<OsStr>
    {
        let dir = PathBuf::from(dir);
        let dir = dir.canonicalize()
            .context(format!("Base temporary directory does not exist: {}", &dir.to_str().unwrap()))
            .unwrap();

        self.base_temp_dir = dir;
        self
    }

    pub fn using_fixture_dir(mut self) -> Self {
        self.using_fixture_dir = true;
        self
    }

    pub fn import_fixture_dir<P>(mut self, namepath: &Namepath) -> Self
    where
        P: ?Sized + AsRef<OsStr>
    {
        let dir = crate::build_fixture_dir(&namepath, self.use_case);
        let dir = dir.canonicalize()
            .context(format!("Base temporary directory does not exist: {}", &dir.to_str().unwrap()))
            .unwrap();

        if self.imported_fixture_dirs.is_none() {
            self.imported_fixture_dirs = Some(HashMap::new());
        }

        self.imported_fixture_dirs.as_mut().expect("Option should exist")
            .insert(namepath.to_owned(), dir);
        
        self
    }

    pub fn using_temp_dir(mut self) -> Self {
        self.using_temp_dir = true;
        self
    }

    pub fn setup(mut self, func: impl FnOnce(&mut Module) + 'func) -> Self {
        self.setup_func = Some(Box::new(func));
        self
    }

    pub fn teardown_static(mut self, func: extern fn()) -> Self {
        assert!(self.is_static, "Module must be static to use a static teardown function.");
        self.static_teardown_func = Some(Box::new(func));
        self
    }

    pub fn nonstatic(mut self) -> Self {
        assert!(self.static_teardown_func.is_none(), "Module must be static to use a static teardown function.");
        self.is_static = false;
        self
    }
}

extern fn default_static_func() {
    let mut teardown_list = STATIC_TEARDOWN_QUEUE.lock().unwrap();
    while let Some(mut teardown) = teardown_list.pop() {
        teardown.destroy();
    }
}

static STATIC_TEARDOWN_QUEUE: Lazy<Mutex<Vec<Teardown>>> = Lazy::new(|| { Mutex::new(Vec::new()) });

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use crate::{self as testing, NamepathTrait, UseCase, strings, namepath};
    use function_name::named;

    #[test] #[should_panic]
    // Should panic if attempting to retrieve the temp_dir() without having configured one manually or by calling ensure_temp_dir().
    fn test_temp_dir_unconfigured() {
        let module = testing::unit(module_path!())
            .nonstatic()
            .build();

        module.temp_dir();  // should panic
    }

    // Should panic if attempting to retrieve the fixture_dir() without having configured one manually or by calling ensure_fixture_dir().
    #[test] #[should_panic]
    fn test_fixture_dir_unconfigured() {
        let module = testing::unit(module_path!())
            .nonstatic()
            .build();

        module.fixture_dir(); // should panic
    }

    // Module base temp dir should be inaccessible if not using a temp dir.
    #[test] #[should_panic]
    fn test_base_temp_dir_unconfigured_temp_dir() {
        testing::unit(module_path!())
            .nonstatic()
            .base_temp_dir(&std::env::temp_dir())
            .build()
            .base_temp_dir();  // should panic
    }

    // Module base temp dir should accept paths of types `Path` and `String`.
    #[test] #[named]
    fn test_base_temp_dir() {
        let expected_base_temp_dir = {
            let base_temp_dir = std::env::temp_dir()
                .join(namepath::squash(&concat!(module_path!(), function_name!())));

            if !base_temp_dir.exists() {
                std::fs::create_dir(&base_temp_dir).unwrap(); // needs manual teardown
            }

            base_temp_dir.canonicalize().unwrap() // for posterity
        };

        let module = testing::unit(module_path!())
            .nonstatic()
            .base_temp_dir(&expected_base_temp_dir)
            .using_temp_dir()
            .build();

        assert_eq!(expected_base_temp_dir, module.base_temp_dir().parent().unwrap(),
            "Module base temp dir should accept paths of type `Path`." );

        let module = testing::unit(module_path!())
            .nonstatic()
            .base_temp_dir(expected_base_temp_dir.to_str().unwrap())
            .using_temp_dir()
            .build();

        assert_eq!(expected_base_temp_dir, module.base_temp_dir().parent().unwrap(),
            "Module base temp dir should accept paths of type `String`." );


        std::fs::remove_dir_all(expected_base_temp_dir).unwrap(); // testing cleanup
    }

    // Module should not allow configuration of base temp dir with a relative path.
    // Only canonical paths are allowed.
    #[test] #[should_panic]
    fn test_base_temp_dir_relative() {
        let _module = testing::unit(module_path!())
            .nonstatic()
            .base_temp_dir("tmp")
            .build();
    }

    // Module should not allow configuration of a base temp dir with a non-existing path.
    #[test] #[should_panic]
    fn test_base_temp_dir_nonexistant() {
        let _module = testing::unit(module_path!())
            .nonstatic()
            .base_temp_dir(&std::env::temp_dir().join("asmovtestingnoandthen"))
            .build();
    }

    // Module namepath should be: `module_path!()`.
    #[test]
    fn test_namepath() {
        let module = testing::unit(module_path!())
            .nonstatic()
            .build();

        assert_eq!(module_path!(), module.namepath().path(),
            "Module namepath should be: `module_path!()`.");
    }

    // Module use-case should match the fascade helper function that was used to create it.
    #[test]
    fn test_use_case() {
        let unit = testing::unit(module_path!()).nonstatic().build();
        let integration = testing::integration(module_path!()).nonstatic().build();
        let benchmark = testing::benchmark(module_path!()).nonstatic().build();

        assert_eq!(testing::UseCase::Unit, *unit.use_case(), 
            "Module use-case should match the fascade helper function (Unit) that was used to create it.");
        assert_eq!(testing::UseCase::Integration, *integration.use_case(),
            "Module use-case should match the fascade helper function (Integration) that was used to create it.");
        assert_eq!(testing::UseCase::Benchmark, *benchmark.use_case(),
            "Module use-case should match the fascade helper function (Benchmark) that was used to create it.");
    }

    // Module should construct Groups properly.
    // Groups constructed by Module should have a reference to it.
    #[test] #[named]
    fn test_group() {
        let unit = testing::unit(module_path!()).nonstatic().build();
        let group = unit.group(function_name!()).build();

        assert_eq!(function_name!(), group.name(),
            "Module should construct Groups properly.");
        assert_eq!(unit.namepath().path(), group.module().namepath().path(),
            "Groups constructed by Module should have a reference to it.");
    }

    // Module configured with `using_temp_dir()` should have a temp path:
    //     `Module.base_temp_dir() + `Module.namepath().path()`
    // Module configured with `using_temp_dir()` should create the temp directory on construction.
    #[test] #[named]
    fn test_temp_dir_using() {
        let namepath = namepath::join(module_path!(), function_name!());
        let unit = testing::unit(&namepath).using_temp_dir().nonstatic().build();
        let expected_tmp_dir = namepath::dir(&unit.base_temp_dir(), &namepath);

        assert_eq!(expected_tmp_dir, unit.temp_dir(),
            "Module configured with `using_temp_dir()` should have a temp path: `Module.base_temp_dir() + `Module.namepath().path()`");
        assert!(unit.temp_dir().exists(),
            "Module configured with `using_temp_dir()` should create the temp directory on construction.");
    }

    // Module configured with `using_fixture_dir()` should have a fixture path:
    //     testing / fixtures / `Module.use_case()` / `Module::namepath().dir()`
    // Module configured with `using_fixture_dir()` should have a pre-existing fixture dir
    #[test]
    fn test_fixture_dir_using() {
        let expected_fixture_dir = PathBuf::from(strings::TESTING).join(strings::FIXTURES)
            .join(UseCase::Unit.to_str())
            .join("module") // equivalent to Namepath::relative_base_module_path()
            .canonicalize().unwrap();

        let unit = testing::unit(module_path!()).using_fixture_dir().nonstatic().build();

        assert_eq!(expected_fixture_dir, unit.fixture_dir(),
            "Module configured with `using_fixture_dir` should have a fixture path: testing / fixtures / `Module.use_case()` / `Module.namepath().dir()`");
         assert!(unit.fixture_dir().exists(),
            "Module configured with `using_fixture_dir` should have a pre-existing fixture dir");
    }

    static mut SETUP_FUNC_CALLED: bool = false;
    fn setup_func(_module: &mut testing::Module) {
        unsafe {
            SETUP_FUNC_CALLED = true;
        }
    }

    #[test]
    // Should run a setup function
    fn test_setup_function() {
        let _module = testing::unit(module_path!()).setup(setup_func).nonstatic().build();

        unsafe {
            assert!(SETUP_FUNC_CALLED);
        }
    }

    #[test]
    // Should run a setup closure 
    fn test_setup_closure() {
        let mut setup_closure_called = false;

        let _module = testing::unit(module_path!())
            .nonstatic()
            .setup(|_| {
                setup_closure_called = true;
            })
            .build();

        assert!(setup_closure_called);
    }

    extern fn static_teardown_func() {
        println!("STATIC_MODULE: teardown_static() ran");
    }

    #[test]
    // Should set a teardown hook. Not testing the actual atexit call here.
    fn test_teardown_static() {
        let _module = testing::unit(module_path!())
            .teardown_static(static_teardown_func)
            .build();
    }

    #[test]
    // Should teardown temp directories
    fn test_teardown() {
        let temp_dir: PathBuf;
        {
            let module = testing::unit(module_path!()).nonstatic().using_temp_dir().build();
            temp_dir = module.temp_dir().into();
            assert!(temp_dir.exists());
        }
        assert!(!temp_dir.exists())
    }
}