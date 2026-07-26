use ignore::WalkBuilder;
use std::path::Path;
use tokio::process::Command;

const MAX_SNAPSHOT_FILES: usize = 20_000;
const MAX_SNAPSHOT_BYTES: u64 = 512 * 1024 * 1024;
const MAX_SNAPSHOT_FILE_BYTES: u64 = 64 * 1024 * 1024;

pub async fn validate_source(project: &Path) -> Result<(), String> {
    let project = project.to_path_buf();
    tokio::task::spawn_blocking(move || {
        let mut files = 0usize;
        let mut bytes = 0u64;
        for entry in WalkBuilder::new(project).hidden(false).build() {
            let entry = entry.map_err(|_| generic_error())?;
            let metadata = entry.metadata().map_err(|_| generic_error())?;
            if !metadata.is_file() {
                continue;
            }
            files = files.saturating_add(1);
            bytes = bytes.saturating_add(metadata.len());
            if files > MAX_SNAPSHOT_FILES
                || bytes > MAX_SNAPSHOT_BYTES
                || metadata.len() > MAX_SNAPSHOT_FILE_BYTES
            {
                return Err(generic_error());
            }
        }
        Ok(())
    })
    .await
    .map_err(|_| generic_error())?
}

pub async fn repository_is_bounded(repository: &Path) -> bool {
    let output = Command::new("git")
        .arg("--git-dir")
        .arg(repository)
        .args(["count-objects", "-v"])
        .kill_on_drop(true)
        .output()
        .await;
    let Ok(output) = output else { return false };
    if !output.status.success() || output.stdout.len() > 8_192 {
        return false;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let kib = text
        .lines()
        .filter_map(|line| {
            let (key, value) = line.split_once(':')?;
            matches!(key, "size" | "size-pack")
                .then(|| value.trim().parse::<u64>().ok())
                .flatten()
        })
        .sum::<u64>();
    kib <= MAX_SNAPSHOT_BYTES / 1024
}

fn generic_error() -> String {
    "Préparation du dossier isolé impossible".to_string()
}
