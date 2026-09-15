use super::{
    benchmarks::{qualifies_for_speed, Benchmark, BenchmarkStore},
    recognizer::ExecutionProfile,
};

fn benchmark(model_id: &str, revision: &str, audio_ms: u64) -> Benchmark {
    Benchmark {
        model_id: model_id.into(),
        revision: revision.into(),
        audio_ms,
        compute_ms: 15_000,
        profile: ExecutionProfile::cpu(2).unwrap(),
    }
}

#[test]
fn a_short_trial_does_not_publish_a_speed_multiplier() {
    assert!(!qualifies_for_speed(29_999));
    assert!(qualifies_for_speed(30_000));
}

#[test]
fn current_revision_and_profile_replace_the_previous_measurement() {
    let data = tempfile::tempdir().unwrap();
    let store = BenchmarkStore;

    assert!(!store
        .record(data.path(), benchmark("parakeet", &"a".repeat(40), 29_999))
        .unwrap());
    assert!(store
        .record(data.path(), benchmark("parakeet", &"a".repeat(40), 30_000))
        .unwrap());
    assert!(store
        .record(data.path(), benchmark("parakeet", &"b".repeat(40), 60_000))
        .unwrap());

    let entries = store.load(data.path()).unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].revision, "b".repeat(40));
    assert_eq!(entries[0].multiplier(), 0.25);
    assert!(store
        .current(
            data.path(),
            "parakeet",
            &"b".repeat(40),
            &ExecutionProfile::cpu(2).unwrap()
        )
        .unwrap()
        .is_some());
    assert!(store
        .current(
            data.path(),
            "parakeet",
            &"a".repeat(40),
            &ExecutionProfile::cpu(2).unwrap()
        )
        .unwrap()
        .is_none());
    store.remove(data.path(), "parakeet").unwrap();
    assert!(store.load(data.path()).unwrap().is_empty());
    assert!(!data
        .path()
        .join("voice-models/voice-benchmarks.json")
        .exists());
}
