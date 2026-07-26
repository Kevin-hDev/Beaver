use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GitDiffPreview {
    pub hunks: Vec<GitDiffHunk>,
    pub truncated: bool,
    pub binary: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GitDiffHunk {
    pub old_start: u32,
    pub old_lines: u32,
    pub new_start: u32,
    pub new_lines: u32,
    pub lines: Vec<GitDiffLine>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GitDiffLine {
    pub kind: String,
    pub content: String,
    pub old_line: Option<u32>,
    pub new_line: Option<u32>,
}
