use serde::Serialize;
use serde_json::Value;

use super::params::*;
use crate::services::forecast::registry::ForecastRuntimeSpec;

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ParamKind {
    Integer,
    Number,
    Boolean,
    Select,
    NumberList,
}

#[derive(Clone)]
pub struct ParamSpec {
    pub id: &'static str,
    pub kind: ParamKind,
    pub default_value: Value,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub options: &'static [&'static str],
}

pub fn specs_for_family(family_id: &str) -> Vec<ParamSpec> {
    match family_id {
        "chronos-bolt" => vec![
            horizon_override(),
            context(),
            quantiles(),
            bool_param("non_negative_output", false),
        ],
        "chronos-2" => vec![
            horizon_override(),
            context(),
            quantiles(),
            bool_param("non_negative_output", false),
        ],
        "timesfm-2-5" => vec![
            horizon_override(),
            context(),
            quantiles(),
            bool_param("non_negative_output", false),
        ],
        "timegpt-2" => vec![
            horizon_override(),
            bool_param("clean_ex_first", true),
            int_param("finetune_steps", 0, 0.0, 1000.0),
            select(
                "finetune_loss",
                "default",
                &["default", "mae", "mse", "rmse"],
            ),
            int_param("finetune_depth", 1, 1.0, 5.0),
            bool_param("feature_contributions", false),
        ],
        "toto-2" => vec![
            horizon_override(),
            context(),
            int_param("decode_block_size", 768, 1.0, 4096.0),
            bool_param("has_missing_values", false),
            quantiles(),
            bool_param("non_negative_output", false),
        ],
        "moirai-2" => vec![
            horizon_override(),
            context(),
            batch_size(),
            quantiles(),
            bool_param("non_negative_output", false),
        ],
        "flowstate" => vec![
            horizon_override(),
            context(),
            number_param("scale_factor", 1.0, 0.0001, 1000.0),
            bool_param("batch_first", true),
            quantiles(),
            bool_param("non_negative_output", false),
        ],
        "tabpfn-ts" => vec![
            horizon_override(),
            bool_param("probabilistic_output", true),
            context(),
            quantiles(),
            bool_param("non_negative_output", false),
        ],
        "tirex" => vec![
            horizon_override(),
            context(),
            select("output_type", "quantiles", &["quantiles", "mean"]),
            quantiles(),
            bool_param("non_negative_output", false),
        ],
        "kairos" => vec![
            horizon_override(),
            context(),
            bool_param("preserve_positivity", true),
            bool_param("average_with_flipped_input", true),
            bool_param("generation", true),
            quantiles(),
            bool_param("non_negative_output", false),
        ],
        "sundial" => vec![
            horizon_override(),
            int_param("num_samples", 64, 1.0, 512.0),
            context(),
            dtype(),
            quantiles(),
            bool_param("non_negative_output", false),
        ],
        _ => Vec::new(),
    }
}

pub fn specs_for_runtime(runtime: &ForecastRuntimeSpec) -> Vec<ParamSpec> {
    specs_for_family(runtime.family_id)
        .into_iter()
        .filter(|spec| runtime.config_params.contains(&spec.id))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::specs_for_family;
    use serde_json::{json, Map};

    #[test]
    fn timegpt_finetune_depth_matches_the_api_contract() {
        let specs = specs_for_family("timegpt-2");
        let depth = specs
            .iter()
            .find(|spec| spec.id == "finetune_depth")
            .unwrap();

        assert_eq!(depth.default_value, 1);
        assert_eq!(depth.min, Some(1.0));
        assert_eq!(depth.max, Some(5.0));

        for invalid in [0, 6] {
            let mut values = Map::new();
            values.insert("finetune_depth".into(), json!(invalid));
            assert!(super::super::validate::sanitize(&specs, values).is_err());
        }
    }
}
