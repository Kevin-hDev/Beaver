use super::durable_fs::{OllamaDurableFs, OllamaFsError, OllamaFsErrorKind};
use super::path_identity::CanonicalDirectory;
use crate::services::paths::ollama_paths;
use std::collections::VecDeque;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Mutex;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum FailurePoint {
    CreateTmp,
    Write,
    SyncFile,
    Rename,
    SyncParent,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ExpectedCall {
    ReadTmp,
    CreateDirectory,
    WriteNew,
    Replace,
    ReadFinal,
    RenameDurable,
    RemoveFileDurable,
    RemoveTree,
    SyncFile,
    SyncParent,
}

#[derive(Default)]
pub(super) struct ScriptedFs {
    expected: Mutex<VecDeque<ExpectedCall>>,
    events: Mutex<Vec<&'static str>>,
    failure: Mutex<Option<FailurePoint>>,
    tmp: Mutex<Option<Vec<u8>>>,
    final_bytes: Mutex<Option<Vec<u8>>>,
    final_override: Mutex<Option<Vec<u8>>>,
    expected_parent: Mutex<Option<PathBuf>>,
    expected_tmp: Mutex<Option<PathBuf>>,
    expected_final: Mutex<Option<PathBuf>>,
}

impl ScriptedFs {
    pub(super) fn scripted(calls: impl IntoIterator<Item = ExpectedCall>) -> Self {
        Self {
            expected: Mutex::new(calls.into_iter().collect()),
            ..Self::default()
        }
    }

    pub(super) fn scripted_at(root: &Path, calls: impl IntoIterator<Item = ExpectedCall>) -> Self {
        let paths = ollama_paths(root);
        Self {
            expected_parent: Mutex::new(paths.journal.parent().map(Path::to_path_buf)),
            expected_tmp: Mutex::new(Some(paths.journal_tmp)),
            expected_final: Mutex::new(Some(paths.journal)),
            ..Self::scripted(calls)
        }
    }

    pub(super) fn fail_at(&self, point: FailurePoint) {
        *self.failure.lock().unwrap() = Some(point);
    }

    fn fail(&self, point: FailurePoint) -> Result<(), OllamaFsError> {
        if *self.failure.lock().unwrap() == Some(point) {
            return Err(OllamaFsError::new(OllamaFsErrorKind::Other));
        }
        Ok(())
    }

    fn expect(&self, actual: ExpectedCall) {
        let expected = self
            .expected
            .lock()
            .unwrap()
            .pop_front()
            .unwrap_or_else(|| panic!("unexpected fake FS call: {actual:?}"));
        assert_eq!(expected, actual, "unexpected fake FS call");
    }

    fn record(&self, event: &'static str) {
        self.events.lock().unwrap().push(event);
    }

    fn expect_path(&self, path: &Path, expected: &Mutex<Option<PathBuf>>) {
        if let Some(expected) = expected.lock().unwrap().as_deref() {
            assert_eq!(path, expected, "unexpected fake FS path");
        }
    }

    fn simulate_write(&self, bytes: &[u8], operation: ExpectedCall) -> Result<(), OllamaFsError> {
        self.expect(operation);
        self.record(match operation {
            ExpectedCall::WriteNew => "write_new",
            ExpectedCall::Replace => "replace",
            _ => unreachable!(),
        });
        self.record("create_tmp");
        self.fail(FailurePoint::CreateTmp)?;
        *self.tmp.lock().unwrap() = Some(Vec::new());
        self.record("write");
        self.fail(FailurePoint::Write)?;
        *self.tmp.lock().unwrap() = Some(bytes.to_vec());
        self.record("sync_file");
        self.fail(FailurePoint::SyncFile)?;
        self.record("rename");
        self.fail(FailurePoint::Rename)?;
        *self.final_bytes.lock().unwrap() = Some(bytes.to_vec());
        self.tmp.lock().unwrap().take();
        self.record("sync_parent");
        self.fail(FailurePoint::SyncParent)?;
        Ok(())
    }

    pub(super) fn events(&self) -> Vec<&'static str> {
        self.events.lock().unwrap().clone()
    }

    pub(super) fn finish(&self) {
        assert!(
            self.expected.lock().unwrap().is_empty(),
            "fake FS script still has queued operations"
        );
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

    pub(super) fn final_is_present(&self) -> bool {
        self.final_bytes.lock().unwrap().is_some()
    }
}

impl OllamaDurableFs for ScriptedFs {
    fn read_bounded(&self, path: &Path, _max_bytes: usize) -> Result<Vec<u8>, OllamaFsError> {
        let is_tmp = path.extension().and_then(|value| value.to_str()) == Some("tmp");
        self.expect_path(
            path,
            if is_tmp {
                &self.expected_tmp
            } else {
                &self.expected_final
            },
        );
        self.expect(if is_tmp {
            ExpectedCall::ReadTmp
        } else {
            ExpectedCall::ReadFinal
        });
        self.record(if is_tmp { "read_tmp" } else { "read_final" });
        let value = if is_tmp {
            self.tmp.lock().unwrap().clone()
        } else {
            self.final_override
                .lock()
                .unwrap()
                .clone()
                .or_else(|| self.final_bytes.lock().unwrap().clone())
        };
        value.ok_or_else(|| OllamaFsError::new(OllamaFsErrorKind::NotFound))
    }

    fn create_directory_durable(&self, path: &Path) -> Result<(), OllamaFsError> {
        self.expect_path(path, &self.expected_parent);
        self.expect(ExpectedCall::CreateDirectory);
        self.record("create_directory");
        Ok(())
    }

    fn write_new_atomic(
        &self,
        tmp: &Path,
        final_path: &Path,
        bytes: &[u8],
    ) -> Result<(), OllamaFsError> {
        self.expect_path(tmp, &self.expected_tmp);
        self.expect_path(final_path, &self.expected_final);
        self.simulate_write(bytes, ExpectedCall::WriteNew)
    }

    fn replace_atomic(
        &self,
        tmp: &Path,
        final_path: &Path,
        bytes: &[u8],
    ) -> Result<(), OllamaFsError> {
        self.expect_path(tmp, &self.expected_tmp);
        self.expect_path(final_path, &self.expected_final);
        self.simulate_write(bytes, ExpectedCall::Replace)
    }

    fn rename_durable(&self, _source: &Path, _destination: &Path) -> Result<(), OllamaFsError> {
        self.expect(ExpectedCall::RenameDurable);
        Ok(())
    }

    fn remove_file_durable(&self, _path: &Path) -> Result<(), OllamaFsError> {
        self.expect(ExpectedCall::RemoveFileDurable);
        Ok(())
    }

    fn remove_tree(&self, _root: &Path) -> Result<(), OllamaFsError> {
        self.expect(ExpectedCall::RemoveTree);
        Ok(())
    }

    fn remove_tree_verified(&self, root: &CanonicalDirectory) -> Result<(), OllamaFsError> {
        self.remove_tree(root.path())
    }

    fn sync_file(&self, _path: &Path) -> Result<(), OllamaFsError> {
        self.expect(ExpectedCall::SyncFile);
        Ok(())
    }

    fn sync_parent(&self, _path: &Path) -> Result<(), OllamaFsError> {
        self.expect(ExpectedCall::SyncParent);
        Ok(())
    }
}
