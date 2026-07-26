use super::remote_oauth;
use serde_json::json;

#[test]
fn absent_kimi_usage_is_not_saved_as_a_success() {
    assert!(remote_oauth::parse("moonshot-oauth", &json!({})).is_none());
}

#[test]
fn kimi_accepts_numeric_strings_and_reset_variants() {
    let reset_at = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(3))
        .unwrap()
        .to_rfc3339();
    let parsed = remote_oauth::parse(
        "moonshot-oauth",
        &json!({
            "usage": {
                "limit": "1000",
                "remaining": "960",
                "resetTime": reset_at
            },
            "limits": [{
                "window": {"duration": "300", "timeUnit": "MINUTE"},
                "detail": {"limit": "100", "used": "4", "reset_in": "600"}
            }]
        }),
    )
    .unwrap();

    assert_eq!(parsed.windows.len(), 2);
    assert_eq!(parsed.windows[0].used, Some(40.0));
    assert_eq!(parsed.windows[0].remaining, Some(960.0));
    assert!(parsed.windows[0].resets_at.is_some());
    assert_eq!(parsed.windows[1].label_code, "rolling_five_hours");
    assert_eq!(parsed.windows[1].used_percent, Some(4.0));
    assert!(parsed.windows[1].resets_at.is_some());
}

#[test]
fn kimi_parses_extra_usage_wallet_without_rounding_away_cents() {
    let parsed = remote_oauth::parse(
        "moonshot-oauth",
        &json!({
            "usage": {"limit": "100", "used": "4"},
            "boosterWallet": {
                "balance": {
                    "type": "BOOSTER",
                    "amount": "20000000000",
                    "amountLeft": "10000000000"
                },
                "monthlyChargeLimitEnabled": true,
                "monthlyChargeLimit": {"currency": "USD", "priceInCents": "20000"},
                "monthlyUsed": {"currency": "USD", "priceInCents": "5001"}
            }
        }),
    )
    .unwrap();

    assert_eq!(parsed.balances.len(), 3);
    assert_eq!(parsed.balances[0].label_code, "extra_usage_balance");
    assert_eq!(parsed.balances[0].amount, "100.00");
    assert_eq!(parsed.balances[1].amount, "200.00");
    assert_eq!(parsed.balances[2].amount, "50.01");
}

#[test]
fn kimi_rejects_unbounded_or_invalid_numeric_strings() {
    let oversized = "9".repeat(64);
    let parsed = remote_oauth::parse(
        "moonshot-oauth",
        &json!({"usage": {"limit": oversized, "used": "not-a-number"}}),
    );
    assert!(parsed.is_none());
}
