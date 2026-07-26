use serde::Serialize;

const FIXED_CENTRAL_LEVELS: &[f64] = &[0.6, 0.8];

#[derive(Debug, Clone, Copy, Serialize)]
pub struct IntervalCapability {
    pub mode: &'static str,
    pub supported_confidence_levels: &'static [f64],
    pub confidence_step: Option<f64>,
}

pub fn for_model(model_id: &str) -> IntervalCapability {
    let fixed = matches!(
        super::registry::find_runtime(model_id).map(|runtime| runtime.family_id),
        Some("timesfm-2-5" | "toto-2" | "flowstate" | "tabpfn-ts" | "tirex" | "kairos")
    );
    if fixed {
        IntervalCapability {
            mode: "fixed_grid",
            supported_confidence_levels: FIXED_CENTRAL_LEVELS,
            confidence_step: None,
        }
    } else {
        IntervalCapability {
            mode: "continuous",
            supported_confidence_levels: &[],
            confidence_step: Some(0.01),
        }
    }
}

pub fn supports(model_id: &str, confidence: f64) -> bool {
    let capability = for_model(model_id);
    if capability.mode == "continuous" {
        return valid_input_level(confidence);
    }
    capability
        .supported_confidence_levels
        .iter()
        .any(|supported| (confidence - supported).abs() < 0.000_001)
}

pub fn valid_input_level(confidence: f64) -> bool {
    (0.5..=0.99).contains(&confidence)
        && (confidence * 100.0 - (confidence * 100.0).round()).abs() < 0.000_001
}

pub fn legacy_label(model_id: &str) -> &'static str {
    if for_model(model_id).mode == "continuous" {
        "continuous"
    } else {
        "central_60_or_80"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_models_expose_only_honest_central_intervals() {
        let capability = for_model("timesfm-2.5-200m");
        assert_eq!(capability.mode, "fixed_grid");
        assert_eq!(capability.supported_confidence_levels, &[0.6, 0.8]);
        assert!(supports("timesfm-2.5-200m", 0.8));
        assert!(!supports("timesfm-2.5-200m", 0.92));
    }

    #[test]
    fn continuous_models_use_whole_percentage_steps() {
        assert!(supports("chronos-2", 0.92));
        assert!(!supports("chronos-2", 0.925));
    }
}
