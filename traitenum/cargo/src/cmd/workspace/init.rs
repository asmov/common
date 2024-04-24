use std::{env, path::{PathBuf, Path}};
use crate::{self as lib, cli, cmd, meta, str};

pub fn init_workspace(mut args: cli::InitWorkspaceCommand, quiet: bool) -> anyhow::Result<()> {
    // clarify to the user that library.lib_name and library_name are the same
    // todo: remove lib_name from the common
    if args.module.lib_name.is_some() {
        lib::log_warn("Using preferred `<LIBRARY_NAME>` argument instead of `--lib-name`")
    } else {
        args.module.lib_name = Some(args.library_name.clone());
    }

    if let Some(ref workspace_path) = args.module.workspace_path {
        if workspace_path.is_relative() {
            args.module.workspace_path = Some(PathBuf::from(env::current_dir().unwrap())
                .join(workspace_path));
        }
    } else {
        args.module.workspace_path = Some(PathBuf::from(env::current_dir().unwrap())
            .join(&args.library_name));
    }

    if args.module.derive_name.is_none() {
        args.module.derive_name = Some(format!("{}-{}", args.library_name, "derive"));
    }

    // Throw an error if `new` should be used instead of `init`.
    let workspace_path = args.module.workspace_path.as_ref().unwrap();
    let workspace_manifest_filepath = cmd::find_cargo_manifest_file(&workspace_path)?;
    let mut workspace_manifest = cmd::read_workspace_manifest(&workspace_manifest_filepath)?;

    lib::log(quiet, "Updating workspace ...");
    update_workspace(&args, &mut workspace_manifest, &workspace_manifest_filepath)?;
    lib::log(quiet, "Creating lib package ...");
    super::make_lib(&args.module)?;
    lib::log(quiet, "Creating derive package ...");
    super::make_derive(&args.module)?;
    lib::log(quiet, "Configuring lib package ...");
    super::config_lib(&args.module)?;
    lib::log(quiet, "Configuring derive package ...");
    super::config_derive(&args.module)?;
    lib::log(quiet, "Building workspace ...");
    super::build_workspace(&args.module)?;
    lib::log(quiet, "Testing workspace ...");
    super::test_workspace(&args.module)?;
    lib::log_success(quiet, "Your traitenum workspace is ready.");

    Ok(())
}

fn update_workspace(
    args: &cli::InitWorkspaceCommand,
    manifest: &mut toml::Value,
    workspace_manifest_filepath: &Path
) -> anyhow::Result<()> {
    let members_data = meta::toml_ensure_array(
        "workspace.members", manifest, "", workspace_manifest_filepath)?;

    members_data.push(toml::Value::String(args.module.lib_dir.to_owned()));
    members_data.push(toml::Value::String(args.module.derive_dir.to_owned()));

    let library_metadata = meta::toml_ensure_array(
        "workspace.metadata.traitenum.library", manifest, "", workspace_manifest_filepath)?;

    let mut library_table = toml::Table::new();
    library_table.insert(str!("derive-dir"), toml::Value::String(args.module.derive_dir.to_owned()));
    library_table.insert(str!("lib-dir"), toml::Value::String(args.module.lib_dir.to_owned()));
    library_table.insert(str!("name"), toml::Value::String(args.library_name.to_owned()));

    library_metadata.push(toml::Value::Table(library_table));

    std::fs::write(workspace_manifest_filepath, toml::to_string_pretty(manifest).unwrap()).unwrap();

    Ok(())
}