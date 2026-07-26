use std::path::PathBuf;

const MAX_DATA_COMPONENT_CHARS: usize = 255;

#[cfg(all(not(test), not(feature = "cef-test-profile")))]
pub fn data_dir() -> PathBuf {
    dirs::home_dir()
        .expect("cannot resolve home directory")
        .join(".local/share/cl-go-dash")
}

pub async fn data_file_for_read(directory: &str, file_name: &str) -> std::io::Result<PathBuf> {
    resolve_data_file(directory, file_name, false).await
}

pub async fn data_file_for_write(directory: &str, file_name: &str) -> std::io::Result<PathBuf> {
    resolve_data_file(directory, file_name, true).await
}

async fn resolve_data_file(
    directory: &str,
    file_name: &str,
    create_directory: bool,
) -> std::io::Result<PathBuf> {
    validate_component(directory)?;
    validate_component(file_name)?;
    let root = data_dir();
    if create_directory {
        tokio::fs::create_dir_all(&root).await?;
    }
    let canonical_root = tokio::fs::canonicalize(&root).await?;
    let requested_directory = canonical_root.join(directory);
    if create_directory {
        tokio::fs::create_dir_all(&requested_directory).await?;
    }
    let canonical_directory = tokio::fs::canonicalize(&requested_directory).await?;
    if !canonical_directory.starts_with(&canonical_root) {
        return Err(invalid_data_path());
    }
    let requested_file = canonical_directory.join(file_name);
    match tokio::fs::canonicalize(&requested_file).await {
        Ok(canonical_file) if canonical_file.starts_with(&canonical_directory) => {
            Ok(canonical_file)
        }
        Ok(_) => Err(invalid_data_path()),
        Err(error) if create_directory && error.kind() == std::io::ErrorKind::NotFound => {
            Ok(requested_file)
        }
        Err(error) => Err(error),
    }
}

fn validate_component(value: &str) -> std::io::Result<()> {
    use std::path::{Component, Path};

    let mut components = Path::new(value).components();
    let valid = value.chars().count() <= MAX_DATA_COMPONENT_CHARS
        && matches!(components.next(), Some(Component::Normal(_)))
        && components.next().is_none();
    valid.then_some(()).ok_or_else(invalid_data_path)
}

fn invalid_data_path() -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid data path")
}

#[cfg(all(not(test), feature = "cef-test-profile"))]
pub fn data_dir() -> PathBuf {
    use std::sync::OnceLock;

    static TEST_DATA_DIR: OnceLock<PathBuf> = OnceLock::new();
    TEST_DATA_DIR
        .get_or_init(|| resolve_cef_test_data_dir().expect("invalid CEF test data directory"))
        .clone()
}

#[cfg(feature = "cef-test-profile")]
fn resolve_cef_test_data_dir() -> Result<PathBuf, ()> {
    let raw = std::env::var_os("CL_GO_CEF_TEST_DATA_DIR").ok_or(())?;
    let requested = PathBuf::from(raw);
    let temp = std::env::temp_dir().canonicalize().map_err(|_| ())?;
    if !clean_cef_test_path(&requested) {
        return Err(());
    }
    let resolved = requested.canonicalize().map_err(|_| ())?;
    allowed_cef_test_path(&resolved, &temp)
        .then_some(resolved)
        .ok_or(())
}

#[cfg(any(test, feature = "cef-test-profile"))]
fn allowed_cef_test_path(path: &std::path::Path, temp: &std::path::Path) -> bool {
    clean_cef_test_path(path) && path.starts_with(temp)
}

#[cfg(any(test, feature = "cef-test-profile"))]
fn clean_cef_test_path(path: &std::path::Path) -> bool {
    use std::path::Component;

    path.is_absolute()
        && path.to_string_lossy().len() <= 4096
        && !path
            .components()
            .any(|part| matches!(part, Component::ParentDir | Component::CurDir))
}

#[cfg(test)]
pub fn data_dir() -> PathBuf {
    use std::sync::OnceLock;

    static TEST_DATA_DIR: OnceLock<PathBuf> = OnceLock::new();
    TEST_DATA_DIR
        .get_or_init(|| {
            let path = std::env::temp_dir().join("cl-go-dash-tests").join(format!(
                "{}-{}",
                std::process::id(),
                uuid::Uuid::new_v4()
            ));
            std::fs::create_dir_all(&path).expect("create isolated test data directory");
            path.canonicalize()
                .expect("canonicalize isolated test data directory")
        })
        .clone()
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    #[test]
    fn test_data_dir_never_targets_user_data() {
        let actual = super::data_dir();
        let production = dirs::home_dir()
            .expect("cannot resolve home directory")
            .join(".local/share/cl-go-dash");

        assert_ne!(actual, production);
        let temp = std::env::temp_dir()
            .canonicalize()
            .expect("canonicalize system temp directory");
        assert!(actual.starts_with(temp));
    }

    #[test]
    fn cef_test_profile_accepts_only_clean_absolute_temp_paths() {
        let temp = Path::new("/private/tmp");

        assert!(super::allowed_cef_test_path(
            Path::new("/private/tmp/cl-go-cef-test"),
            temp,
        ));
        assert!(!super::allowed_cef_test_path(
            Path::new("/private/tmp/../Users/kevinh"),
            temp,
        ));
        assert!(!super::allowed_cef_test_path(
            Path::new("/Users/kevinh/.local/share/cl-go-dash"),
            temp,
        ));
    }

    #[test]
    fn data_components_reject_traversal_and_nested_paths() {
        assert!(super::validate_component("forecast-analyses").is_ok());
        assert!(super::validate_component("analysis.json").is_ok());
        assert!(super::validate_component("../analysis.json").is_err());
        assert!(super::validate_component("nested/analysis.json").is_err());
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn data_file_read_rejects_a_symlink_outside_its_directory() {
        use std::os::unix::fs::symlink;

        let directory = format!("path-guard-{}", uuid::Uuid::new_v4());
        let trusted = super::data_dir().join(&directory);
        tokio::fs::create_dir_all(&trusted).await.unwrap();
        let outside = tempfile::NamedTempFile::new().unwrap();
        symlink(outside.path(), trusted.join("escape.json")).unwrap();

        assert!(super::data_file_for_read(&directory, "escape.json")
            .await
            .is_err());
        tokio::fs::remove_dir_all(trusted).await.unwrap();
    }
}
