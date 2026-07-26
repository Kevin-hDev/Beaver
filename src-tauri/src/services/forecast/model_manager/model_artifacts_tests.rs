use super::{manifest, model, sha256_matches, total_size};
use crate::services::forecast::catalog;

#[test]
fn approved_manifest_covers_every_local_catalog_model() {
    let manifest = manifest().unwrap();
    let local_count = catalog::FORECAST_MODELS
        .iter()
        .filter(|entry| !entry.is_cloud)
        .count();
    assert_eq!(manifest.models.len(), local_count);
    for catalog_model in catalog::FORECAST_MODELS
        .iter()
        .filter(|entry| !entry.is_cloud)
    {
        let approved = model(catalog_model.id).unwrap();
        assert_eq!(approved.repository, catalog_model.hf_repo.unwrap());
        assert_eq!(approved.revision, catalog_model.hf_revision.unwrap());
        let bytes = total_size(catalog_model.id).unwrap();
        let rounded_mb = (bytes + 500_000) / 1_000_000;
        assert_eq!(
            u64::from(catalog_model.size_mb),
            rounded_mb,
            "{}",
            catalog_model.id
        );
    }
}

#[test]
fn legacy_tabpfn_id_uses_the_same_approved_artifact() {
    assert_eq!(model("tabpfn-ts").unwrap(), model("tabpfn-ts-3").unwrap());
}

#[test]
fn manifest_contains_only_the_needed_tabpfn_checkpoint() {
    let tabpfn = model("tabpfn-ts-3").unwrap();
    assert_eq!(tabpfn.artifacts.len(), 1);
    assert!(tabpfn.artifacts[0].path.ends_with("_timeseries.ckpt"));
}

#[test]
fn artifact_hashes_are_compared_from_exact_bytes() {
    let bytes = [0xabu8; 32];
    assert!(sha256_matches(&bytes, &"ab".repeat(32)));
    assert!(!sha256_matches(&bytes, &"ac".repeat(32)));
    assert!(!sha256_matches(&bytes, "not-a-hash"));
}
