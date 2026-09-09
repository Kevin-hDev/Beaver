// LiteLLM currently publishes fewer than 4,000 entries. Ten thousand leaves
// growth room while rejecting an unbounded external map before publication.
pub(crate) const MAX_LITELLM_CATALOG_ENTRIES: usize = 10_000;
