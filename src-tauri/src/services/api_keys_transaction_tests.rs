use super::*;

fn state_with_old_secret() -> VaultState {
    VaultState {
        master_key: Zeroizing::new(vec![7_u8; 32]),
        keys: HashMap::from([(
            "openai".to_string(),
            Zeroizing::new("old-secret".to_string()),
        )]),
    }
}

#[test]
fn failed_persistence_keeps_previous_memory_state() {
    let mut state = state_with_old_secret();
    let result = commit_candidate_with(
        &mut state,
        |candidate| {
            candidate.insert("openai".to_string(), "new-secret".to_string());
            Ok(())
        },
        |_, _| Err("écriture refusée".to_string()),
    );

    assert!(result.is_err());
    assert_eq!(
        state.keys.get("openai").map(|value| value.as_str()),
        Some("old-secret")
    );
}

#[test]
fn successful_persistence_replaces_memory_after_write() {
    let mut state = state_with_old_secret();
    let result = commit_candidate_with(
        &mut state,
        |candidate| {
            candidate.insert("openai".to_string(), "new-secret".to_string());
            Ok(())
        },
        |_, candidate| {
            assert_eq!(
                candidate.get("openai").map(String::as_str),
                Some("new-secret")
            );
            Ok(())
        },
    );

    assert!(result.is_ok());
    assert_eq!(
        state.keys.get("openai").map(|value| value.as_str()),
        Some("new-secret")
    );
}
