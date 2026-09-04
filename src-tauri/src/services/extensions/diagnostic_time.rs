pub(super) fn now() -> String {
    chrono::Utc::now().to_rfc3339()
}
