pub(super) fn safe_csv_cell(value: &str) -> String {
    if value
        .chars()
        .next()
        .is_some_and(|character| matches!(character, '=' | '+' | '-' | '@' | '\t' | '\r'))
    {
        format!("'{value}")
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefixes_formula_like_cells() {
        for value in [
            "=HYPERLINK(\"https://example.invalid\")",
            "+1+1",
            "-2+3",
            "@SUM(A1:A2)",
            "\tformula",
            "\rformula",
        ] {
            assert!(safe_csv_cell(value).starts_with('\''));
        }
    }

    #[test]
    fn keeps_regular_text_unchanged() {
        assert_eq!(safe_csv_cell("Prévision normale"), "Prévision normale");
        assert_eq!(safe_csv_cell("2026-07-23"), "2026-07-23");
    }
}
