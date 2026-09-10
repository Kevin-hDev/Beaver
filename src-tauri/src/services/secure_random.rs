use rand::TryRng;

/// Fill with the operating system CSPRNG. Failure stops generation instead of
/// returning predictable bytes.
pub(crate) fn fill(bytes: &mut [u8]) {
    try_fill(bytes).expect("system CSPRNG unavailable");
}

pub(crate) fn try_fill(bytes: &mut [u8]) -> Result<(), ()> {
    rand::rngs::SysRng.try_fill_bytes(bytes).map_err(|_| ())
}

#[cfg(test)]
mod tests {
    #[test]
    fn fills_distinct_buffers() {
        let mut first = [0_u8; 32];
        let mut second = [0_u8; 32];

        super::fill(&mut first);
        super::fill(&mut second);

        assert_ne!(first, [0_u8; 32]);
        assert_ne!(first, second);
    }
}
