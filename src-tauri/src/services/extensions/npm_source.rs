use super::npm_runner::NpmRunner;
use super::source_validation::NpmSource;
use std::path::{Path, PathBuf};

pub fn materialize(
    source: &NpmSource,
    destination: &Path,
    npm: &NpmRunner,
) -> Result<PathBuf, super::OperationFailure> {
    let package = npm.install_package(destination, source)?;
    super::managed_tree::validate(destination)?;
    Ok(package)
}
