use super::super::error::OllamaErrorCode;
use super::relative_components;
use std::fs::{File, OpenOptions};
use std::io;
use std::os::windows::fs::{MetadataExt, OpenOptionsExt};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use windows_sys::Win32::Storage::FileSystem::{
    FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_REPARSE_POINT, FILE_FLAG_BACKUP_SEMANTICS,
    FILE_FLAG_OPEN_REPARSE_POINT, FILE_LIST_DIRECTORY, FILE_READ_ATTRIBUTES, FILE_SHARE_READ,
    FILE_SHARE_WRITE,
};

const DIRECTORY_FLAGS: u32 = FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT;
const SHARE_MODE: u32 = FILE_SHARE_READ | FILE_SHARE_WRITE;

pub(super) struct PlatformExtractionRoot {
    root_path: PathBuf,
    allow_existing_files: bool,
    directories: Mutex<Vec<(PathBuf, Arc<File>)>>,
}

impl PlatformExtractionRoot {
    pub(super) fn open(path: &Path, require_empty: bool) -> Result<Self, OllamaErrorCode> {
        let root = open_directory(path)?;
        if require_empty && std::fs::read_dir(path).map_err(map_error)?.next().is_some() {
            return Err(OllamaErrorCode::OllamaUpdateRecoveryRequired);
        }
        Ok(Self {
            root_path: path.to_path_buf(),
            allow_existing_files: !require_empty,
            directories: Mutex::new(vec![(path.to_path_buf(), Arc::new(root))]),
        })
    }

    pub(super) fn create_directory_all(&self, path: &Path) -> Result<(), OllamaErrorCode> {
        let mut current = self.root_path.clone();
        for component in relative_components(path)? {
            current.push(component);
            self.open_or_create_directory(&current)?;
        }
        Ok(())
    }

    pub(super) fn create_file(&self, path: &Path, _mode: u32) -> Result<File, OllamaErrorCode> {
        let components = relative_components(path)?;
        let (_name, parent) = components
            .split_last()
            .ok_or(OllamaErrorCode::OllamaBundleInvalid)?;
        let parent = parent
            .iter()
            .fold(self.root_path.clone(), |mut path, part| {
                path.push(part);
                path
            });
        self.create_directory_all(
            parent
                .strip_prefix(&self.root_path)
                .unwrap_or(parent.as_path()),
        )?;
        let target = self.root_path.join(path);
        let mut options = OpenOptions::new();
        options
            .write(true)
            .share_mode(SHARE_MODE)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT);
        if self.allow_existing_files {
            options.create(true).truncate(true);
        } else {
            options.create_new(true);
        }
        let file = options
            .open(&target)
            .map_err(|error| map_file_error(&target, error))?;
        let metadata = file.metadata().map_err(map_error)?;
        if metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
            || metadata.file_attributes() & FILE_ATTRIBUTE_DIRECTORY != 0
        {
            return Err(OllamaErrorCode::OllamaBundleInvalid);
        }
        Ok(file)
    }

    fn open_or_create_directory(&self, path: &Path) -> Result<Arc<File>, OllamaErrorCode> {
        if let Some(existing) = self.find_directory(path)? {
            return Ok(existing);
        }
        match std::fs::create_dir(path) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(map_error(error)),
        }
        let handle = Arc::new(open_directory(path)?);
        let mut directories = self
            .directories
            .lock()
            .map_err(|_| OllamaErrorCode::OllamaStorageUnavailable)?;
        if let Some((_, existing)) = directories.iter().find(|(known, _)| known == path) {
            return Ok(Arc::clone(existing));
        }
        directories.push((path.to_path_buf(), Arc::clone(&handle)));
        Ok(handle)
    }

    fn find_directory(&self, path: &Path) -> Result<Option<Arc<File>>, OllamaErrorCode> {
        let directories = self
            .directories
            .lock()
            .map_err(|_| OllamaErrorCode::OllamaStorageUnavailable)?;
        Ok(directories
            .iter()
            .find(|(known, _)| known == path)
            .map(|(_, handle)| Arc::clone(handle)))
    }
}

fn open_directory(path: &Path) -> Result<File, OllamaErrorCode> {
    let file = OpenOptions::new()
        .read(true)
        .share_mode(SHARE_MODE)
        .access_mode(FILE_LIST_DIRECTORY | FILE_READ_ATTRIBUTES)
        .custom_flags(DIRECTORY_FLAGS)
        .open(path)
        .map_err(map_error)?;
    let metadata = file.metadata().map_err(map_error)?;
    if metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
        return Err(OllamaErrorCode::OllamaBundleInvalid);
    }
    if metadata.file_attributes() & FILE_ATTRIBUTE_DIRECTORY == 0 {
        return Err(OllamaErrorCode::OllamaBundleInvalid);
    }
    Ok(file)
}

fn map_file_error(path: &Path, error: io::Error) -> OllamaErrorCode {
    if error.kind() == io::ErrorKind::AlreadyExists
        && std::fs::symlink_metadata(path)
            .map(|metadata| metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0)
            .unwrap_or(false)
    {
        OllamaErrorCode::OllamaBundleInvalid
    } else {
        map_error(error)
    }
}

fn map_error(error: io::Error) -> OllamaErrorCode {
    match error.kind() {
        io::ErrorKind::AlreadyExists => OllamaErrorCode::OllamaUpdateRecoveryRequired,
        io::ErrorKind::NotFound => OllamaErrorCode::OllamaStorageUnavailable,
        _ => OllamaErrorCode::OllamaStorageUnavailable,
    }
}
