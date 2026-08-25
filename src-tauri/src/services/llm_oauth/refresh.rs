use super::{kimi, lifecycle, store, xai, AccessToken, LlmOAuthProvider, OAuthFailure};

pub async fn access_token(provider: LlmOAuthProvider) -> Result<AccessToken, String> {
    let tokens = store::load(provider)?.ok_or_else(not_connected)?;
    let generation = store::generation(provider);
    if tokens.is_fresh() {
        if provider == LlmOAuthProvider::Xai && tokens.user_id.is_none() {
            return enrich_existing_xai(generation).await;
        }
        return Ok(AccessToken {
            value: tokens.access,
            generation,
            user_id: tokens.user_id,
        });
    }
    refresh_locked(provider, generation).await
}

async fn enrich_existing_xai(expected_generation: u64) -> Result<AccessToken, String> {
    let provider = LlmOAuthProvider::Xai;
    let _guard = lifecycle::lock(provider).await;
    let mut tokens = store::load(provider)?.ok_or_else(not_connected)?;
    let current_generation = store::generation(provider);
    if current_generation != expected_generation && tokens.user_id.is_some() {
        return Ok(AccessToken {
            value: tokens.access,
            generation: current_generation,
            user_id: tokens.user_id,
        });
    }
    // Login antérieur à l'identité OAuth : sans principal sauvegardé à comparer,
    // la première réponse /user authentifiée devient l'autorité de migration.
    super::xai_identity::enrich(&mut tokens, None)
        .await
        .map_err(|_| "Connexion impossible".to_string())?;
    let generation = store::save_if_generation(provider, &tokens, current_generation)?;
    Ok(AccessToken {
        value: tokens.access,
        generation,
        user_id: tokens.user_id,
    })
}

pub async fn force_refresh(
    provider: LlmOAuthProvider,
    used_generation: u64,
) -> Result<AccessToken, String> {
    refresh_locked(provider, used_generation).await
}

async fn refresh_locked(
    provider: LlmOAuthProvider,
    expected_generation: u64,
) -> Result<AccessToken, String> {
    let _guard = lifecycle::lock(provider).await;
    let current = store::load(provider)?.ok_or_else(not_connected)?;
    let current_generation = store::generation(provider);
    if current_generation != expected_generation && current.is_fresh() {
        return Ok(AccessToken {
            value: current.access,
            generation: current_generation,
            user_id: current.user_id,
        });
    }
    let refreshed = match provider {
        LlmOAuthProvider::Xai => xai::refresh(&current.refresh).await,
        LlmOAuthProvider::Kimi => kimi::refresh(&current.refresh).await,
    };
    match refreshed {
        Ok(mut tokens) => {
            tokens.preserve_credential_scope_from(&current);
            if provider == LlmOAuthProvider::Xai {
                let previous = current.user_id.as_ref().map(|value| value.as_str());
                if let Err(error) = super::xai_identity::enrich(&mut tokens, previous).await {
                    if error == OAuthFailure::Unauthorized {
                        let _ = store::clear(provider);
                        return Err(not_connected());
                    }
                    return Err("Renouvellement impossible".to_string());
                }
            }
            let generation = store::save_if_generation(provider, &tokens, current_generation)?;
            Ok(AccessToken {
                value: tokens.access,
                generation,
                user_id: tokens.user_id,
            })
        }
        Err(OAuthFailure::Unauthorized) => {
            let _ = store::clear(provider);
            Err(not_connected())
        }
        Err(_) => Err("Renouvellement impossible".to_string()),
    }
}

fn not_connected() -> String {
    "Connexion requise".to_string()
}
