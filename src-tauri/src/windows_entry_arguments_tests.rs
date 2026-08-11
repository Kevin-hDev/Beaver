use super::*;
use std::os::windows::ffi::OsStringExt;

#[test]
fn bootstrap_arguments_only_forward_validated_values() {
    let arguments = bootstrap_arguments(
        OsStr::new("bootstrap.exe"),
        vec![OsString::from("--inspect")],
    )
    .expect("bootstrap arguments");

    assert_eq!(arguments, vec![OsString::from("--inspect")]);
}

#[test]
fn bootstrap_arguments_reject_an_external_module_override() {
    assert!(bootstrap_arguments(
        OsStr::new("bootstrap.exe"),
        vec![OsString::from("--module=other")]
    )
    .is_err());
}

#[test]
fn bootstrap_role_accepts_only_a_paired_cef_type_and_marker() {
    let parent = classify_bootstrap(vec![OsString::from("beaver.exe")]);
    assert!(matches!(parent, Ok(BootstrapRole::Parent)));

    let helper = classify_bootstrap(vec![
        OsString::from("beaver.exe"),
        OsString::from("--type=renderer"),
        OsString::from("--beaver-cef-admission=secret"),
    ]);
    let Ok(BootstrapRole::CefHelper(marker)) = helper else {
        panic!("valid helper role expected");
    };
    assert_eq!(marker.as_str(), "secret");
}

#[test]
fn bootstrap_role_rejects_unsupervised_or_ambiguous_helpers() {
    assert!(classify_bootstrap(vec![
        OsString::from("beaver.exe"),
        OsString::from("--type=renderer")
    ])
    .is_err());
    assert!(classify_bootstrap(vec![
        OsString::from("beaver.exe"),
        OsString::from("--beaver-cef-admission=secret")
    ])
    .is_err());
    assert!(classify_bootstrap(vec![
        OsString::from("beaver.exe"),
        OsString::from("--type=renderer"),
        OsString::from("--type=gpu-process"),
        OsString::from("--beaver-cef-admission=secret"),
    ])
    .is_err());
    assert!(classify_bootstrap(vec![
        OsString::from("beaver.exe"),
        OsString::from("--type=renderer"),
        OsString::from("--beaver-cef-admission=first"),
        OsString::from("--beaver-cef-admission=second"),
    ])
    .is_err());
}

#[test]
fn shell_sandbox_process_is_never_classified_as_a_cef_helper() {
    assert!(classify_bootstrap(vec![
        OsString::from("beaver.exe"),
        OsString::from("--beaver-shell-sandbox"),
        OsString::from("--type=renderer"),
        OsString::from("--beaver-cef-admission=secret"),
    ])
    .is_err());
}

#[test]
fn shell_sandbox_process_has_its_own_role() {
    let role = classify_bootstrap(vec![
        OsString::from("beaver.exe"),
        OsString::from("--beaver-shell-sandbox"),
        OsString::from(r"C:\temporary"),
        OsString::from("--"),
        OsString::from("cmd.exe"),
    ]);

    assert!(matches!(role, Ok(BootstrapRole::ShellSandbox)));
}

#[test]
fn private_switches_are_classified_case_insensitively() {
    let helper = classify_bootstrap(vec![
        OsString::from("beaver.exe"),
        OsString::from("--TYPE=renderer"),
        OsString::from("--BEAVER-CEF-ADMISSION=secret"),
    ]);
    assert!(matches!(helper, Ok(BootstrapRole::CefHelper(_))));
}

#[test]
fn bootstrap_role_only_bounds_private_beaver_switches() {
    let mut helper = (0..70)
        .map(|index| OsString::from(format!("--chromium-option-{index}")))
        .collect::<Vec<_>>();
    helper.insert(0, OsString::from("beaver.exe"));
    helper.push(OsString::from("x".repeat(MAX_PRIVATE_ARG_UTF16 + 1)));
    helper.push(OsString::from_wide(&[0xD800]));
    helper.push(OsString::from("--type=renderer"));
    helper.push(OsString::from("--beaver-cef-admission=secret"));

    assert!(matches!(
        classify_bootstrap(helper),
        Ok(BootstrapRole::CefHelper(_))
    ));
    assert!(classify_bootstrap(vec![
        OsString::from("beaver.exe"),
        OsString::from(format!(
            "--beaver-cef-admission={}",
            "x".repeat(MAX_PRIVATE_ARG_UTF16 + 1)
        ))
    ])
    .is_err());
}

#[test]
fn shell_payload_is_not_reinterpreted_as_cef_metadata() {
    let role = classify_bootstrap(vec![
        OsString::from("beaver.exe"),
        OsString::from("--beaver-shell-sandbox"),
        OsString::from(r"C:\temporary"),
        OsString::from("--"),
        OsString::from("command.exe"),
        OsString::from("--type=renderer"),
        OsString::from("--beaver-cef-admission=ordinary-command-value"),
    ]);

    assert!(matches!(role, Ok(BootstrapRole::ShellSandbox)));
}

#[test]
fn development_forwarding_uses_the_create_process_limit() {
    let many_arguments = (0..70)
        .map(|index| OsString::from(format!("--safe-{index}")))
        .collect::<Vec<_>>();
    assert!(bootstrap_arguments(OsStr::new("bootstrap.exe"), many_arguments).is_ok());

    let fixed_units =
        encoded_argument_len(OsStr::new("bootstrap.exe"), true).expect("executable length") + 2;
    let exact = vec![OsString::from(
        "x".repeat(CREATE_PROCESS_UTF16_LIMIT - fixed_units),
    )];
    assert!(bootstrap_arguments(OsStr::new("bootstrap.exe"), exact).is_ok());

    let oversized = vec![OsString::from(
        "x".repeat(CREATE_PROCESS_UTF16_LIMIT - fixed_units + 1),
    )];
    assert!(bootstrap_arguments(OsStr::new("bootstrap.exe"), oversized).is_err());
}

#[test]
fn encoded_argument_length_accounts_for_windows_escaping() {
    assert_eq!(encoded_argument_len(OsStr::new("plain"), false), Ok(5));
    assert_eq!(encoded_argument_len(OsStr::new("two words"), false), Ok(11));
    assert_eq!(encoded_argument_len(OsStr::new(r#"a\"b"#), false), Ok(6));
    assert_eq!(encoded_argument_len(OsStr::new(r"tail\"), true), Ok(8));
}
