//! A traitenum library is comprised of a pair of crates ("lib" and "derive") contained within a cargo workspace.
//! 
//! The "lib" crate exports traits that are defined using the `#[enumtrait]` macro.
//! 
//! The "derive" crate exports the associated derive macros for each enumtrait exported by the "lib" crate. End-users
//! will use these macros to define their own enums, using `#[traitenum]` helper attributes to define properties.
//! 
//! The "lib" crate is the primary product of an enumtrait library. The traitenum library's name and the "lib" package
//! name are, by default, the same.
//! 
//! The "derive" crate depends on the "lib" crate. Its name is, by default, the library's name appended with "-derive".
//! 
//! Package names and directory paths are customizable.
//! 
use std::path::{PathBuf, Path};
use anyhow::Context;
use crate::{self as lib, cmd};

#[derive(Debug)]
pub struct WorkspaceMeta {
    path: PathBuf,
    libraries: Vec<LibraryMeta>
}

#[derive(Debug)]
pub struct LibraryMeta {
    name: String,
    lib_name: String,
    derive_name: String,
    lib_dir: String,
    derive_dir: String,
    traits: Vec<TraitMeta>
}

#[derive(Debug)]
pub struct TraitMeta {
    name: String,
}

impl WorkspaceMeta {
    pub fn path(&self) -> &Path { &self.path }
    pub fn libraries(&self) -> &Vec<LibraryMeta> { &self.libraries }

    pub fn lib_path(&self, library: &LibraryMeta) -> PathBuf {
        self.path.join(library.lib_dir())
    }

    pub fn derive_path(&self, library: &LibraryMeta) -> PathBuf {
        self.path.join(library.derive_dir())
    }
}

impl LibraryMeta {
    pub fn name(&self) -> &str { &self.name }
    pub fn lib_name(&self) -> &str { &self.lib_name }
    pub fn derive_name(&self) -> &str { &self.derive_name }
    pub fn lib_dir(&self) -> &str { &self.lib_dir }
    pub fn derive_dir(&self) -> &str { &self.derive_dir }
    pub fn traits(&self) -> &Vec<TraitMeta> { &self.traits }
}

impl TraitMeta {
    pub fn name(&self) -> &str { &self.name }
}

mod build {
    use std::path::PathBuf;

    #[derive(Debug)]
    pub struct WorkspaceMeta {
        path: Option<PathBuf>,
        libraries: Vec<LibraryMeta>
    }

    impl WorkspaceMeta {
        pub fn new() -> Self {
            Self {
                path: None,
                libraries: Vec::new()
            }
        }

        pub fn path(&mut self, path: PathBuf) -> &mut Self { self.path = Some(path); self }
        /// Panics if path is not set.
        pub fn libraries(&mut self, mut libraries: Vec<LibraryMeta>) -> &mut Self { self.libraries.append(&mut libraries); self }

        /// Panics if path or library.lib_dir is not set.
        pub fn get_lib_path(&self, library: &LibraryMeta) -> PathBuf {
            self.path.as_ref().unwrap().join(library.lib_dir.as_ref().unwrap())
        }

        /// Panics if path or library.derive_dir is not set.
        pub fn get_derive_path(&self, library: &LibraryMeta) -> PathBuf {
            self.path.as_ref().unwrap().join(library.derive_dir.as_ref().unwrap())
        }


        pub fn build(self) -> super::WorkspaceMeta {
            super::WorkspaceMeta {
                path: self.path.unwrap(),
                libraries: self.libraries.into_iter().map(|l| l.build()).collect(),
            }
        }
    }

    #[derive(Debug)]
    pub struct LibraryMeta {
        name: Option<String>,
        lib_name: Option<String>,
        derive_name: Option<String>,
        lib_dir: Option<String>,
        derive_dir: Option<String>,
        traits: Vec<TraitMeta>
    }

    impl LibraryMeta {
        pub fn new() -> Self {
            Self {
                name: None,
                lib_name: None,
                derive_name: None,
                lib_dir: None,
                derive_dir: None,
                traits: Vec::new()
            }
        }

        pub fn name(&mut self, name: String) -> &mut Self { self.name = Some(name); self }
        pub fn lib_name(&mut self, lib_name: String) -> &mut Self { self.lib_name = Some(lib_name); self }
        pub fn derive_name(&mut self, derive_name: String) -> &mut Self { self.derive_name = Some(derive_name); self }
        pub fn lib_dir(&mut self, lib_dir: String) -> &mut Self { self.lib_dir = Some(lib_dir); self }
        pub fn derive_dir(&mut self, derive_dir: String) -> &mut Self { self.derive_dir = Some(derive_dir); self }
        pub fn traits(&mut self, mut traits: Vec<TraitMeta>) -> &mut Self { self.traits.append(&mut traits); self }

        pub fn build(self) -> super::LibraryMeta {
            super::LibraryMeta {
                name: self.name.unwrap(),
                lib_name: self.lib_name.unwrap(),
                derive_name: self.derive_name.unwrap(),
                lib_dir: self.lib_dir.unwrap(),
                derive_dir: self.derive_dir.unwrap(),
                traits: self.traits.into_iter().map(|t| t.build()).collect()
            }
        }
    }

    #[derive(Debug)]
    pub struct TraitMeta {
        name: Option<String>
    }

    impl TraitMeta {
        pub fn new() -> Self {
            Self {
                name: None
            }
        }

        pub fn name(&mut self, name: String) -> &mut Self { self.name = Some(name); self }

        pub fn build(self) -> super::TraitMeta {
            super::TraitMeta {
                name: self.name.unwrap()
            }
        }
    }
}

pub fn build(from_dir: &Path) -> anyhow::Result<WorkspaceMeta> {
    let (workspace_manifest, manifest_path) = cmd::find_cargo_workspace_manifest(&from_dir)?;

    let mut workspace = build::WorkspaceMeta::new();
    workspace.path(manifest_path.parent().unwrap().to_path_buf());

    let libraries_metadata = toml_array("workspace.metadata.traitenum.library", &workspace_manifest, "", &manifest_path)?;

    let mut libraries: Vec<build::LibraryMeta> = Vec::new();
    let mut i = 0;
    for library_metadata in libraries_metadata {
        let context = format!("workspace.metadata.traitenum.library[{}]", i);

        let name = toml_str("name", library_metadata, &context, &manifest_path)?;
        let lib_dir = toml_str("lib-dir", library_metadata, &context, &manifest_path)?;
        let derive_dir = toml_str("derive-dir", library_metadata, &context, &manifest_path)?;

        let mut library = build::LibraryMeta::new();
        library.name(name.to_owned());
        library.lib_dir(lib_dir.to_owned());
        library.derive_dir(derive_dir.to_owned());
        libraries.push(library);
        i += 1;
    }

    for library in &mut libraries {
        let lib_path = workspace.get_lib_path(&library);
        let derive_path = workspace.get_derive_path(&library);

        let manifest_filepath = &lib_path.join("Cargo.toml");
        let manifest = cmd::read_manifest(&manifest_filepath)?;
        let lib_name = toml_str("package.name", &manifest, "", &manifest_filepath)?.to_owned();

        let mut traits: Vec<build::TraitMeta> = Vec::new();
        if let Ok(traits_metadata) = toml_array("package.metadata.traitenum.trait", &manifest, "", &manifest_filepath) {
            let mut i = 0;
            for trait_metadata in traits_metadata {
                let context = format!("package.metadata.traitenum.trait[{}]", i);
                let trait_name = toml_str("name", trait_metadata, &context, &manifest_filepath)?.to_owned();

                let mut trait_meta = build::TraitMeta::new();
                trait_meta.name(trait_name);
                traits.push(trait_meta);
                i += 0;
            }
        }

        let manifest_filepath = &derive_path.join("Cargo.toml");
        let manifest = cmd::read_manifest(&manifest_filepath)?;
        let derive_name = toml_str("package.name", &manifest, "", &manifest_filepath)?.to_owned();

        library.lib_name(lib_name);
        library.derive_name(derive_name);
        library.traits(traits);
    }

    workspace.libraries(libraries);

    Ok(workspace.build())
}

pub(crate) fn toml_path<'toml>(
    path: &str,
    toml: &'toml toml::Value,
    cargo_manifest_filepath: &Path
) -> anyhow::Result<&'toml toml::Value> {
    let mut value = toml;
    for key in path.split(".") {
        value = value.get(key)
            .with_context(|| lib::Errors::MissingCargoMetadata(path.to_owned(), cargo_manifest_filepath.to_owned()))?;
    }

    Ok(value)
}

/*TODO: Remove this if not used
pub(crate) fn toml_path_mut<'toml>(
    path: &str,
    toml: &'toml mut toml::Value,
    cargo_manifest_filepath: &Path
) -> anyhow::Result<&'toml mut toml::Value> {
    let mut value = toml;
    for key in path.split(".") {
        value = value.get_mut(key)
            .with_context(|| lib::Errors::MissingCargoMetadata(path.to_owned(), cargo_manifest_filepath.to_owned()))?;
    }

    Ok(value)
}*/

pub(crate) fn toml_array<'toml>(
    path: &str,
    toml: &'toml toml::Value,
    context: &str,
    cargo_manifest_filepath: &Path
) -> anyhow::Result<&'toml toml::value::Array> {
    let array = toml_path(path, toml, cargo_manifest_filepath)?
        .as_array()
        .with_context(|| lib::Errors::InvalidCargoMetadata(
            format!("{}.{}", context, path), cargo_manifest_filepath.to_owned()))?;

    Ok(array)
}

/*TODO
pub(crate) fn toml_array_mut<'toml>(
    path: &str,
    toml: &'toml mut toml::Value,
    context: &str,
    cargo_manifest_filepath: &Path
) -> anyhow::Result<&'toml mut toml::value::Array> {
    let array = toml_path_mut(path, toml, cargo_manifest_filepath)?
        .as_array_mut()
        .with_context(|| lib::Errors::InvalidCargoMetadata(
            format!("{}.{}", context, path), cargo_manifest_filepath.to_owned()))?;

    Ok(array)
}*/

pub(crate) fn toml_ensure_array<'toml>(
    path: &str,
    toml: &'toml mut toml::Value,
    context: &str,
    cargo_manifest_filepath: &Path
) -> anyhow::Result<&'toml mut toml::value::Array> {
    let context = format!("{}.{}", context, path);
    let mut value = toml;
    let keys: Vec<&str> = path.split(".").collect();
    let last_idx = keys.len() - 1;
    let mut i = 0;

    for key in keys {
        // attempt to retrieve an existing value for this key
        if let Some(table) = value.as_table() {
            if let Some(current_value) = table.get(key) {
                if (i == last_idx && current_value.is_array()) || (i != last_idx && current_value.is_table()) {
                    value = value.as_table_mut().unwrap().get_mut(key).unwrap();
                    i += 1;
                    continue;
                } else {
                    // we would be overwriting something here unexpectedly
                    anyhow::bail!(lib::Errors::InvalidCargoManifestKey(context.to_owned(), cargo_manifest_filepath.to_owned()))
                }
            }
        }

        // otherwise, create a new value for the key; tables for each until the last key, then an array.
        if i != last_idx {
            value.as_table_mut()
                .with_context(|| lib::Errors::InvalidCargoManifestKey(context.to_owned(), cargo_manifest_filepath.to_owned()))?
                .insert(key.to_owned(), toml::Value::Table(toml::Table::new()));
            value = value.get_mut(key).unwrap();
        } else {  // last key
            value.as_table_mut()
                .with_context(|| lib::Errors::InvalidCargoManifestKey(context.to_owned(), cargo_manifest_filepath.to_owned()))?
                .insert(key.to_owned(), toml::Value::Array(toml::value::Array::new()));
            value = value.get_mut(key).unwrap();
        }

        i += 1;
    }

    Ok(value.as_array_mut().unwrap())
}

pub(crate) fn toml_str<'toml>(
    path: &str,
    toml: &'toml toml::Value,
    context: &str,
    cargo_manifest_filepath: &Path
) -> anyhow::Result<&'toml str> {
    let string = toml_path(path, toml, cargo_manifest_filepath)?
        .as_str()
        .with_context(|| lib::Errors::InvalidCargoMetadata(
            format!("{}.{}", context, path), cargo_manifest_filepath.to_owned()))?;
    Ok(string)
}