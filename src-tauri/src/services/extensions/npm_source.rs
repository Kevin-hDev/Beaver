use super::npm_runner::NpmRunner;
use super::source_validation::NpmSource;
use std::path::{Path, PathBuf};

use crate::services::work_registry::ServiceWorkCancellation;

pub fn materialize(
    source: &NpmSource,
    destination: &Path,
    npm: &NpmRunner,
    cancellation: &ServiceWorkCancellation,
) -> Result<PathBuf, super::OperationFailure> {
    let package = npm.install_package(destination, source, cancellation)?;
    super::managed_tree::validate(destination)?;
    Ok(package)
}
