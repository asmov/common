use std::{env, fs, path::{self, PathBuf}};
use anyhow;

use crate::{self as lib, cli, cmd, str};

pub fn new_workspace(mut args: cli::NewWorkspaceCommand, quiet: bool) -> anyhow::Result<()> {
    // a common mistake is to specify a path instead of a name as the positional parameter, which we can't handle well
    if args.workspace_name.contains(path::MAIN_SEPARATOR) {
        anyhow::bail!(lib::Errors::InvalidArgument(
            str!("workspace-name"), str!("Try `--workspace-path` instead"), args.workspace_name))
    }

    if let Some(ref workspace_path) = args.library.workspace_path {
        if workspace_path.is_relative() {
            args.library.workspace_path = Some(PathBuf::from(env::current_dir().unwrap())
                .join(workspace_path));
        }
    } else {
        args.library.workspace_path = Some(PathBuf::from(env::current_dir().unwrap())
            .join(&args.workspace_name));
    }

    if args.library.lib_name.is_none() {
        args.library.lib_name = Some(args.workspace_name.clone());
    }

    if args.library.derive_name.is_none() {
        args.library.derive_name = Some(format!("{}-{}", args.workspace_name, "derive"));
    }

    // Throw an error if `init` should be used instead of `new`.
    let workspace_path = args.library.workspace_path.as_ref().unwrap();
    if cmd::find_cargo_manifest_file(workspace_path).is_ok() {
        anyhow::bail!(lib::Errors::CargoManifestExists(workspace_path.to_owned()));
    }

    lib::log(quiet, "Creating workspace ...");
    make_workspace(&args)?;
    lib::log(quiet, "Creating lib package ...");
    super::make_lib(&args.library)?;
    lib::log(quiet, "Creating derive package ...");
    super::make_derive(&args.library)?;
    lib::log(quiet, "Configuring lib package ...");
    super::config_lib(&args.library)?;
    lib::log(quiet, "Configuring derive package ...");
    super::config_derive(&args.library)?;
    lib::log(quiet, "Adding new enumtrait ...");
    super::add_enumtrait(&args.library)?;
    lib::log(quiet, "Building workspace ...");
    super::build_workspace(&args.library)?;
    lib::log(quiet, "Testing workspace ...");
    super::test_workspace(&args.library)?;
    lib::log_success(quiet, "Your traitenum workspace is ready.");

    Ok(())
}


// There may be multiple traitenum lib/derive pairs in a workspace, so even when we create our own workspace, we need
// to configure it the same way that we would with "cargo traitenum init". This allows our cargo addon-on to find
// what it's looking for without guessing.
// 
// metadata.traitenum.workspaces: Lists each traitenum workspace by <workspace_name>. Typically the <lib_name>.
// metadata.traitenum.<workspace_name>: Stores the workspace members for a pair of traitenum lib and derive packages.
const WORKSPACE_MANIFEST_TEMPLATE: &'static str =
r#"[workspace]
resolver = "2"
members = [ "%{LIB_DIR}%", "%{DERIVE_DIR}%" ]

[[workspace.metadata.traitenum.library]]
name = "%{LIBRARY_NAME}%"
lib-dir = "%{LIB_DIR}%"
derive-dir = "%{DERIVE_DIR}%"
"#;

fn make_workspace(args: &cli::NewWorkspaceCommand) -> anyhow::Result<()> {
    let workspace_path = args.library.workspace_path.as_ref().unwrap();

    let cmdout = super::cargo_new(workspace_path, None)?;
    if !cmdout.status.success() {
        anyhow::bail!(lib::Errors::CargoNewError(cmd::quote_error_output(cmdout)))
    }

    fs::remove_dir_all(workspace_path.join("src"))?;

    let workspace_manifest = WORKSPACE_MANIFEST_TEMPLATE
        .replace(super::VAR_LIBRARY_NAME, args.library.lib_name.as_ref().unwrap())
        .replace(super::VAR_LIB_DIR, &args.library.lib_dir)
        .replace(super::VAR_DERIVE_DIR, &args.library.derive_dir);

    fs::write(workspace_path.join("Cargo.toml"), workspace_manifest)?;

    Ok(())
}

