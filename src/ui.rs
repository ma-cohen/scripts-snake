use anyhow::{Context, Result};
use inquire::{Confirm, Select, Text};

use crate::store::ScriptAlias;

pub fn prompt_alias() -> Result<String> {
    Text::new("Alias:").prompt().context("failed to read alias")
}

pub fn prompt_command() -> Result<String> {
    Text::new("Command:")
        .prompt()
        .context("failed to read command")
}

pub fn confirm_overwrite(alias: &str) -> Result<bool> {
    Confirm::new(&format!("Alias `{alias}` already exists. Overwrite it?"))
        .with_default(false)
        .prompt()
        .context("failed to confirm overwrite")
}

pub fn select_alias(
    message: &str,
    scripts: &[ScriptAlias],
    initial_query: Option<&str>,
) -> Result<Option<ScriptAlias>> {
    if scripts.is_empty() {
        return Ok(None);
    }

    let options = scripts.to_vec();
    let mut prompt = Select::new(message, options).with_page_size(12);

    if let Some(query) = initial_query.filter(|query| !query.trim().is_empty()) {
        prompt = prompt.with_starting_filter_input(query);
    }

    prompt
        .prompt_skippable()
        .context("failed to select command")
}
