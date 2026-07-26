use super::types::{ProviderBalance, BALANCE_LIMIT};

const FIXED_POINT_CENTS: u64 = 1_000_000;

pub fn parse(body: &serde_json::Value) -> Vec<ProviderBalance> {
    let Some(wallet) = body
        .get("boosterWallet")
        .and_then(serde_json::Value::as_object)
    else {
        return Vec::new();
    };
    let Some(balance) = wallet.get("balance").and_then(serde_json::Value::as_object) else {
        return Vec::new();
    };
    if balance.get("type").and_then(serde_json::Value::as_str) != Some("BOOSTER") {
        return Vec::new();
    }
    let Some(total) = balance.get("amount").and_then(unsigned_integer) else {
        return Vec::new();
    };
    if total == 0 {
        return Vec::new();
    }

    let currency = wallet_currency(wallet);
    let mut balances = Vec::new();
    if let Some(left) = balance.get("amountLeft").and_then(unsigned_integer) {
        push_balance(
            &mut balances,
            "extra_usage_balance",
            fixed_point_decimal(left),
            &currency,
        );
    }
    if wallet
        .get("monthlyChargeLimitEnabled")
        .and_then(serde_json::Value::as_bool)
        == Some(true)
    {
        if let Some(cents) = money_cents(wallet.get("monthlyChargeLimit")) {
            push_balance(
                &mut balances,
                "extra_usage_monthly_limit",
                cents_decimal(cents),
                &currency,
            );
        }
    }
    if let Some(cents) = money_cents(wallet.get("monthlyUsed")) {
        push_balance(
            &mut balances,
            "extra_usage_monthly_used",
            cents_decimal(cents),
            &currency,
        );
    }
    balances
}

fn wallet_currency(wallet: &serde_json::Map<String, serde_json::Value>) -> String {
    ["monthlyChargeLimit", "monthlyUsed"]
        .iter()
        .find_map(|key| {
            wallet.get(*key)?.get("currency")?.as_str().filter(|value| {
                value.len() == 3 && value.bytes().all(|byte| byte.is_ascii_uppercase())
            })
        })
        .unwrap_or("USD")
        .to_string()
}

fn money_cents(value: Option<&serde_json::Value>) -> Option<u64> {
    value?.get("priceInCents").and_then(unsigned_integer)
}

fn unsigned_integer(value: &serde_json::Value) -> Option<u64> {
    value.as_u64().or_else(|| {
        let raw = value.as_str()?;
        if raw.is_empty() || raw.len() > 20 || !raw.bytes().all(|byte| byte.is_ascii_digit()) {
            return None;
        }
        raw.parse::<u64>().ok()
    })
}

fn fixed_point_decimal(value: u64) -> String {
    let cents = value / FIXED_POINT_CENTS;
    cents_decimal(if value > 0 && cents == 0 { 1 } else { cents })
}

fn cents_decimal(value: u64) -> String {
    format!("{}.{:02}", value / 100, value % 100)
}

fn push_balance(
    balances: &mut Vec<ProviderBalance>,
    label_code: &str,
    amount: String,
    currency: &str,
) {
    if balances.len() >= BALANCE_LIMIT {
        return;
    }
    balances.push(ProviderBalance {
        label_code: label_code.to_string(),
        amount,
        currency: currency.to_string(),
    });
}
