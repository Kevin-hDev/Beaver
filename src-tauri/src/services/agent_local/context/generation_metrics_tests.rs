use super::*;

fn tracked_result(
    first_count: u32,
    final_count: u32,
    elapsed: Duration,
    exact_count: Option<u32>,
    native_duration_ns: Option<u64>,
) -> StreamResult {
    let started = Instant::now();
    let mut result = StreamResult::default();
    result.generation.record_activity_at(first_count, started);
    result
        .generation
        .record_activity_at(final_count, started + elapsed);
    result.generated_units = final_count as usize * 4;
    result.eval_count = exact_count;
    if let Some(duration_ns) = native_duration_ns {
        result.generation.record_native_duration(duration_ns);
    }
    result
}

#[test]
fn live_rate_ignores_the_first_batched_chunk() {
    let result = tracked_result(10, 30, Duration::from_secs(2), None, None);

    assert_eq!(result.generation.live_tps(30), 10.0);
}

#[test]
fn native_duration_and_exact_count_are_not_estimated() {
    let result = tracked_result(4, 20, Duration::from_secs(3), Some(20), Some(2_000_000_000));
    let mut aggregate = GenerationAggregate::default();
    aggregate.add_result(&result);

    assert_eq!(aggregate.summary(), GenerationSummary {
        duration_ns: 2_000_000_000,
        tps: 10.0,
        estimated: false,
    });
}

#[test]
fn observed_duration_is_marked_as_estimated() {
    let result = tracked_result(5, 25, Duration::from_secs(2), Some(25), None);
    let mut aggregate = GenerationAggregate::default();
    aggregate.add_result(&result);

    assert_eq!(aggregate.summary(), GenerationSummary {
        duration_ns: 2_000_000_000,
        tps: 10.0,
        estimated: true,
    });
}

#[test]
fn aggregate_is_weighted_by_generation_duration() {
    let fast = tracked_result(1, 20, Duration::from_secs(1), Some(20), Some(1_000_000_000));
    let slow = tracked_result(1, 20, Duration::from_secs(3), Some(20), Some(3_000_000_000));
    let mut aggregate = GenerationAggregate::default();
    aggregate.add_result(&fast);
    aggregate.add_result(&slow);

    assert_eq!(aggregate.summary().tps, 10.0);
}

#[test]
fn missing_provider_count_uses_the_bounded_estimate() {
    let result = tracked_result(4, 24, Duration::from_secs(2), None, Some(2_000_000_000));
    let mut aggregate = GenerationAggregate::default();
    aggregate.add_result(&result);

    assert_eq!(aggregate.summary().tps, 12.0);
    assert!(aggregate.summary().estimated);
}

#[test]
fn rejects_invalid_provider_durations() {
    assert_eq!(seconds_to_duration_ns(f64::NAN), None);
    assert_eq!(seconds_to_duration_ns(-1.0), None);
    assert_eq!(seconds_to_duration_ns(8.0 * 24.0 * 60.0 * 60.0), None);
}

#[test]
fn unavailable_request_marks_an_existing_aggregate_as_estimated() {
    let exact = tracked_result(1, 20, Duration::from_secs(1), Some(20), Some(1_000_000_000));
    let mut unavailable = StreamResult {
        eval_count: Some(10),
        ..Default::default()
    };
    unavailable
        .generation
        .record_activity_at(0, Instant::now());
    let mut aggregate = GenerationAggregate::default();
    aggregate.add_result(&exact);
    aggregate.add_result(&unavailable);

    assert_eq!(aggregate.summary().tps, 20.0);
    assert!(aggregate.summary().estimated);
}
