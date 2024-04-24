use std::{env, process, path::{PathBuf, Path}};
use anyhow::Context;
use crate::{self as lib, str};

pub mod workspace;
pub mod enumtrait;

pub use workspace::new::new_workspace;
pub use workspace::init::init_workspace;
pub use enumtrait::add::add_trait;
pub use enumtrait::remove::remove_trait;

fn quote_error(errmsg: String) -> String {
    let errmsg = errmsg.replace("error: ", "");
    if let Some(offset) = errmsg.find("\n") {
        errmsg[0 .. offset].to_owned()
    } else {
        errmsg
    }
}

fn quote_error_output(output: process::Output) -> String {
    quote_error(String::from_utf8(output.stderr).unwrap())
}

fn find_cargo_manifest_file(from_dir: &Path) -> anyhow::Result<PathBuf> {
    let mut current_dir = from_dir.to_owned();

    while current_dir.exists() {
        let cargo_manifest_filepath = current_dir.join("Cargo.toml");
        if cargo_manifest_filepath.exists() {
            return Ok(cargo_manifest_filepath);
        }

        current_dir = current_dir.join("..");
    }

    Err(lib::Errors::NoCargoManifestExists(from_dir.into()).into())
}

pub(crate) fn read_manifest(filepath: &Path) -> anyhow::Result<toml::Value> {
    let contents = std::fs::read_to_string(filepath)?;
    toml::from_str(&contents).map_err(|e| anyhow::format_err!("{}", e.message()))
}

pub(crate) fn read_workspace_manifest(filepath: &Path) -> anyhow::Result<toml::Value> {
    let manifest = read_manifest(&filepath)?;
    if manifest.as_table()
        .with_context(|| lib::Errors::InvalidCargoManifest(filepath.to_owned()))?
        .contains_key("workspace") {
            Ok(manifest)
    } else {
        anyhow::bail!(lib::Errors::CargoManifestNotWorkspace(filepath.to_owned()))
    }
}

pub(crate) fn find_cargo_workspace_manifest(from_dir: &Path) -> anyhow::Result<(toml::Value, PathBuf)> {
    // if first manifest found is a package, we'll try once more to find a parent workspace
    let mut dir = from_dir;

    while let Ok(manifest_file) = find_cargo_manifest_file(dir) {
        let manifest = read_workspace_manifest(&manifest_file)?;
        if manifest.as_table()
                .with_context(|| lib::Errors::NoCargoManifestExists(manifest_file))?
                .contains_key("workspace") {
            return Ok((manifest, from_dir.join("Cargo.toml")))
        }

        dir = match dir.parent() { Some(d) => d, None => break };
    }

    Err(lib::Errors::NoCargoWorkspaceExists(from_dir.into()).into())
}

fn cargo_test(dir: &Path) -> anyhow::Result<()> {
    env::set_current_dir(dir)?;
    let output = process::Command::new("cargo")
        .arg("test")
        .output()
        .context(lib::Errors::CargoRunError())?;

    if !output.status.success() {
        anyhow::bail!(lib::Errors::CargoError(str!("test")))
    }

    Ok(())
}

pub(crate) fn rustfmt(filepath: &Path) -> anyhow::Result<()> {
    let output = process::Command::new("rustfmt")
        .arg(filepath.to_str().unwrap())
        .output()
        .context(lib::Errors::RustfmtRunError())?;

    if output.status.success() {
       Ok(()) 
    } else {
        return Err(lib::Errors::RustfmtRunError().into())
    }
}


