use super::route_profile::FragmentMode;

const MAX_CUMULATIVE_FRAGMENT_BYTES: usize = 32 * 1024 * 1024;

struct FragmentAccumulator {
    mode: FragmentMode,
    previous: String,
}

impl FragmentAccumulator {
    fn new(mode: FragmentMode) -> Self {
        Self {
            mode,
            previous: String::new(),
        }
    }

    fn push(&mut self, fragment: &str) -> Result<String, String> {
        if self.mode != FragmentMode::CumulativeFragments {
            return Ok(fragment.to_string());
        }
        if fragment.len() > MAX_CUMULATIVE_FRAGMENT_BYTES {
            return Err("provider_stream_invalid".to_string());
        }
        let suffix = fragment
            .strip_prefix(&self.previous)
            .ok_or_else(|| "provider_stream_invalid".to_string())?
            .to_string();
        self.previous.clear();
        self.previous.push_str(fragment);
        Ok(suffix)
    }
}

pub(crate) struct StreamFragmentState {
    content: FragmentAccumulator,
    thinking: FragmentAccumulator,
}

impl StreamFragmentState {
    pub(in crate::services::llm) fn new(mode: FragmentMode) -> Self {
        Self {
            content: FragmentAccumulator::new(mode),
            thinking: FragmentAccumulator::new(mode),
        }
    }

    pub(crate) fn ollama() -> Self {
        let mode = super::route_profile::find("ollama")
            .expect("the built-in Ollama route profile must exist")
            .wire
            .fragments;
        Self::new(mode)
    }

    #[cfg(test)]
    pub(crate) fn cumulative_fixture() -> Self {
        Self::new(FragmentMode::CumulativeFragments)
    }

    pub(crate) fn content(&mut self, fragment: &str) -> Result<String, String> {
        self.content.push(fragment)
    }

    pub(crate) fn thinking(&mut self, fragment: &str) -> Result<String, String> {
        self.thinking.push(fragment)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn collect(mode: FragmentMode, fragments: &[&str]) -> Result<String, String> {
        let mut accumulator = FragmentAccumulator::new(mode);
        let mut result = String::new();
        for fragment in fragments {
            result.push_str(&accumulator.push(fragment)?);
        }
        Ok(result)
    }

    #[test]
    fn differential_fragments_are_appended_without_transformation() {
        assert_eq!(
            collect(FragmentMode::DifferentialFragments, &["Bon", "jour", " !"]).unwrap(),
            "Bonjour !"
        );
    }

    #[test]
    fn cumulative_fragments_only_emit_the_new_suffix() {
        assert_eq!(
            collect(
                FragmentMode::CumulativeFragments,
                &["Bon", "Bonjour", "Bonjour !"]
            )
            .unwrap(),
            "Bonjour !"
        );
    }

    #[test]
    fn cumulative_fragments_fail_closed_when_the_prefix_changes() {
        assert_eq!(
            collect(FragmentMode::CumulativeFragments, &["Bonjour", "Bonsoir"]).unwrap_err(),
            "provider_stream_invalid"
        );
    }
}
