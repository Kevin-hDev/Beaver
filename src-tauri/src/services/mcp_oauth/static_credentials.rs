use zeroize::Zeroizing;

use crate::services::api_keys;

pub struct StaticCredentials {
    pub client_id: Zeroizing<String>,
    pub client_secret: Zeroizing<String>,
    pub scopes: &'static str,
}

const VAULT_GOOGLE_ID: &str = "_oauth_google_client_id";
const VAULT_GOOGLE_SECRET: &str = "_oauth_google_client_secret";
const VAULT_GITHUB_ID: &str = "_oauth_github_client_id";
const VAULT_GITHUB_SECRET: &str = "_oauth_github_client_secret";

pub fn for_endpoint(endpoint: &str) -> Option<StaticCredentials> {
    let host = reqwest::Url::parse(endpoint).ok()?.host_str()?.to_string();

    if host.ends_with(".googleapis.com") {
        let scopes = google_scopes_for_host(&host);
        let client_id = load_credential(VAULT_GOOGLE_ID, "CLGO_GOOGLE_CLIENT_ID")?;
        let client_secret = load_credential(VAULT_GOOGLE_SECRET, "CLGO_GOOGLE_CLIENT_SECRET")?;
        return Some(StaticCredentials {
            client_id,
            client_secret,
            scopes,
        });
    }

    if host == "api.githubcopilot.com" || host.ends_with(".githubcopilot.com") {
        let client_id = load_credential(VAULT_GITHUB_ID, "CLGO_GITHUB_CLIENT_ID")?;
        let client_secret = load_credential(VAULT_GITHUB_SECRET, "CLGO_GITHUB_CLIENT_SECRET")?;
        return Some(StaticCredentials {
            client_id,
            client_secret,
            scopes: "repo read:user",
        });
    }

    None
}

fn load_credential(vault_key: &str, env_var: &str) -> Option<Zeroizing<String>> {
    if let Ok(val) = api_keys::get_raw(vault_key) {
        return Some(val);
    }
    if let Ok(val) = std::env::var(env_var) {
        if !val.is_empty() {
            let _ = api_keys::set_raw(vault_key, &val);
            return Some(Zeroizing::new(val));
        }
    }
    None
}

fn google_scopes_for_host(host: &str) -> &'static str {
    if host.starts_with("gmail") {
        return "https://www.googleapis.com/auth/gmail.readonly https://www.googleapis.com/auth/gmail.compose";
    }
    if host.starts_with("drive") {
        return "https://www.googleapis.com/auth/drive.readonly https://www.googleapis.com/auth/drive.file";
    }
    if host.starts_with("calendar") {
        return "https://www.googleapis.com/auth/calendar.calendarlist.readonly https://www.googleapis.com/auth/calendar.events.freebusy https://www.googleapis.com/auth/calendar.events.readonly";
    }
    "https://www.googleapis.com/auth/userinfo.email"
}
