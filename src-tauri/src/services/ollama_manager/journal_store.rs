#![allow(dead_code)]

use super::blocking::run_ollama_blocking;
use super::constants::MAX_DURABLE_DOCUMENT_BYTES;
use super::durable_fs::{
    platform_fs, OllamaDurableFs, OllamaFsError, OllamaFsErrorKind, PlatformOllamaDurableFs,
};
use super::error::OllamaErrorCode;
use super::journal::OllamaTransactionJournal;
use crate::services::paths::OllamaPaths;
use std::sync::Arc;

pub(super) struct OllamaJournalStore<F: OllamaDurableFs> {
    fs: Arc<F>,
    paths: OllamaPaths,
}

impl<F: OllamaDurableFs + 'static> OllamaJournalStore<F> {
    pub(super) fn new(fs: Arc<F>, paths: OllamaPaths) -> Self {
        Self { fs, paths }
    }

    pub(super) async fn read(&self) -> Result<Option<OllamaTransactionJournal>, OllamaErrorCode> {
        let fs = Arc::clone(&self.fs);
        let path = self.paths.journal.clone();
        run_ollama_blocking(
            move || match fs.read_bounded(&path, MAX_DURABLE_DOCUMENT_BYTES) {
                Ok(bytes) => OllamaTransactionJournal::parse_bounded(&bytes)
                    .map(Some)
                    .map_err(|_| OllamaErrorCode::OllamaJournalInvalid),
                Err(error) if error.kind() == OllamaFsErrorKind::NotFound => Ok(None),
                Err(error) => Err(storage_error(error)),
            },
        )
        .await
    }

    pub(super) async fn write_new(
        &self,
        journal: &OllamaTransactionJournal,
    ) -> Result<(), OllamaErrorCode> {
        self.write(journal, false).await
    }

    pub(super) async fn replace(
        &self,
        journal: &OllamaTransactionJournal,
    ) -> Result<(), OllamaErrorCode> {
        self.write(journal, true).await
    }

    async fn write(
        &self,
        journal: &OllamaTransactionJournal,
        replace: bool,
    ) -> Result<(), OllamaErrorCode> {
        let bytes = serialize(journal)?;
        let expected = journal.clone();
        let fs = Arc::clone(&self.fs);
        let paths = self.paths.clone();
        run_ollama_blocking(move || {
            let parent = paths
                .journal
                .parent()
                .ok_or(OllamaErrorCode::OllamaInternal)?;
            fs.create_directory_durable(parent).map_err(storage_error)?;
            refuse_existing_tmp(&*fs, &paths)?;
            if replace {
                fs.replace_atomic(&paths.journal_tmp, &paths.journal, &bytes)
            } else {
                fs.write_new_atomic(&paths.journal_tmp, &paths.journal, &bytes)
            }
            .map_err(storage_error)?;
            let committed = fs
                .read_bounded(&paths.journal, MAX_DURABLE_DOCUMENT_BYTES)
                .map_err(storage_error)?;
            let parsed = OllamaTransactionJournal::parse_bounded(&committed)
                .map_err(|_| OllamaErrorCode::OllamaJournalInvalid)?;
            (parsed == expected)
                .then_some(())
                .ok_or(OllamaErrorCode::OllamaJournalInvalid)
        })
        .await
    }

    pub(super) async fn remove(&self) -> Result<(), OllamaErrorCode> {
        let fs = Arc::clone(&self.fs);
        let path = self.paths.journal.clone();
        run_ollama_blocking(move || match fs.remove_file_durable(&path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == OllamaFsErrorKind::NotFound => Ok(()),
            Err(error) => Err(storage_error(error)),
        })
        .await
    }
}

impl OllamaJournalStore<PlatformOllamaDurableFs> {
    pub(super) fn platform(paths: OllamaPaths) -> Self {
        Self::new(Arc::new(platform_fs()), paths)
    }
}

fn serialize(journal: &OllamaTransactionJournal) -> Result<Vec<u8>, OllamaErrorCode> {
    journal
        .validate()
        .map_err(|_| OllamaErrorCode::OllamaJournalInvalid)?;
    let bytes = serde_json::to_vec(journal).map_err(|_| OllamaErrorCode::OllamaInternal)?;
    (bytes.len() <= MAX_DURABLE_DOCUMENT_BYTES)
        .then_some(bytes)
        .ok_or(OllamaErrorCode::OllamaJournalInvalid)
}

fn refuse_existing_tmp<F: OllamaDurableFs>(
    fs: &F,
    paths: &OllamaPaths,
) -> Result<(), OllamaErrorCode> {
    match fs.read_bounded(&paths.journal_tmp, MAX_DURABLE_DOCUMENT_BYTES) {
        Err(error) if error.kind() == OllamaFsErrorKind::NotFound => Ok(()),
        Ok(_) => Err(OllamaErrorCode::OllamaStorageUnavailable),
        Err(error) => Err(storage_error(error)),
    }
}

fn storage_error(error: OllamaFsError) -> OllamaErrorCode {
    ::log::error!(
        "[ollama] durable storage failure kind={:?} operation={:?} os_code={:?}",
        error.kind(),
        error.operation(),
        error.os_code()
    );
    OllamaErrorCode::OllamaStorageUnavailable
}
