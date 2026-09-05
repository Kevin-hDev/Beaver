use serde::Serialize;

pub(crate) fn serialize_bounded_result<T: Serialize>(value: &T) -> Result<String, ()> {
    let serialized = serde_json::to_vec(value).map_err(|_| ())?;
    if serialized.len() > super::discovery_contract::MAX_SERIALIZED_RESULT_BYTES {
        return Err(());
    }
    String::from_utf8(serialized).map_err(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_a_result_at_the_exact_serialized_limit() {
        let value = "x".repeat(super::super::discovery_contract::MAX_SERIALIZED_RESULT_BYTES - 2);

        let serialized = serialize_bounded_result(&value).expect("exactly bounded JSON is valid");

        assert_eq!(
            serialized.len(),
            super::super::discovery_contract::MAX_SERIALIZED_RESULT_BYTES
        );
        assert_eq!(serde_json::from_str::<String>(&serialized).unwrap(), value);
    }

    #[test]
    fn rejects_a_result_past_the_serialized_limit_without_truncating_json() {
        let value = "x".repeat(super::super::discovery_contract::MAX_SERIALIZED_RESULT_BYTES - 1);

        assert!(serialize_bounded_result(&value).is_err());
    }
}
