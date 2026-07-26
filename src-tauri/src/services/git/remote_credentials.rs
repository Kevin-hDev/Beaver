use git2::{Config, Cred, CredentialType};
use zeroize::Zeroizing;

use super::repo;

pub(super) struct CredentialProvider {
    config: Config,
    github_token: Option<Zeroizing<String>>,
    helper_attempted: bool,
    token_attempted: bool,
    ssh_attempted: bool,
    default_attempted: bool,
}

impl CredentialProvider {
    pub(super) fn new(config: Config, github_token: Option<Zeroizing<String>>) -> Self {
        Self {
            config,
            github_token,
            helper_attempted: false,
            token_attempted: false,
            ssh_attempted: false,
            default_attempted: false,
        }
    }

    pub(super) fn credentials(
        &mut self,
        url: &str,
        username: Option<&str>,
        allowed: CredentialType,
    ) -> Result<Cred, git2::Error> {
        if allowed.contains(CredentialType::USER_PASS_PLAINTEXT) {
            if !self.helper_attempted {
                self.helper_attempted = true;
                if let Ok(credential) = Cred::credential_helper(&self.config, url, username) {
                    return Ok(credential);
                }
            }
            if repo::is_github_url(url) && !self.token_attempted {
                self.token_attempted = true;
                if let Some(token) = self.github_token.as_ref() {
                    return Cred::userpass_plaintext("x-access-token", token.as_str());
                }
            }
        }
        if allowed.contains(CredentialType::SSH_KEY) && !self.ssh_attempted {
            self.ssh_attempted = true;
            return Cred::ssh_key_from_agent(username.unwrap_or("git"));
        }
        if allowed.contains(CredentialType::USERNAME) {
            return Cred::username(username.unwrap_or("git"));
        }
        if allowed.contains(CredentialType::DEFAULT) && !self.default_attempted {
            self.default_attempted = true;
            return Cred::default();
        }
        Err(git2::Error::from_str("no supported credential available"))
    }
}

#[cfg(test)]
mod tests {
    use super::CredentialProvider;
    use git2::{Config, CredentialType};
    use zeroize::Zeroizing;

    #[test]
    fn github_token_is_a_bounded_fallback_after_the_credential_helper() {
        let config = Config::new().expect("config");
        let mut provider =
            CredentialProvider::new(config, Some(Zeroizing::new("test-token".to_string())));
        provider.helper_attempted = true;

        let credential = provider.credentials(
            "https://github.com/example/project.git",
            Some("git"),
            CredentialType::USER_PASS_PLAINTEXT,
        );

        assert!(credential.is_ok());
        assert!(provider.token_attempted);
    }

    #[test]
    fn missing_credentials_fail_closed() {
        let config = Config::new().expect("config");
        let mut provider = CredentialProvider::new(config, None);
        provider.helper_attempted = true;

        let credential = provider.credentials(
            "https://github.com/example/project.git",
            None,
            CredentialType::USER_PASS_PLAINTEXT,
        );

        assert!(credential.is_err());
    }
}
