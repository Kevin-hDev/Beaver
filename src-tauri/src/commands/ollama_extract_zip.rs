use std::path::Path;
use tokio_util::sync::CancellationToken;

const MAX_ZIP_ENTRIES: usize = 50_000;
const MAX_UNPACKED_BYTES: u64 = 4 * 1024 * 1024 * 1024;
const COPY_BUFFER_BYTES: usize = 64 * 1024;

pub(crate) fn extract_zip(
    archive: &Path,
    dest: &Path,
    cancel: &CancellationToken,
) -> Result<(), String> {
    let file = std::fs::File::open(archive).map_err(|e| {
        eprintln!("[ollama-extract] open zip: {e}");
        "ollama-extract-error".to_string()
    })?;
    let mut zip_archive = zip::ZipArchive::new(file).map_err(|e| {
        eprintln!("[ollama-extract] read zip: {e}");
        "ollama-extract-error".to_string()
    })?;
    safe_unpack_zip(&mut zip_archive, dest, cancel)
}

fn safe_unpack_zip(
    archive: &mut zip::ZipArchive<std::fs::File>,
    dest: &Path,
    cancel: &CancellationToken,
) -> Result<(), String> {
    let canonical_dest = std::fs::canonicalize(dest).map_err(|e| {
        eprintln!("[ollama-extract] canonicalize dest: {e}");
        "ollama-extract-error".to_string()
    })?;
    if archive.len() > MAX_ZIP_ENTRIES {
        return Err("zip contient trop d'entrées — extraction refusée".into());
    }
    let mut total_bytes = 0u64;

    for i in 0..archive.len() {
        ensure_not_cancelled(cancel)?;
        let mut entry = archive.by_index(i).map_err(|e| {
            eprintln!("[ollama-extract] zip entry: {e}");
            "ollama-extract-error".to_string()
        })?;

        let raw_path = match entry.enclosed_name() {
            Some(p) => p.to_path_buf(),
            None => return Err("zip contient un chemin non sûr — extraction refusée".into()),
        };

        let target = canonical_dest.join(&raw_path);
        if !target.starts_with(&canonical_dest) {
            return Err("zip contient un chemin non sûr — extraction refusée".into());
        }
        total_bytes += entry.size();
        if total_bytes > MAX_UNPACKED_BYTES {
            return Err("zip trop volumineux — extraction refusée".into());
        }

        if entry.is_dir() {
            std::fs::create_dir_all(&target).map_err(|e| {
                eprintln!("[ollama-extract] mkdir zip: {e}");
                "ollama-extract-error".to_string()
            })?;
            continue;
        }

        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                eprintln!("[ollama-extract] mkdir zip parent: {e}");
                "ollama-extract-error".to_string()
            })?;
        }

        let mut outfile = std::fs::File::create(&target).map_err(|e| {
            eprintln!("[ollama-extract] create zip file: {e}");
            "ollama-extract-error".to_string()
        })?;
        copy_zip_entry(&mut entry, &mut outfile, cancel)?;
    }
    Ok(())
}

fn copy_zip_entry(
    entry: &mut zip::read::ZipFile<'_, std::fs::File>,
    output: &mut std::fs::File,
    cancel: &CancellationToken,
) -> Result<(), String> {
    let mut buffer = [0u8; COPY_BUFFER_BYTES];
    loop {
        ensure_not_cancelled(cancel)?;
        let read = std::io::Read::read(entry, &mut buffer).map_err(|e| {
            eprintln!("[ollama-extract] read zip entry: {e}");
            "ollama-extract-error".to_string()
        })?;
        if read == 0 {
            break;
        }
        std::io::Write::write_all(output, &buffer[..read]).map_err(|e| {
            eprintln!("[ollama-extract] write zip entry: {e}");
            "ollama-extract-error".to_string()
        })?;
    }
    Ok(())
}

fn ensure_not_cancelled(cancel: &CancellationToken) -> Result<(), String> {
    if cancel.is_cancelled() {
        return Err(super::ollama_setup_cancel::cancelled_error());
    }
    Ok(())
}
