pub fn compute_tps(count: u32, first: Option<std::time::Instant>) -> f64 {
    match first {
        Some(t) => {
            let elapsed = t.elapsed().as_secs_f64();
            if elapsed > 0.1 && count > 1 {
                (count - 1) as f64 / elapsed
            } else {
                0.0
            }
        }
        None => 0.0,
    }
}

pub enum FilteredChunk {
    Content(String),
    Thinking(String),
}

pub struct ThinkTagFilter {
    active_close_tag: Option<&'static str>,
    pending: String,
}

const THINK_TAGS: &[(&str, &str)] = &[("<think>", "</think>"), ("<thought>", "</thought>")];

impl ThinkTagFilter {
    pub fn new() -> Self {
        Self {
            active_close_tag: None,
            pending: String::new(),
        }
    }

    pub fn feed(&mut self, chunk: &str) -> Vec<FilteredChunk> {
        self.pending.push_str(chunk);
        let mut output = Vec::new();

        loop {
            if let Some(close_tag) = self.active_close_tag {
                if let Some(pos) = self.pending.find(close_tag) {
                    let thinking = self.pending[..pos].to_string();
                    if !thinking.is_empty() {
                        output.push(FilteredChunk::Thinking(thinking));
                    }
                    self.pending = self.pending[pos + close_tag.len()..].to_string();
                    self.active_close_tag = None;
                    continue;
                }
                let safe = safe_flush_len(&self.pending, close_tag);
                if safe > 0 {
                    output.push(FilteredChunk::Thinking(self.pending[..safe].to_string()));
                    self.pending = self.pending[safe..].to_string();
                }
                break;
            } else {
                if let Some((pos, open_tag, close_tag)) = find_open_tag(&self.pending) {
                    let content = self.pending[..pos].to_string();
                    if !content.is_empty() {
                        output.push(FilteredChunk::Content(content));
                    }
                    self.pending = self.pending[pos + open_tag.len()..].to_string();
                    self.active_close_tag = Some(close_tag);
                    continue;
                }
                let safe = safe_content_flush_len(&self.pending);
                if safe > 0 {
                    output.push(FilteredChunk::Content(self.pending[..safe].to_string()));
                    self.pending = self.pending[safe..].to_string();
                }
                break;
            }
        }

        output
    }

    pub fn flush(&mut self) -> Vec<FilteredChunk> {
        if self.pending.is_empty() {
            return vec![];
        }
        let text = std::mem::take(&mut self.pending);
        if self.active_close_tag.is_some() {
            vec![FilteredChunk::Thinking(text)]
        } else {
            vec![FilteredChunk::Content(text)]
        }
    }
}

fn find_open_tag(text: &str) -> Option<(usize, &'static str, &'static str)> {
    THINK_TAGS
        .iter()
        .filter_map(|(open, close)| text.find(open).map(|pos| (pos, *open, *close)))
        .min_by_key(|(pos, _, _)| *pos)
}

fn safe_content_flush_len(text: &str) -> usize {
    THINK_TAGS
        .iter()
        .map(|(open, _)| safe_flush_len(text, open))
        .min()
        .unwrap_or(text.len())
}

fn safe_flush_len(text: &str, tag: &str) -> usize {
    let len = text.len();
    for i in (1..=tag.len().min(len)).rev() {
        if text.ends_with(&tag[..i]) {
            return len - i;
        }
    }
    len
}

#[cfg(test)]
#[path = "stream_utils_tests.rs"]
mod tests;
