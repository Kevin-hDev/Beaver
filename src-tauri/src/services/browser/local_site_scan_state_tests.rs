use super::{
    local_site_scan_state::LocalSiteScanState,
    local_site_types::{LocalSite, LocalSiteProtocol, MAX_LOCAL_RESULTS},
};

#[test]
fn publishes_only_changes_and_removes_after_two_failures() {
    let mut state = LocalSiteScanState::default();
    let first = state.merge(vec![site(3_000)]).unwrap();
    assert!(first.changed);
    assert_eq!(first.generation, 1);

    let unchanged = state.merge(vec![site(3_000)]).unwrap();
    assert!(!unchanged.changed);
    assert_eq!(unchanged.generation, 1);

    let first_failure = state.merge(Vec::new()).unwrap();
    assert!(!first_failure.changed);
    assert_eq!(first_failure.sites.len(), 1);

    let removed = state.merge(Vec::new()).unwrap();
    assert!(removed.changed);
    assert!(removed.sites.is_empty());
    assert_eq!(removed.generation, 2);
}

#[test]
fn sorts_results_and_rejects_unbounded_or_duplicate_input() {
    let mut state = LocalSiteScanState::default();
    let result = state
        .merge(vec![site(4_000), site(2_000), site(3_000)])
        .unwrap();
    assert_eq!(
        result
            .sites
            .iter()
            .map(|site| site.port)
            .collect::<Vec<_>>(),
        vec![2_000, 3_000, 4_000]
    );
    assert!(state.merge(vec![site(5_000), site(5_000)]).is_err());
    let too_many = (0..=MAX_LOCAL_RESULTS)
        .map(|index| site(20_000 + index as u16))
        .collect();
    assert!(state.merge(too_many).is_err());
}

fn site(port: u16) -> LocalSite {
    LocalSite {
        url: format!("http://localhost:{port}/"),
        title: format!("Site {port}"),
        port,
        protocol: LocalSiteProtocol::Http,
    }
}
