use super::{
    checkpoint::{load_checkpoint, save_checkpoint},
    inspect_partial, Checkpoint, PartialDecision,
};

fn checkpoint_fixture(bytes: u64) -> Checkpoint {
    let mut checkpoint = Checkpoint::new(
        "parakeet-tdt-v3-int8".into(),
        "parakeet-tdt-v3-int8".into(),
        "2bda32ec70b097a55adaa07d9a7173915b43cc78".into(),
        487_170_055,
        "5793d0fd397c5778d2cf2126994d58e9d56b1be7c04d13c7a15bb1b4eafb16bf".into(),
    )
    .expect("valid checkpoint");
    checkpoint.durable_bytes = bytes;
    checkpoint
}

#[test]
fn durable_position_wins_over_a_crash_tail() {
    let checkpoint = checkpoint_fixture(4_194_304);
    assert_eq!(
        inspect_partial(&checkpoint, Some(4_300_000)),
        PartialDecision::Truncate {
            durable_bytes: 4_194_304
        }
    );
}

#[test]
fn missing_or_short_partial_is_discarded() {
    let checkpoint = checkpoint_fixture(4_194_304);
    assert_eq!(inspect_partial(&checkpoint, None), PartialDecision::Discard);
    assert_eq!(
        inspect_partial(&checkpoint, Some(4_000_000)),
        PartialDecision::Discard
    );
}

#[test]
fn exact_durable_length_is_resumed() {
    let checkpoint = checkpoint_fixture(4_194_304);
    assert_eq!(
        inspect_partial(&checkpoint, Some(4_194_304)),
        PartialDecision::Resume {
            durable_bytes: 4_194_304
        }
    );
}

#[test]
fn invalid_checkpoint_is_never_reused() {
    let mut checkpoint = checkpoint_fixture(0);
    checkpoint.revision = "moving-target".into();
    assert_eq!(
        inspect_partial(&checkpoint, Some(0)),
        PartialDecision::Discard
    );
}

#[test]
fn revision_requires_an_exact_immutable_identifier() {
    let mut checkpoint = checkpoint_fixture(0);
    checkpoint.revision = "a".repeat(71);
    assert!(!checkpoint.is_valid());

    checkpoint.revision = format!("sha256:{}", "a".repeat(64));
    assert!(checkpoint.is_valid());
}

#[test]
fn checkpoint_round_trip_is_durable_and_bounded() {
    let directory = tempfile::tempdir().expect("directory");
    let path = directory.path().join("checkpoint.json");
    let checkpoint = checkpoint_fixture(4_194_304);

    save_checkpoint(&path, &checkpoint).expect("durable checkpoint");
    assert_eq!(load_checkpoint(&path).unwrap(), Some(checkpoint));
}

#[test]
fn queued_voice_request_has_a_durable_zero_before_any_partial_file() {
    let data = tempfile::tempdir().unwrap();
    let resource_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let entry = super::load_catalog(resource_dir).unwrap().entries.remove(0);

    super::prepare_download(&entry, &entry.id, data.path()).unwrap();

    let checkpoint = super::discover_checkpoints(data.path()).remove(0);
    assert_eq!(checkpoint.durable_bytes, 0);
    assert_eq!(checkpoint.requested_model_id, entry.id);
    assert!(!super::partial_path(data.path(), &entry.id).exists());
}
