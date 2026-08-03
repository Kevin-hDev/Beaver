const PROCESS_INJECTION_ENVS: [&str; 3] =
    ["LD_PRELOAD", "LD_AUDIT", "DYLD_INSERT_LIBRARIES"];

pub(crate) fn is_process_injection_env(name: &str) -> bool {
    PROCESS_INJECTION_ENVS.contains(&name)
}

pub(super) fn protect_helper(command: &mut tokio::process::Command) {
    for name in PROCESS_INJECTION_ENVS {
        command.env_remove(name);
    }
}

#[cfg(unix)]
pub(super) fn protect_helper_std(command: &mut std::process::Command) {
    for name in PROCESS_INJECTION_ENVS {
        command.env_remove(name);
    }
}

#[cfg(all(test, unix))]
mod tests {
    #[test]
    fn recognizes_every_process_injection_variable() {
        for name in ["LD_PRELOAD", "LD_AUDIT", "DYLD_INSERT_LIBRARIES"] {
            assert!(super::is_process_injection_env(name));
        }
        assert!(!super::is_process_injection_env("HTTPS_PROXY"));
    }

    #[tokio::test]
    async fn helper_keeps_work_environment_but_removes_process_injection() {
        let mut command = tokio::process::Command::new("/usr/bin/env");
        command
            .env("HTTPS_PROXY", "http://proxy.example")
            .env("SSH_AUTH_SOCK", "/private/agent.sock")
            .env("LD_PRELOAD", "/outside/injection.so")
            .env("LD_AUDIT", "/outside/audit.so")
            .env("DYLD_INSERT_LIBRARIES", "/outside/injection.dylib");

        super::protect_helper(&mut command);
        let output = command.output().await.expect("environment");
        let environment = String::from_utf8_lossy(&output.stdout);

        assert!(output.status.success());
        assert!(environment.contains("HTTPS_PROXY=http://proxy.example"));
        assert!(environment.contains("SSH_AUTH_SOCK=/private/agent.sock"));
        assert!(!environment.contains("LD_PRELOAD="));
        assert!(!environment.contains("LD_AUDIT="));
        assert!(!environment.contains("DYLD_INSERT_LIBRARIES="));
    }
}
