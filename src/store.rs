use std::{fs, path::PathBuf};

use anyhow::{Context, Result, bail};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ScriptAlias {
    pub alias: String,
    pub command: String,
}

impl ScriptAlias {
    pub fn new(alias: impl Into<String>, command: impl Into<String>) -> Result<Self> {
        let alias = alias.into().trim().to_string();
        let command = command.into().trim().to_string();

        if alias.is_empty() {
            bail!("alias cannot be empty");
        }

        if alias.contains('\n') || alias.contains('\r') {
            bail!("alias must be a single line");
        }

        if command.is_empty() {
            bail!("command cannot be empty");
        }

        Ok(Self { alias, command })
    }
}

impl std::fmt::Display for ScriptAlias {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}  {}", self.alias, self.command)
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct AliasFile {
    scripts: Vec<ScriptAlias>,
}

#[derive(Debug)]
pub struct AliasStore {
    path: PathBuf,
    file: AliasFile,
}

impl AliasStore {
    pub fn open_default() -> Result<Self> {
        Self::open(default_store_path()?)
    }

    pub fn open(path: PathBuf) -> Result<Self> {
        let file = if path.exists() {
            let contents = fs::read_to_string(&path)
                .with_context(|| format!("failed to read {}", path.display()))?;

            if contents.trim().is_empty() {
                AliasFile::default()
            } else {
                serde_json::from_str(&contents)
                    .with_context(|| format!("failed to parse {}", path.display()))?
            }
        } else {
            AliasFile::default()
        };

        Ok(Self { path, file })
    }

    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }

        let contents =
            serde_json::to_string_pretty(&self.file).context("failed to serialize aliases")?;

        fs::write(&self.path, format!("{contents}\n"))
            .with_context(|| format!("failed to write {}", self.path.display()))?;

        Ok(())
    }

    pub fn scripts(&self) -> &[ScriptAlias] {
        &self.file.scripts
    }

    pub fn is_empty(&self) -> bool {
        self.file.scripts.is_empty()
    }

    pub fn contains_alias(&self, alias: &str) -> bool {
        self.get(alias).is_some()
    }

    pub fn get(&self, alias: &str) -> Option<&ScriptAlias> {
        self.file
            .scripts
            .iter()
            .find(|script| script.alias == alias)
    }

    pub fn upsert(&mut self, script: ScriptAlias) -> bool {
        if let Some(existing) = self
            .file
            .scripts
            .iter_mut()
            .find(|existing| existing.alias == script.alias)
        {
            *existing = script;
            true
        } else {
            self.file.scripts.push(script);
            self.file
                .scripts
                .sort_by(|left, right| left.alias.cmp(&right.alias));
            false
        }
    }

    pub fn remove(&mut self, alias: &str) -> bool {
        let previous_len = self.file.scripts.len();
        self.file.scripts.retain(|script| script.alias != alias);
        self.file.scripts.len() != previous_len
    }
}

fn default_store_path() -> Result<PathBuf> {
    let project_dirs = ProjectDirs::from("dev", "ma-cohen", "snake")
        .context("could not resolve a user config directory")?;

    Ok(project_dirs.config_dir().join("aliases.json"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store_path(directory: &tempfile::TempDir) -> PathBuf {
        directory.path().join("aliases.json")
    }

    #[test]
    fn creates_scripts_with_trimmed_values() {
        let script = ScriptAlias::new("  build  ", "  cargo build  ").expect("script");

        assert_eq!(script.alias, "build");
        assert_eq!(script.command, "cargo build");
    }

    #[test]
    fn rejects_empty_values() {
        assert!(ScriptAlias::new("", "cargo test").is_err());
        assert!(ScriptAlias::new("test", "").is_err());
    }

    #[test]
    fn saves_and_loads_aliases() {
        let directory = tempfile::tempdir().expect("tempdir");
        let path = store_path(&directory);
        let mut store = AliasStore::open(path.clone()).expect("open");

        store.upsert(ScriptAlias::new("test", "cargo test").expect("script"));
        store.save().expect("save");

        let loaded = AliasStore::open(path).expect("reload");

        assert_eq!(loaded.scripts().len(), 1);
        assert_eq!(
            loaded.get("test").expect("test alias").command,
            "cargo test"
        );
    }

    #[test]
    fn upsert_replaces_existing_alias() {
        let directory = tempfile::tempdir().expect("tempdir");
        let mut store = AliasStore::open(store_path(&directory)).expect("open");

        assert!(!store.upsert(ScriptAlias::new("test", "cargo test").expect("script")));
        assert!(store.upsert(ScriptAlias::new("test", "cargo test --all").expect("script")));

        assert_eq!(store.scripts().len(), 1);
        assert_eq!(
            store.get("test").expect("test alias").command,
            "cargo test --all"
        );
    }

    #[test]
    fn remove_deletes_matching_alias() {
        let directory = tempfile::tempdir().expect("tempdir");
        let mut store = AliasStore::open(store_path(&directory)).expect("open");
        store.upsert(ScriptAlias::new("fmt", "cargo fmt").expect("script"));

        assert!(store.remove("fmt"));
        assert!(!store.remove("fmt"));
        assert!(store.is_empty());
    }
}
