use url::Url;

const MAX_GIT_REF_CHARS: usize = 200;
const MAX_NPM_NAME_CHARS: usize = 214;
const MAX_NPM_SELECTOR_CHARS: usize = 96;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitSource {
    pub locator: String,
    pub clone_url: String,
    pub reference: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NpmSource {
    pub locator: String,
    pub package_name: String,
}

pub fn git(input: &str) -> Result<GitSource, String> {
    bounded_input(input, super::types::MAX_GIT_LOCATOR_CHARS)?;
    let (remote, reference) = split_reference(input)?;
    let clone_url = if !remote.contains("://") && remote.contains('@') {
        validate_scp_remote(remote)?;
        remote.to_string()
    } else {
        validate_url_remote(remote)?
    };
    Ok(GitSource {
        locator: input.to_string(),
        clone_url,
        reference,
    })
}

pub fn npm(input: &str) -> Result<NpmSource, String> {
    bounded_input(input, super::types::MAX_NPM_SPEC_CHARS)?;
    let selector_at = if input.starts_with('@') {
        let slash = input
            .find('/')
            .ok_or_else(|| "Package npm invalide.".to_string())?;
        input[slash + 1..].rfind('@').map(|index| slash + 1 + index)
    } else {
        input.rfind('@').filter(|index| *index > 0)
    };
    let (name, selector) = selector_at
        .map(|index| (&input[..index], Some(&input[index + 1..])))
        .unwrap_or((input, None));
    validate_package_name(name)?;
    if selector.is_some_and(|value| !valid_selector(value)) {
        return Err("Version ou tag npm invalide.".to_string());
    }
    Ok(NpmSource {
        locator: input.to_string(),
        package_name: name.to_string(),
    })
}

fn split_reference(input: &str) -> Result<(&str, Option<String>), String> {
    let Some((remote, reference)) = input.rsplit_once('#') else {
        return Ok((input, None));
    };
    if remote.contains('#') || !valid_git_reference(reference) {
        return Err("Référence Git invalide.".to_string());
    }
    Ok((remote, Some(reference.to_string())))
}

fn validate_url_remote(input: &str) -> Result<String, String> {
    let normalized = input.strip_prefix("git+").unwrap_or(input).to_string();
    let parsed = Url::parse(&normalized).map_err(|_| "URL Git invalide.".to_string())?;
    if !matches!(parsed.scheme(), "https" | "ssh")
        || parsed.host_str().is_none()
        || parsed.password().is_some()
        || parsed.query().is_some()
        || (parsed.scheme() == "https" && !parsed.username().is_empty())
        || parsed.path().trim_matches('/').is_empty()
    {
        return Err("URL Git invalide.".to_string());
    }
    Ok(normalized)
}

fn validate_scp_remote(input: &str) -> Result<(), String> {
    let (account, path) = input
        .split_once(':')
        .ok_or_else(|| "URL Git invalide.".to_string())?;
    let (user, host) = account
        .split_once('@')
        .ok_or_else(|| "URL Git invalide.".to_string())?;
    let valid = valid_segment(user)
        && valid_host(host)
        && !path.is_empty()
        && path
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "/._~-".contains(character))
        && !path
            .split('/')
            .any(|segment| segment.is_empty() || segment == "..");
    valid
        .then_some(())
        .ok_or_else(|| "URL Git invalide.".to_string())
}

fn validate_package_name(name: &str) -> Result<(), String> {
    if name.chars().count() > MAX_NPM_NAME_CHARS {
        return Err("Nom de package npm invalide.".to_string());
    }
    let valid = if let Some(scoped) = name.strip_prefix('@') {
        scoped
            .split_once('/')
            .is_some_and(|(scope, package)| valid_segment(scope) && valid_segment(package))
    } else {
        valid_segment(name)
    };
    valid
        .then_some(())
        .ok_or_else(|| "Nom de package npm invalide.".to_string())
}

fn valid_segment(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .next()
            .is_some_and(|character| character.is_ascii_lowercase() || character.is_ascii_digit())
        && value.chars().all(|character| {
            character.is_ascii_lowercase()
                || character.is_ascii_digit()
                || "._~-".contains(character)
        })
}

fn valid_host(value: &str) -> bool {
    !value.is_empty()
        && !value.starts_with('-')
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || ".-".contains(character))
}

fn valid_selector(value: &str) -> bool {
    !value.is_empty()
        && value.chars().count() <= MAX_NPM_SELECTOR_CHARS
        && value
            .chars()
            .next()
            .is_some_and(|character| character.is_ascii_alphanumeric())
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "._+-".contains(character))
}

fn valid_git_reference(value: &str) -> bool {
    let commit_hash = matches!(value.len(), 40 | 64)
        && value.chars().all(|character| character.is_ascii_hexdigit());
    !value.is_empty()
        && value.chars().count() <= MAX_GIT_REF_CHARS
        && !commit_hash
        && !value.starts_with(['-', '/', '.'])
        && !value.ends_with(['/', '.'])
        && !value.contains("..")
        && !value.contains("@{")
        && !value.contains("//")
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "/._-".contains(character))
}

fn bounded_input(input: &str, maximum: usize) -> Result<(), String> {
    let valid = !input.is_empty()
        && input.trim() == input
        && input.chars().count() <= maximum
        && !input.chars().any(char::is_control);
    valid
        .then_some(())
        .ok_or_else(|| "Source d'extension invalide.".to_string())
}
