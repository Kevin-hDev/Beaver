use crate::services::forecast::intervals::{lower_level, quantile_key, upper_level};
use serde::Serialize;

#[derive(Serialize)]
pub(super) struct QuantileLabels {
    pub lower: String,
    pub median: String,
    pub upper: String,
}

impl QuantileLabels {
    pub(super) fn for_confidence(confidence: f64) -> Self {
        Self {
            lower: quantile_key(lower_level(confidence)),
            median: quantile_key(0.5),
            upper: quantile_key(upper_level(confidence)),
        }
    }

    pub(super) fn table_headers(&self) -> [&str; 3] {
        [&self.lower, &self.median, &self.upper]
    }

    pub(super) fn uppercase_headers(&self) -> [String; 3] {
        [
            self.lower.to_ascii_uppercase(),
            self.median.to_ascii_uppercase(),
            self.upper.to_ascii_uppercase(),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels_follow_the_requested_confidence() {
        let labels = QuantileLabels::for_confidence(0.9);

        assert_eq!(labels.table_headers(), ["q05", "q50", "q95"]);
    }

    #[test]
    fn labels_keep_basis_point_precision() {
        let labels = QuantileLabels::for_confidence(0.99);

        assert_eq!(labels.table_headers(), ["q0050", "q50", "q9950"]);
    }
}
