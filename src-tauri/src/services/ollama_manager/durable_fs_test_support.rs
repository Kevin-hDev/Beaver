use super::durable_fs::{OllamaDurableFs, OllamaFsError, OllamaFsErrorKind};
use std::path::Path;
use std::sync::Mutex;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum FailurePoint {
    CreateTmp,
    Write,
    SyncFile,
    Rename,
    SyncParent,
}

#[derive(Default)]
pub(super) struct ScriptedFs {
    events: Mutex<Vec<&'static str>>,
    failure: Mutex<Option<FailurePoint>>,
    tmp: Mutex<Option<Vec<u8>>>,
    final_bytes: Mutex<Option<Vec<u8>>>,
    final_override: Mutex<Option<Vec<u8>>>,
}

impl ScriptedFs {
    pub(super) fn fail_at(&self, point: FailurePoint) {
        *self.failure.lock().unwrap() = Some(point);
    }

    fn fail(&self, point: FailurePoint) -> Result<(), OllamaFsError> {
        if *self.failure.lock().unwrap() == Some(point) {
            return Err(OllamaFsError::new(OllamaFsErrorKind::Other));
        }
        Ok(())
    }

    fn fail_after_temp(&self, point: FailurePoint) -> Result<(), OllamaFsError> {
        let result = self.fail(point);
        if result.is_err() {
            *self.tmp.lock().unwrap() = None;
        }
        result
    }

    fn record(&self, event: &'static str) {
        self.events.lock().unwrap().push(event);
    }

    pub(super) fn events(&self) -> Vec<&'static str> {
        self.events.lock().unwrap().clone()
    }

    pub(super) fn set_tmp(&self, bytes: Vec<u8>) {
        *self.tmp.lock().unwrap() = Some(bytes);
    }

    pub(super) fn set_final_override(&self, bytes: Vec<u8>) {
        *self.final_override.lock().unwrap() = Some(bytes);
    }

    pub(super) fn temp_is_absent(&self) -> bool {
        self.tmp.lock().unwrap().is_none()
    }
}

impl OllamaDurableFs for ScriptedFs {
    fn read_bounded(&self, path: &Path, _max_bytes: usize) -> Result<Vec<u8>, OllamaFsError> {
        if path.extension().and_then(|v| v.to_str()) == Some("tmp") {
            self.record("read_tmp");
            return self
                .tmp
                .lock()
                .unwrap()
                .clone()
                .ok_or_else(|| OllamaFsError::new(OllamaFsErrorKind::NotFound));
        }
        self.record("read_final");
        self.final_override
            .lock()
            .unwrap()
            .clone()
            .or_else(|| self.final_bytes.lock().unwrap().clone())
            .ok_or_else(|| OllamaFsError::new(OllamaFsErrorKind::NotFound))
    }

    fn create_directory_durable(&self, _path: &Path) -> Result<(), OllamaFsError> {
        Ok(())
    }

    fn write_new_atomic(
        &self,
        _tmp: &Path,
        _final_path: &Path,
        bytes: &[u8],
    ) -> Result<(), OllamaFsError> {
        self.record("create_tmp");
        self.fail(FailurePoint::CreateTmp)?;
        self.record("write");
        self.fail(FailurePoint::Write)?;
        *self.tmp.lock().unwrap() = Some(bytes.to_vec());
        self.record("sync_file");
        self.fail_after_temp(FailurePoint::SyncFile)?;
        self.record("rename");
        self.fail_after_temp(FailurePoint::Rename)?;
        *self.final_bytes.lock().unwrap() = self.tmp.lock().unwrap().take();
        self.record("sync_parent");
        self.fail(FailurePoint::SyncParent)
    }

    fn replace_atomic(
        &self,
        tmp: &Path,
        final_path: &Path,
        bytes: &[u8],
    ) -> Result<(), OllamaFsError> {
        self.record("replace");
        self.write_new_atomic(tmp, final_path, bytes)
    }

    fn rename_durable(&self, _source: &Path, _destination: &Path) -> Result<(), OllamaFsError> {
        Ok(())
    }

    fn remove_file_durable(&self, _path: &Path) -> Result<(), OllamaFsError> {
        Ok(())
    }

    fn remove_tree(&self, _root: &Path) -> Result<(), OllamaFsError> {
        Ok(())
    }

    fn sync_file(&self, _path: &Path) -> Result<(), OllamaFsError> {
        Ok(())
    }

    fn sync_parent(&self, _path: &Path) -> Result<(), OllamaFsError> {
        Ok(())
    }
}
