use std::path::Path;

pub fn detect_shell() -> String {
    if cfg!(target_os = "windows") {
        if std::env::var("PSModulePath").is_ok() {
            "PowerShell".to_string()
        } else {
            std::env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".to_string())
        }
    } else {
        std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string())
    }
}

pub fn detect_os_version() -> String {
    if cfg!(target_os = "macos") {
        detect_macos_version()
    } else if cfg!(target_os = "linux") {
        detect_linux_version()
    } else if cfg!(target_os = "windows") {
        detect_windows_version()
    } else {
        fallback_version()
    }
}

fn detect_macos_version() -> String {
    std::process::Command::new("sw_vers")
        .arg("-productVersion")
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                let v = String::from_utf8_lossy(&o.stdout).trim().to_string();
                if v.is_empty() {
                    None
                } else {
                    Some(format!("macOS {v}"))
                }
            } else {
                None
            }
        })
        .unwrap_or_else(fallback_version)
}

fn detect_linux_version() -> String {
    let path = Path::new("/etc/os-release");
    if let Ok(content) = std::fs::read_to_string(path) {
        for line in content.lines() {
            if let Some(value) = line.strip_prefix("PRETTY_NAME=") {
                let trimmed = value.trim_matches('"').trim();
                if !trimmed.is_empty() {
                    return trimmed.to_string();
                }
            }
        }
    }
    fallback_version()
}

fn detect_windows_version() -> String {
    std::env::var("OS").unwrap_or_else(|_| fallback_version())
}

fn fallback_version() -> String {
    format!("{} {}", std::env::consts::OS, std::env::consts::ARCH)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_shell_returns_nonempty() {
        let shell = detect_shell();
        assert!(
            !shell.is_empty(),
            "detect_shell() doit retourner une string non-vide"
        );
    }

    #[test]
    fn test_detect_os_version_returns_nonempty() {
        let version = detect_os_version();
        assert!(
            !version.is_empty(),
            "detect_os_version() doit retourner une string non-vide"
        );
    }

    #[cfg(unix)]
    #[test]
    fn test_detect_shell_unix() {
        let shell = detect_shell();
        assert!(
            shell.contains("sh") || shell.starts_with('/'),
            "Sur Unix, le shell doit contenir 'sh' ou commencer par '/' — got: {shell}"
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_detect_macos_version() {
        let version = detect_os_version();
        assert!(
            version.starts_with("macOS"),
            "Sur macOS, la version doit commencer par 'macOS' — got: {version}"
        );
    }
}
