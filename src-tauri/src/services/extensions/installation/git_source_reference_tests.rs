#[test]
fn hexadecimal_branches_are_not_assumed_to_be_full_commits() {
    assert!(!super::git_source::looks_like_full_commit("20260728"));
    assert!(!super::git_source::looks_like_full_commit("abcdef1"));
    assert!(super::git_source::looks_like_short_commit("abcdef1"));
    assert!(super::git_source::looks_like_full_commit(&"a".repeat(40)));
    assert!(super::git_source::looks_like_full_commit(&"b".repeat(64)));
}
