use super::memory_types::MemoryMode;
use std::collections::VecDeque;
use std::sync::{LazyLock, Mutex};

const MAX_ACTIVE_TURNS: usize = 32;

#[derive(Debug, Clone)]
struct TurnPolicy {
    session_id: String,
    nonce: uuid::Uuid,
    mode: MemoryMode,
    write_authorized: bool,
    budget_tokens: usize,
    used_tokens: usize,
}

static POLICIES: LazyLock<Mutex<VecDeque<TurnPolicy>>> =
    LazyLock::new(|| Mutex::new(VecDeque::new()));

pub struct MemoryTurnGuard {
    session_id: String,
    nonce: uuid::Uuid,
}

pub fn begin(
    session_id: &str,
    mode: MemoryMode,
    write_authorized: bool,
    budget_tokens: usize,
    injected_tokens: usize,
) -> MemoryTurnGuard {
    let nonce = uuid::Uuid::new_v4();
    let policy = TurnPolicy {
        session_id: session_id.to_string(),
        nonce,
        mode,
        write_authorized,
        budget_tokens,
        used_tokens: injected_tokens.min(budget_tokens),
    };
    let mut policies = lock_policies();
    policies.retain(|entry| entry.session_id != session_id);
    if policies.len() >= MAX_ACTIVE_TURNS {
        policies.pop_front();
    }
    policies.push_back(policy);
    MemoryTurnGuard {
        session_id: session_id.to_string(),
        nonce,
    }
}

pub fn read_allowed(session_id: &str) -> bool {
    policy(session_id).is_some_and(|policy| policy.mode.is_active())
}

pub fn write_allowed(session_id: &str) -> bool {
    policy(session_id).is_some_and(|policy| {
        policy.mode == MemoryMode::Automatic
            || (policy.mode == MemoryMode::Manual && policy.write_authorized)
    })
}

pub fn consume_result(session_id: &str, content: &str) -> (String, bool) {
    let mut policies = lock_policies();
    let Some(policy) = policies
        .iter_mut()
        .find(|policy| policy.session_id == session_id)
    else {
        return (
            "[résultat mémoire omis : budget épuisé]".to_string(),
            true,
        );
    };
    let remaining = policy.budget_tokens.saturating_sub(policy.used_tokens);
    if remaining == 0 {
        return (
            "[résultat mémoire omis : budget épuisé]".to_string(),
            true,
        );
    }
    let tokens = estimate(content);
    let (output, truncated) = if tokens <= remaining {
        (content.to_string(), false)
    } else {
        (truncated_result(content, remaining), true)
    };
    let consumed = if truncated {
        remaining
    } else {
        estimate(&output).min(remaining)
    };
    policy.used_tokens = policy.used_tokens.saturating_add(consumed);
    (output, truncated)
}

fn truncated_result(content: &str, max_tokens: usize) -> String {
    const NOTICE: &str = "[résultat mémoire tronqué : budget atteint]";
    let notice_tokens = estimate(NOTICE);
    if notice_tokens >= max_tokens {
        return truncate_to_tokens(NOTICE, max_tokens);
    }
    let body = truncate_to_tokens(
        content,
        max_tokens.saturating_sub(notice_tokens.saturating_add(1)),
    );
    truncate_to_tokens(&format!("{body}\n{NOTICE}"), max_tokens)
}

pub fn truncate_to_tokens(content: &str, max_tokens: usize) -> String {
    if max_tokens == 0 {
        return String::new();
    }
    if estimate(content) <= max_tokens {
        return content.to_string();
    }
    let mut keep = max_tokens.saturating_mul(4).min(content.chars().count());
    loop {
        let candidate = content.chars().take(keep).collect::<String>();
        if estimate(&candidate) <= max_tokens || keep == 0 {
            return candidate;
        }
        keep = keep.saturating_mul(9) / 10;
    }
}

pub fn has_explicit_request(message: &str) -> bool {
    let normalized = message.to_lowercase();
    [
        "souviens-toi",
        "souviens toi",
        "retiens ",
        "mémoris",
        "memorise",
        "dans ta memoire",
        "mémoire",
        "corrige ma préférence",
        "corrige mon souvenir",
        "ne garde pas ça",
        "oublie ",
        "remember ",
        "save this preference",
        "update my preference",
        "do not remember",
        "forget ",
        "recuerda ",
        "corrige mi preferencia",
        "olvida ",
        "erinnere dich",
        "korrigiere meine präferenz",
        "vergiss ",
        "ricorda ",
        "correggi la mia preferenza",
        "dimentica ",
        "记住",
        "更正我的偏好",
        "忘记",
        "覚えて",
        "好みを修正",
        "忘れて",
    ]
    .iter()
    .any(|marker| normalized.contains(marker))
}

fn policy(session_id: &str) -> Option<TurnPolicy> {
    lock_policies()
        .iter()
        .find(|entry| entry.session_id == session_id)
        .cloned()
}

fn estimate(content: &str) -> usize {
    crate::services::token_counting::estimate_text_tokens(content)
}

fn lock_policies() -> std::sync::MutexGuard<'static, VecDeque<TurnPolicy>> {
    POLICIES.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

impl Drop for MemoryTurnGuard {
    fn drop(&mut self) {
        let mut policies = lock_policies();
        policies.retain(|entry| {
            entry.session_id != self.session_id || entry.nonce != self.nonce
        });
    }
}

#[cfg(test)]
#[path = "memory_runtime_tests.rs"]
mod tests;
