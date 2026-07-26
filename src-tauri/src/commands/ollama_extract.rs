use std::io::Read;
use std::path::Path;
use tokio_util::sync::CancellationToken;

const MAX_TAR_ENTRIES: usize = 50_000;
const MAX_UNPACKED_BYTES: u64 = 4 * 1024 * 1024 * 1024;
const COPY_BUFFER_BYTES: usize = 64 * 1024;

pub fn extract_overlay(
    archive: &Path,
    dest: &Path,
    name: &str,
    cancel: &CancellationToken,
) -> Result<(), String> {
    if name.ends_with(".tgz") || name.ends_with(".tar.gz") {
        extract_tar_gz(archive, dest, cancel)?;
    } else if name.ends_with(".tar.zst") {
        extract_tar_zst(archive, dest, cancel)?;
    } else if name.ends_with(".zip") {
        super::ollama_extract_zip::extract_zip(archive, dest, cancel)?;
    } else {
        return Err(format!("format inconnu: {name}"));
    }
    Ok(())
}

fn extract_tar_gz(archive: &Path, dest: &Path, cancel: &CancellationToken) -> Result<(), String> {
    let file = std::fs::File::open(archive).map_err(|e| {
        eprintln!("[ollama-extract] open tar.gz: {e}");
        "ollama-extract-error".to_string()
    })?;
    let gz = flate2::read::GzDecoder::new(file);
    let tar = tar::Archive::new(gz);
    safe_unpack_tar(tar, dest, cancel)
}

fn extract_tar_zst(archive: &Path, dest: &Path, cancel: &CancellationToken) -> Result<(), String> {
    let file = std::fs::File::open(archive).map_err(|e| {
        eprintln!("[ollama-extract] open tar.zst: {e}");
        "ollama-extract-error".to_string()
    })?;
    let zst = zstd::Decoder::new(file).map_err(|e| {
        eprintln!("[ollama-extract] zstd decode: {e}");
        "ollama-extract-error".to_string()
    })?;
    let tar = tar::Archive::new(zst);
    safe_unpack_tar(tar, dest, cancel)
}

pub(crate) fn safe_unpack_tar<R: Read>(
    mut archive: tar::Archive<R>,
    dest: &Path,
    cancel: &CancellationToken,
) -> Result<(), String> {
    let canonical_dest = std::fs::canonicalize(dest).map_err(|e| {
        eprintln!("[ollama-extract] canonicalize dest: {e}");
        "ollama-extract-error".to_string()
    })?;

    let mut entry_count: usize = 0;
    let mut total_bytes: u64 = 0;

    ensure_not_cancelled(cancel)?;
    let entries = archive.entries().map_err(|e| {
        eprintln!("[ollama-extract] tar entries: {e}");
        "ollama-extract-error".to_string()
    })?;

    for entry_result in entries {
        ensure_not_cancelled(cancel)?;
        entry_count += 1;
        if entry_count > MAX_TAR_ENTRIES {
            return Err("archive contient trop d'entrées — extraction refusée".into());
        }
        let mut entry = entry_result.map_err(|e| {
            eprintln!("[ollama-extract] tar entry: {e}");
            "ollama-extract-error".to_string()
        })?;
        let raw_path = entry
            .path()
            .map_err(|e| {
                eprintln!("[ollama-extract] tar path: {e}");
                "ollama-extract-error".to_string()
            })?
            .into_owned();

        if super::ollama_extract_tar_safe::has_parent_dir(&raw_path) {
            return Err("archive contient un chemin avec '..' — extraction refusée".into());
        }

        let entry_type = entry.header().entry_type();
        if matches!(entry_type, tar::EntryType::Symlink) {
            let target =
                super::ollama_extract_tar_safe::safe_target_path(&canonical_dest, dest, &raw_path)?;
            super::ollama_extract_tar_safe::unpack_safe_symlink(&entry, &canonical_dest, &target)?;
            continue;
        }
        if matches!(
            entry_type,
            tar::EntryType::Link
                | tar::EntryType::Block
                | tar::EntryType::Char
                | tar::EntryType::Fifo
        ) {
            return Err(format!(
                "type d'entrée non autorisé dans l'archive: {:?}",
                entry_type
            ));
        }

        let target_canonical =
            super::ollama_extract_tar_safe::safe_target_path(&canonical_dest, dest, &raw_path)?;

        total_bytes += entry.header().size().unwrap_or(0);
        if total_bytes > MAX_UNPACKED_BYTES {
            return Err("archive trop volumineuse — extraction refusée".into());
        }

        unpack_entry(&mut entry, &target_canonical, cancel)?;
    }
    Ok(())
}

fn ensure_not_cancelled(cancel: &CancellationToken) -> Result<(), String> {
    if cancel.is_cancelled() {
        return Err(super::ollama_setup_cancel::cancelled_error());
    }
    Ok(())
}

fn unpack_entry<R: Read>(
    entry: &mut tar::Entry<R>,
    target: &Path,
    cancel: &CancellationToken,
) -> Result<(), String> {
    let entry_type = entry.header().entry_type();
    if entry_type.is_dir() {
        std::fs::create_dir_all(target).map_err(|e| {
            eprintln!("[ollama-extract] mkdir entry: {e}");
            "ollama-extract-error".to_string()
        })?;
        return Ok(());
    }

    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            eprintln!("[ollama-extract] mkdir parent: {e}");
            "ollama-extract-error".to_string()
        })?;
    }
    let mut output = std::fs::File::create(target).map_err(|e| {
        eprintln!("[ollama-extract] create file: {e}");
        "ollama-extract-error".to_string()
    })?;
    let mut buffer = [0u8; COPY_BUFFER_BYTES];
    loop {
        ensure_not_cancelled(cancel)?;
        let read = entry.read(&mut buffer).map_err(|e| {
            eprintln!("[ollama-extract] read entry: {e}");
            "ollama-extract-error".to_string()
        })?;
        if read == 0 {
            break;
        }
        std::io::Write::write_all(&mut output, &buffer[..read]).map_err(|e| {
            eprintln!("[ollama-extract] write entry: {e}");
            "ollama-extract-error".to_string()
        })?;
    }

    #[cfg(unix)]
    if let Ok(mode) = entry.header().mode() {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(target, std::fs::Permissions::from_mode(mode));
    }

    Ok(())
}
