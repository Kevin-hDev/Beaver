//! Tests de build_future_dates (PURE) : génération de dates futures selon la
//! fréquence. Cœur métier du forecast — beaucoup d'edge cases de parsing.

use super::build_future_dates;

// --- Fréquences simples -----------------------------------------------------

#[test]
fn daily_frequency_advances_by_days() {
    let dates = build_future_dates("2026-01-01", "D", 3);
    assert_eq!(dates, vec!["2026-01-02", "2026-01-03", "2026-01-04"]);
}

#[test]
fn business_frequency_skips_weekends() {
    let dates = build_future_dates("2026-07-17", "B", 3);
    assert_eq!(dates, vec!["2026-07-20", "2026-07-21", "2026-07-22"]);
}

#[test]
fn weekly_frequency_advances_by_weeks() {
    let dates = build_future_dates("2026-01-01", "W", 2);
    assert_eq!(dates, vec!["2026-01-08", "2026-01-15"]);
}

#[test]
fn monthly_frequency_advances_by_months() {
    let dates = build_future_dates("2026-01-31", "M", 2);
    // 31 jan + 1 mois = 28 fév (fév clampé), + 1 mois = 31 mars (chrono
    // retient le jour d'origine 31 quand le mois le permet).
    assert_eq!(dates, vec!["2026-02-28", "2026-03-31"]);
}

#[test]
fn monthly_frequency_from_first_of_month() {
    // Cas plus simple : du 1er, pas de clamping.
    let dates = build_future_dates("2026-01-01", "M", 3);
    assert_eq!(dates, vec!["2026-02-01", "2026-03-01", "2026-04-01"]);
}

#[test]
fn quarterly_frequency_advances_by_3_months() {
    let dates = build_future_dates("2026-01-01", "Q", 2);
    assert_eq!(dates, vec!["2026-04-01", "2026-07-01"]);
}

#[test]
fn yearly_frequency_advances_by_year() {
    let dates = build_future_dates("2026-01-01", "Y", 2);
    assert_eq!(dates, vec!["2027-01-01", "2028-01-01"]);
}

#[test]
fn hourly_frequency_advances_by_hours() {
    let dates = build_future_dates("2026-01-01T00:00:00", "H", 3);
    assert_eq!(
        dates,
        vec![
            "2026-01-01T01:00:00",
            "2026-01-01T02:00:00",
            "2026-01-01T03:00:00"
        ]
    );
}

// --- Fréquences composées (ex: 15T, 2W) ------------------------------------

#[test]
fn compound_minutes_frequency() {
    // 15T = 15 minutes par step.
    let dates = build_future_dates("2026-01-01T00:00:00", "15T", 2);
    assert_eq!(dates, vec!["2026-01-01T00:15:00", "2026-01-01T00:30:00"]);
}

#[test]
fn compound_weeks_frequency() {
    // 2W = 2 semaines par step.
    let dates = build_future_dates("2026-01-01", "2W", 2);
    assert_eq!(dates, vec!["2026-01-15", "2026-01-29"]);
}

#[test]
fn compound_days_frequency() {
    // 3D = 3 jours par step.
    let dates = build_future_dates("2026-01-01", "3D", 2);
    assert_eq!(dates, vec!["2026-01-04", "2026-01-07"]);
}

// --- Cas limites ------------------------------------------------------------

#[test]
fn horizon_zero_returns_empty() {
    let dates = build_future_dates("2026-01-01", "D", 0);
    assert!(dates.is_empty());
}

#[test]
fn lowercase_frequency_normalized() {
    // La fréquence est normalisée en majuscule en interne.
    let dates = build_future_dates("2026-01-01", "d", 2);
    assert_eq!(dates, vec!["2026-01-02", "2026-01-03"]);
}

#[test]
fn frequency_with_whitespace_trimmed() {
    let dates = build_future_dates("2026-01-01", "  D  ", 2);
    assert_eq!(dates, vec!["2026-01-02", "2026-01-03"]);
}

// --- Dates non parsables → fallback T+index --------------------------------

#[test]
fn invalid_date_returns_t_plus_index_fallback() {
    let dates = build_future_dates("not-a-date", "D", 3);
    assert_eq!(dates, vec!["T+1", "T+2", "T+3"]);
}

#[test]
fn unknown_unit_in_compound_falls_back_to_t_plus() {
    // "5X" : unit inconnue → fallback T+index.
    let dates = build_future_dates("2026-01-01", "5X", 2);
    assert_eq!(dates, vec!["T+1", "T+2"]);
}

// --- Format de sortie suit le format d'entrée ------------------------------

#[test]
fn output_format_matches_input_datetime_format() {
    // Entrée sans 'T' ni ':' → sortie date seule (YYYY-MM-DD).
    let dates = build_future_dates("2026-01-01", "D", 1);
    assert_eq!(dates[0], "2026-01-02");
    assert!(!dates[0].contains('T'));
    assert!(!dates[0].contains(':'));

    // Entrée avec 'T' → sortie au format ISO T.
    let dates = build_future_dates("2026-01-01T10:00:00", "D", 1);
    assert!(dates[0].contains('T'));
}

#[test]
fn slash_date_format_supported() {
    let dates = build_future_dates("2026/01/01", "D", 2);
    // Le parsing accepte YYYY/MM/DD, la sortie est normalisée en YYYY-MM-DD.
    assert_eq!(dates.len(), 2);
    assert!(dates[0].starts_with("2026-01-02"));
}
