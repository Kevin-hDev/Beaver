//! Outils de fixture debug : aucune entrée ne désigne une commande ou un chemin.

use serde_json::{json, Value};
use std::path::PathBuf;
use tempfile::TempDir;

const WRITE_NOTE: &str = "fixture.write_note";
const READ_NOTE: &str = "fixture.read_note";
const NOTE_FILE: &str = "fixture-note.txt";
const MAX_NOTE_BYTES: usize = 1024;
const MAX_STALE_DIRECTORIES: usize = 64;

pub struct FixtureToolset {
    directory: TempDir,
}

impl FixtureToolset {
    pub async fn execute(&mut self, tool_id: &str, arguments: &Value) -> Result<Value, String> {
        match tool_id {
            WRITE_NOTE => self.write_note(arguments).await,
            READ_NOTE => self.read_note(arguments).await,
            _ => Err(unavailable()),
        }
    }

    async fn write_note(&self, arguments: &Value) -> Result<Value, String> {
        let value = write_value(arguments)?;
        crate::services::private_store::atomic_write_async(
            self.note_path(),
            value.as_bytes().to_vec(),
        )
        .await
        .map_err(|_| unavailable())?;
        Ok(json!({ "written": true }))
    }

    async fn read_note(&self, arguments: &Value) -> Result<Value, String> {
        exact_empty_object(arguments)?;
        let crate::services::private_store::BoundedFile::Content(value) =
            crate::services::private_store::read_bounded_regular_async(
                self.note_path(),
                MAX_NOTE_BYTES as u64,
            )
            .await
            .map_err(|_| unavailable())?
        else {
            return Err(unavailable());
        };
        let value = String::from_utf8(value).map_err(|_| unavailable())?;
        Ok(json!({ "value": value }))
    }

    fn note_path(&self) -> PathBuf {
        self.directory.path().join(NOTE_FILE)
    }

    #[cfg(test)]
    pub(crate) fn root_for_test(&self) -> PathBuf {
        self.directory.path().to_path_buf()
    }

    #[cfg(all(test, unix))]
    pub(crate) fn note_path_for_test(&self) -> PathBuf {
        self.note_path()
    }
}

pub async fn isolated_toolset() -> Result<FixtureToolset, String> {
    isolated_toolset_in(crate::services::paths::data_dir().join("reasoning-fixture-runtime")).await
}

#[cfg(debug_assertions)]
pub(crate) fn purge_stale_runtime() -> Result<(), String> {
    purge_stale_runtime_in(&crate::services::paths::data_dir().join("reasoning-fixture-runtime"))
}

#[cfg(any(debug_assertions, test))]
fn purge_stale_runtime_in(runtime: &std::path::Path) -> Result<(), String> {
    let metadata = match std::fs::symlink_metadata(runtime) {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(_) => return Err(unavailable()),
    };
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(unavailable());
    }
    for (index, entry) in std::fs::read_dir(runtime)
        .map_err(|_| unavailable())?
        .enumerate()
    {
        if index >= MAX_STALE_DIRECTORIES {
            return Err(unavailable());
        }
        let entry = entry.map_err(|_| unavailable())?;
        let file_type = entry.file_type().map_err(|_| unavailable())?;
        let is_fixture = entry
            .file_name()
            .to_str()
            .is_some_and(|name| name.starts_with("fixture-"));
        if file_type.is_symlink() || !file_type.is_dir() || !is_fixture {
            return Err(unavailable());
        }
        std::fs::remove_dir_all(entry.path()).map_err(|_| unavailable())?;
    }
    Ok(())
}

pub(crate) async fn isolated_toolset_in(runtime: PathBuf) -> Result<FixtureToolset, String> {
    create_in(runtime).await
}

async fn create_in(runtime: PathBuf) -> Result<FixtureToolset, String> {
    crate::services::private_store::ensure_private_dir_async(runtime.clone())
        .await
        .map_err(|_| unavailable())?;
    tokio::task::spawn_blocking(move || {
        let directory = tempfile::Builder::new()
            .prefix("fixture-")
            .tempdir_in(runtime)
            .map_err(|_| unavailable())?;
        crate::services::private_store::repair_path(directory.path()).map_err(|_| unavailable())?;
        Ok(FixtureToolset { directory })
    })
    .await
    .map_err(|_| unavailable())?
}

fn write_value(arguments: &Value) -> Result<&str, String> {
    let Some(object) = arguments.as_object() else {
        return Err(unavailable());
    };
    if object.len() != 1 {
        return Err(unavailable());
    }
    let Some(value) = object.get("value").and_then(Value::as_str) else {
        return Err(unavailable());
    };
    (!value.is_empty() && value.len() <= MAX_NOTE_BYTES && !value.chars().any(char::is_control))
        .then_some(value)
        .ok_or_else(unavailable)
}

fn exact_empty_object(arguments: &Value) -> Result<(), String> {
    arguments
        .as_object()
        .filter(|object| object.is_empty())
        .map(|_| ())
        .ok_or_else(unavailable)
}

fn unavailable() -> String {
    "Outil de fixture indisponible".to_string()
}

#[cfg(test)]
#[path = "reasoning_fixture_tools_tests.rs"]
mod tests;
