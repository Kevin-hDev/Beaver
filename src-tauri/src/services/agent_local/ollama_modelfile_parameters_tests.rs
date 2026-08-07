use super::ollama_modelfile_parameters::rewrite;
use super::ollama_parameter_summary::parse;

#[test]
fn empty_payload_removes_every_parameter_without_touching_prompt_text() {
    let input = concat!(
        "FROM C:/models/blobs/sha256-base\n",
        "SYSTEM \"\"\"Keep this prompt.\n",
        "PARAMETER num_ctx 777 must remain prompt text.\n",
        "\"\"\"\n",
        "PARAMETER num_ctx 100000\n",
        "PARAMETER temperature 0.8\n",
        "MESSAGE user hello\n",
    );
    let current = vec![
        ("temperature".into(), "0.8".into()),
        ("num_ctx".into(), "100000".into()),
    ];

    let output = rewrite(input, &current, &[]).expect("matching normalized block");

    assert_eq!(
        output,
        concat!(
            "FROM C:/models/blobs/sha256-base\n",
            "SYSTEM \"\"\"Keep this prompt.\n",
            "PARAMETER num_ctx 777 must remain prompt text.\n",
            "\"\"\"\n",
            "MESSAGE user hello\n",
        )
    );
}

#[test]
fn raw_quote_does_not_hide_the_following_parameter() {
    let input = concat!(
        "FROM x\n",
        "PARAMETER stop \"\n",
        "PARAMETER temperature 0.9\n",
    );
    let current = vec![
        ("stop".into(), "\"".into()),
        ("temperature".into(), "0.9".into()),
    ];
    let new = vec![
        ("stop".into(), "\"".into()),
        ("temperature".into(), "0.4".into()),
    ];

    let output = rewrite(input, &current, &new).expect("literal quote is semantic data");

    assert_eq!(
        output,
        concat!(
            "FROM x\n",
            "PARAMETER stop \"\"\"\"\"\"\"\n",
            "PARAMETER temperature 0.4\n",
        )
    );
    assert!(!output.contains("temperature 0.9"));
}

#[test]
fn source_rendering_preserves_values_wrapped_in_literal_quotes() {
    let input = concat!(
        "FROM x\n",
        "PARAMETER stop \"User:\"\n",
        "PARAMETER stop \"\"\n",
    );
    let current = vec![
        ("stop".into(), "\"User:\"".into()),
        ("stop".into(), "\"\"".into()),
    ];

    let output = rewrite(input, &current, &current).expect("literal outer quotes");

    assert_eq!(
        output,
        concat!(
            "FROM x\n",
            "PARAMETER stop \"\"\"\"User:\"\"\"\"\n",
            "PARAMETER stop \"\"\"\"\"\"\"\"\n",
        )
    );
}

#[test]
fn removes_multiline_overrides_and_preserves_following_directives() {
    let input = concat!(
        "FROM x\n",
        "PARAMETER stop \"\n",
        "end\n",
        "\"\n",
        "PARAMETER temperature 0.7\n",
        "LICENSE Apache-2.0\n",
    );
    let current = vec![
        ("temperature".into(), "0.7".into()),
        ("stop".into(), "\nend\n".into()),
    ];

    let output = rewrite(input, &current, &[]).expect("complete multiline block");

    assert_eq!(output, "FROM x\nLICENSE Apache-2.0\n");
}

#[test]
fn renders_edge_whitespace_whitespace_only_quotes_and_multiline_values() {
    let input = "FROM x\r\nPARAMETER temperature 0.7\r\n";
    let current = vec![("temperature".into(), "0.7".into())];
    let new = vec![
        ("stop".into(), "Assistant: ".into()),
        ("stop".into(), " ".into()),
        ("stop".into(), "\"User:".into()),
        ("stop".into(), "line one\nline two".into()),
        ("future_option".into(), "say \"hi\"".into()),
    ];

    let output = rewrite(input, &current, &new).expect("representable text values");

    assert_eq!(
        output,
        concat!(
            "FROM x\r\n",
            "PARAMETER stop \"Assistant: \"\r\n",
            "PARAMETER stop \" \"\r\n",
            "PARAMETER stop \"\"\"\"User:\"\"\"\r\n",
            "PARAMETER stop \"line one\r\n",
            "line two\"\r\n",
            "PARAMETER future_option say \"hi\"\r\n",
        )
    );
}

#[test]
fn renders_multiline_values_with_quotes_using_triple_quotes() {
    let input = "FROM x\n";
    let new = vec![("stop".into(), "line \"one\"\nline two".into())];

    let output = rewrite(input, &[], &new).expect("quoted multiline value");

    assert_eq!(
        output,
        concat!(
            "FROM x\n",
            "PARAMETER stop \"\"\"line \"one\"\n",
            "line two\"\"\"\n",
        )
    );
}

#[test]
fn matches_duplicate_entries_when_summary_order_differs_from_modelfile_order() {
    let input = concat!(
        "FROM x\n",
        "PARAMETER temperature 0.8\n",
        "PARAMETER stop User:\n",
        "PARAMETER stop User:\n",
        "RENDERER gemma4\n",
    );
    let current = vec![
        ("stop".into(), "User:".into()),
        ("stop".into(), "User:".into()),
        ("temperature".into(), "0.8".into()),
    ];

    let output = rewrite(input, &current, &[("temperature".into(), "0.4".into())])
        .expect("unordered summary entries");

    assert_eq!(
        output,
        concat!(
            "FROM x\n",
            "PARAMETER temperature 0.4\n",
            "RENDERER gemma4\n",
        )
    );
}

#[test]
fn fails_closed_when_the_summary_does_not_match_one_complete_block() {
    let partial = "FROM x\nPARAMETER temperature 0.8\n";
    let current = vec![("temperature".into(), "0.7".into())];
    assert!(rewrite(partial, &current, &[]).is_err());

    let duplicated = concat!(
        "FROM x\n",
        "PARAMETER temperature 0.8\n",
        "SYSTEM safe\n",
        "PARAMETER temperature 0.8\n",
    );
    let current = vec![("temperature".into(), "0.8".into())];
    assert!(rewrite(duplicated, &current, &[]).is_err());
}

#[test]
fn complete_ollama_normalization_cycle_has_no_value_drift() {
    let summary = concat!(
        "stop                           \"Assistant: \"\n",
        "stop                           \"\\\"User:\\\"\"\n",
        "temperature                    0.4",
    );
    let semantic = parse(summary).expect("authoritative Ollama summary");
    let normalized = concat!(
        "FROM x\n",
        "PARAMETER temperature 0.4\n",
        "PARAMETER stop \"Assistant: \"\n",
        "PARAMETER stop \"User:\"\n",
    );

    let first = rewrite(normalized, &semantic, &semantic).expect("first save");
    let ollama_normalized_again = normalized;
    let second = rewrite(ollama_normalized_again, &semantic, &semantic).expect("second save");

    assert_eq!(first, second);
    assert!(second.contains("PARAMETER stop \"Assistant: \"\n"));
    assert!(second.contains("PARAMETER stop \"\"\"\"User:\"\"\"\"\n"));
}
