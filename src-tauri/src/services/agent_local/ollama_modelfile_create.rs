use std::io::Write;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;

const MAX_MODELFILE_BYTES: usize = 2 * 1024 * 1024;
const CREATE_TIMEOUT: Duration = Duration::from_secs(10 * 60);

pub async fn create_from_modelfile(name: &str, content: &str) -> Result<(), String> {
    super::model_customizations::validate_model_name(name)?;
    validate_content(content)?;

    let mut file = tempfile::NamedTempFile::new().map_err(|error| {
        eprintln!("[ollama-modelfile] temporary file: {error}");
        "ollama-create-error".to_string()
    })?;
    file.write_all(content.as_bytes()).map_err(|error| {
        eprintln!("[ollama-modelfile] temporary write: {error}");
        "ollama-create-error".to_string()
    })?;
    file.flush().map_err(|error| {
        eprintln!("[ollama-modelfile] temporary flush: {error}");
        "ollama-create-error".to_string()
    })?;

    let binary = crate::services::ollama_lifecycle::ollama_binary_path()
        .map_err(|_| "ollama-create-error".to_string())?;
    let mut command = Command::new(binary);
    command
        .arg("create")
        .arg(name)
        .arg("--file")
        .arg(file.path())
        .env("OLLAMA_HOST", super::ollama_base_url())
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .kill_on_drop(true);

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        command.as_std_mut().creation_flags(0x08000000);
    }

    let status = tokio::time::timeout(CREATE_TIMEOUT, command.status())
        .await
        .map_err(|_| "ollama-create-timeout".to_string())?
        .map_err(|error| {
            eprintln!("[ollama-modelfile] command unavailable: {error}");
            "ollama-create-error".to_string()
        })?;
    if status.success() {
        Ok(())
    } else {
        Err("ollama-create-error".into())
    }
}

pub fn use_updated_base(content: &str, name: &str) -> String {
    let mut replaced = false;
    content
        .lines()
        .map(|line| {
            if !replaced && is_from_directive(line) {
                replaced = true;
                format!("FROM {name}")
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn validate_content(content: &str) -> Result<(), String> {
    if content.trim().is_empty()
        || content.len() > MAX_MODELFILE_BYTES
        || content.contains('\0')
    {
        return Err("ollama-modelfile-invalid".into());
    }
    Ok(())
}

fn is_from_directive(line: &str) -> bool {
    line.trim_start()
        .split_ascii_whitespace()
        .next()
        .is_some_and(|word| word.eq_ignore_ascii_case("FROM"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_modelfile_size_and_nul_bytes() {
        assert!(validate_content("FROM gemma4").is_ok());
        assert!(validate_content("").is_err());
        assert!(validate_content("FROM gemma4\0SYSTEM test").is_err());
        assert!(validate_content(&"x".repeat(MAX_MODELFILE_BYTES + 1)).is_err());
    }

    #[test]
    fn update_preserves_every_directive_except_base() {
        let input = "FROM old\nADAPTER ./adapter.gguf\nMESSAGE user hello\nRENDERER llama3";
        let result = use_updated_base(input, "gemma4:e2b");
        assert_eq!(
            result,
            "FROM gemma4:e2b\nADAPTER ./adapter.gguf\nMESSAGE user hello\nRENDERER llama3"
        );
    }
}
