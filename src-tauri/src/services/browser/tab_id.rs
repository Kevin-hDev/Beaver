use rand::RngCore;

const TAB_ID_BYTES: usize = 16;
const TAB_ID_LENGTH: usize = TAB_ID_BYTES * 2;

pub(super) fn new_secure_tab_id() -> String {
    let mut bytes = [0_u8; TAB_ID_BYTES];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    hex::encode(bytes)
}

pub(super) fn validate_tab_id(id: &str) -> Result<(), ()> {
    if id.len() != TAB_ID_LENGTH
        || !id
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(());
    }
    Ok(())
}
