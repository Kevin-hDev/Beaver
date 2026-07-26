pub const CHRONOS_BOLT_PARAMS: &[&str] = &[
    "horizon_max_override",
    "context_length",
    "quantiles",
    "non_negative_output",
];
pub const CHRONOS_2_PARAMS: &[&str] = CHRONOS_BOLT_PARAMS;
pub const TIMESFM_PARAMS: &[&str] = CHRONOS_BOLT_PARAMS;

pub const TIMEGPT_PARAMS: &[&str] = &[
    "horizon_max_override",
    "clean_ex_first",
    "finetune_steps",
    "finetune_loss",
    "finetune_depth",
    "feature_contributions",
];

pub const TOTO_PARAMS: &[&str] = &[
    "horizon_max_override",
    "context_length",
    "decode_block_size",
    "has_missing_values",
    "quantiles",
    "non_negative_output",
];

pub const MOIRAI_PARAMS: &[&str] = &[
    "horizon_max_override",
    "context_length",
    "batch_size",
    "quantiles",
    "non_negative_output",
];

pub const FLOWSTATE_PARAMS: &[&str] = &[
    "horizon_max_override",
    "context_length",
    "scale_factor",
    "batch_first",
    "quantiles",
    "non_negative_output",
];

pub const TABPFN_PARAMS: &[&str] = &[
    "horizon_max_override",
    "context_length",
    "probabilistic_output",
    "quantiles",
    "non_negative_output",
];

pub const TIREX_PARAMS: &[&str] = &[
    "horizon_max_override",
    "context_length",
    "output_type",
    "quantiles",
    "non_negative_output",
];

pub const KAIROS_PARAMS: &[&str] = &[
    "horizon_max_override",
    "context_length",
    "preserve_positivity",
    "average_with_flipped_input",
    "generation",
    "quantiles",
    "non_negative_output",
];

pub const SUNDIAL_PARAMS: &[&str] = &[
    "horizon_max_override",
    "context_length",
    "num_samples",
    "dtype",
    "quantiles",
    "non_negative_output",
];
