use std::{fs, env};
use anyhow::Context;
use syn;
use quote::{self, ToTokens};
use crate::{self as lib, cli, meta, str, cmd};


pub fn remove_trait(args: cli::RemoveTraitCommand, quiet: bool) -> anyhow::Result<()> {
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

    // find the trait
    let trait_meta = match library.traits().iter().find(|t| t.name() == args.module.trait_name) {
        Some(v) => v,
        None => anyhow::bail!(lib::Errors::UnknownTrait(args.module.trait_name, library.name().to_owned()))
    };

    lib::log(quiet, "Removing trait from lib package ...");
    rm_lib_trait(trait_meta, library, &workspace)?;
    /*lib::log("Removing trait from lib manifest ...");
    update_lib_manifest()?;
    lib::log("Removing macro from derive package ...");
    rm_derive_macro()?;
    lib::log("Removing integration test from derive package ...");
    rm_derive_integration_test()?;
    lib::log("Testing workspace ...");
    test_workspace()?;
    lib::log_success("The enumtrait has been removed.");
    */

    Ok(())
}

const ENUMTRAIT_ATTR_IDENT: &'static str = "enumtrait";

fn rm_lib_trait(
    trait_meta: &meta::TraitMeta,
    library: &meta::LibraryMeta,
    workspace: &meta::WorkspaceMeta
) -> anyhow::Result<()> {
    let trait_name = trait_meta.name();
    let src_filepath = workspace.lib_path(library).join("src").join("lib.rs");
    let mut src = syn::parse_file(&fs::read_to_string(&src_filepath)?)?;
    
    // build a new vector of items, excluding the trait
    let mut found = false;
    src.items = src.items.into_iter()
        .filter_map(|item| {
            // return None when it's found, Some(item) otherwise to retain that item 
            match item {
                syn::Item::Trait(ref item_trait) => {
                    // match against the trait ident by name
                    if item_trait.ident.to_string() != trait_name {
                        Some(item)
                    }
                    // ensure that this trait has an #[enumtrait] attribute
                    // if it doesn't, warn that there's a potential error
                    else if item_trait.attrs.iter().find(|attr| attr.path().is_ident(ENUMTRAIT_ATTR_IDENT)).is_none() {
                        lib::log_warn(&format!(
                            "Trait `{}` found, but it does not have an `#[enumtrait]` attribute. Skipped",
                            trait_name));

                        Some(item) 
                    } else {
                        found = true;
                        None
                    }
                },
                _ => Some(item)
            }
        })
        .collect();

    if !found {
        anyhow::bail!(lib::Errors::SourceParsing(str!("Trait not found"), src_filepath.to_owned()));
    }

    fs::write(&src_filepath, src.to_token_stream().to_string())?;
    cmd::rustfmt(&src_filepath)?;

    Ok(())
}