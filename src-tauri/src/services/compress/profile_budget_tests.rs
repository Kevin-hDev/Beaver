use super::profile_budget::{resolve_budget, summary_output_limit};
use super::profile_defaults::beaver_profile;
use super::profile_types::{BudgetMode, TokenBudget};

#[test]
fn token_budget_modes_are_integer_only() {
    let fixed = TokenBudget::fixed(4_096);
    let percentage = TokenBudget::percentage(750);
    let minimum = TokenBudget::minimum(5_000, 1_500);

    assert_eq!(resolve_budget(&fixed, 64_000), 4_096);
    assert_eq!(resolve_budget(&percentage, 64_000), 4_800);
    assert_eq!(resolve_budget(&minimum, 64_000), 5_000);
    assert_eq!(minimum.mode, BudgetMode::Minimum);
}

#[test]
fn evidence_envelopes_match_the_three_policies() {
    let profile = beaver_profile();
    assert_eq!(
        resolve_budget(&profile.under_64k.evidence_envelope, 32_000),
        2_000
    );
    assert_eq!(
        resolve_budget(&profile.under_64k.evidence_envelope, 500_000),
        10_000
    );
    assert_eq!(
        resolve_budget(&profile.compact.evidence_envelope, 64_000),
        4_000
    );
    assert_eq!(
        resolve_budget(&profile.large.evidence_envelope, 400_000),
        20_000
    );
}

#[test]
fn summary_output_preserves_existing_formula_and_halves_tiny_windows() {
    let profile = beaver_profile();
    assert_eq!(
        summary_output_limit(&profile.compact.summary_output, 128_000, 3_000),
        1_000
    );
    assert_eq!(
        summary_output_limit(&profile.large.summary_output, 400_000, 60_000),
        16_000
    );
    assert_eq!(
        summary_output_limit(&profile.under_64k.summary_output, 32_000, 3_000),
        500
    );
    assert_eq!(
        summary_output_limit(&profile.under_64k.summary_output, 63_999, 60_000),
        4_799
    );
}
