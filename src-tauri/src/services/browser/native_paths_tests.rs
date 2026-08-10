#[cfg(target_os = "macos")]
use super::native_paths::{framework_candidates, helper_executable, resolve_runtime_files};
use super::native_paths::{resolve_windows_runtime_files, WINDOWS_RUNTIME_FILES};
#[cfg(target_os = "macos")]
use std::path::{Path, PathBuf};

#[cfg(target_os = "macos")]
#[test]
fn framework_candidates_prefer_the_installed_bundle() {
    let executable = Path::new("/Applications/Beaver.app/Contents/MacOS/cl-go-dash");
    let downloaded = Path::new("/private/tmp/cef-cache");

    let candidates = framework_candidates(executable, Some(downloaded));

    assert_eq!(
        candidates,
        vec![
            PathBuf::from(
                "/Applications/Beaver.app/Contents/Frameworks/Chromium Embedded Framework.framework/Chromium Embedded Framework",
            ),
            downloaded.join(
                "Chromium Embedded Framework.framework/Chromium Embedded Framework",
            ),
        ]
    );
}

#[cfg(target_os = "macos")]
#[test]
fn helper_path_stays_inside_the_application_bundle() {
    let executable = Path::new("/Applications/Beaver.app/Contents/MacOS/cl-go-dash");

    assert_eq!(
        helper_executable(executable),
        Some(PathBuf::from(
            "/Applications/Beaver.app/Contents/Frameworks/Beaver Helper.app/Contents/MacOS/Beaver Helper",
        ))
    );
    assert_eq!(helper_executable(Path::new("/tmp/cl-go-dash")), None);
}

#[cfg(target_os = "macos")]
#[test]
fn runtime_files_fail_closed_when_the_helper_is_missing() {
    let temp = tempfile::tempdir().expect("tempdir");
    let contents = temp.path().join("Beaver.app/Contents");
    let executable = contents.join("MacOS/cl-go-dash");
    let framework = contents
        .join("Frameworks/Chromium Embedded Framework.framework/Chromium Embedded Framework");
    std::fs::create_dir_all(executable.parent().expect("executable parent")).expect("macos");
    std::fs::create_dir_all(framework.parent().expect("framework parent")).expect("framework");
    std::fs::write(&executable, []).expect("executable");
    std::fs::write(&framework, []).expect("framework binary");

    assert!(resolve_runtime_files(&executable, None).is_none());

    let helper = helper_executable(&executable).expect("helper path");
    std::fs::create_dir_all(helper.parent().expect("helper parent")).expect("helper dirs");
    std::fs::write(&helper, []).expect("helper binary");
    let resolved = resolve_runtime_files(&executable, None).expect("runtime files");

    assert_eq!(
        resolved.framework,
        framework.canonicalize().expect("framework")
    );
    assert_eq!(resolved.helper, helper.canonicalize().expect("helper"));
}

#[test]
fn windows_runtime_validation_requires_every_pinned_file() {
    let temp = tempfile::tempdir().expect("tempdir");
    let executable = temp.path().join("cl-go-dash.exe");
    std::fs::write(&executable, [1]).expect("bootstrap");
    for relative in WINDOWS_RUNTIME_FILES {
        let path = temp.path().join(relative);
        std::fs::create_dir_all(path.parent().expect("runtime parent")).expect("runtime directory");
        std::fs::write(path, [1]).expect("runtime file");
    }

    assert_eq!(
        resolve_windows_runtime_files(&executable),
        executable.canonicalize().ok()
    );
    std::fs::remove_file(temp.path().join("locales/fr.pak")).expect("remove locale");
    assert!(resolve_windows_runtime_files(&executable).is_none());
}

#[test]
fn windows_development_runtime_validates_the_loaded_development_module() {
    let temp = tempfile::tempdir().expect("tempdir");
    let executable = temp.path().join("cl_go_dash_lib.exe");
    std::fs::write(&executable, [1]).expect("development bootstrap");
    for relative in WINDOWS_RUNTIME_FILES {
        if relative == "cl-go-dash.dll" {
            continue;
        }
        let path = temp.path().join(relative);
        std::fs::create_dir_all(path.parent().expect("runtime parent")).expect("runtime directory");
        std::fs::write(path, [1]).expect("runtime file");
    }
    std::fs::write(temp.path().join("cl_go_dash_lib.dll"), [1]).expect("development module");

    assert_eq!(
        resolve_windows_runtime_files(&executable),
        executable.canonicalize().ok()
    );
}

#[test]
fn windows_development_runtime_rejects_the_release_module() {
    let temp = tempfile::tempdir().expect("tempdir");
    let executable = temp.path().join("cl_go_dash_lib.exe");
    std::fs::write(&executable, [1]).expect("development bootstrap");
    for relative in WINDOWS_RUNTIME_FILES {
        let path = temp.path().join(relative);
        std::fs::create_dir_all(path.parent().expect("runtime parent")).expect("runtime directory");
        std::fs::write(path, [1]).expect("runtime file");
    }

    assert!(resolve_windows_runtime_files(&executable).is_none());
}

#[test]
fn windows_runtime_validation_rejects_an_unknown_bootstrap_name() {
    let temp = tempfile::tempdir().expect("tempdir");
    let executable = temp.path().join("unexpected.exe");
    std::fs::write(&executable, [1]).expect("unknown bootstrap");
    for relative in WINDOWS_RUNTIME_FILES {
        let path = temp.path().join(relative);
        std::fs::create_dir_all(path.parent().expect("runtime parent")).expect("runtime directory");
        std::fs::write(path, [1]).expect("runtime file");
    }

    assert!(resolve_windows_runtime_files(&executable).is_none());
}
