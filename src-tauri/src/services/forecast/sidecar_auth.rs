use rand::RngCore;
use zeroize::Zeroizing;

pub fn generate_auth_token() -> Zeroizing<String> {
    let mut bytes = [0u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    let token = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
    bytes.fill(0);
    Zeroizing::new(token)
}
