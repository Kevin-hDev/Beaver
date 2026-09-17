use regex::Regex;
use std::sync::LazyLock;
use zeroize::{Zeroize, Zeroizing};

const REDACTED: &str = "[REDACTED]";

static SECRET_VALUE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r#"(?i)\b((?:[a-z0-9]+[_-])*(?:api[_-]?key|apikey|token|secret|password|authorization|client_secret|access_token|refresh_token))("?\s*[:=]\s*)("[^"]*"|'[^']*'|[^\s,}]+)"#,
    )
    .expect("secret value regex")
});

static TOKEN_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(?ix)
        \b(?:
            sk-(?:proj-)?[a-z0-9_-]{8,}
          | gsk_[a-z0-9_-]{16,}
          | xai-[a-z0-9_-]{16,}
          | csk-[a-z0-9_-]{16,}
          | hf_[a-z0-9_-]{16,}
          | bearer[\t ]+[a-z0-9._~+/=-]{8,}
          | gh(?:p|o|u|s|r)_[a-z0-9_-]{8,}
          | github_pat_[a-z0-9_-]{8,}
          | glpat-[a-z0-9_-]{8,}
          | xapp-[a-z0-9-]{8,}
          | xox[a-z]-[a-z0-9-]{8,}
          | [0-9]{5,}:[a-z0-9_-]{20,}
          | (?:AKIA|ASIA)[A-Z0-9]{16}
          | AIza[a-z0-9_-]{35}
          | https://hooks[.]slack[.]com/services/[a-z0-9_-]{8,64}/[a-z0-9_-]{8,64}/[a-z0-9_-]{8,128}
          | [a-z0-9_-]{20,}\.[a-z0-9_-]{5,}\.[a-z0-9_-]{20,}
        )",
    )
    .expect("token regex")
});

static BEARER_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?i)Bearer\s+[^\s,}\"']+"#).expect("bearer regex")
});

static PEM_BLOCK_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?is)-----BEGIN [A-Z0-9 ]*(PRIVATE KEY|CERTIFICATE)[A-Z0-9 ]*-----.*?-----END [A-Z0-9 ]*(PRIVATE KEY|CERTIFICATE)[A-Z0-9 ]*-----")
        .expect("pem regex")
});

pub fn redact_text(content: &str) -> String {
    let mut current = redact_high_confidence(content);
    replace_all(&mut current, &SECRET_VALUE_RE, "$1$2[REDACTED]");
    current.to_string()
}

pub fn redact_high_confidence_text(content: &str) -> String {
    redact_high_confidence(content).to_string()
}

fn redact_high_confidence(content: &str) -> Zeroizing<String> {
    let mut current = Zeroizing::new(content.to_string());
    replace_all(&mut current, &PEM_BLOCK_RE, REDACTED);
    replace_all(&mut current, &BEARER_RE, REDACTED);
    replace_all(&mut current, &TOKEN_RE, REDACTED);
    current
}

pub fn redact_string(content: &mut String) {
    let redacted = redact_text(content);
    content.zeroize();
    *content = redacted;
}

pub fn redact_high_confidence_string(content: &mut String) {
    let redacted = redact_high_confidence_text(content);
    content.zeroize();
    *content = redacted;
}

fn replace_all(current: &mut Zeroizing<String>, regex: &Regex, replacement: &str) {
    let next = regex.replace_all(current.as_str(), replacement).into_owned();
    current.zeroize();
    **current = next;
}

include!("sensitive_data_redaction_json.rs");
