use serde::Deserialize;

pub(crate) use super::types::ExtensionResultFilePurpose as FilePurpose;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub(crate) enum ToolResultContent {
    Text(String),
    Blocks(Vec<ToolResultBlock>),
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase", deny_unknown_fields)]
pub(crate) enum ToolResultBlock {
    Text {
        text: String,
    },
    File {
        path: String,
        purpose: FilePurpose,
        #[serde(default, rename = "displayName")]
        display_name: Option<String>,
    },
}

pub(crate) fn checked_add(total: usize, next: usize, limit: usize) -> Result<usize, ()> {
    let value = total.checked_add(next).ok_or(())?;
    (value <= limit).then_some(value).ok_or(())
}

pub(crate) fn validate(content: &ToolResultContent) -> Result<(), ()> {
    let ToolResultContent::Blocks(blocks) = content else {
        return Ok(());
    };
    if blocks.len() > super::types::MAX_RESULT_BLOCKS {
        return Err(());
    }
    let mut files = 0usize;
    let mut bytes = 0usize;
    for block in blocks {
        match block {
            ToolResultBlock::Text { text } => {
                bytes = checked_add(bytes, text.len(), super::types::MAX_RESULT_TEXT_BYTES)?;
            }
            ToolResultBlock::File {
                path,
                purpose,
                display_name,
            } => {
                files = checked_add(files, 1, super::types::MAX_RESULT_FILES)?;
                if path.is_empty()
                    || path.chars().count() > super::types::MAX_PATH_CHARS
                    || display_name.as_ref().is_some_and(|name| {
                        name.is_empty()
                            || name.chars().count() > super::types::MAX_EXTENSION_NAME_CHARS
                            || name.chars().any(char::is_control)
                    })
                {
                    return Err(());
                }
                let _ = purpose;
            }
        }
    }
    let _ = (files, bytes);
    Ok(())
}
