use super::public_error::not_authorized;

// Les labels Tauri sont courts ; cette borne limite aussi toute fixture propriétaire.
const MAX_TERMINAL_OWNER_BYTES: usize = 128;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TerminalOwner(String);

impl TerminalOwner {
    #[cfg(test)]
    pub(crate) fn for_test(label: &str) -> Result<Self, String> {
        bounded_owner(label)
    }
}

pub(crate) fn authorize(label: &str) -> Result<TerminalOwner, String> {
    if label != "main" {
        return Err(not_authorized());
    }
    bounded_owner(label)
}

fn bounded_owner(label: &str) -> Result<TerminalOwner, String> {
    (label.len() <= MAX_TERMINAL_OWNER_BYTES)
        .then(|| TerminalOwner(label.to_string()))
        .ok_or_else(not_authorized)
}
