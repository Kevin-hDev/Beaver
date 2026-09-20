pub(super) const FALLBACK_TTL: std::time::Duration = std::time::Duration::from_secs(300);

use std::collections::HashMap;
use std::time::Instant;

use super::transport::{McpToolCatalog, McpToolDef};

const MAX_CACHE: usize = 32;

pub(super) struct CachedTools {
    tools: Vec<McpToolDef>,
    generation: u64,
    published_at: Instant,
    expires_at: Instant,
}

#[derive(Default)]
pub(super) struct CacheState {
    generation: u64,
    closed: bool,
    entries: HashMap<String, CachedTools>,
}

impl CacheState {
    pub(super) fn generation(&self) -> u64 {
        self.generation
    }

    pub(super) fn is_closed(&self) -> bool {
        self.closed
    }

    pub(super) fn invalidate(&mut self, connector_id: &str) {
        match self.generation.checked_add(1) {
            Some(next) => self.generation = next,
            None => {
                self.closed = true;
                self.entries.clear();
                return;
            }
        }
        // ponytail: one global generation for at most 32 entries; split by account only if contention is measured.
        self.entries.remove(connector_id);
    }

    pub(super) fn publish(
        &mut self,
        connector_id: &str,
        generation: u64,
        catalog: McpToolCatalog,
        now: Instant,
    ) -> Result<(), String> {
        if self.closed || generation != self.generation {
            return Err("identité MCP modifiée".to_string());
        }
        let Some(ttl) = catalog.cache_ttl.filter(|ttl| !ttl.is_zero()) else {
            self.entries.remove(connector_id);
            return Ok(());
        };
        let expires_at = now.checked_add(ttl).ok_or("durée de cache MCP invalide")?;
        if self.entries.len() >= MAX_CACHE && !self.entries.contains_key(connector_id) {
            if let Some(oldest) = self
                .entries
                .iter()
                .min_by_key(|(id, entry)| (entry.published_at, *id))
                .map(|(id, _)| id.clone())
            {
                self.entries.remove(&oldest);
            }
        }
        self.entries.insert(
            connector_id.to_owned(),
            CachedTools {
                tools: catalog.tools,
                generation,
                published_at: now,
                expires_at,
            },
        );
        Ok(())
    }

    pub(super) fn get(
        &self,
        connector_id: &str,
        generation: u64,
        now: Instant,
    ) -> Option<Vec<McpToolDef>> {
        if self.closed || generation != self.generation {
            return None;
        }
        self.entries.get(connector_id).and_then(|entry| {
            (entry.generation == generation && now < entry.expires_at).then(|| entry.tools.clone())
        })
    }
}
