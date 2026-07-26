use super::diff_preview_model::{GitDiffHunk, GitDiffLine, GitDiffPreview};
use git2::{DiffLineType, Patch};

pub(super) const MAX_HUNKS: usize = 100;
pub(super) const MAX_LINES: usize = 2_000;
pub(super) const MAX_LINE_BYTES: usize = 16 * 1024;
pub(super) const MAX_TOTAL_BYTES: usize = 2 * 1024 * 1024;

pub(super) fn serialize_patch(patch: &Patch<'_>) -> GitDiffPreview {
    let mut preview = GitDiffPreview {
        hunks: Vec::new(),
        truncated: false,
        binary: false,
    };
    let mut total_lines = 0;
    let mut total_bytes = 0;
    if patch.num_hunks() > MAX_HUNKS {
        preview.truncated = true;
    }
    for hunk_index in 0..patch.num_hunks().min(MAX_HUNKS) {
        let Ok((header, line_count)) = patch.hunk(hunk_index) else {
            preview.truncated = true;
            break;
        };
        let mut lines = Vec::new();
        for line_index in 0..line_count {
            if total_lines >= MAX_LINES || total_bytes >= MAX_TOTAL_BYTES {
                preview.truncated = true;
                break;
            }
            let Ok(line) = patch.line_in_hunk(hunk_index, line_index) else {
                preview.truncated = true;
                break;
            };
            let kind = match line.origin_value() {
                DiffLineType::Context => "context",
                DiffLineType::Addition => "added",
                DiffLineType::Deletion => "deleted",
                _ => continue,
            };
            let (content, shortened) = bounded_line(line.content());
            if total_bytes.saturating_add(content.len()) > MAX_TOTAL_BYTES {
                preview.truncated = true;
                break;
            }
            preview.truncated |= shortened;
            total_bytes += content.len();
            total_lines += 1;
            lines.push(GitDiffLine {
                kind: kind.to_string(),
                content,
                old_line: line.old_lineno(),
                new_line: line.new_lineno(),
            });
        }
        preview.hunks.push(GitDiffHunk {
            old_start: header.old_start(),
            old_lines: header.old_lines(),
            new_start: header.new_start(),
            new_lines: header.new_lines(),
            lines,
        });
        if total_lines >= MAX_LINES || total_bytes >= MAX_TOTAL_BYTES {
            break;
        }
    }
    preview
}

fn bounded_line(bytes: &[u8]) -> (String, bool) {
    let end = bytes
        .iter()
        .rposition(|byte| *byte != b'\n' && *byte != b'\r')
        .map_or(0, |i| i + 1);
    let capped = end.min(MAX_LINE_BYTES);
    (
        String::from_utf8_lossy(&bytes[..capped]).into_owned(),
        capped < end,
    )
}
