use super::ForecastModelSpec;

pub const TIMESFM_2_5: ForecastModelSpec = ForecastModelSpec {
    id: "timesfm-2.5-200m",
    provider_id: "google",
    family_id: "timesfm-2-5",
    display_name: "TimesFM 2.5 200M",
    params: "200M",
    size_mb: 925,
    ram_mb: 4200,
    vram_mb: Some(1800),
    cpu_supported: true,
    gpu_supported: true,
    multivariate: false,
    covariates: true,
    horizon_max: 1000,
    frequencies: "Toutes",
    hf_repo: Some("google/timesfm-2.5-200m-pytorch"),
    hf_revision: Some("1d952420fba87f3c6dee4f240de0f1a0fbc790e3"),
    github_repo: Some("google-research/timesfm"),
    github_revision: Some("3dae50b20d7a724981e8ea36cda75578f80dd2dc"),
    is_cloud: false,
};
