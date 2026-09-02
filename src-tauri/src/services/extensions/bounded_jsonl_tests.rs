use serde::Serialize;

#[derive(Serialize)]
struct Entry<'a> {
    sequence: usize,
    value: &'a str,
}

#[test]
fn appending_rotates_only_complete_oldest_lines_within_sixty_four_kib() {
    let temporary = tempfile::tempdir().unwrap();
    let path = temporary.path().join("bounded.jsonl");
    let initial = (0..4)
        .map(|sequence| {
            format!(
                "{{\"sequence\":{sequence},\"value\":\"{}\"}}\n",
                "x".repeat(16_352)
            )
        })
        .collect::<String>();
    std::fs::write(&path, initial).unwrap();

    super::bounded_jsonl::write(
        &path,
        &Entry {
            sequence: 9,
            value: "new",
        },
    )
    .unwrap();

    let bytes = std::fs::read(&path).unwrap();
    assert!(bytes.len() <= 64 * 1024);
    let lines = String::from_utf8(bytes).unwrap();
    assert!(!lines.contains("{\"sequence\":0"));
    assert!(lines.starts_with("{\"sequence\":1"));
    assert!(lines.ends_with("{\"sequence\":9,\"value\":\"new\"}\n"));
    assert!(lines
        .lines()
        .all(|line| serde_json::from_str::<serde_json::Value>(line).is_ok()));
}

#[test]
fn oversized_single_record_is_refused_without_changing_the_log() {
    let temporary = tempfile::tempdir().unwrap();
    let path = temporary.path().join("bounded.jsonl");
    std::fs::write(&path, b"{\"safe\":true}\n").unwrap();

    let result = super::bounded_jsonl::write(
        &path,
        &Entry {
            sequence: 1,
            value: &"s".repeat(70_000),
        },
    );

    assert!(result.is_err());
    assert_eq!(std::fs::read(&path).unwrap(), b"{\"safe\":true}\n");
}

#[test]
fn concurrent_writers_do_not_overwrite_each_others_records() {
    let temporary = tempfile::tempdir().unwrap();
    let path = temporary.path().join("concurrent.jsonl");
    let threads = (0..16)
        .map(|sequence| {
            let path = path.clone();
            std::thread::spawn(move || {
                super::bounded_jsonl::write(
                    &path,
                    &Entry {
                        sequence,
                        value: "safe",
                    },
                )
                .unwrap();
            })
        })
        .collect::<Vec<_>>();
    for thread in threads {
        thread.join().unwrap();
    }

    let text = std::fs::read_to_string(path).unwrap();
    assert_eq!(text.lines().count(), 16);
    for sequence in 0..16 {
        assert!(text.contains(&format!("\"sequence\":{sequence},")));
    }
}
