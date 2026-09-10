use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

const BLOCK_BYTES: u64 = 64 * 1024;
const MAX_READ_BYTES: u64 = 8 * 1024 * 1024;
pub const MAX_LINES: usize = 5_000;

pub fn last_lines(path: &Path, requested: usize) -> std::io::Result<Vec<String>> {
    let metadata = std::fs::symlink_metadata(path)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(std::io::Error::from(std::io::ErrorKind::InvalidInput));
    }
    let limit = requested.min(MAX_LINES);
    if limit == 0 || metadata.len() == 0 {
        return Ok(Vec::new());
    }
    let mut file = std::fs::File::open(path)?;
    let end = metadata.len();
    let lower = end.saturating_sub(MAX_READ_BYTES);
    let mut start = end;
    let mut newlines = 0_usize;
    let mut block = vec![0_u8; BLOCK_BYTES as usize];
    while start > lower && newlines <= limit {
        let next = start.saturating_sub(BLOCK_BYTES).max(lower);
        let length = (start - next) as usize;
        file.seek(SeekFrom::Start(next))?;
        file.read_exact(&mut block[..length])?;
        newlines = newlines.saturating_add(
            block[..length]
                .iter()
                .filter(|byte| **byte == b'\n')
                .count(),
        );
        start = next;
    }
    let starts_mid_line = if start == 0 {
        false
    } else {
        file.seek(SeekFrom::Start(start - 1))?;
        let mut previous = [0_u8; 1];
        file.read_exact(&mut previous)?;
        previous[0] != b'\n'
    };
    file.seek(SeekFrom::Start(start))?;
    let mut bytes = vec![0_u8; (end - start) as usize];
    file.read_exact(&mut bytes)?;
    if starts_mid_line {
        if let Some(newline) = bytes.iter().position(|byte| *byte == b'\n') {
            bytes.drain(..=newline);
        } else {
            bytes.clear();
        }
    }
    let mut parts = bytes.rsplit(|byte| *byte == b'\n');
    if bytes.last() == Some(&b'\n') {
        parts.next();
    }
    let mut lines = parts
        .take(limit)
        .map(|line| {
            let line = line.strip_suffix(b"\r").unwrap_or(line);
            String::from_utf8_lossy(line).into_owned()
        })
        .collect::<Vec<_>>();
    lines.reverse();
    Ok(lines)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dernieres_lignes_simples() {
        let directory = tempfile::TempDir::new().expect("temporary directory");
        let file = directory.path().join("x.log");
        std::fs::write(&file, "a\nb\nc\nd\n").expect("log");
        assert_eq!(last_lines(&file, 2).expect("tail"), vec!["c", "d"]);
    }

    #[test]
    fn n_plus_grand_que_le_fichier() {
        let directory = tempfile::TempDir::new().expect("temporary directory");
        let file = directory.path().join("x.log");
        std::fs::write(&file, "a\nb\n").expect("log");
        assert_eq!(last_lines(&file, 50).expect("tail"), vec!["a", "b"]);
    }

    #[test]
    fn fichier_absent_est_une_erreur_io() {
        let directory = tempfile::TempDir::new().expect("temporary directory");
        assert!(last_lines(&directory.path().join("missing.log"), 5).is_err());
    }

    #[test]
    fn utf8_multioctets_intact() {
        let directory = tempfile::TempDir::new().expect("temporary directory");
        let file = directory.path().join("x.log");
        std::fs::write(&file, "été\nnoël\n").expect("log");
        assert_eq!(last_lines(&file, 1).expect("tail"), vec!["noël"]);
    }

    #[cfg(unix)]
    #[test]
    fn refuse_un_journal_symbolique() {
        use std::os::unix::fs::symlink;

        let directory = tempfile::TempDir::new().expect("temporary directory");
        let target = directory.path().join("target.log");
        std::fs::write(&target, "secret\n").expect("target");
        let link = directory.path().join("linked.log");
        symlink(target, &link).expect("symlink");
        assert!(last_lines(&link, 1).is_err());
    }
}
