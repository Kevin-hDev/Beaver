use super::{
    session_order::ordered_tabs,
    session_types::{blank_tab, BrowserTabState, MAX_BROWSER_TABS},
};

fn id(index: usize) -> String {
    format!("{index:032x}")
}

fn tabs(count: usize) -> Vec<BrowserTabState> {
    (1..=count).map(|index| blank_tab(id(index))).collect()
}

#[test]
fn builds_the_requested_permutation_without_changing_tabs() {
    let original = tabs(3);
    let result = ordered_tabs(&original, &[id(3), id(1), id(2)]).unwrap();

    assert_eq!(
        result,
        vec![
            original[2].clone(),
            original[0].clone(),
            original[1].clone()
        ]
    );
}

#[test]
fn accepts_an_identical_order_for_one_and_ten_tabs() {
    for count in [1, MAX_BROWSER_TABS] {
        let original = tabs(count);
        let ordered_ids = original
            .iter()
            .map(|tab| tab.id.clone())
            .collect::<Vec<_>>();

        assert_eq!(ordered_tabs(&original, &ordered_ids), Ok(original));
    }
}

#[test]
fn rejects_an_empty_or_oversized_order() {
    assert!(ordered_tabs(&[], &[]).is_err());

    let original = tabs(MAX_BROWSER_TABS);
    let oversized = (1..=MAX_BROWSER_TABS + 1).map(id).collect::<Vec<_>>();
    assert!(ordered_tabs(&original, &oversized).is_err());
}

#[test]
fn rejects_duplicate_and_omitted_identifiers() {
    let original = tabs(2);

    assert!(ordered_tabs(&original, &[id(1), id(1)]).is_err());
    assert!(ordered_tabs(&original, &[id(1)]).is_err());
}

#[test]
fn rejects_unknown_or_invalid_identifiers() {
    let original = tabs(2);

    assert!(ordered_tabs(&original, &[id(1), id(3)]).is_err());
    assert!(ordered_tabs(&original, &[id(1), "invalid".into()]).is_err());
}
