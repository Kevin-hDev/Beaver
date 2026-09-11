# Rapports de sous-agent et continuité de raisonnement — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` (recommended) or `superpowers:executing-plans` to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Empêcher l'arrêt du parent lorsqu'un rapport de sous-agent revient pendant une conversation avec continuité de raisonnement, sur tous les fournisseurs actifs, sans affaiblir la validation des vraies réponses du modèle ni modifier le comportement de compression.

**Architecture:** `subagent_report_context::report_batch_to_message` reste l'unique autorité du rôle des rapports injectés et les produit comme entrées `user`. Le registre `reasoning_continuity::active_routes` reste l'autorité de la couverture fournisseur : un test unique le parcourt et exerce la famille de payload correspondant à chaque politique active. Les validateurs Responses, Chat et Anthropic restent stricts pour tout vrai message `assistant` sans enveloppe. Le budget de contexte continue de protéger les rapports par leur préfixe, indépendamment de leur rôle.

**Tech Stack:** Rust, Tokio, Serde JSON et tests Rust existants ; aucune dépendance, aucun appel provider réel et aucune modification frontend.

**Spec:** [Investigation de l'interruption au retour d'un sous-agent](../../bug/sous-agent/2026-09-11-interruption-parent-au-retour-sous-agent-continuite-raisonnement.md).

État historique de l'enquête : **11 septembre 2026, branche `codex/fix-agent-stream-stops` à `e457053f`**.

État d'implémentation : **base initiale `d7b27545`, base actuelle après rebase `a5060ddf`, branche `codex/fix-subagent-report-continuity`**. Les tâches 1 à 4 sont appliquées dans `6f928742`, `3913232b`, `96bf7a16` et `f070d49f`. La tâche 5 consigne séparément le contrôle global parallèle resté rouge et la validation complète en série réussie.

## Global Constraints

- Vérifier `git status --short --branch` avant l'implémentation. Conserver le document d'investigation déjà indexé et tout travail sans rapport ; ne jamais utiliser `git add .`.
- Corriger le rôle une seule fois dans `subagent_report_context::report_batch_to_message`. Motif : les boucles cloud et Ollama passent déjà par cette autorité commune.
- Ne fabriquer aucune enveloppe de raisonnement pour un rapport, ne reconnaître aucun rapport par son texte dans les validateurs fournisseur et ne créer aucune exception propre à Codex, Kimi, Grok, Mistral ou un autre fournisseur.
- Ne relâcher aucune validation de continuité. Un vrai message `assistant` sans enveloppe doit toujours fermer les routes dont la politique est `Required`.
- Parcourir `reasoning_continuity::active_routes()` dans le test de matrice. Ne pas recopier une liste de fournisseurs ou de modèles qui divergerait du registre.
- Tester seulement les politiques `LiveValidated` avec `ContinuationUse::UserContinuation`, car l'injection du rapport `user` devient le dernier message et l'appel sortant est alors une continuation utilisateur. Les politiques outil restent couvertes par les tests de rejeu existants.
- Garder le préfixe `SUBAGENT_REPORT_CONTEXT_PREFIX`, le XML échappé, le lot unique, la politique système et le cycle `pending -> injecté -> acquitté` inchangés.
- La compression durable lit `subagent_hidden_reports` séparément dans `compress::checkpoint_subagents`; aucune modification du domaine `compress` n'est nécessaire. La non-régression à verrouiller ici est le passage dans `context_budget` avec un rapport intact et toujours `user`.
- Le correctif de diagnostic reste borné à un code stable et à un résumé sûr. Les traductions et `KNOWN_ERROR_KEYS.reasoning_continuity_invalid` existent déjà et doivent seulement être vérifiés.
- Le travail `session_capacity_reached` qui chevauchait initialement le diagnostic est maintenant isolé dans le commit `e457053f`. Vérifier tout de même les quatre fichiers concernés avant la tâche 4 afin de ne pas mélanger un nouveau travail concurrent.
- Respecter le seuil de 230 lignes pour chaque fichier de code. Les fichiers de tests sont exemptés par `AGENTS.md`, mais le nouveau test de matrice doit rester centré sur ce contrat unique.
- Après chaque lot vert : relire le diff et les appelants directs, exécuter les filtres indiqués, puis faire un commit ciblé avec une git note contenant la cause et les commandes réellement exécutées.
- Après les modifications de code, exécuter `graphify update .` selon `AGENTS.md`. Ne pas ajouter `graphify-out` au commit.

## Comportement attendu

| Scénario | Résultat attendu |
| --- | --- |
| Premier rapport reçu en direct | Un seul message `user`, sans continuité ni appel d'outil |
| Rapport relu depuis le stockage | Même rôle `user` qu'en direct |
| Toute politique `LiveValidated` de continuation utilisateur | Construction locale du payload réussie |
| Vrai assistant sans enveloppe sur Responses obligatoire | Requête bloquée avant réseau |
| Vrai assistant sans enveloppe sur Chat obligatoire | Requête bloquée avant réseau |
| Vrai assistant sans enveloppe sur Anthropic obligatoire | Requête bloquée avant réseau |
| Ollama et Z.AI, continuité optionnelle | Payload construit avec le rapport au rôle `user` |
| Deux enfants, premier rapport prêt | Le parent reprend une fois et peut attendre le second |
| Échec ou annulation avant succès | Rapport non acquitté, donc encore livrable |
| Réduction du contexte | Rapport présent en entier et toujours `user` |
| Rapport trop grand pour le contexte | Échec fermé, aucun tronquage silencieux |
| Erreur `reasoning_continuity_invalid` | Code conservé et résumé situé avant l'appel modèle, sans accuser le dernier outil |

---

## Task 1: Corriger uniquement le constructeur partagé

**Files:**

- Modify: `src-tauri/src/services/agent_local/subagent_hidden_reports_tests.rs`
- Modify: `src-tauri/src/services/agent_local/subagent_report_context.rs`

**Interfaces:**

- Consumes: `report_to_message(SubagentHiddenReport) -> ChatMessage`, façade de test existante vers `report_batch_to_message`.
- Produces: tout rapport injecté par `append_context(&mut Vec<ChatMessage>, &[SubagentHiddenReport])` est un `ChatMessage` de rôle `user`, sans `continuation` ni `tool_calls`.

- [x] Renommer `report_context_is_assistant_and_xml_escaped` en `report_context_is_user_without_continuation_and_xml_escaped`. Conserver les assertions d'échappement et ajouter :

```rust
assert_eq!(message.role, "user");
assert!(message.continuation.is_none());
assert!(message.tool_calls.is_none());
```

- [x] Dans `multiple_ready_reports_share_one_batch`, remplacer l'attente `assistant` par `user`. Garder la preuve qu'un seul lot contient les deux identifiants.
- [x] Exécuter depuis `src-tauri` avant la correction :

```bash
node ../scripts/ci/run-rust-test-filter.mjs --filter report_context_is_user_without_continuation_and_xml_escaped
```

Échec attendu : le constructeur retourne encore `assistant`.

- [x] Remplacer seulement cette ligne dans `report_batch_to_message` :

```rust
ChatMessage::user(report_batch_content(reports))
```

- [x] Relancer :

```bash
node ../scripts/ci/run-rust-test-filter.mjs --filter report_context_is_user_without_continuation_and_xml_escaped
node ../scripts/ci/run-rust-test-filter.mjs --filter multiple_ready_reports_share_one_batch
```

- [x] Vérifier que les deux commandes exécutent un test et se terminent avec le code 0. Relire tous les appelants de `append_context`; le diff de production attendu est une ligne. Commit ciblé :

```bash
git add src-tauri/src/services/agent_local/subagent_report_context.rs \
  src-tauri/src/services/agent_local/subagent_hidden_reports_tests.rs
git commit --only -m "fix(agent): classify subagent reports as user input" -- \
  src-tauri/src/services/agent_local/subagent_report_context.rs \
  src-tauri/src/services/agent_local/subagent_hidden_reports_tests.rs
git notes add -m "Cause: Beaver-generated subagent reports were mislabeled as provider assistant output. Validation: report role and batching tests passed."
```

---

## Task 2: Verrouiller tous les contrats fournisseur actifs

**Files:**

- Create: `src-tauri/src/services/llm/reasoning_wire/subagent_report_contract_tests.rs`
- Modify: `src-tauri/src/services/llm/reasoning_wire/mod.rs`
- Modify: `src-tauri/src/services/llm/reasoning_wire/chat_contract_tests.rs`
- Modify: `src-tauri/src/services/llm/reasoning_wire/anthropic_contract_tests.rs`
- Modify: `src-tauri/src/services/codex_client/reasoning_continuity_tests.rs`
- Modify: `src-tauri/src/services/agent_local/ollama_wire_tests.rs`
- Read for contract verification: `src-tauri/src/services/reasoning_continuity/registry.rs`
- Read for contract verification: `src-tauri/src/services/llm/reasoning_wire/responses.rs`
- Read for contract verification: `src-tauri/src/services/llm/reasoning_wire/chat_text.rs`
- Read for contract verification: `src-tauri/src/services/llm/reasoning_wire/replay_apply_anthropic.rs`
- Read for contract verification: `src-tauri/src/services/agent_local/ollama_wire.rs`

**Interfaces:**

- Consumes: le `ChatMessage` de rapport `user` produit par la tâche 1 ; `active_routes() -> &'static [RouteContract]` ; `ContinuationTarget::Replay(ReplayTarget)`.
- Produces: le test `every_live_user_continuation_accepts_the_shared_subagent_report` échoue si une politique active refuse le rapport ou si un adaptateur nouveau n'est pas traité ; les tests par famille prouvent le scénario `[assistant natif valide, rapport]`.

- [x] Déclarer sous `#[cfg(test)]` le nouveau module `subagent_report_contract_tests` dans `reasoning_wire/mod.rs`.
- [x] Ajouter le test de matrice complet ci-dessous. Garder le `match` exhaustif, `ContinuationTarget::Replay` et `Some(&target)` tels quels : ils empêchent un faux vert qui contournerait la validation.

```rust
use super::{chat_text, replay, responses};
use crate::services::agent_local::ollama_wire;
use crate::services::agent_local::subagent_hidden_reports::{build_report, report_to_message};
use crate::services::agent_local::types_ollama::{ChatRequest, OllamaThink};
use crate::services::reasoning_continuity::contract::{
    ContinuationTarget, ContinuationUse, CredentialScope, ReplayTarget, RouteId,
};
use crate::services::reasoning_continuity::registry::{
    active_routes, ActivationState, AdapterId, ReplayRequirement,
};
use serde_json::json;

#[test]
fn every_live_user_continuation_accepts_the_shared_subagent_report() {
    let report = report_to_message(build_report(
        "child".into(),
        "Geminitor".into(),
        "explorer".into(),
        "completed".into(),
        "Rapport vérifié".into(),
    ));
    let mut checked = 0usize;

    for route in active_routes() {
        for model in route.models.iter().filter(|model| {
            model.activation == ActivationState::LiveValidated
                && model.continuation_use == ContinuationUse::UserContinuation
                && model.requirement != ReplayRequirement::Forbidden
        }) {
            let replay_target = ReplayTarget {
                route_id: route.route_id,
                model_id: model.model_id.into(),
                credential_scope: if route.route_id == RouteId::Ollama {
                    CredentialScope::local_uncredentialed()
                } else {
                    CredentialScope::authenticated("subagent-report-fixture").unwrap()
                },
                reasoning_mode: model.reasoning_mode,
                continuation_use: ContinuationUse::UserContinuation,
            };
            let target = ContinuationTarget::Replay(replay_target.clone());
            let messages = std::slice::from_ref(&report);

            let accepted = match route.adapter {
                AdapterId::ResponsesLocal => match responses::target_for_request(
                    messages,
                    Some(&target),
                ) {
                    Ok(Some(_)) => Ok(()),
                    Ok(None) => Err(replay::ReplayApplyError::PayloadMismatch),
                    Err(error) => Err(error),
                },
                AdapterId::AnthropicBlocks => {
                    let mut payload = [json!({
                        "role": report.role.as_str(),
                        "content": [{"type": "text", "text": report.content.as_str()}],
                    })];
                    replay::apply_anthropic_messages(messages, Some(&target), &mut payload)
                        .and_then(|_| {
                            (payload[0]["role"] == "user")
                                .then_some(())
                                .ok_or(replay::ReplayApplyError::PayloadMismatch)
                        })
                }
                AdapterId::OllamaNative => {
                    let request = ChatRequest {
                        model: model.model_id.into(),
                        messages: Vec::new(),
                        stream: true,
                        tools: None,
                        options: None,
                        keep_alive: None,
                        think: Some(OllamaThink::Level(model.reasoning_mode.as_name().into())),
                        capture_reasoning: false,
                        live_replay_target: Some(replay_target),
                        #[cfg(debug_assertions)]
                        fixture_candidate: None,
                    };
                    ollama_wire::chat_request(&request, messages).and_then(|payload| {
                        (payload["messages"][0]["role"] == "user")
                            .then_some(())
                            .ok_or(replay::ReplayApplyError::PayloadMismatch)
                    })
                }
                AdapterId::GeminiParts
                | AdapterId::MistralChunks
                | AdapterId::CerebrasReasoning
                | AdapterId::OpenRouterDetails
                | AdapterId::ChatReasoning => {
                    let mut payload = json!({
                        "messages": [{
                            "role": report.role.as_str(),
                            "content": report.content.as_str(),
                        }],
                    });
                    chat_text::apply_continuity(messages, Some(&target), &mut payload)
                        .and_then(|_| {
                            (payload["messages"][0]["role"] == "user")
                                .then_some(())
                                .ok_or(replay::ReplayApplyError::PayloadMismatch)
                        })
                }
            };

            assert!(
                accepted.is_ok(),
                "route={:?} model={} mode={:?} error={accepted:?}",
                route.route_id,
                model.model_id,
                model.reasoning_mode,
            );
            checked += 1;
        }
    }

    assert!(checked > 0, "the active registry must exercise at least one policy");
}
```

- [x] Ajouter dans `chat_contract_tests.rs` le scénario Chat réel suivant, avec les imports `build_report` et `report_to_message` :

```rust
#[test]
fn chat_native_assistant_then_subagent_report_is_valid() {
    let target = ContinuationTarget::Replay(replay_target(
        RouteId::Moonshot,
        "kimi-k2.7-code",
        ReasoningModeId::Auto,
        ContinuationUse::UserContinuation,
    ));
    let report = crate::services::agent_local::subagent_hidden_reports::report_to_message(
        crate::services::agent_local::subagent_hidden_reports::build_report(
            "child".into(),
            "Geminitor".into(),
            "explorer".into(),
            "completed".into(),
            "Rapport Kimi".into(),
        ),
    );
    let messages = [assistant(), report];

    let body = payload(
        "moonshot",
        "kimi-k2.7-code",
        &messages,
        &target,
        "auto",
    )
    .expect("native Kimi assistant followed by a Beaver report");

    assert_eq!(body["messages"][0]["reasoning_content"], "opaque-kimi");
    assert_eq!(body["messages"][1]["role"], "user");
}
```

- [x] Dans `anthropic_contract_tests.rs`, ajouter cette cible de production puis les deux tests :

```rust
fn user_target() -> ContinuationTarget {
    ContinuationTarget::Replay(ReplayTarget {
        route_id: RouteId::Anthropic,
        model_id: "claude-haiku-4-5-20251001".into(),
        credential_scope: CredentialScope::authenticated("fixture-scope").unwrap(),
        reasoning_mode: ReasoningModeId::Low,
        continuation_use: ContinuationUse::UserContinuation,
    })
}

#[test]
fn anthropic_native_assistant_then_subagent_report_is_valid() {
    let target = user_target();
    let native_blocks = blocks();
    let replay = target.replay().unwrap();
    let envelope = ReasoningEnvelope::new(
        ContractId::AnthropicMessagesV1,
        ReasoningSource::from_target(replay),
        CompletionState::Complete,
        ContinuationState::AnthropicBlocks {
            blocks: native_blocks.clone(),
        },
        vec![ToolLink {
            provider_call_id: "toolu_1".into(),
            tool_name: "read_file".into(),
        }],
    );
    let report = crate::services::agent_local::subagent_hidden_reports::report_to_message(
        crate::services::agent_local::subagent_hidden_reports::build_report(
            "child".into(),
            "Geminitor".into(),
            "explorer".into(),
            "completed".into(),
            "Rapport Anthropic".into(),
        ),
    );
    let messages = [assistant(envelope), report];
    let mut payload = [
        json!({"role":"assistant","content":[{"type":"text","text":"reconstructed"}]}),
        json!({"role":"user","content":[{"type":"text","text":"Rapport Anthropic"}]}),
    ];

    let replayed = super::replay::apply_anthropic_messages(
        &messages,
        Some(&target),
        &mut payload,
    )
    .expect("native Anthropic assistant followed by a Beaver report");

    assert_eq!(replayed.len(), 1);
    assert_eq!(payload[0]["content"], Value::Array(native_blocks));
    assert_eq!(payload[1]["role"], "user");
}

#[test]
fn required_anthropic_replay_rejects_an_assistant_without_an_envelope() {
    let target = user_target();
    let messages = [
        ChatMessage::assistant("lost state".into(), None, None, None, None),
        ChatMessage::user("continue".into()),
    ];
    let mut payload = [
        json!({"role":"assistant","content":[{"type":"text","text":"lost state"}]}),
        json!({"role":"user","content":[{"type":"text","text":"continue"}]}),
    ];

    assert!(matches!(
        super::replay::apply_anthropic_messages(&messages, Some(&target), &mut payload),
        Err(super::replay::ReplayApplyError::Blocked)
    ));
}
```

- [x] Ajouter dans `ollama_wire_tests.rs` le scénario Ollama réel suivant :

```rust
#[test]
fn ollama_native_assistant_then_subagent_report_is_valid() {
    let target = target("qwen3.5:4b");
    let report = crate::services::agent_local::subagent_hidden_reports::report_to_message(
        crate::services::agent_local::subagent_hidden_reports::build_report(
            "child".into(),
            "Geminitor".into(),
            "explorer".into(),
            "completed".into(),
            "Rapport Ollama".into(),
        ),
    );
    let messages = [message(&target, "opaque historic"), report];
    let mut request = request();
    request.live_replay_target = Some(target);

    let body = chat_request(&request, &messages)
        .expect("native Ollama assistant followed by a Beaver report");

    assert_eq!(body["messages"][0]["thinking"], "opaque historic");
    assert_eq!(body["messages"][1]["role"], "user");
}
```

- [x] Ajouter dans `codex_client/reasoning_continuity_tests.rs` ce test qui reproduit la frontière de la session témoin sans réseau :

```rust
#[test]
fn codex_parent_payload_accepts_a_subagent_report_after_native_reasoning() {
    let target = ContinuationTarget::Replay(
        target("codex-scope").replay().unwrap().clone(),
    );
    let report = crate::services::agent_local::subagent_hidden_reports::report_to_message(
        crate::services::agent_local::subagent_hidden_reports::build_report(
            "child".into(),
            "Geminitor".into(),
            "explorer".into(),
            "completed".into(),
            "Rapport Codex".into(),
        ),
    );
    let request = build_codex_request_with_continuity(
        "gpt-5.6-luna",
        &[assistant(&target), report],
        &[],
        Some("medium"),
        Some("session"),
        FastModeRequest::Standard,
        Some(&target),
    )
    .expect("native Codex assistant followed by a Beaver report");

    assert_eq!(request.input[0]["type"], "reasoning");
    assert_eq!(request.input[1]["type"], "message");
    assert_eq!(request.input.last().unwrap()["role"], "user");
}
```
- [x] Ces tests ajoutent la couverture transversale après le correctif minimal de la tâche 1. Le témoin rouge de la tâche 1 prouve déjà la régression sur `e457053f`; ne créer aucun worktree temporaire uniquement pour refaire cette preuve.
- [x] Exécuter depuis `src-tauri` après la correction :

```bash
node ../scripts/ci/run-rust-test-filter.mjs --filter every_live_user_continuation_accepts_the_shared_subagent_report
node ../scripts/ci/run-rust-test-filter.mjs --filter codex_parent_payload_accepts_a_subagent_report_after_native_reasoning
node ../scripts/ci/run-rust-test-filter.mjs --filter chat_native_assistant_then_subagent_report_is_valid
node ../scripts/ci/run-rust-test-filter.mjs --filter anthropic_native_assistant_then_subagent_report_is_valid
node ../scripts/ci/run-rust-test-filter.mjs --filter ollama_native_assistant_then_subagent_report_is_valid
node ../scripts/ci/run-rust-test-filter.mjs --filter responses_continuity_blocks_wrong_scope_and_required_missing_state
node ../scripts/ci/run-rust-test-filter.mjs --filter required_missing_envelope_blocks_before_the_transport_records_a_request
node ../scripts/ci/run-rust-test-filter.mjs --filter required_anthropic_replay_rejects_an_assistant_without_an_envelope
node ../scripts/ci/run-rust-test-filter.mjs --filter only_exact_live_fixture_pairs_are_activated
```

- [x] Vérifier que chaque commande annonce au moins un test exécuté et se termine avec le code 0. Commit ciblé :

```bash
git add src-tauri/src/services/llm/reasoning_wire/subagent_report_contract_tests.rs \
  src-tauri/src/services/llm/reasoning_wire/mod.rs \
  src-tauri/src/services/llm/reasoning_wire/chat_contract_tests.rs \
  src-tauri/src/services/llm/reasoning_wire/anthropic_contract_tests.rs \
  src-tauri/src/services/codex_client/reasoning_continuity_tests.rs \
  src-tauri/src/services/agent_local/ollama_wire_tests.rs
git commit --only -m "test(agent): cover subagent reports across reasoning providers" -- \
  src-tauri/src/services/llm/reasoning_wire/subagent_report_contract_tests.rs \
  src-tauri/src/services/llm/reasoning_wire/mod.rs \
  src-tauri/src/services/llm/reasoning_wire/chat_contract_tests.rs \
  src-tauri/src/services/llm/reasoning_wire/anthropic_contract_tests.rs \
  src-tauri/src/services/codex_client/reasoning_continuity_tests.rs \
  src-tauri/src/services/agent_local/ollama_wire_tests.rs
git notes add -m "Coverage: active registry matrix plus native assistant and subagent report scenarios for Responses, Chat, Anthropic and Ollama."
```

---

## Task 3: Verrouiller le rechargement, le cycle parent et le budget de contexte

**Files:**

- Modify: `src-tauri/src/services/agent_local/subagent_orchestration_race_tests.rs`
- Modify: `src-tauri/src/services/agent_local/subagent_terminal_wait_tests.rs`
- Modify: `src-tauri/src/services/agent_local/context_budget_tests.rs`
- Modify: `src-tauri/src/services/agent_local/conversation_history_continuity_tests.rs`
- Read for contract verification: `src-tauri/src/services/agent_local/subagent_orchestration.rs`
- Read for contract verification: `src-tauri/src/services/agent_local/subagent_report_delivery.rs`
- Read for contract verification: `src-tauri/src/services/agent_local/context_budget_prune.rs`
- Read for compression verification: `src-tauri/src/services/compress/checkpoint_subagents.rs`

**Interfaces:**

- Consumes: le rapport `user` produit par la tâche 1 ; `conversation_history_build::from_continuation(&AgentSession, &ContinuationTarget) -> Result<ConversationHistory, ConversationHistoryError>` ; `ParentSubagentOrchestrator::prepare_for_model_request(&mut Vec<ChatMessage>)`.
- Produces: une preuve de parité direct/disque, une preuve de réveil avec deux enfants et une preuve que `context_budget::prepare_for_request` conserve le rapport entier avec son rôle.

- [x] Ajouter `delivered_subagent_report_reloads_as_user_context` dans `conversation_history_continuity_tests.rs` :

```rust
#[tokio::test]
async fn delivered_subagent_report_reloads_as_user_context() {
    let mut session = create_session().await;
    let mut report = super::super::subagent_hidden_reports::build_report(
        "child".into(),
        "Geminitor".into(),
        "explorer".into(),
        "completed".into(),
        "Rapport durable".into(),
    );
    report.delivered = true;
    session.subagent_hidden_reports.push(report);
    super::super::session_store::save(&session)
        .await
        .expect("persist delivered report");
    let reloaded = super::super::session_store::get(&session.id)
        .await
        .expect("reload delivered report");
    let target = ContinuationTarget::Replay(target("model-a"));

    let history = super::super::conversation_history_build::from_continuation(
        &reloaded,
        &target,
    )
    .expect("delivered report history");
    let context = history
        .messages
        .iter()
        .find(|message| {
            message
                .content
                .starts_with(super::super::subagent_report_context::SUBAGENT_REPORT_POLICY_PREFIX)
        })
        .expect("durable subagent report context");

    assert_eq!(context.role, conversation_history::ProviderRole::User);
    assert!(context.continuation.is_none());
    assert!(context.content.contains("Rapport durable"));
    cleanup(&session.id).await;
}
```

- [x] Renforcer `report_policy_and_body_match_api_and_ollama_payloads` : localiser le rapport injecté, vérifier son rôle `user`, puis vérifier le rôle `user` dans le payload Ollama et dans le payload OpenAI compatible. Garder les assertions sur la politique système et le contenu exact.
- [x] Renforcer `first_report_resumes_once_then_waits_for_the_second_child` : après chaque réveil, retrouver le lot correspondant, vérifier son rôle `user`, puis conserver les preuves existantes qu'un rapport ne réveille qu'une fois et que le second enfant reste attendu.
- [x] Dans `oversized_subagent_report_fails_closed_instead_of_truncating` et `fitting_subagent_report_survives_saturated_context_intact`, construire le rapport avec `msg("user", ...)`. Après `prepare_for_request`, vérifier à la fois le contenu exact et `role == "user"`.
- [x] Ne modifier ni `context_budget_prune::is_required_report` ni le domaine `compress`. Motif : l'autorité de priorité est déjà le préfixe du rapport et le checkpoint collecte les rapports cachés séparément de l'historique de messages.
- [x] Exécuter :

```bash
node ../scripts/ci/run-rust-test-filter.mjs --filter report_policy_and_body_match_api_and_ollama_payloads
node ../scripts/ci/run-rust-test-filter.mjs --filter first_report_resumes_once_then_waits_for_the_second_child
node ../scripts/ci/run-rust-test-filter.mjs --filter delivered_subagent_report_reloads_as_user_context
node ../scripts/ci/run-rust-test-filter.mjs --filter oversized_subagent_report_fails_closed_instead_of_truncating
node ../scripts/ci/run-rust-test-filter.mjs --filter fitting_subagent_report_survives_saturated_context_intact
node ../scripts/ci/run-rust-test-filter.mjs --filter cancellation_after_stream_completion_keeps_unacknowledged_report
node ../scripts/ci/run-rust-test-filter.mjs --filter missing_report_from_real_payload_blocks_acknowledgement
node ../scripts/ci/run-rust-test-filter.mjs --filter active_subagents_keep_mission_activity_and_identity
```

- [x] Vérifier dans chaque sortie qu'au moins un test a réellement été exécuté. Commit ciblé :

```bash
git add src-tauri/src/services/agent_local/subagent_orchestration_race_tests.rs \
  src-tauri/src/services/agent_local/subagent_terminal_wait_tests.rs \
  src-tauri/src/services/agent_local/context_budget_tests.rs \
  src-tauri/src/services/agent_local/conversation_history_continuity_tests.rs
git commit --only -m "test(agent): preserve subagent report delivery across context paths" -- \
  src-tauri/src/services/agent_local/subagent_orchestration_race_tests.rs \
  src-tauri/src/services/agent_local/subagent_terminal_wait_tests.rs \
  src-tauri/src/services/agent_local/context_budget_tests.rs \
  src-tauri/src/services/agent_local/conversation_history_continuity_tests.rs
git notes add -m "Coverage: direct and reloaded report roles, two-child wakeup ordering, acknowledgement failures and context-budget preservation."
```

---

## Task 4: Conserver le vrai code de diagnostic

**Files:**

- Modify: `src-tauri/src/services/agent_local/stream_diagnostics_failure_tests.rs`
- Modify: `src-tauri/src/services/agent_local/stream_diagnostics_failure.rs`
- Read for UI contract verification: `src/lib/agent-error-codes.ts`
- Read for translation verification: `src/lib/agent-error-codes.test.ts`

**Interfaces:**

- Consumes: `classify_error(&str, bool) -> String`, `safe_code(&str) -> String` et `safe_summary(&AgentDiagnosticRun, &str, &str) -> String`.
- Produces: `reasoning_continuity_invalid` reste identique dans les diagnostics et le résumé exact est `Interruption avant l'appel du modèle (reasoning_continuity_invalid).`.

- [x] Avant toute modification, exécuter depuis la racine :

```bash
git diff -- src-tauri/src/services/agent_local/stream_diagnostics_failure.rs \
  src-tauri/src/services/agent_local/stream_diagnostics_failure_tests.rs \
  src/lib/agent-error-codes.ts \
  src/lib/agent-error-codes.test.ts
```

Résultat requis : aucun hunk non indexé appartenant à `session_capacity_reached` ou à une autre tâche. S'il en reste, ne pas modifier ni indexer ces quatre fichiers avant que leur propriétaire ait isolé son travail.

- [x] Ajouter le test rouge `reasoning_continuity_failure_keeps_its_specific_code_and_phase` :

```rust
#[test]
fn reasoning_continuity_failure_keeps_its_specific_code_and_phase() {
    let diagnostic = run("model_request", "completed");
    assert_eq!(
        classify_error("reasoning_continuity_invalid", false),
        "reasoning_continuity_invalid"
    );
    assert_eq!(
        safe_code("reasoning_continuity_invalid"),
        "reasoning_continuity_invalid"
    );
    let summary = safe_summary(
        &diagnostic,
        "reasoning_continuity_invalid",
        "reasoning_continuity_invalid",
    );
    assert_eq!(
        summary,
        "Interruption avant l'appel du modèle (reasoning_continuity_invalid)."
    );
}
```

- [x] Exécuter :

```bash
node ../scripts/ci/run-rust-test-filter.mjs --filter reasoning_continuity_failure_keeps_its_specific_code_and_phase
```

Échec attendu : `classify_error` retourne `unknown`, `safe_code` retourne `stream_error` et le résumé désigne encore le dernier outil.

- [x] Ajouter `reasoning_continuity_invalid` à la liste fermée des codes stables de `classify_error`.
- [x] Dans `safe_summary`, traiter ce code avant `last_tool` et retourner exactement `Interruption avant l'appel du modèle (reasoning_continuity_invalid).` via `support::clip`. Motif : `get_subagent` a réussi et ne doit plus apparaître comme la cause de l'interruption.
- [x] Ne reformater aucun code voisin. Recompter `stream_diagnostics_failure.rs` : 205 lignes dans `e457053f`, 197 à la base `d7b27545` et 203 après le correctif ; il reste sous 230 lignes.
- [x] L'implémentation de production attendue est limitée à ces deux ajouts :

```rust
if error_type == "reasoning_continuity_invalid" {
    return support::clip(
        "Interruption avant l'appel du modèle (reasoning_continuity_invalid).",
    );
}
```

et, dans la liste des codes stables :

```rust
| "reasoning_continuity_invalid"
```

- [x] Vérifier sans modification que `KNOWN_ERROR_KEYS.reasoning_continuity_invalid` pointe déjà vers `errors.reasoningContinuityInvalid` et que cette clé existe dans les sept catalogues.
- [x] Relancer :

```bash
node ../scripts/ci/run-rust-test-filter.mjs --filter reasoning_continuity_failure_keeps_its_specific_code_and_phase
node ../scripts/ci/run-rust-test-filter.mjs --filter stream_diagnostics_failure
cd .. && npx vitest run src/lib/agent-error-codes.test.ts
```

- [x] Vérifier que chaque commande exécute au moins un test et se termine avec le code 0. Refaire `git diff --` sur les quatre fichiers de précondition, puis indexer uniquement les deux fichiers Rust si la précondition est toujours satisfaite :

```bash
git add src-tauri/src/services/agent_local/stream_diagnostics_failure.rs \
  src-tauri/src/services/agent_local/stream_diagnostics_failure_tests.rs
git commit --only -m "fix(agent): preserve reasoning continuity diagnostics" -- \
  src-tauri/src/services/agent_local/stream_diagnostics_failure.rs \
  src-tauri/src/services/agent_local/stream_diagnostics_failure_tests.rs
git notes add -m "Cause: reasoning_continuity_invalid fell through to generic diagnostics. Validation: exact code, exact safe summary and seven existing translations."
```

---

## Task 5: Vérification finale sans provider payant

**Files:**

- Verify only: all files modified in Tasks 1–4
- Maintain graph: `graphify-out/` through `graphify update .`, without staging it

**Interfaces:**

- Consumes: les commits indépendants des tâches 1 à 4.
- Produces: une validation locale complète avec sorties et nombres de tests vérifiés, sans requête provider réelle.

- [x] Relire le diff complet et confirmer : une seule modification de production pour le rôle, une petite modification du diagnostic, aucune exception fournisseur, aucune modification du domaine `compress`, aucune dépendance nouvelle.
- [x] Vérifier les occurrences restantes :

```bash
rg -n 'report_context_is_assistant|SUBAGENT_REPORT_CONTEXT_PREFIX.*assistant|ChatMessage::assistant\(report_batch_content' src-tauri/src
rg -n 'reasoning_continuity_invalid' src-tauri/src/services/agent_local/stream_diagnostics_failure.rs src/lib/agent-error-codes.ts
```

Résultat attendu : la première commande ne trouve rien ; la seconde trouve le code dans les deux autorités de diagnostic backend et frontend.

- [x] Exécuter les suites Rust ciblées :

```bash
node ../scripts/ci/run-rust-test-filter.mjs --filter subagent_hidden_reports
node ../scripts/ci/run-rust-test-filter.mjs --filter subagent_orchestration
node ../scripts/ci/run-rust-test-filter.mjs --filter subagent_event_terminal_tests
node ../scripts/ci/run-rust-test-filter.mjs --filter context_budget
node ../scripts/ci/run-rust-test-filter.mjs --filter reasoning_wire
node ../scripts/ci/run-rust-test-filter.mjs --filter checkpoint_sources_tests
node ../scripts/ci/run-rust-test-filter.mjs --filter checkpoint_reasoning_tests
```

- [ ] Exécuter les contrôles globaux :

```bash
cargo test --lib
cargo clippy --all-targets -- -D warnings
cd .. && npx vitest run src/lib/agent-error-codes.test.ts
```

Résultat réel : `cargo clippy --all-targets -- -D warnings` et les 11 tests frontend ont réussi. `cargo test --lib` en parallèle a réussi 5470 tests mais a échoué deux fois sur `private_store::tests::app_storage_repairs_the_forecast_notes_directory` avec `Outil de fixture indisponible`; cette case reste donc décochée. Le test isolé a réussi, puis `cargo test --lib -- --test-threads=1` a réussi avec 5471 tests, 0 échec et 21 ignorés.

- [x] Vérifier les codes de sortie et le nombre de tests exécutés. Si une commande est rouge, conserver la sortie et ne pas déclarer le plan validé.
- [x] Depuis la racine du dépôt, exécuter :

```bash
git diff --check
graphify update .
git status --short
```

- [x] Vérifier que `graphify-out` n'est pas indexé. Indexer seulement les fichiers du correctif avec des chemins explicites, conserver les documents demandés, puis créer le commit final ou confirmer que les commits ciblés couvrent tout le plan.
- [x] Ajouter une git note finale qui explique : rapport Beaver classé comme entrée `user`, validations des vraies sorties modèle conservées, matrice dérivée du registre actif et compression inchangée. Inclure uniquement les commandes réellement exécutées et leurs résultats.
