#[cfg(test)]
mod tests {
    use crate::services::agent_local::security::*;

    // --- check_destructive_command ---

    #[test]
    fn allows_rm_rf_root_targets() {
        assert!(check_destructive_command("rm -rf /").is_ok());
        assert!(check_destructive_command("rm  -rf  /").is_ok());
        assert!(check_destructive_command("rm -rf $HOME").is_ok());
        assert!(check_destructive_command("rm -rf ~").is_ok());
    }

    #[test]
    fn allows_rm_rf_wildcard() {
        assert!(check_destructive_command("rm -rf *").is_ok());
    }

    #[test]
    fn blocks_sudo_rm() {
        assert!(check_destructive_command("sudo rm file.txt").is_err());
    }

    #[test]
    fn blocks_chmod_777() {
        assert!(check_destructive_command("chmod 777 file").is_err());
        assert!(check_destructive_command("chmod 777 /etc").is_err());
    }

    #[test]
    fn blocks_disk_operations() {
        assert!(check_destructive_command("dd if=/dev/zero of=/dev/sda").is_err());
        assert!(check_destructive_command("mkfs.ext4 /dev/sda1").is_err());
        assert!(check_destructive_command("echo > /dev/sda").is_err());
        assert!(check_destructive_command("fdisk /dev/sda").is_err());
        assert!(check_destructive_command("dd if=/tmp/file of=/dev/sda").is_err());
    }

    #[test]
    fn blocks_delete_scans_and_rsync_delete() {
        assert!(check_destructive_command("find . -delete").is_err());
        assert!(check_destructive_command("rsync -a --delete src/ dst/").is_err());
    }

    #[test]
    fn blocks_system_control() {
        assert!(check_destructive_command("shutdown now").is_err());
        assert!(check_destructive_command("reboot").is_err());
        assert!(check_destructive_command("init 0").is_err());
        assert!(check_destructive_command("init 6").is_err());
    }

    #[test]
    fn blocks_fork_bomb() {
        assert!(check_destructive_command(":(){:|:&};:").is_err());
    }

    #[test]
    fn blocks_eval_expansion() {
        assert!(check_destructive_command("eval $cmd").is_err());
        assert!(check_destructive_command(r#"eval "$user_input""#).is_err());
        assert!(check_destructive_command("eval  $var").is_err());
    }

    #[test]
    fn allows_safe_commands() {
        assert!(check_destructive_command("ls -la").is_ok());
        assert!(check_destructive_command("echo hello").is_ok());
        assert!(check_destructive_command("cat file.txt").is_ok());
        assert!(check_destructive_command("grep pattern *.rs").is_ok());
        assert!(check_destructive_command("eval 'echo static'").is_ok());
    }

    #[test]
    fn allows_rm_rf_subpath() {
        // rm -rf <sous-chemin> doit passer : /tmp, /home/user/x, ./build, etc.
        assert!(check_destructive_command("rm -rf /tmp/x").is_ok());
        assert!(check_destructive_command("rm -rf /tmp/office-tools-retest.xlsx").is_ok());
        assert!(check_destructive_command("rm -rf /home/user/projet/node_modules").is_ok());
        assert!(check_destructive_command("rm -rf ./build").is_ok());
        assert!(check_destructive_command("rm -rf target/debug").is_ok());
    }

    #[test]
    fn allows_rm_rf_root_wildcard_targets() {
        assert!(check_destructive_command("rm -rf /").is_ok());
        assert!(check_destructive_command("rm -rf /*").is_ok());
        assert!(check_destructive_command("rm -rf *").is_ok());
        assert!(check_destructive_command("rm -rf $HOME").is_ok());
        assert!(check_destructive_command("rm -rf ~").is_ok());
        assert!(check_destructive_command("rm -rf / ; echo done").is_ok());
        assert!(check_destructive_command("rm -rf / && echo done").is_ok());
    }

    // --- validate_write_path (default = full disk) ---

    #[test]
    fn write_path_allows_temp_dir() {
        let tmp = std::env::temp_dir();
        let p = tmp.join("test-cl-go-security.txt");
        let _ = std::fs::remove_file(&p);
        assert!(validate_write_path(&p).is_ok());
    }

    #[test]
    fn write_path_allows_data_dir() {
        let p = crate::services::paths::data_dir().join("test.json");
        assert!(validate_write_path(&p).is_ok());
    }

    #[test]
    fn write_path_allows_app_data_dir() {
        let data = crate::services::paths::data_dir();
        let _ = std::fs::create_dir_all(&data);
        let p = data.join("write-test.json");
        assert!(validate_write_path(&p).is_ok());
    }

    // --- sanitize_error ---

    #[test]
    fn sanitize_masks_paths() {
        let e = std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "No such file or directory (os error 2): /Users/kevinh/secret",
        );
        assert_eq!(sanitize_error(e), "Fichier introuvable");
    }

    // --- validate_read_path (default = full disk) ---

    #[test]
    fn read_path_allows_temp() {
        let tmp = std::env::temp_dir();
        let working = &tmp;
        let p = tmp.join("some-file.txt");
        assert!(validate_read_path(&p, working).is_ok());
    }

    #[test]
    fn read_path_allows_file_under_working_dir() {
        let working = std::env::temp_dir();
        let raw = working.join(".");
        assert!(validate_read_path(&raw, &working).is_ok());
    }

    #[test]
    fn read_path_allows_app_data_dir() {
        let data = crate::services::paths::data_dir();
        let _ = std::fs::create_dir_all(&data);
        let p = data.join("read-test.json");
        assert!(validate_read_path(&p, &std::env::temp_dir()).is_ok());
    }

    #[test]
    fn read_path_allows_nested_app_data_subpath() {
        let nested = crate::services::paths::data_dir()
            .join(format!("security-test-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&nested).expect("create nested app data directory");
        let target = nested.join("test.json");
        let working = std::env::temp_dir();
        assert!(validate_read_path(&target, &working).is_ok());
        std::fs::remove_dir(nested).expect("remove nested app data directory");
    }

    // --- implicit paths always allowed ---

    #[test]
    fn data_dir_always_allowed() {
        let data = crate::services::paths::data_dir();
        let _ = std::fs::create_dir_all(&data);
        let p = data.join("test-security-check.json");
        assert!(validate_read_path(&p, &std::env::temp_dir()).is_ok());
        assert!(validate_write_path(&p).is_ok());
    }

    #[test]
    fn temp_dir_always_allowed() {
        let tmp = std::env::temp_dir();
        let p = tmp.join("cl-go-test-file.txt");
        assert!(validate_read_path(&p, &tmp).is_ok());
        assert!(validate_write_path(&p).is_ok());
    }
}
