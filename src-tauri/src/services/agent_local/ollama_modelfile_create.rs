use std::io::Write;
use std::time::Duration;

const MAX_MODELFILE_BYTES: usize = 2 * 1024 * 1024;
const CREATE_TIMEOUT: Duration = Duration::from_secs(10 * 60);

pub async fn create_from_modelfile(
    ollama: &super::ollama_client::OllamaClient,
    name: &str,
    content: &str,
) -> Result<(), String> {
    super::model_customizations::validate_model_name(name)?;
    validate_content(content)?;

    let mut file = tempfile::NamedTempFile::new().map_err(|error| {
        ::log::error!("[ollama-modelfile] temporary file: {error}");
        "ollama-create-error".to_string()
    })?;
    file.write_all(content.as_bytes()).map_err(|error| {
        ::log::error!("[ollama-modelfile] temporary write: {error}");
        "ollama-create-error".to_string()
    })?;
    file.flush().map_err(|error| {
        ::log::error!("[ollama-modelfile] temporary flush: {error}");
        "ollama-create-error".to_string()
    })?;

    let args = crate::services::ollama_manager::OllamaCliArgs::Create {
        model: name.to_string(),
        modelfile: file.path().to_path_buf(),
    };
    let result = tokio::time::timeout(CREATE_TIMEOUT, ollama.manager().run_cli(args))
        .await
        .map_err(|_| "ollama-create-timeout".to_string())?
        .map_err(|error| error.as_str().to_string())?;
    if result.success {
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
