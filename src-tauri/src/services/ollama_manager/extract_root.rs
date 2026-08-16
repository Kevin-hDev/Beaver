#![allow(dead_code)]

use super::error::OllamaErrorCode;
use std::fs::File;
use std::path::{Path, PathBuf};

#[cfg(unix)]
#[path = "extract_root_unix.rs"]
mod platform;
#[cfg(windows)]
#[path = "extract_root_windows.rs"]
mod platform;

pub(super) struct ExtractionRoot {
    inner: platform::PlatformExtractionRoot,
}

impl ExtractionRoot {
    pub(super) fn open(staging: &Path, require_empty: bool) -> Result<Self, OllamaErrorCode> {
        platform::PlatformExtractionRoot::open(staging, require_empty).map(|inner| Self { inner })
    }

    pub(super) fn create_directory_all(&self, path: &Path) -> Result<(), OllamaErrorCode> {
        self.inner.create_directory_all(path)
    }

    pub(super) fn create_file(&self, path: &Path, mode: u32) -> Result<File, OllamaErrorCode> {
        self.inner.create_file(path, mode)
    }

    #[cfg(unix)]
    pub(super) fn create_symlink(&self, path: &Path, target: &Path) -> Result<(), OllamaErrorCode> {
        self.inner.create_symlink(path, target)
    }
}

pub(super) fn relative_components(path: &Path) -> Result<Vec<PathBuf>, OllamaErrorCode> {
    path.components()
        .map(|component| match component {
            std::path::Component::Normal(value) => Ok(PathBuf::from(value)),
            _ => Err(OllamaErrorCode::OllamaBundleInvalid),
        })
        .collect()
}
