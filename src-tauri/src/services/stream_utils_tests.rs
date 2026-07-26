use super::*;

fn collect(chunks: Vec<FilteredChunk>) -> (String, String) {
    let thinking = chunks
        .iter()
        .filter_map(|chunk| match chunk {
            FilteredChunk::Thinking(text) => Some(text.as_str()),
            _ => None,
        })
        .collect();
    let content = chunks
        .iter()
        .filter_map(|chunk| match chunk {
            FilteredChunk::Content(text) => Some(text.as_str()),
            _ => None,
        })
        .collect();
    (thinking, content)
}

#[test]
fn filter_complete_tags_single_chunk() {
    let mut filter = ThinkTagFilter::new();
    let out: Vec<_> = filter
        .feed("<think>my thoughts</think>visible answer")
        .into_iter()
        .chain(filter.flush())
        .collect();

    assert_eq!(
        collect(out),
        ("my thoughts".into(), "visible answer".into())
    );
}

#[test]
fn filter_thought_tags_single_chunk() {
    let mut filter = ThinkTagFilter::new();
    let out: Vec<_> = filter
        .feed("<thought>reasoning</thought>final")
        .into_iter()
        .chain(filter.flush())
        .collect();

    assert_eq!(collect(out), ("reasoning".into(), "final".into()));
}

#[test]
fn filter_split_across_chunks() {
    let mut filter = ThinkTagFilter::new();
    let mut all = vec![];
    for token in [
        "<",
        "think>",
        "reasoning here",
        "</",
        "think>",
        "the answer",
    ] {
        all.extend(filter.feed(token));
    }
    all.extend(filter.flush());

    assert_eq!(collect(all), ("reasoning here".into(), "the answer".into()));
}

#[test]
fn filter_split_thought_tags() {
    let mut filter = ThinkTagFilter::new();
    let mut all = vec![];
    for token in ["<", "thought>", "hidden", "</", "thought>", "visible"] {
        all.extend(filter.feed(token));
    }
    all.extend(filter.flush());

    assert_eq!(collect(all), ("hidden".into(), "visible".into()));
}

#[test]
fn filter_no_tags() {
    let mut filter = ThinkTagFilter::new();
    let out: Vec<_> = filter
        .feed("just normal content")
        .into_iter()
        .chain(filter.flush())
        .collect();

    assert_eq!(collect(out), ("".into(), "just normal content".into()));
}

#[test]
fn filter_multiple_think_blocks() {
    let mut filter = ThinkTagFilter::new();
    let out: Vec<_> = filter
        .feed("<think>first</think>middle<thought>second</thought>end")
        .into_iter()
        .chain(filter.flush())
        .collect();

    assert_eq!(collect(out), ("firstsecond".into(), "middleend".into()));
}

#[test]
fn filter_token_by_token() {
    let mut filter = ThinkTagFilter::new();
    let tokens = [
        "<", "think", ">", "I ", "need ", "to ", "think", "</", "think", ">", "Hello",
    ];
    let mut all = vec![];
    for token in tokens {
        all.extend(filter.feed(token));
    }
    all.extend(filter.flush());

    assert_eq!(collect(all), ("I need to think".into(), "Hello".into()));
}
