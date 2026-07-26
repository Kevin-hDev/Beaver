use sha2::{Digest, Sha256};

use super::types::ForecastRequest;

pub fn for_request(request: &ForecastRequest) -> String {
    let mut digest = Sha256::new();
    add(&mut digest, b"cl-go-forecast-input-v1");
    add(
        &mut digest,
        request.data.as_deref().unwrap_or_default().as_bytes(),
    );
    add(&mut digest, request.target_column.as_bytes());
    add(&mut digest, request.date_column.as_bytes());
    add(
        &mut digest,
        request
            .series_column
            .as_deref()
            .unwrap_or_default()
            .as_bytes(),
    );
    for column in &request.covariate_columns {
        add(&mut digest, column.as_bytes());
    }
    add(&mut digest, request.frequency.as_bytes());
    add(&mut digest, &request.horizon.to_be_bytes());
    format!("{:x}", digest.finalize())
}

fn add(digest: &mut Sha256, value: &[u8]) {
    digest.update((value.len() as u64).to_be_bytes());
    digest.update(value);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn framing_prevents_ambiguous_inputs() {
        let mut first = Sha256::new();
        add(&mut first, b"ab");
        add(&mut first, b"c");
        let mut second = Sha256::new();
        add(&mut second, b"a");
        add(&mut second, b"bc");
        assert_ne!(first.finalize(), second.finalize());
    }
}
