use super::*;
use std::os::unix::ffi::OsStringExt;

#[test]
fn opaque_chromium_arguments_do_not_hide_the_private_marker() {
    let mut arguments = (0..70)
        .map(|index| OsString::from(format!("--chromium-option-{index}")))
        .collect::<Vec<_>>();
    arguments.insert(0, OsString::from("cl-go-dash-helper"));
    arguments.push(OsString::from_vec(vec![0xff]));
    arguments.push(OsString::from("--type=renderer"));
    arguments.push(OsString::from("--beaver-cef-admission=secret"));

    let marker = parse_helper_marker_from(arguments).expect("private marker");

    assert_eq!(marker.as_str(), "secret");
}

#[test]
fn private_arguments_remain_bounded_and_unambiguous() {
    assert!(parse_helper_marker_from(vec![
        OsString::from("cl-go-dash-helper"),
        OsString::from("--type=renderer"),
        OsString::from(format!(
            "--beaver-cef-admission={}",
            "x".repeat(MAX_PRIVATE_ARGUMENT_BYTES + 1)
        )),
    ])
    .is_err());
    assert!(parse_helper_marker_from(vec![
        OsString::from("cl-go-dash-helper"),
        OsString::from("--type=renderer"),
        OsString::from("--beaver-cef-admission=first"),
        OsString::from("--beaver-cef-admission=second"),
    ])
    .is_err());
}
