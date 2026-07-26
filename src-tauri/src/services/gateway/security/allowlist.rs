use std::collections::{HashSet, VecDeque};

const MAX_ENTRIES: usize = 1000;
const MAX_ENTRY_LEN: usize = 128;

pub struct Allowlist {
    set: HashSet<String>,
    order: VecDeque<String>,
    allow_wildcard: bool,
}

impl Allowlist {
    pub fn new(allow_wildcard: bool) -> Self {
        Self {
            set: HashSet::new(),
            order: VecDeque::new(),
            allow_wildcard,
        }
    }

    pub fn from_list(entries: &[String], allow_wildcard: bool) -> Self {
        let mut al = Self::new(allow_wildcard);
        for entry in entries {
            al.add(entry);
        }
        al
    }

    pub fn add(&mut self, entry: &str) -> bool {
        let normalized = Self::normalize(entry);
        if normalized.is_empty() || normalized.len() > MAX_ENTRY_LEN {
            return false;
        }
        if normalized == "*" && !self.allow_wildcard {
            return false;
        }
        if self.set.contains(&normalized) {
            return false;
        }
        while self.set.len() >= MAX_ENTRIES {
            if let Some(evicted) = self.order.pop_front() {
                self.set.remove(&evicted);
            }
        }
        self.set.insert(normalized.clone());
        self.order.push_back(normalized);
        true
    }

    #[cfg(test)]
    pub fn remove(&mut self, entry: &str) -> bool {
        let normalized = Self::normalize(entry);
        if self.set.remove(&normalized) {
            self.order.retain(|e| e != &normalized);
            true
        } else {
            false
        }
    }

    pub fn contains(&self, entry: &str) -> bool {
        if self.set.contains("*") {
            return true;
        }
        self.set.contains(&Self::normalize(entry))
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.set.len()
    }

    #[cfg(test)]
    pub fn is_empty(&self) -> bool {
        self.set.is_empty()
    }

    fn normalize(entry: &str) -> String {
        entry.trim().to_lowercase()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_and_contains() {
        let mut al = Allowlist::new(false);
        assert!(al.add("User123"));
        assert!(al.contains("user123"));
        assert!(al.contains("USER123"));
    }

    #[test]
    fn reject_empty_and_too_long() {
        let mut al = Allowlist::new(false);
        assert!(!al.add(""));
        assert!(!al.add("  "));
        let long = "a".repeat(MAX_ENTRY_LEN + 1);
        assert!(!al.add(&long));
    }

    #[test]
    fn reject_wildcard_by_default() {
        let mut al = Allowlist::new(false);
        assert!(!al.add("*"));
        assert!(!al.contains("anything"));
    }

    #[test]
    fn allow_wildcard_when_enabled() {
        let mut al = Allowlist::new(true);
        assert!(al.add("*"));
        assert!(al.contains("anything"));
    }

    #[test]
    fn eviction_on_overflow() {
        let mut al = Allowlist::new(false);
        for i in 0..MAX_ENTRIES {
            al.add(&format!("user{i}"));
        }
        assert_eq!(al.len(), MAX_ENTRIES);
        al.add("overflow_user");
        assert_eq!(al.len(), MAX_ENTRIES);
        assert!(!al.contains("user0"));
        assert!(al.contains("overflow_user"));
    }

    #[test]
    fn no_duplicates() {
        let mut al = Allowlist::new(false);
        assert!(al.add("bob"));
        assert!(!al.add("Bob"));
        assert_eq!(al.len(), 1);
    }

    #[test]
    fn remove_entry() {
        let mut al = Allowlist::new(false);
        al.add("alice");
        assert!(al.remove("Alice"));
        assert!(!al.contains("alice"));
        assert!(al.is_empty());
    }

    #[test]
    fn from_list_initializes() {
        let entries = vec!["a".into(), "B".into(), "c".into()];
        let al = Allowlist::from_list(&entries, false);
        assert_eq!(al.len(), 3);
        assert!(al.contains("A"));
        assert!(al.contains("b"));
    }
}
