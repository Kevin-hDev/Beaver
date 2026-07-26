use super::ForecastModelSpec;

pub const MOIRAI_2_0_R_SMALL: ForecastModelSpec = ForecastModelSpec {
    id: "moirai-2.0-r-small",
    provider_id: "salesforce",
    family_id: "moirai-2",
    display_name: "MOIRAI 2.0 R Small",
    params: "—",
    size_mb: 46,
    ram_mb: 700,
    vram_mb: Some(280),
    cpu_supported: true,
    gpu_supported: true,
    multivariate: false,
    covariates: false,
    horizon_max: 1024,
    frequencies: "Toutes",
    hf_repo: Some("Salesforce/moirai-2.0-R-small"),
    hf_revision: Some("30f43ff08c8494f4943ae1521e9d4e94a0fbb389"),
    github_repo: Some("SalesforceAIResearch/uni2ts"),
    github_revision: Some("8062ef5a5660d2fea395fd1288ec9c397396c168"),
    is_cloud: false,
};
