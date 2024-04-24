//! Handles all workspace commands (init, new).
 
use std::{env, fs, process, path::{PathBuf, Path}};
use anyhow::{self, Context};
use convert_case::{self as case, Casing};

use crate::{self as lib, cli, cmd, str};

pub mod new;
pub mod init;

const VAR_LIB_DIR: &'static str = "%{LIB_DIR}%";
const VAR_DERIVE_DIR: &'static str = "%{DERIVE_DIR}%";
const VAR_LIBRARY_NAME: &'static str = "%{LIBRARY_NAME}%";
const VAR_LIB_NAME: &'static str = "%{LIB_NAME}%";
const VAR_DERIVE_NAME: &'static str = "%{DERIVE_NAME}%";
const VAR_LIB_CRATE_NAME: &'static str = "%{LIB_CRATE_NAME}%";
const VAR_DERIVE_CRATE_NAME: &'static str = "%{DERIVE_CRATE_NAME}%";


const LIB_MANIFEST_TEMPLATE: &'static str =
r#"[package]
name = "%{LIB_NAME}%"
version = "0.1.0"
edition = "2021"

[package.metadata.traitenum]
purpose = "lib"
"#;

const LIB_SRC_TEMPLATE: &'static str =
r#"use traitenum::enumtrait;
"#;

fn make_lib(library: &cli::WorkspaceCommand) -> anyhow::Result<()> {
    let lib_path = library.workspace_path.as_ref().unwrap().join(&library.lib_dir);
    let lib_name = library.lib_name.as_ref().unwrap();

    let cmdout = cargo_new(&lib_path, Some(lib_name))?;
    if !cmdout.status.success() {
        anyhow::bail!(lib::Errors::CargoNewError(cmd::quote_error_output(cmdout)))
    }

    let lib_manifest = LIB_MANIFEST_TEMPLATE
        .replace(VAR_LIB_NAME, lib_name);

    fs::write(lib_path.join("Cargo.toml"), lib_manifest)?;
    fs::write(lib_path.join("src").join("lib.rs"), LIB_SRC_TEMPLATE)?;

    Ok(())
}

const DERIVE_MANIFEST_TEMPLATE: &'static str =
r#"[package]
name = "%{DERIVE_NAME}%"
version = "0.1.0"
edition = "2021"

[lib]
proc-macro = true

[package.metadata.traitenum]
purpose = "derive"
"#;

const DERIVE_SRC_TEMPLATE: &'static str =
r#"traitenum_lib::gen_require!(%{LIB_CRATE_NAME}%, %{DERIVE_CRATE_NAME}%);
"#;


fn make_derive(library: &cli::WorkspaceCommand) -> anyhow::Result<()> {
    let derive_path = library.workspace_path.as_ref().unwrap().join(&library.derive_dir);
    let derive_name = library.derive_name.as_ref().unwrap();
    let lib_name = library.lib_name.as_ref().unwrap();

    let cmdout = cargo_new(&derive_path, Some(derive_name))?;
    if !cmdout.status.success() {
        anyhow::bail!(lib::Errors::CargoNewError(cmd::quote_error_output(cmdout)))
    }

    let derive_manifest = DERIVE_MANIFEST_TEMPLATE
        .replace(VAR_DERIVE_NAME, derive_name);

    fs::write(derive_path.join("Cargo.toml"), derive_manifest)?;

    let derive_src = DERIVE_SRC_TEMPLATE
        .replace(VAR_LIB_CRATE_NAME, &lib_name.to_case(case::Case::Snake))
        .replace(VAR_DERIVE_CRATE_NAME, &derive_name.to_case(case::Case::Snake));

    fs::write(derive_path.join("src").join("lib.rs"), derive_src)?;

    // create the integration test dir
    fs::create_dir_all(derive_path.join("tests"))?;
    fs::write(derive_path.join("tests").join(".gitignore"), "")?;

    Ok(())
}

fn config_lib(library: &cli::WorkspaceCommand) -> anyhow::Result<()> {
    let lib_path = library.workspace_path.as_ref().unwrap().join(&library.lib_dir);

    //todo
    let traitenum_crate_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("macro");

    cargo_add(&lib_path, None, Some(&traitenum_crate_path))?;

    Ok(())
}

fn config_derive(library: &cli::WorkspaceCommand) -> anyhow::Result<()> {
    let derive_path = library.workspace_path.as_ref().unwrap().join(&library.derive_dir);
    let lib_name = library.lib_name.as_ref().unwrap();

    //todo
    let traitenum_lib_crate_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("lib");

    cargo_add(&derive_path, Some("proc-macro2"), None)?;
    cargo_add(&derive_path, None, Some(&traitenum_lib_crate_path))?;
    cargo_add(&derive_path, Some(lib_name), None)?;

    Ok(())
}

fn add_enumtrait(library: &cli::WorkspaceCommand) -> anyhow::Result<()> {
    let cmd = cli::AddTraitCommand {
        module: cli::TraitCommand {
            trait_name: cmd::enumtrait::EXAMPLE_TRAIT_NAME.to_owned(),
            workspace_path: library.workspace_path.to_owned(),
            library_name: library.lib_name.to_owned(),
        },
    };

    cmd::add_trait(cmd, true, false)
}

fn build_workspace(library: &cli::WorkspaceCommand) -> anyhow::Result<()> {
    let workspace_path = library.workspace_path.as_ref().unwrap();

    env::set_current_dir(workspace_path)?;
    let output = process::Command::new("cargo")
        .arg("build")
        .output()
        .context(lib::Errors::CargoRunError())?;

    if !output.status.success() {
        anyhow::bail!(lib::Errors::CargoError(str!("build")))
    }

    Ok(())
}

fn test_workspace(library: &cli::WorkspaceCommand) -> anyhow::Result<()> {
    let workspace_path = library.workspace_path.as_ref().unwrap();
    cmd::cargo_test(workspace_path)
}

fn cargo_new(path: &Path, name: Option<&str>) -> anyhow::Result<process::Output> {
    let mut cmd = process::Command::new("cargo");
    cmd.args(["-q", "new", "--lib"]);

    if let Some(name) = name {
        cmd.args(["--name", &name]);
    }
    
    let output = cmd
        .arg(path.to_str().unwrap())
        .output()
        .context(lib::Errors::CargoRunError())?;

    if !output.status.success() {
        anyhow::bail!(lib::Errors::CargoNewError(cmd::quote_error_output(output)))
    }

    Ok(output)
}


fn cargo_add(manifest_dir: &PathBuf, name: Option<&str>, path: Option<&Path>) -> anyhow::Result<process::Output> {
    let mut cmd = process::Command::new("cargo");
    cmd.args([
        "-q",
        "add",
        "--manifest-path",
        manifest_dir.join("Cargo.toml").to_str().unwrap() ]);

    let target;
    if let Some(name) = name {
        target = name;
        cmd.arg(&name);
    } else if let Some(path) = path {
        target = path.to_str().unwrap();
        cmd.args(["--path", &target]);
    } else {
        unreachable!("Neither name nor path was passed as a parameter");
    }
    
    let output = cmd
        .output()
        .context(lib::Errors::CargoRunError())?;

    if !output.status.success() {
        anyhow::bail!(lib::Errors::CargoAddError(target.to_string(), cmd::quote_error_output(output)))
    }

    Ok(output)
}

