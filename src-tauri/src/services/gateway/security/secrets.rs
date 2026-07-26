use subtle::ConstantTimeEq;
use zeroize::Zeroize;

pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    let max_len = a.len().max(b.len());
    let mut padded_a = vec![0u8; max_len];
    let mut padded_b = vec![0u8; max_len];

    padded_a[..a.len()].copy_from_slice(a);
    padded_b[..b.len()].copy_from_slice(b);

    let ct_match = padded_a.ct_eq(&padded_b).into();
    let len_match = a.len() == b.len();

    padded_a.zeroize();
    padded_b.zeroize();

    ct_match && len_match
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_secrets_match() {
        assert!(constant_time_eq(b"secret123", b"secret123"));
    }

    #[test]
    fn different_secrets_no_match() {
        assert!(!constant_time_eq(b"secret123", b"secret456"));
    }

    #[test]
    fn different_lengths_no_match() {
        assert!(!constant_time_eq(b"short", b"much_longer_secret"));
    }

    #[test]
    fn empty_secrets_match() {
        assert!(constant_time_eq(b"", b""));
    }

    #[test]
    fn one_empty_no_match() {
        assert!(!constant_time_eq(b"", b"notempty"));
    }
}
