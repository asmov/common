use std::{fs, process, env};
use anyhow::Context;
use syn;
use quote::{self, ToTokens};
use convert_case::{self as case, Casing};
use crate::{self as lib, cli, meta::{self, LibraryMeta}, str, cmd};

use super::has_enumtrait;

pub fn add_trait(args: cli::AddTraitCommand, quiet: bool, test: bool) -> anyhow::Result<()> {
    let dir = if let Some(ref workspace_path) = args.module.workspace_path {
        workspace_path.to_owned()
    } else {
        env::current_dir()?
    };

    let workspace = meta::build(&dir)?;

    // find the library
    let library = if workspace.libraries().len() == 1 {
        workspace.libraries().first().unwrap()
    } else if workspace.libraries().len() > 1 {
        let library_name = match &args.module.library_name {
            Some(name) => name,
            None => anyhow::bail!(lib::Errors::AmbiguousLibrary)
        };

        workspace.libraries().iter().find(|lib| lib.name() == library_name)
            .context(lib::Errors::LibraryNotFound(library_name.to_owned()))?
    } else {
        anyhow::bail!(lib::Errors::MisconfiguredCargoMetadata(str!("No traitenum libraries found")))
    };

    if library.traits().iter().find(|t| t.name() == args.module.trait_name).is_some() {
        anyhow::bail!(lib::Errors::DuplicateTrait(args.module.trait_name, library.name().to_owned()))
    }

    let trait_name = &args.module.trait_name;
    let trait_ident: syn::Ident = syn::parse_str(&args.module.trait_name).unwrap();

    // remove the Example trait/macro if it exists
    //TODO: remove false && to begin
    if false && has_enumtrait(super::EXAMPLE_TRAIT_NAME, &workspace, library)? {
        lib::log(quiet, "Removing example trait ...");
        remove_example_trait(&trait_ident, &workspace, library)?;
    }

    lib::log(quiet, "Adding trait to lib package...");
    add_lib_trait(&trait_ident, &workspace, library)?;
    lib::log(quiet, "Adding macro to derive package...");
    add_derive_macro(&trait_ident, &workspace, library)?;
    lib::log(quiet, "Creating integration test for derive package ...");
    build_derive_test(&args, &trait_name, &workspace, library)?;
    lib::log(quiet, "Updating lib package manifest ...");
    update_cargo_manifest(&args, &trait_ident, &workspace, library)?;

    if test {
        lib::log(quiet, "Testing workspace ...");
        test_workspace(workspace)?;
    }

    lib::log_success(quiet, "Your enumtrait is ready.");
   
    Ok(())
}

fn add_lib_trait(
    trait_ident: &syn::Ident, 
    workspace: &meta::WorkspaceMeta,
    library: &LibraryMeta
) -> anyhow::Result<()> {
    let lib_src_path = workspace.lib_path(library).join("src").join("lib.rs");
    let lib_src = std::fs::read_to_string(&lib_src_path).unwrap();
    let trait_item = trait_item(trait_ident);

    let mut lib_src_file = syn::parse_file(&lib_src).unwrap();
    lib_src_file.items.push(trait_item);
    fs::write(&lib_src_path, lib_src_file.to_token_stream().to_string())?;

    process::Command::new("rustfmt")
        .arg(lib_src_path.to_str().unwrap())
        .output()
        .expect("Unable to run: rustfmt");
 
    Ok(())
}

fn add_derive_macro(
    trait_ident: &syn::Ident,
    workspace: &meta::WorkspaceMeta,
    library: &LibraryMeta
) -> anyhow::Result<()> {
    let derive_src_path = workspace.derive_path(library).join("src").join("lib.rs");
    let derive_src = std::fs::read_to_string(&derive_src_path).unwrap();
    let trait_item = derive_item(trait_ident);

    let mut derive_src_file = syn::parse_file(&derive_src).unwrap();
    derive_src_file.items.push(trait_item);

    fs::write(&derive_src_path, derive_src_file.to_token_stream().to_string())?;
    cmd::rustfmt(&derive_src_path)?;

    Ok(())
}

fn remove_example_trait(
    _trait_ident: &syn::Ident,
    _workspace: &meta::WorkspaceMeta,
    _library: &LibraryMeta
) -> anyhow::Result<()> {
   todo!() 
}

fn update_cargo_manifest(
    args: &cli::AddTraitCommand,
    _trait_ident: &syn::Ident,
    workspace: &meta::WorkspaceMeta,
    library: &LibraryMeta
) -> anyhow::Result<()> {
    let lib_path = workspace.lib_path(&library);
    let manifest_filepath = &lib_path.join("Cargo.toml");
    let mut manifest = cmd::read_manifest(&manifest_filepath)?;
    let traits_metadata = meta::toml_ensure_array("package.metadata.traitenum.trait", &mut manifest, "", &manifest_filepath)?;

    let mut trait_table = toml::Table::new();
    trait_table.insert(str!("name"), toml::Value::String(args.module.trait_name.to_owned()));
    traits_metadata.push(toml::Value::Table(trait_table));

    fs::write(manifest_filepath, toml::to_string_pretty(&manifest).unwrap())?;

    Ok(())
}

fn build_derive_test(
    _args: &cli::AddTraitCommand,
    trait_name: &str,
    workspace: &meta::WorkspaceMeta,
    library: &LibraryMeta
) -> anyhow::Result<()> {
    let trait_snake_name = trait_name.to_case(case::Case::Snake);

    let test_src = DERIVE_TEST_TEMPLATE
        .replace(VAR_DERIVE_CRATE_NAME, &library.derive_name().to_case(case::Case::Snake))
        .replace(VAR_LIB_CRATE_NAME, &library.lib_name().to_case(case::Case::Snake))
        .replace(VAR_TRAIT_NAME, &trait_name)
        .replace(VAR_TRAIT_SNAKE_NAME, &trait_snake_name);

    let test_src_path = workspace.derive_path(library).join("tests")
        .join(format!("{}{}", trait_snake_name, ".rs"));

    fs::write(test_src_path, test_src)?;

    Ok(())
}

fn test_workspace(workspace: meta::WorkspaceMeta) -> anyhow::Result<()> {
    cmd::cargo_test(workspace.path())
}

fn trait_item(trait_ident: &syn::Ident) -> syn::Item {
    let item = quote::quote!{
        #[enumtrait]
        pub trait #trait_ident {
            #[enumtrait::Str(preset(Variant))]
            fn name(&self) -> &'static str;
            #[enumtrait::Num(preset(Ordinal))]
            fn ordinal(&self) -> usize;
        }
    };

    syn::parse2(item).unwrap()
}

const DERIVE_MACRO_FN_PREFIX: &'static str = "derive_traitenum_";
const DERIVE_MODEL_BYTES_PREFIX: &'static str = "TRAITENUM_MODEL_BYTES_";

fn derive_item(derive_ident: &syn::Ident) -> syn::Item {
    let trait_name = derive_ident.to_string();
    let derive_macro_fn_ident = syn::Ident::new(
        &format!("{}{}", DERIVE_MACRO_FN_PREFIX, trait_name.to_case(case::Case::Snake)),
        proc_macro2::Span::call_site());
    let derive_model_const_ident = syn::Ident::new(
        &format!("{}{}", DERIVE_MODEL_BYTES_PREFIX, trait_name.to_case(case::Case::ScreamingSnake)),
        proc_macro2::Span::call_site());

    let item = quote::quote!{
        traitenum_lib::gen_derive_macro!(
            #derive_ident,
            #derive_macro_fn_ident,
            traitlib::#derive_model_const_ident
        );
    };

    syn::parse2(item).unwrap()
}

const VAR_LIB_CRATE_NAME: &'static str = "%{LIB_CRATE_NAME}%";
const VAR_DERIVE_CRATE_NAME: &'static str = "%{DERIVE_CRATE_NAME}%";
const VAR_TRAIT_NAME: &'static str = "%{TRAIT_NAME}%";
const VAR_TRAIT_SNAKE_NAME: &'static str = "%{TRAIT_SNAKE_NAME}%";

const DERIVE_TEST_TEMPLATE: &'static str =
r#"
#[cfg(test)]
mod tests {
    use %{LIB_CRATE_NAME}%::%{TRAIT_NAME}%;

    #[test]
    fn test_%{TRAIT_SNAKE_NAME}%() {
        #[derive(%{DERIVE_CRATE_NAME}%::%{TRAIT_NAME}%)]
        enum MyEnum {
            Alpha,
            Bravo,
            Charlie
        }

        assert_eq!("Alpha", MyEnum::Alpha.name());
        assert_eq!("Bravo", MyEnum::Bravo.name());
        assert_eq!("Charlie", MyEnum::Charlie.name());

        assert_eq!(0, MyEnum::Alpha.ordinal());
        assert_eq!(1, MyEnum::Bravo.ordinal());
        assert_eq!(2, MyEnum::Charlie.ordinal());
    }
}
"#;
