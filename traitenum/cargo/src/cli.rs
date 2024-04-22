use clap;
use std::path::PathBuf;
use syn;

use crate::str;

#[derive(clap::Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(global = true, short, long)]
    pub quiet: bool,

    #[command(subcommand)]
    pub module: CommandModules
}

#[derive(clap::Subcommand)]
pub enum CommandModules {
    Workspace(WorkspaceCommandModule),
    Trait(TraitCommandModule)
}

#[derive(clap::Args)]
#[command(about = "Manage traitenum workspaces")]
pub struct WorkspaceCommandModule {
    #[command(subcommand)]
    pub command: WorkspaceCommands
}

#[derive(clap::Args)]
#[command(about = "Manage traitenum traits")]
pub struct TraitCommandModule {
    #[command(subcommand)]
    pub command: TraitCommands 
}

#[derive(clap::Subcommand)]
pub enum WorkspaceCommands {
    #[command(about = "Create a new traitenum workspace containing traits and derive macros")]
    New(NewWorkspaceCommand),
    #[command(about = "Create new traitenum lib and derive packages in an existing workspace")]
    Init(InitWorkspaceCommand),
}

#[derive(clap::Subcommand)]
pub enum TraitCommands {
    Add(AddTraitCommand),
    Remove(RemoveTraitCommand),
}

#[derive(clap::Args)]
pub struct WorkspaceCommand {
     #[arg(long)]
    pub workspace_path: Option<PathBuf>,
     #[arg(long)]
    pub lib_name: Option<String>,
    #[arg(long)]
    pub derive_name: Option<String>,
    #[arg(long, default_value_t = str!("lib"))]
    pub lib_dir: String,
    #[arg(long, default_value_t = str!("derive"))]
    pub derive_dir: String
}

#[derive(clap::Args)]
pub struct NewWorkspaceCommand {
    pub workspace_name: String,
    #[clap(flatten)]
    pub library: WorkspaceCommand,
}

#[derive(clap::Args)]
pub struct InitWorkspaceCommand {
    pub library_name: String,
    #[clap(flatten)]
    pub module: WorkspaceCommand,
}

#[derive(clap::Args)]
pub struct TraitCommand {
    #[arg(value_parser = validate_ident)]
    pub trait_name: String,
    #[arg(long)]
    pub workspace_path: Option<PathBuf>,
    #[arg(long)]
    pub library_name: Option<String>,
}

#[derive(clap::Args)]
#[command(about = "Add a new trait and derive macro to an existing traitenum workspace")]
pub struct AddTraitCommand {
    #[clap(flatten)]
    pub module: TraitCommand
}

#[derive(clap::Args)]
#[command(about = "Remove an existing trait and derive macro to an existing traitenum workspace")]
pub struct RemoveTraitCommand {
    #[clap(flatten)]
    pub module: TraitCommand
}

fn validate_ident(s: &str) -> Result<String, String> {
    syn::parse_str::<syn::Ident>(s)
        .map(|_| s.to_string())
        .map_err(|e| e.to_string() )
}