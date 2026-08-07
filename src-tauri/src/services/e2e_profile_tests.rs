use super::e2e_profile::{external_home_dir, load_dotenv, run_host_mutation};
use std::cell::Cell;

#[test]
fn e2e_profile_skips_dotenv_and_host_mutations() {
    let dotenv_called = Cell::new(false);
    let mutation_called = Cell::new(false);

    load_dotenv(|| dotenv_called.set(true));
    run_host_mutation(|| mutation_called.set(true));

    assert!(!dotenv_called.get());
    assert!(!mutation_called.get());
}

#[test]
fn e2e_external_home_is_confined_to_the_test_profile() {
    let profile = crate::services::paths::data_dir();
    let home = external_home_dir().expect("isolated home");

    assert!(home.starts_with(profile));
    assert_eq!(
        home.file_name().and_then(|name| name.to_str()),
        Some("e2e-home")
    );
}
