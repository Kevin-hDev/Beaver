use super::access_is_narrower;
use std::path::PathBuf;

#[test]
fn adding_a_root_does_not_interrupt_running_work() {
    let old = vec![PathBuf::from("/work/a")];
    let new = vec![PathBuf::from("/work/a"), PathBuf::from("/work/b")];

    assert!(!access_is_narrower(&old, &new));
}

#[test]
fn replacing_a_parent_with_a_child_is_a_real_restriction() {
    let old = vec![PathBuf::from("/work")];
    let new = vec![PathBuf::from("/work/a")];

    assert!(access_is_narrower(&old, &new));
}

#[test]
fn replacing_children_with_their_parent_is_an_expansion() {
    let old = vec![PathBuf::from("/work/a"), PathBuf::from("/work/b")];
    let new = vec![PathBuf::from("/work")];

    assert!(!access_is_narrower(&old, &new));
}
