use super::ephemeral_vault_state;

#[test]
fn e2e_vault_state_is_empty_and_uses_a_volatile_master_key() {
    let state = ephemeral_vault_state();

    assert!(state.keys.is_empty());
    assert_eq!(state.master_key.len(), 32);
}
