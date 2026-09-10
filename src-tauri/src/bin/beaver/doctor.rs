use crate::output::Out;
use std::path::Path;

#[path = "doctor_checks.rs"]
mod checks;

pub struct CheckResult {
    pub label_fr: String,
    pub label_en: String,
    pub passed: bool,
    pub advice_fr: String,
    pub advice_en: String,
}

fn result(fr: &str, en: &str, passed: bool, advice_fr: &str, advice_en: &str) -> CheckResult {
    CheckResult {
        label_fr: fr.to_string(),
        label_en: en.to_string(),
        passed,
        advice_fr: advice_fr.to_string(),
        advice_en: advice_en.to_string(),
    }
}

pub fn run_checks(root: &Path) -> Vec<CheckResult> {
    checks::run_checks(root)
}

pub fn run(out: &Out) -> i32 {
    let checks = run_checks(&cl_go_dash_lib::cli_support::data_dir());
    for check in &checks {
        let label = out.t(&check.label_fr, &check.label_en);
        if check.passed {
            out.ok(label);
        } else {
            out.fail(label, out.t(&check.advice_fr, &check.advice_en));
        }
    }
    i32::from(checks.iter().any(|check| !check.passed))
}

#[cfg(test)]
#[path = "doctor_tests.rs"]
mod tests;
