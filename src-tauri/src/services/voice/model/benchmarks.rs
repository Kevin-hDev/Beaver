use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::services::{
    private_store::{
        atomic_write_with_durability, read_bounded_regular, BoundedFile, PublicationDurability,
    },
    voice::limits::MAX_CATALOG_ENTRIES,
};

use super::recognizer::ExecutionProfile;

const VERSION: u8 = 1;
const MAX_BYTES: u64 = 64 * 1024;
const QUALIFYING_AUDIO_MS: u64 = 30_000;
const MAX_AUDIO_MS: u64 = 30 * 60 * 1_000;
const MAX_COMPUTE_MS: u64 = 24 * 60 * 60 * 1_000;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Benchmark {
    pub model_id: String,
    pub revision: String,
    pub audio_ms: u64,
    pub compute_ms: u64,
    pub profile: ExecutionProfile,
}

impl Benchmark {
    pub fn multiplier(&self) -> f64 {
        self.compute_ms as f64 / self.audio_ms as f64
    }

    fn valid(&self) -> bool {
        qualifies_for_speed(self.audio_ms)
            && self.compute_ms > 0
            && self.audio_ms <= MAX_AUDIO_MS
            && self.compute_ms <= MAX_COMPUTE_MS
            && valid_id(&self.model_id)
            && valid_revision(&self.revision)
            && !self.profile.engine_version.is_empty()
            && self.profile.engine_version.len() <= 32
            && self.profile.backend == "cpu"
            && (1..=64).contains(&self.profile.num_threads)
    }
}

#[derive(Default)]
pub(super) struct BenchmarkStore;

#[derive(Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct BenchmarkFile {
    version: u8,
    entries: Vec<Benchmark>,
}

impl BenchmarkStore {
    pub fn load(&self, data_dir: &Path) -> Result<Vec<Benchmark>, String> {
        match read_bounded_regular(&path(data_dir), MAX_BYTES)? {
            BoundedFile::Missing => Ok(Vec::new()),
            BoundedFile::Content(bytes) => {
                let file: BenchmarkFile = serde_json::from_slice(&bytes).map_err(|_| error())?;
                if file.version != VERSION
                    || file.entries.len() > MAX_CATALOG_ENTRIES
                    || file.entries.iter().any(|entry| !entry.valid())
                {
                    return Err(error());
                }
                Ok(file.entries)
            }
        }
    }

    pub fn record(&self, data_dir: &Path, benchmark: Benchmark) -> Result<bool, String> {
        if !qualifies_for_speed(benchmark.audio_ms) {
            return Ok(false);
        }
        if !benchmark.valid() {
            return Err(error());
        }
        let mut entries = self.load(data_dir)?;
        entries.retain(|entry| entry.model_id != benchmark.model_id);
        if entries.len() >= MAX_CATALOG_ENTRIES {
            return Err(error());
        }
        entries.push(benchmark);
        save(data_dir, entries)?;
        Ok(true)
    }

    pub fn current(
        &self,
        data_dir: &Path,
        model_id: &str,
        revision: &str,
        profile: &ExecutionProfile,
    ) -> Result<Option<Benchmark>, String> {
        Ok(self.load(data_dir)?.into_iter().find(|entry| {
            entry.model_id == model_id && entry.revision == revision && entry.profile == *profile
        }))
    }

    pub fn remove(&self, data_dir: &Path, model_id: &str) -> Result<(), String> {
        let mut entries = self.load(data_dir)?;
        let before = entries.len();
        entries.retain(|entry| entry.model_id != model_id);
        if entries.len() != before {
            save(data_dir, entries)?;
        }
        Ok(())
    }
}

fn save(data_dir: &Path, entries: Vec<Benchmark>) -> Result<(), String> {
    let bytes = serde_json::to_vec(&BenchmarkFile {
        version: VERSION,
        entries,
    })
    .map_err(|_| error())?;
    match atomic_write_with_durability(&path(data_dir), &bytes)? {
        PublicationDurability::Durable => Ok(()),
        PublicationDurability::PublishedDurabilityUnconfirmed => Err(error()),
    }
}

pub fn qualifies_for_speed(audio_ms: u64) -> bool {
    audio_ms >= QUALIFYING_AUDIO_MS
}

fn path(data_dir: &Path) -> PathBuf {
    data_dir.join("voice-benchmarks.json")
}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn valid_revision(value: &str) -> bool {
    let digest = value.strip_prefix("sha256:").unwrap_or(value);
    matches!(digest.len(), 40 | 64)
        && digest
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn error() -> String {
    "voice-benchmarks-unavailable".into()
}
