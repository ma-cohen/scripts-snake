mod runner;
mod store;
mod ui;

use std::process;

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use store::AliasStore;

#[derive(Debug, Parser)]
#[command(
    name = "snake",
    version,
    about = "Save, search, and run command aliases from your terminal"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Add a new command alias.
    Add,
    /// List saved command aliases.
    List,
    /// Remove a saved alias.
    #[command(alias = "rm")]
    Remove {
        /// Alias to remove.
        alias: String,
    },
    /// Run an alias directly or search with an initial query.
    Run {
        /// Exact alias or initial search query.
        query: Option<String>,
    },
}

fn main() {
    if let Err(error) = try_main() {
        eprintln!("snake: {error:#}");
        process::exit(1);
    }
}

fn try_main() -> Result<()> {
    let cli = Cli::parse();
    let mut store = AliasStore::open_default()?;

    match cli.command {
        Some(Commands::Add) => add_alias(&mut store),
        Some(Commands::List) => list_aliases(&store),
        Some(Commands::Remove { alias }) => remove_alias(&mut store, &alias),
        Some(Commands::Run { query }) => run_alias(&store, query.as_deref()),
        None => pick_and_run(&store, None),
    }
}

fn add_alias(store: &mut AliasStore) -> Result<()> {
    let alias = ui::prompt_alias()?;
    let command = ui::prompt_command()?;

    if store.contains_alias(&alias) && !ui::confirm_overwrite(&alias)? {
        println!("No changes made.");
        return Ok(());
    }

    let script = store::ScriptAlias::new(alias, command)?;
    let was_update = store.upsert(script);
    store.save()?;

    if was_update {
        println!("Updated alias.");
    } else {
        println!("Added alias.");
    }

    Ok(())
}

fn list_aliases(store: &AliasStore) -> Result<()> {
    if store.is_empty() {
        println!("No aliases saved yet. Add one with `snake add`.");
        return Ok(());
    }

    for script in store.scripts() {
        println!("{}\t{}", script.alias, script.command);
    }

    Ok(())
}

fn remove_alias(store: &mut AliasStore, alias: &str) -> Result<()> {
    if !store.remove(alias) {
        bail!("alias `{alias}` was not found");
    }

    store.save()?;
    println!("Removed `{alias}`.");
    Ok(())
}

fn run_alias(store: &AliasStore, query: Option<&str>) -> Result<()> {
    if let Some(query) = query
        && let Some(script) = store.get(query)
    {
        return run_script(script);
    }

    pick_and_run(store, query)
}

fn pick_and_run(store: &AliasStore, initial_query: Option<&str>) -> Result<()> {
    let script = ui::select_alias(store.scripts(), initial_query)?
        .context("no aliases saved yet; add one with `snake add`")?;

    run_script(&script)
}

fn run_script(script: &store::ScriptAlias) -> Result<()> {
    println!("$ {}", script.command);
    let exit_code = runner::run(&script.command)?;

    if exit_code != 0 {
        process::exit(exit_code);
    }

    Ok(())
}
