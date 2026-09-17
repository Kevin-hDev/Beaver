#[test]
fn powershell_wrapper_preserves_native_exit_codes() {
    let script = super::powershell_script("native-command");

    assert!(script.contains("$global:LASTEXITCODE = $null"));
    assert!(script.contains("exit [int]$beaverStatus"));
    assert!(script.contains("if ($beaverSucceeded) { exit 0 }"));
    assert!(script.contains("native-command"));
}

#[cfg(unix)]
#[test]
fn unix_wrapper_keeps_user_command_out_of_the_wrapper_argument() {
    let arguments = super::shell_arguments("printf private-command");

    assert!(!arguments[1].contains("private-command"));
    assert_eq!(arguments.last().map(String::as_str), Some("printf private-command"));
}
