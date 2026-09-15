use std::path::Path;

use crate::services::voice::download::remove_installation;

use super::{
    benchmarks::{Benchmark, BenchmarkStore},
    lifecycle::ModelLifecycle,
    recognizer::ExecutionProfile,
};

impl ModelLifecycle {
    pub fn remove(&self, data_dir: &Path, model_id: &str) -> Result<(), String> {
        remove_installation(data_dir, model_id, self)?;
        BenchmarkStore.remove(data_dir, model_id)
    }

    pub fn prepare_reinstall(&self, data_dir: &Path, model_id: &str) -> Result<(), String> {
        self.invalidate_installation(model_id)
            .map_err(|_| "model-download-model-busy".to_string())?;
        BenchmarkStore.remove(data_dir, model_id)
    }

    pub fn record_benchmark(&self, data_dir: &Path, benchmark: Benchmark) -> Result<bool, String> {
        BenchmarkStore.record(data_dir, benchmark)
    }

    pub fn current_benchmark(
        &self,
        data_dir: &Path,
        model_id: &str,
        revision: &str,
        profile: &ExecutionProfile,
    ) -> Result<Option<Benchmark>, String> {
        BenchmarkStore.current(data_dir, model_id, revision, profile)
    }
}
