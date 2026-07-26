use std::fs::File;
use std::io::{ErrorKind, Read};
use std::path::Path;
use subtle::{Choice, ConstantTimeEq};

const READ_CHUNK_BYTES: usize = 64 * 1024;
const COOKIE_STORE_FILES: [&str; 4] = ["Cookies", "Cookies-journal", "Cookies-wal", "Cookies-shm"];

pub(super) const PROBE_VALUE_BYTES: usize = 64;
pub(super) const MAX_COOKIE_STORE_BYTES: u64 = 256 * 1024 * 1024;

pub(super) fn cookie_store_hides_probe(profile: &Path, probe: &[u8]) -> Result<bool, ()> {
    if !valid_probe(probe) {
        return Err(());
    }
    let store_root = profile.join("Default");
    let main_store = store_root.join(COOKIE_STORE_FILES[0]);
    if !main_store.is_file() {
        return Err(());
    }

    let mut plaintext_found = Choice::from(0);
    for name in COOKIE_STORE_FILES {
        let path = store_root.join(name);
        match scan_file(&path, probe) {
            Ok(Some(found)) => plaintext_found |= found,
            Ok(None) if name != COOKIE_STORE_FILES[0] => {}
            _ => return Err(()),
        }
    }
    Ok(!bool::from(plaintext_found))
}

fn valid_probe(probe: &[u8]) -> bool {
    probe.len() == PROBE_VALUE_BYTES && probe.iter().all(u8::is_ascii_hexdigit)
}

fn scan_file(path: &Path, probe: &[u8]) -> Result<Option<Choice>, ()> {
    let metadata = match path.symlink_metadata() {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(()),
    };
    if !metadata.file_type().is_file() || metadata.len() > MAX_COOKIE_STORE_BYTES {
        return Err(());
    }
    let mut file = File::open(path).map_err(|_| ())?;
    let mut buffer = [0_u8; READ_CHUNK_BYTES + PROBE_VALUE_BYTES - 1];
    let mut tail_len = 0_usize;
    let mut total_read = 0_u64;
    let mut found = Choice::from(0);

    loop {
        let read = file
            .read(&mut buffer[tail_len..tail_len + READ_CHUNK_BYTES])
            .map_err(|_| ())?;
        if read == 0 {
            break;
        }
        total_read = total_read.checked_add(read as u64).ok_or(())?;
        if total_read > MAX_COOKIE_STORE_BYTES {
            return Err(());
        }
        let available = tail_len + read;
        for window in buffer[..available].windows(probe.len()) {
            found |= window.ct_eq(probe);
        }
        tail_len = available.min(probe.len() - 1);
        buffer.copy_within(available - tail_len..available, 0);
    }
    Ok(Some(found))
}
