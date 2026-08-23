# Phase 2 — mode Rapide OpenAI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ajouter une bascule « Rapide » indépendante et persistante par session, puis transmettre le tier Fast exact sur OpenAI API et Codex OAuth uniquement pour les modèles qui le déclarent.

**Architecture:** `AgentSession.fast_mode_enabled` est l'unique préférence durable. `ModelInfo.supports_fast_mode` est l'unique capacité publiée à l'interface. Au début de chaque génération, Rust capture le provider, la capacité et la préférence dans un `FastModeRequest` fermé (`Unsupported | Standard | Fast`) qui traverse sans être recalculé les tours outils, retries, compressions et transports : OpenAI API utilise Responses et sérialise toujours `default` hors Fast effectif, tandis que Codex OAuth sérialise `priority` ou omet le champ et dérive le même tier dans `x-codex-routing-hint`.

**Tech Stack:** Rust, Tauri 2, serde, reqwest, OpenAI Responses HTTP, Codex Responses HTTP/WebSocket, React 19, TypeScript, Vitest, Testing Library.

**Correction issue de la validation réelle du 23 août 2026 :** le chemin OpenAI API Chat Completions a été remplacé par Responses pour toutes les générations. Les essais discriminants ont prouvé que Fast fonctionnait sans raisonnement sur Chat Completions, tandis que `reasoning_effort` y provoquait HTTP 400; Responses a terminé avec Fast servi sous `priority` et `reasoning.effort = medium`. Le module `llm/openai_responses.rs` partage la conversion, le lecteur SSE, les outils, le replay et les métriques Responses avec Codex, mais charge uniquement la clé API OpenAI et appelle uniquement `api.openai.com/v1/responses`.

**Spec:** `docs/providers/openai-fast/SPEC.md`

## Global Constraints

- La branche de départ est `main` au commit de phase 1 `5b27453d` ou un descendant entièrement vert.
- `fast_mode_enabled` est un booléen de session, vaut `false` par défaut et reste inchangé lors d'un changement de modèle ou de provider.
- Une session créée, clonée, sous-agent, heartbeat ou gateway commence à `false`; aucune session n'hérite de la préférence d'une autre.
- Une session incompatible conserve sa préférence cachée; revenir sur un modèle compatible la réactive.
- `supports_fast_mode` vient du registre exact pour OpenAI API et du catalogue réel du compte pour Codex OAuth; aucun préfixe de nom ne décide.
- OpenAI API : Fast effectivement actif → `service_tier: "fast"`; tout autre état, y compris une capacité non déclarée → `service_tier: "default"`.
- Codex OAuth compatible : actif → `service_tier: "priority"`, inactif → champ absent.
- Codex OAuth non compatible ou autre provider : champ absent. Beaver ne produit jamais `auto`, `flex` ou `ultrafast` depuis cette bascule.
- Le catalogue OAuth autorise Fast uniquement avec `service_tiers[].id == "priority"`; `additional_speed_tiers` reste informatif et ne décide pas.
- Le transport Codex envoie `x-codex-routing-hint` sur HTTP et WebSocket, dérivé de `CodexRequest`: `model=<slug>;tier=priority` en Fast, sinon `model=<slug>`.
- Le tier est capturé une fois au début d'une génération. Une modification pendant le flux ne concerne que la génération suivante.
- Aucun prix, crédit, multiplicateur de vitesse ou texte secondaire dans le sélecteur.
- Réutiliser `ToggleSwitch`; ne pas créer une seconde primitive de bascule.
- Convertir exactement le tracé de `/Users/kevinh/Downloads/typcn--flash-outline.svg` en `FastModeIcon` via `InlineIcon`; ne pas ajouter de masque CSS, de second asset ni embarquer le PNG de licence.
- Ajouter l'attribution Typicons/Stephen Hutchings/CC BY-SA 4.0 dans `THIRD_PARTY_NOTICES.md`.
- Sept langues, clavier, VoiceOver, focus visible et thèmes clair/sombre.
- Toute erreur visible utilise un code stable et une traduction; aucun corps provider, token, route privée ou identité de compte dans les logs.
- Les catalogues et journaux restent bornés. Toute lecture de capacité invalide échoue fermé pour Fast.
- Aucun replay automatique en Standard après un refus du tier.
- Les coûts GPT-5.6 API et OAuth restent indisponibles tant que Beaver ne possède pas une observation tarifaire exacte.
- Un fichier de code reste sous 230 lignes. Créer un module ciblé avant de dépasser la limite.
- Chaque correction commence par un test rouge, puis le test doit passer avant le commit.
- Après toute modification de code, exécuter `graphify update .`.
- Les appels réels potentiellement facturés restent bloqués jusqu'à l'accord explicite de Kevin.
- Préserver les changements utilisateur déjà présents dans `docs/beaver-site/`.
- Pour les preuves provider, suivre `docs/providers/plan-de-tests.md` (`P01` à `P13`).

## File Structure

### Nouveaux fichiers

- `src-tauri/src/services/llm/fast_mode.rs` — décision fermée `Unsupported | Standard | Fast`, capacité effective et valeurs réseau.
- `src-tauri/src/services/llm/openai_responses.rs` — payload, authentification par clé et transport Responses OpenAI API; aucun jeton OAuth.
- `src-tauri/src/services/codex_client/model_catalog_fast.rs` — lecture bornée des tiers Fast/Priority du catalogue Codex, séparée de `model_catalog.rs` déjà proche de 230 lignes.
- `src-tauri/src/services/codex_client/routing_hint.rs` — valeur validée de `x-codex-routing-hint`, dérivée uniquement de `CodexRequest`.
- `src-tauri/src/services/agent_local/session_fast_mode_tests.rs` — persistance, indépendance, concurrence et sérialisation de la préférence.
- `src/hooks/use-session-fast-mode.ts` — mutation IPC bornée, état en vol et rechargement confirmé.
- `src/components/ui/fast-mode-icon.tsx` — tracé tiers unique rendu par la primitive `InlineIcon`.
- `src/i18n/openai-fast-translations.test.ts` — présence des textes Fast et des erreurs dans les sept langues.
- `src/components/agent-local/__tests__/agent-local-fast-mode.test.tsx` — câblage de la session affichée, y compris un onglet clone.
- `docs/providers/openai-fast/fixtures/` — preuves réelles anonymisées, créées seulement après accord de coût.

### Autorités modifiées

- `src-tauri/src/services/agent_local/types_session.rs` — préférence durable.
- `src-tauri/src/services/llm/types.rs` — capacité modèle normalisée.
- `src-tauri/src/services/llm/provider_model_registry.rs` et `src-tauri/resources/provider-models/openai.json` — capacité API exacte.
- `src-tauri/src/services/codex_client/model_catalog_wire.rs` — forme bornée du catalogue OAuth.
- `src-tauri/src/services/codex_client/types.rs` — payload Responses commun à HTTP et WebSocket.
- `src-tauri/src/services/codex_client/request_http.rs` et `websocket_connect.rs` — pose du même routage dérivé, sans reconstruire sa valeur.
- `src-tauri/src/services/provider_usage/request_journal.rs` — tier demandé et tier réellement servi.
- `src/components/agent-local/reasoning-selector.tsx` — ligne Rapide en tête du menu existant.
- `src/components/ui/__tests__/icon-authority.test.ts` — test existant étendu, jamais recréé ni écrasé.

### Sens des dépendances

```text
registre API / catalogue OAuth
            ↓
ModelInfo.supports_fast_mode ─────→ interface
            ↓
AgentSession.fast_mode_enabled ───→ FastModeRequest capturé
                                      ↓
                      API Responses / Codex HTTP / Codex WS
                                      ↓
                         tier réellement servi → journal borné
```

---

### Task 1: Publier une capacité Fast unique dans les catalogues

**Files:**
- Modify: `src-tauri/src/services/llm/types.rs:5`
- Modify: `src-tauri/src/services/llm/provider_model_registry.rs:12`
- Modify: `src-tauri/src/services/llm/provider_model_registry_tests.rs`
- Modify: `src-tauri/src/services/llm/provider_model_registry_inventory_tests.rs`
- Modify: `src-tauri/src/services/llm/openai_compat_models.rs:26`
- Modify: `src-tauri/src/services/llm/openai_compat_model_parser.rs:59`
- Modify: `src-tauri/src/services/llm/provider_model_lookup.rs:3`
- Modify: `src-tauri/src/services/llm/providers/openai.rs`
- Modify: `src-tauri/src/services/llm/runtime_models.rs`
- Modify: `src-tauri/src/services/llm/kimi_models.rs`
- Modify: `src-tauri/src/services/llm/litellm_catalog_search.rs`
- Modify: `src-tauri/src/services/llm/types_tests.rs`
- Modify: `src-tauri/src/commands/llm.rs:20`
- Modify: `src-tauri/resources/provider-models/openai.json`
- Modify: `src-tauri/src/services/codex_client/model_catalog_wire.rs:7`
- Create: `src-tauri/src/services/codex_client/model_catalog_fast.rs`
- Modify: `src-tauri/src/services/codex_client/model_catalog.rs:11`
- Modify: `src-tauri/src/services/codex_client/model_catalog_tests.rs`
- Modify: `src-tauri/src/services/codex_client/model_catalog_fallback.rs`
- Modify: `src-tauri/src/services/codex_client/mod.rs`
- Modify: `src-tauri/src/commands/oauth_provider_models.rs:7`
- Modify: `src-tauri/src/services/llm_oauth/xai_catalog.rs`
- Modify: `src-tauri/src/services/agent_local/ollama_model_helpers.rs`
- Modify: `src-tauri/src/services/agent_local/types_ollama.rs`
- Modify: `src-tauri/src/services/reasoning_tests.rs`
- Modify: `src/hooks/available-model-types.ts`
- Modify: `src/hooks/oauth-models.ts`
- Modify: `src/hooks/use-available-models.ts`
- Modify: `src/hooks/use-context-progress.ts`
- Modify: `src/hooks/__tests__/use-available-models-oauth.test.ts`

**Interfaces:**
- Consumes: registre JSON OpenAI et champ OAuth `service_tiers[].id`.
- Produces: `ModelInfo.supports_fast_mode: bool`, transporté sans recalcul jusqu'à `AvailableModel.supports_fast_mode`.

- [ ] **Step 1: Écrire les tests rouges du registre API**

Ajouter les assertions exactes suivantes :

```rust
assert!(provider_model_registry::lookup("openai", "gpt-5.6-sol")
    .unwrap()
    .supports_fast_mode);
assert!(provider_model_registry::lookup("openai", "gpt-5.6")
    .unwrap()
    .supports_fast_mode);
assert!(provider_model_registry::lookup("openai", "gpt-5.6-terra")
    .unwrap()
    .supports_fast_mode);
assert!(provider_model_registry::lookup("openai", "gpt-5.6-luna")
    .unwrap()
    .supports_fast_mode);
assert!(provider_model_registry::lookup("openai", "gpt-5.6-terra-pro").is_none());
assert!(!provider_model_lookup::supports_fast_mode(
    "openrouter",
    "openai/gpt-5.6-sol",
));
```

Ne pas encoder GPT-5.5 ou GPT-5.4 comme « incompatibles Fast » : ils sont compatibles en OAuth Codex, mais leur support Fast par clé API n'est pas confirmé par la grille API ouverte le 23 août 2026. Le registre API les laisse simplement non annoncés jusqu'à une source officielle ou une preuve réelle datée.

Ajouter aussi un JSON de test avec `supports_fast_mode: true` sur un provider autre qu'OpenAI et exiger `Err("fast_mode_provider")` afin qu'une copie accidentelle ne publie pas Fast ailleurs.

- [ ] **Step 2: Vérifier que les tests échouent**

Run:

```bash
cd src-tauri
cargo test provider_model_registry --lib
cargo test provider_model_registry_inventory --lib
```

Expected: compilation ou assertions rouges car le champ n'existe pas.

- [ ] **Step 3: Ajouter le champ statique et sa validation**

Ajouter exactement :

```rust
#[serde(default)]
pub supports_fast_mode: bool,
```

à `ProviderModelConfig` et `ModelInfo`. Dans `validate_file`, refuser `supports_fast_mode == true` lorsque `file.provider != "openai"`.

Ajouter l'autorité provider dans `providers/openai.rs` :

```rust
pub const PROVIDER_ID: &str = "openai";
```

Réutiliser cette constante et `codex_client::PROVIDER_ID` dans tout le domaine Fast; ne pas créer de littéraux provider concurrents. Ajouter ensuite une fonction stricte dans `provider_model_lookup.rs` :

```rust
pub fn supports_fast_mode(provider_id: &str, model_id: &str) -> bool {
    provider_id == crate::services::llm::providers::openai::PROVIDER_ID
        && direct_entry(provider_id, model_id)
            .is_some_and(|model| model.supports_fast_mode)
}
```

`list_llm_models` recopie cette valeur dans chaque `ModelInfo`. Ne pas l'ajouter à l'agrégation amont de `ModelCapabilities` : OpenRouter hérite des capacités outils/vision de ses modèles amont, mais ne doit pas hériter du contrat commercial Fast d'OpenAI. Initialiser le champ à `false` dans les constructeurs génériques et laisser `cargo check --all-targets` signaler chaque littéral restant.

Dans `openai.json`, ajouter `"supports_fast_mode": true` seulement aux entrées canoniques `gpt-5.6-sol`, `gpt-5.6-terra` et `gpt-5.6-luna`. L'alias `gpt-5.6` hérite de l'entrée `sol`; ne pas créer une quatrième entrée visible.

- [ ] **Step 4: Écrire les tests rouges du catalogue OAuth borné**

Étendre la fixture `WireModel` avec :

```json
{
  "slug": "gpt-5.6-sol",
  "display_name": "GPT-5.6 Sol",
  "context_window": 272000,
  "effective_context_window_percent": 95,
  "service_tiers": [
    { "id": "priority", "name": "Fast", "description": "faster" }
  ],
  "additional_speed_tiers": ["fast"]
}
```

Exiger :

```rust
assert!(parsed.info.supports_fast_mode);
assert!(!without_tiers.info.supports_fast_mode);
assert!(!fallback_models().iter().any(|model| model.supports_fast_mode));
```

Tester aussi neuf tiers, un identifiant de tier de plus de 32 caractères, les doublons et les valeurs `flex`/`ultrafast`. Seul `service_tiers[].id == "priority"` donne `true`. Une fixture avec uniquement `additional_speed_tiers: ["fast"]` doit rester `false`, car le client Codex filtre les tiers réseau contre `service_tiers` et marque l'autre champ comme obsolète.

- [ ] **Step 5: Implémenter le parseur OAuth dans un module dédié**

Dans `model_catalog_wire.rs`, ajouter :

```rust
const MAX_SERVICE_TIERS: usize = 8;

#[derive(Debug, Deserialize)]
pub(super) struct WireServiceTier {
    pub id: String,
}
```

et sur `WireModel` :

```rust
#[serde(default)]
pub service_tiers: BoundedVec<WireServiceTier, MAX_SERVICE_TIERS>,
```

Dans le nouveau module :

```rust
pub(super) fn supports_fast_mode(model: &WireModel) -> bool {
    model.service_tiers.0.iter().any(|tier| {
        valid_tier_id(&tier.id) && tier.id == "priority"
    })
}

fn valid_tier_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 32
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || matches!(byte, b'-' | b'_')
        })
}
```

Le `ModelInfo` construit par `convert_model` reçoit ce booléen. Tous les modèles de `model_catalog_fallback.rs` reçoivent explicitement `supports_fast_mode: false` avec un commentaire indiquant que le fallback ne prouve pas l'éligibilité du compte.

Exposer dans `model_catalog.rs` une lecture qui réutilise exactement `load_catalog()` et son cache borné :

```rust
pub async fn supports_fast_mode(model_id: &str) -> Result<bool, String> {
    if !crate::services::llm::runtime_models::valid_model_id(model_id) {
        return Ok(false);
    }
    Ok(load_catalog()
        .await?
        .iter()
        .find(|model| model.info.id == model_id)
        .is_some_and(|model| model.info.supports_fast_mode))
}
```

Cette fonction ne lance pas un second catalogue et ne consulte jamais le fallback.

- [ ] **Step 6: Propager la capacité dans les deux frontières frontend**

Ajouter `supports_fast_mode: bool` à `OAuthProviderModel` et `OAuthModelInfo`, puis `supports_fast_mode?: boolean` à `AvailableModel`. Déplacer l'unique interface frontend `LlmModelInfo` dans `available-model-types.ts`, y ajouter `supports_fast_mode: boolean`, puis l'importer dans `use-available-models.ts` et `use-context-progress.ts` au lieu de conserver deux interfaces privées. Les deux mappeurs de modèles recopient seulement le champ reçu :

```ts
supports_fast_mode: model.supports_fast_mode,
```

Ne jamais ajouter de test `model.id.startsWith(...)` côté TypeScript.

- [ ] **Step 7: Vérifier toute la chaîne de sérialisation**

Run:

```bash
cd src-tauri
cargo test provider_model_registry --lib
cargo test provider_model_registry_inventory --lib
cargo test model_catalog --lib
cargo test oauth_provider_models --lib
cargo check --all-targets
cd ..
npx vitest run src/hooks/__tests__/use-available-models-oauth.test.ts
npx tsc --noEmit
```

Expected: les modèles API exacts et le catalogue OAuth réel transportent le booléen; les fallbacks et autres providers restent `false`.

- [ ] **Step 8: Commit**

```bash
git add src-tauri/src/services/llm src-tauri/src/services/codex_client src-tauri/src/commands src-tauri/resources/provider-models/openai.json src/hooks
git commit -m "feat(openai): publier la capacité du mode rapide"
```

---

### Task 2: Persister une préférence indépendante par session

**Files:**
- Modify: `src-tauri/src/services/agent_local/types_session.rs:43`
- Modify: `src-tauri/src/services/agent_local/session_store.rs:23`
- Modify: `src-tauri/src/services/agent_local/session_store_updates.rs:1`
- Modify: `src-tauri/src/services/agent_local/session_store_update_race_tests.rs`
- Modify: `src-tauri/src/services/agent_local/session_index.rs:53`
- Modify: `src-tauri/src/services/agent_local/session_index_tests.rs`
- Modify: `src-tauri/src/services/agent_local/session_index_reconcile_tests.rs`
- Modify: `src-tauri/src/services/agent_local/clone_session_build.rs:12`
- Modify: `src-tauri/src/services/agent_local/clone_session_tests.rs`
- Modify: `src-tauri/src/services/agent_local/subagent_inheritance_tests.rs`
- Create: `src-tauri/src/services/agent_local/session_fast_mode_tests.rs`
- Modify: `src-tauri/src/services/agent_local/agent_local_modules_sessions.rs`
- Modify: `src-tauri/src/commands/agent_sessions.rs:37`
- Modify: `src-tauri/src/invoke_handler.rs:76`
- Modify: tous les littéraux `AgentSession` et `AgentSessionMeta` signalés par `cargo check --all-targets`
- Modify: `src/types/agent-session.ts`
- Create: `src/hooks/use-session-fast-mode.ts`
- Create: `src/hooks/__tests__/use-session-fast-mode.test.tsx`
- Modify: `src/hooks/use-agent-sessions.ts`
- Modify: `src/hooks/__tests__/use-agent-sessions.test.ts`

**Interfaces:**
- Consumes: `id: String`, `enabled: bool` depuis IPC.
- Produces: `AgentSession.fast_mode_enabled: bool`, `AgentSessionMeta.fast_mode_enabled: bool`, `set_session_fast_mode(id, enabled) -> Result<bool, String>`.

- [ ] **Step 1: Écrire les tests rouges de lecture et sérialisation**

Dans `session_fast_mode_tests.rs`, tester un vrai aller-retour serde :

```rust
let legacy = serde_json::from_value::<AgentSession>(legacy_session_json()).unwrap();
assert!(!legacy.fast_mode_enabled);

let mut enabled = legacy;
enabled.fast_mode_enabled = true;
let json = serde_json::to_value(&enabled).unwrap();
assert_eq!(json["fast_mode_enabled"], true);

enabled.fast_mode_enabled = false;
let json = serde_json::to_value(&enabled).unwrap();
assert_eq!(json["fast_mode_enabled"], false);
```

Le test doit prouver que `false` est sérialisé, pas transformé en champ absent.

- [ ] **Step 2: Écrire les tests rouges de cycle de vie**

Ajouter des cas qui exigent :

```rust
assert!(!new_session.fast_mode_enabled);
assert!(!clone.fast_mode_enabled);
assert!(!subagent.fast_mode_enabled);
assert!(!heartbeat.fast_mode_enabled);
assert!(!gateway.fast_mode_enabled);
```

Puis activer la session A et vérifier que la session B reste fausse, qu'un changement de modèle de A conserve `true`, et que la reconstruction de `index.json` expose encore `true`.

Traverser aussi les vraies fonctions IPC Rust : appeler `set_session_fast_mode` sur A, puis `list_agent_sessions`, et exiger que la métadonnée sérialisée de A porte `true` tandis que B reste `false`. Simuler un échec de sauvegarde via un writer injecté sous `#[cfg(test)]`; l'appel doit renvoyer une erreur et une nouvelle lecture du fichier doit conserver l'ancienne valeur.

- [ ] **Step 3: Vérifier les échecs**

Run:

```bash
cd src-tauri
cargo test session_fast_mode --lib
cargo test session_index --lib
cargo test clone_session --lib
cargo test subagent_inheritance --lib
```

Expected: compilation ou assertions rouges car la préférence n'existe pas.

- [ ] **Step 4: Ajouter la préférence sans seconde autorité**

Ajouter aux deux structs, sans `skip_serializing_if` :

```rust
#[serde(default)]
pub fast_mode_enabled: bool,
```

`meta_from_session` recopie la valeur et `index_meta_drifted` la compare. `create_full` continue d'avoir sa signature actuelle et appelle une fonction privée dont le dernier argument est `false`. Ajouter une voie réservée à la création interactive :

```rust
pub async fn create_with_project_and_fast_mode(
    name: &str,
    model: &str,
    provider: &str,
    project_id: Option<String>,
    fast_mode_enabled: bool,
) -> Result<AgentSession, String>
```

Cette fonction construit la session avec la bonne valeur avant sa première écriture. `create_full`, `create_gateway`, scheduler, heartbeat et sous-agents gardent `false`.

Dans `clone_session_build.rs`, écrire explicitement :

```rust
clone.fast_mode_enabled = false;
```

La raison doit rester à côté de la ligne : un clone est une nouvelle session et ne reprend pas une préférence de coût/vitesse.

- [ ] **Step 5: Centraliser tous les writers lecture → mutation → sauvegarde**

Dans `session_store_updates.rs`, ajouter un guichet verrouillé :

```rust
pub(super) async fn update_locked<R>(
    id: &str,
    mutate: impl FnOnce(&mut AgentSession) -> R,
) -> Result<R, String> {
    validate_session_id(id)?;
    let lock = super::session_store::lock_session(id).await;
    let _guard = lock.lock().await;
    let mut session = get(id).await?;
    let result = mutate(&mut session);
    save(&session).await?;
    Ok(result)
}

pub async fn update_fast_mode(id: &str, enabled: bool) -> Result<bool, String> {
    update_locked(id, |session| {
        session.fast_mode_enabled = enabled;
        session.fast_mode_enabled
    })
    .await
}
```

Faire passer `update_model`, `update_reasoning` et `session_store::rename` par ce même guichet afin qu'une mutation simultanée ne réécrive pas une ancienne valeur Fast. Le changement de modèle ne touche jamais `fast_mode_enabled`.

Faire ensuite l'inventaire complet avant de poursuivre :

```bash
rg -n "session_store::get|\bget\(id\).*await" src-tauri/src/services/agent_local src-tauri/src/commands
rg -n "session_store::save|\bsave\(&session\)" src-tauri/src/services/agent_local src-tauri/src/commands
```

Classer chaque writer trouvé : soit il passe par `update_locked`, soit il possède déjà le même `lock_session(id)` sur toute la séquence lecture → écriture. Ajouter dans `session_store_update_race_tests.rs` une course entre `update_fast_mode` et `rename`, puis une assertion de structure sur la liste finie des writers. Retirer l'appel verrouillé de `rename`, `update_model`, `update_reasoning` ou `save_agent_session` doit faire échouer un test ou la compilation; ne pas laisser l'adoption à moitié.

- [ ] **Step 6: Protéger la sauvegarde générale périmée**

Dans `save_agent_session`, prendre le même verrou avant `get`, conserver les champs possédés par Rust, puis sauvegarder :

```rust
session.fast_mode_enabled = current.fast_mode_enabled;
session.working_dir = current.working_dir;
session.working_dir_managed = current.working_dir_managed;
```

Ajouter un test : un objet frontend chargé avec `false`, suivi d'une activation Rust à `true`, ne peut pas réécrire `false` lors d'un `save_agent_session` tardif.

- [ ] **Step 7: Exposer une commande étroite**

Ajouter :

```rust
#[tauri::command]
pub async fn set_session_fast_mode(id: String, enabled: bool) -> Result<bool, String> {
    crate::services::agent_local::session_user_write::ensure_allowed(&id).await?;
    session_store::update_fast_mode(&id, enabled).await
}
```

Enregistrer la commande dans `invoke_handler.rs`. Étendre `create_agent_session` avec `fast_mode_enabled: Option<bool>` et utiliser `unwrap_or(false)` uniquement à cette frontière. La commande appelle `create_with_project_and_fast_mode`; elle ne crée pas d'abord une session fausse pour la corriger dans une seconde écriture.

- [ ] **Step 8: Transporter la valeur au frontend**

Ajouter `fast_mode_enabled: boolean` aux interfaces `AgentSession` et `AgentSessionMeta`. Dans `useAgentSessions.create`, ajouter le dernier argument `fastModeEnabled = false` et envoyer :

```ts
fastModeEnabled,
```

Créer `use-session-fast-mode.ts` afin de ne pas faire dépasser `use-agent-sessions.ts`. `pendingIdsRef` est l'autorité transitoire; un compteur React sert uniquement à redessiner. La collection est bornée :

```ts
const MAX_PENDING_FAST_MODE_MUTATIONS = 32;
const pendingIdsRef = useRef(new Set<string>());
const [, refreshPendingState] = useReducer((value: number) => value + 1, 0);
```

`setFastMode(id, enabled)` ignore un doublon. Si 32 identifiants distincts sont déjà en vol, il refuse la nouvelle mutation sans modifier l'état visible et sans afficher `errors.sessionSaveFailed`, qui décrirait faussement un échec disque; ce cas de capacité interne est couvert par un test et ne contient aucun identifiant dans les traces. Sinon, il ajoute l'identifiant avant l'IPC, appelle `invoke<boolean>("set_session_fast_mode", { id, enabled })`, attend `refresh()`, puis retire l'identifiant dans `finally`. Une vraie erreur IPC affiche `errors.sessionSaveFailed`; aucune branche ne modifie localement la préférence persistée. Exposer `isFastModePending(id)` au lieu de rendre le `Set` mutable aux composants.

- [ ] **Step 9: Vérifier persistance, concurrence et frontière IPC**

Run:

```bash
cd src-tauri
cargo test session_fast_mode --lib
cargo test session_store_update_race --lib
cargo test session_index --lib
cargo test clone_session --lib
cargo test subagent_inheritance --lib
cargo check --all-targets
cd ..
npx vitest run src/hooks/__tests__/use-agent-sessions.test.ts
npx vitest run src/hooks/__tests__/use-session-fast-mode.test.tsx
npx tsc --noEmit
```

Expected: A/B indépendantes, redémarrage simulé fidèle, clone/sous-agent faux, renommage et autres writers concurrents sans perte, plafond sans faux message de sauvegarde et sauvegarde périmée neutralisée.

- [ ] **Step 10: Commit**

```bash
git add src-tauri/src/services/agent_local src-tauri/src/commands/agent_sessions.rs src-tauri/src/invoke_handler.rs src/types/agent-session.ts src/hooks/use-agent-sessions.ts src/hooks/use-session-fast-mode.ts src/hooks/__tests__/use-agent-sessions.test.ts src/hooks/__tests__/use-session-fast-mode.test.tsx
git commit -m "feat(sessions): persister le mode rapide par conversation"
```

---

### Task 3: Capturer le tier une fois et couvrir OpenAI API

**Files:**
- Create: `src-tauri/src/services/llm/fast_mode.rs`
- Modify: `src-tauri/src/services/llm/mod.rs`
- Modify: `src-tauri/src/commands/agent_chat_task/api.rs:5`
- Modify: `src-tauri/src/services/llm/agent_loop.rs:15`
- Modify: `src-tauri/src/services/llm/agent_loop_request.rs:9`
- Modify: `src-tauri/src/services/llm/retry.rs:39`
- Modify: `src-tauri/src/services/llm/stream.rs:12`
- Modify: `src-tauri/src/services/llm/stream_http.rs:6`
- Modify: `src-tauri/src/services/llm/stream_http_payload.rs:4`
- Modify: `src-tauri/src/services/llm/stream_http_tests.rs`
- Modify: `src-tauri/src/services/llm/stream_silent.rs:11`
- Modify: `src-tauri/src/services/llm/stream_silent_consume_tests.rs`
- Modify: `src-tauri/src/services/llm/agent_loop_compression.rs:10`
- Modify: `src-tauri/src/services/llm/compress_hook.rs:12`
- Modify: `src-tauri/src/services/agent_local/tool_executor_compression.rs:5`
- Modify: `src-tauri/src/commands/agent_chat_task/compress.rs:89`
- Modify: `src-tauri/src/services/agent_local/clone_session.rs:150`

**Interfaces:**
- Consumes: préférence persistée et capacité backend.
- Produces: `FastModeRequest::{Unsupported, Standard, Fast}`, copiable et immuable pendant une génération.

- [ ] **Step 1: Écrire les tests rouges de la décision fermée**

Tester la matrice exacte :

```rust
assert_eq!(FastModeRequest::for_api(false, false), FastModeRequest::Standard);
assert_eq!(FastModeRequest::for_api(false, true), FastModeRequest::Standard);
assert_eq!(FastModeRequest::for_api(true, false), FastModeRequest::Standard);
assert_eq!(FastModeRequest::for_api(true, true), FastModeRequest::Fast);
assert_eq!(FastModeRequest::for_codex(false, true), FastModeRequest::Unsupported);
assert_eq!(FastModeRequest::for_codex(true, false), FastModeRequest::Standard);
assert_eq!(FastModeRequest::for_codex(true, true), FastModeRequest::Fast);
assert_eq!(FastModeRequest::Standard.api_value(), Some("default"));
assert_eq!(FastModeRequest::Fast.api_value(), Some("fast"));
assert_eq!(FastModeRequest::Standard.codex_value(), None);
assert_eq!(FastModeRequest::Fast.codex_value(), Some("priority"));
```

La séparation des constructeurs est normative : l'API doit neutraliser le défaut du projet même quand la capacité n'est pas publiée, tandis qu'OAuth doit omettre le champ lorsqu'il n'est pas actif.

- [ ] **Step 2: Implémenter l'enum et les deux sérialisations**

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FastModeRequest {
    Unsupported,
    Standard,
    Fast,
}

impl FastModeRequest {
    pub const fn for_api(supported: bool, enabled: bool) -> Self {
        if supported && enabled {
            Self::Fast
        } else {
            Self::Standard
        }
    }

    pub const fn for_codex(supported: bool, enabled: bool) -> Self {
        match (supported, enabled) {
            (false, _) => Self::Unsupported,
            (true, false) => Self::Standard,
            (true, true) => Self::Fast,
        }
    }

    pub const fn api_value(self) -> Option<&'static str> {
        match self {
            Self::Unsupported => None,
            Self::Standard => Some("default"),
            Self::Fast => Some("fast"),
        }
    }

    pub const fn codex_value(self) -> Option<&'static str> {
        match self {
            Self::Unsupported | Self::Standard => None,
            Self::Fast => Some("priority"),
        }
    }

    pub const fn fast_requested(self) -> bool {
        matches!(self, Self::Fast)
    }
}
```

Ajouter `for_session(session_id, provider_id, model) -> Result<Self, String>`. Il charge la préférence une fois. Pour `providers::openai::PROVIDER_ID`, appeler `for_api(provider_model_lookup::supports_fast_mode(...), enabled)` : un modèle non annoncé produit donc `Standard`, jamais `Unsupported`. Pour `codex_client::PROVIDER_ID`, transformer un catalogue valide avec `for_codex(capability, enabled)` et transformer une erreur de catalogue en `Unsupported`; ne pas utiliser `?` sur cette lecture. Tout autre provider renvoie `Unsupported` sans lire de tier frontend. Une erreur de lecture de session reste une erreur; une erreur de catalogue OAuth conserve la préférence et échoue fermé uniquement pour Fast.

Ajouter `fn standard_for_internal(provider_id) -> Self` : un appel interne OpenAI API ou Codex OAuth reçoit `Standard`, jamais la préférence d'une autre session; un autre provider reçoit `Unsupported`. Il réutilise les deux constantes provider et ne consulte pas de catalogue, car aucun appel interne ne peut demander Fast.

- [ ] **Step 3: Écrire les tests payload API rouges**

Dans `stream_http_tests.rs`, construire trois `RequestConfig` et exiger :

```rust
assert_eq!(fast_payload["service_tier"], "fast");
assert_eq!(standard_payload["service_tier"], "default");
assert_eq!(unadvertised_openai_payload["service_tier"], "default");
assert!(other_provider_payload.get("service_tier").is_none());
```

Ajouter une assertion négative sur `auto`, `flex`, `priority` et `ultrafast` dans le chemin API.

- [ ] **Step 4: Propager la capture sans relecture**

Dans `commands/agent_chat_task/api.rs`, calculer une fois avant `run_agent_loop` :

```rust
let fast_mode = llm::fast_mode::for_session(
    &params.session_id,
    &params.provider,
    &params.model,
)
.await?;
```

Ajouter un paramètre `FastModeRequest` à `run_agent_loop`, `ApiRequestParams`, `retry_stream`, `stream_chat_no_done` et `RequestConfig`. Le retry réutilise la valeur reçue; il ne rappelle jamais `for_session`.

Dans `build_chat_payload` :

```rust
if let Some(value) = cfg.fast_mode.api_value() {
    payload["service_tier"] = value.into();
}
```

La valeur `FastModeRequest` est `Standard` pour toute requête OpenAI API sans Fast effectif et `Unsupported` pour les autres providers; le payload ne fait aucune détection par nom.

- [ ] **Step 5: Propager la même capture aux compressions du tour**

Ajouter le champ à `LoopCompression` et à `ToolCompressionProvider::Cloud` :

```rust
fast_mode: crate::services::llm::fast_mode::FastModeRequest,
```

Le chemin auto-compression, l'interruption pour compression et la compression pendant les outils transmettent cette copie. La commande `/compress` calcule sa propre capture au début de sa génération. Le résumé interne d'un clone utilise `standard_for_internal` et n'hérite jamais du parent.

- [ ] **Step 6: Tester la stabilité pendant retries et compression**

Ajouter un test qui démarre avec `Fast`, modifie la session persistée vers `false` après la première tentative simulée, puis exige encore `"fast"` sur le retry et la compression de cette génération. La requête suivante doit produire `"default"`. Ajouter aussi une requête OpenAI API sur un modèle non annoncé et exiger `"default"`, quelle que soit la préférence persistée.

Run:

```bash
cd src-tauri
cargo test fast_mode --lib
cargo test stream_http --lib
cargo test retry --lib
cargo test compress_hook --lib
cargo test tool_executor_compression --lib
```

- [ ] **Step 7: Vérifier que les appels internes ne prennent pas Fast**

Tester le résumé de clone et tout appel silencieux sans génération utilisateur : pour OpenAI API, le champ vaut `default`; pour Codex OAuth et les autres providers, il est absent.

- [ ] **Step 8: Commit**

```bash
git add src-tauri/src/services/llm src-tauri/src/services/agent_local/tool_executor_compression.rs src-tauri/src/commands/agent_chat_task src-tauri/src/services/agent_local/clone_session.rs
git commit -m "feat(openai): capturer et envoyer Fast par clé API"
```

---

### Task 4: Brancher Codex OAuth sur le payload canonique HTTP/WS

**Files:**
- Modify: `src-tauri/src/services/codex_client/types.rs:5`
- Modify: `src-tauri/src/services/codex_client/request.rs:10`
- Create: `src-tauri/src/services/codex_client/routing_hint.rs`
- Modify: `src-tauri/src/services/codex_client/mod.rs`
- Modify: `src-tauri/src/services/codex_client/request_http.rs:72`
- Modify: `src-tauri/src/services/codex_client/request_http_tests.rs`
- Modify: `src-tauri/src/services/codex_client/stream.rs:18`
- Modify: `src-tauri/src/services/codex_client/websocket.rs:31`
- Modify: `src-tauri/src/services/codex_client/websocket_tests.rs`
- Modify: `src-tauri/src/services/codex_client/websocket_connect.rs`
- Modify: `src-tauri/src/services/codex_client/websocket_connect_tests.rs`
- Modify: `src-tauri/src/services/codex_client/stream_silent.rs:13`
- Modify: `src-tauri/src/services/codex_client/stream_silent_tests.rs`
- Modify: appels Codex dans `src-tauri/src/services/llm/stream.rs` et `src-tauri/src/services/llm/stream_silent.rs`

**Interfaces:**
- Consumes: `FastModeRequest` capturé dans Task 3.
- Produces: `CodexRequest.service_tier: Option<String>`, unique source du corps et de `x-codex-routing-hint` pour HTTP et WebSocket.

- [ ] **Step 1: Écrire les tests rouges de l'objet canonique**

Ajouter aux tests de `build_codex_request` :

```rust
let fast = build_codex_request(
    "gpt-5.6-sol", &[], &[], None, None, FastModeRequest::Fast,
);
let standard = build_codex_request(
    "gpt-5.6-sol", &[], &[], None, None, FastModeRequest::Standard,
);
let unsupported = build_codex_request(
    "gpt-5.4-mini", &[], &[], None, None, FastModeRequest::Unsupported,
);

assert_eq!(serde_json::to_value(&fast).unwrap()["service_tier"], "priority");
assert!(serde_json::to_value(&standard).unwrap().get("service_tier").is_none());
assert!(serde_json::to_value(&unsupported).unwrap().get("service_tier").is_none());
```

Ajouter aussi les assertions de routage issues des mêmes objets :

```rust
assert_eq!(routing_hint::for_request(&fast).unwrap(), "model=gpt-5.6-sol;tier=priority");
assert_eq!(routing_hint::for_request(&standard).unwrap(), "model=gpt-5.6-sol");
assert_eq!(routing_hint::for_request(&unsupported).unwrap(), "model=gpt-5.4-mini");
```

- [ ] **Step 2: Ajouter le champ à `CodexRequest`**

```rust
#[serde(skip_serializing_if = "Option::is_none")]
pub service_tier: Option<String>,
```

`build_codex_request` reçoit obligatoirement `FastModeRequest` et remplit :

```rust
service_tier: fast_mode.codex_value().map(str::to_string),
```

Ne pas injecter de tier dans `request_http.rs` ou `websocket.rs`; ces modules sérialisent l'objet canonique.

Créer `routing_hint.rs` avec une seule fonction `for_request(&CodexRequest) -> Result<String, String>`. Elle valide `request.model` avec l'autorité existante `runtime_models::valid_model_id`, accepte seulement `None` ou `Some("priority")`, borne la sortie à 160 octets — 128 pour l'identifiant validé plus le préfixe et le tier — et renvoie `provider_configuration_invalid` en cas d'écart. La raison reste dans le module : le client Codex officiel envoie cet en-tête sur les deux transports, et le dériver du payload empêche le corps et le routage de diverger.

- [ ] **Step 3: Faire échouer les tests de branchement du corps et de l'en-tête HTTP/WS**

Dans `websocket_tests.rs`, appeler le vrai `build_payload(&CodexRequest)` et vérifier `type: "response.create"` plus le tier. Dans `request_http_tests.rs`, capturer le body reçu par le serveur de test et vérifier le même tier. Ces tests doivent traverser les fonctions appelées par le runtime, pas seulement sérialiser un objet fabriqué à la main.

Faire aussi traverser la pose réelle de `x-codex-routing-hint` : le serveur HTTP de test reçoit `model=gpt-5.6-sol;tier=priority`, et `connect_loopback_at` capture la même valeur pendant la poignée de main WebSocket. Les cas Standard reçoivent `model=gpt-5.6-sol` sans `;tier=`. Retirer l'appel qui pose l'en-tête dans `send_once` ou `connect_at` doit rendre le test correspondant rouge.

- [ ] **Step 4: Propager le paramètre obligatoire**

Ajouter `fast_mode: FastModeRequest` aux signatures `post_codex_stream`, `post_codex_stream_with_timeout`, `stream_chat_with_budget`, `websocket::stream_chat` et `collect_chat_silent_for_compression`. Le fallback WebSocket → HTTP transmet exactement la même copie.

HTTP construit une fois `CodexRequest`, puis passe à `request_http::post` le corps sérialisé et la valeur de `routing_hint::for_request(&request)`. WebSocket construit le même type, puis passe cette valeur à `websocket_connect::connect`; `request_http` et `websocket_connect` posent seulement la chaîne déjà validée. Aucun transport ne reconstruit `model=...;tier=...`.

Le compilateur doit empêcher l'oubli : aucune surcharge et aucune valeur par défaut sur ces chemins.

- [ ] **Step 5: Vérifier les frontières OAuth inchangées**

Les tests existants doivent toujours prouver :

```rust
assert_eq!(request_origin, "https://chatgpt.com");
assert!(has_header("chatgpt-account-id"));
assert!(has_header("originator"));
assert!(!body.contains("access_token"));
```

Run:

```bash
cd src-tauri
cargo test codex_client::request --lib
cargo test request_http --lib
cargo test websocket --lib
cargo test websocket_connect --lib
cargo test stream_silent --lib
cargo test codex_client --lib
```

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/services/codex_client src-tauri/src/services/llm/stream.rs src-tauri/src/services/llm/stream_silent.rs
git commit -m "feat(codex): transmettre Fast sur HTTP et WebSocket"
```

---

### Task 5: Observer le tier servi et classer les refus sans replay

**Files:**
- Modify: `src-tauri/src/services/provider_usage/request_journal.rs:15`
- Modify: `src-tauri/src/services/provider_usage/request_journal_validation.rs`
- Modify: `src-tauri/src/services/provider_usage/request_journal_store.rs`
- Modify: `src-tauri/src/services/provider_usage/request_journal_tests.rs`
- Modify: `src-tauri/src/services/provider_usage/request_measurement.rs:6`
- Modify: `src-tauri/src/services/provider_usage/request_measurement_tests.rs`
- Modify: `src-tauri/src/services/llm/stream_metrics.rs:8`
- Modify: `src-tauri/src/services/llm/stream_consume.rs:45`
- Modify: `src-tauri/src/services/llm/stream_silent_consume.rs`
- Modify: `src-tauri/src/services/codex_client/stream_measurement.rs:7`
- Modify: `src-tauri/src/services/codex_client/stream_accumulator_tests.rs`
- Modify: `src-tauri/src/services/codex_client/stream_silent.rs`
- Modify: `src-tauri/src/services/llm/provider_error.rs:7`
- Modify: `src-tauri/src/services/llm/provider_error_tests.rs`
- Modify: `src-tauri/src/services/llm/stream_http.rs:136`
- Modify: `src-tauri/src/services/llm/stream_http_classification_tests.rs`
- Modify: `src-tauri/src/services/codex_client/http_error.rs:25`
- Modify: `src-tauri/src/services/codex_client/http_error_tests.rs`
- Modify: `src-tauri/src/services/llm/retry.rs:25`
- Modify: `src/types/provider-usage.ts`
- Modify: `src/lib/agent-error-codes.ts`
- Modify: `src/lib/agent-error-codes.test.ts`
- Modify: sept fichiers `src/i18n/*.json`

**Interfaces:**
- Consumes: `FastModeRequest.fast_requested()` et les champs réponse `service_tier` / `response.service_tier`.
- Produces: `fast_requested: bool`, `service_tier_served: "fast" | "default" | "unknown"`, erreur stable `service_tier_unavailable`.

- [ ] **Step 1: Écrire les tests rouges de normalisation du tier servi**

Définir et tester :

```rust
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceTierServed {
    Fast,
    Default,
    #[default]
    #[serde(other)]
    Unknown,
}
```

Matrice :

```rust
assert_eq!(served_tier("fast"), ServiceTierServed::Fast);
assert_eq!(served_tier("priority"), ServiceTierServed::Fast);
assert_eq!(served_tier("default"), ServiceTierServed::Default);
assert_eq!(served_tier("auto"), ServiceTierServed::Unknown);
assert_eq!(served_tier("ultrafast"), ServiceTierServed::Unknown);
```

- [ ] **Step 2: Étendre le journal borné de façon rétrocompatible**

Ajouter à `ProviderRequestMetric`, sous `#[serde(default)]` déjà appliqué à la struct :

```rust
pub fast_requested: bool,
pub service_tier_served: ServiceTierServed,
```

`RequestMeasurementContext` reçoit `fast_mode: FastModeRequest`. `start` initialise `fast_requested`; `observe_response_metadata` lit :

```rust
value.get("service_tier")
    .or_else(|| value.pointer("/response/service_tier"))
```

Une observation `Unknown` n'efface jamais une observation `Fast` ou `Default` déjà reçue. Les limites existantes restent `REQUEST_LIMIT = 1_000`, `SESSION_REQUEST_LIMIT = 200`, `SNAPSHOT_LIMIT = 50`.

- [ ] **Step 3: Brancher l'observation sur les parseurs réellement exécutés**

Le chemin Chat Completions appelle déjà `measurement.observe_response_metadata(&value)` dans `stream_consume` et `stream_silent_consume`; étendre cette méthode plutôt que créer un second parseur.

Dans `codex_client::stream_measurement::apply`, appeler l'observation avant `accumulator.apply`, ce qui couvre HTTP et WebSocket avec le même code. Dans le flux silencieux, observer aussi chaque événement final.

- [ ] **Step 4: Écrire les tests rouges du refus de tier**

Tester uniquement les champs structurés, jamais le texte libre :

```rust
let by_param = r#"{"error":{"code":"invalid_request_error","param":"service_tier","message":"private"}}"#;
let by_code = r#"{"error":{"code":"unsupported_service_tier","message":"private"}}"#;

assert!(is_service_tier_rejection(by_param));
assert!(is_service_tier_rejection(by_code));
assert!(!is_service_tier_rejection(
    r#"{"error":{"message":"service tier unavailable"}}"#
));
```

`param == "service_tier"` est le critère principal documenté par la forme des erreurs API. `unsupported_service_tier` est conservé comme hypothèse défensive clairement commentée, pas comme valeur officielle; la campagne réelle doit confirmer ou retirer cette branche. Aucun message libre ne participe à la classification.

Ajouter `ProviderErrorCode::ServiceTierUnavailable` → `"service_tier_unavailable"`. Le classifieur générique et `codex_client::http_error` utilisent ce code avant le rejet générique. Les événements Responses examinent `/response/error/param` en premier, puis `/response/error/code` avec la petite liste fermée et documentée comme hypothèse.

- [ ] **Step 5: Verrouiller l'absence de replay Standard**

Ajouter :

```rust
assert!(!is_retryable_error("service_tier_unavailable"));
```

et un test d'orchestration qui compte les envois : un refus structuré de tier produit exactement une requête, même avec des outils présents. Ne jamais modifier la requête en `default` après ce refus.

- [ ] **Step 6: Propager le code utilisateur et les champs TypeScript**

Ajouter dans `KNOWN_ERROR_KEYS` :

```ts
service_tier_unavailable: "errors.serviceTierUnavailable",
```

Ajouter `fast_requested` et `service_tier_served` à `ProviderRequestMetric` TypeScript. Le test `agent-error-codes.test.ts` doit vérifier la nouvelle traduction dans les sept langues et `isKnownAgentErrorCode("service_tier_unavailable") === true`.

Ajouter les textes exacts, sans détail provider :

| Langue | `errors.serviceTierUnavailable` |
| --- | --- |
| fr | Le mode Rapide n'est pas disponible pour cette requête. Désactive-le ou choisis un modèle compatible. |
| en | Fast mode is not available for this request. Turn it off or choose a compatible model. |
| es | El modo Rápido no está disponible para esta solicitud. Desactívalo o elige un modelo compatible. |
| de | Der Schnellmodus ist für diese Anfrage nicht verfügbar. Deaktiviere ihn oder wähle ein kompatibles Modell. |
| it | La modalità Rapida non è disponibile per questa richiesta. Disattivala o scegli un modello compatibile. |
| zh | 快速模式不适用于此请求。请将其关闭或选择兼容的模型。 |
| ja | 高速モードはこのリクエストでは利用できません。無効にするか、対応モデルを選択してください。 |

- [ ] **Step 7: Verrouiller l'absence d'estimation de coût**

Relancer les tests existants qui exigent un coût indisponible pour GPT-5.6 et OAuth :

```bash
cd src-tauri
cargo test provider_usage::tests --lib
```

Ne modifier ni `provider_usage/pricing.rs` ni les prix du sélecteur.

- [ ] **Step 8: Vérifier observation, erreurs et retries**

Run:

```bash
cd src-tauri
cargo test request_measurement --lib
cargo test request_journal --lib
cargo test stream_http_classification --lib
cargo test http_error --lib
cargo test retry --lib
cargo test codex_client --lib
cd ..
npx vitest run src/lib/agent-error-codes.test.ts
npx tsc --noEmit
```

- [ ] **Step 9: Commit**

```bash
git add src-tauri/src/services/provider_usage src-tauri/src/services/llm src-tauri/src/services/codex_client src/types/provider-usage.ts src/lib/agent-error-codes.ts src/lib/agent-error-codes.test.ts src/i18n
git commit -m "feat(openai): observer le tier Fast réellement servi"
```

---

### Task 6: Ajouter la ligne Rapide et la relier à la session affichée

**Files:**
- Create: `src/components/ui/fast-mode-icon.tsx`
- Modify: `src/components/ui/__tests__/icon-authority.test.ts`
- Modify: `src/components/agent-local/reasoning-selector.tsx:14`
- Modify: `src/components/agent-local/reasoning-selector.css:55`
- Modify: `src/components/agent-local/__tests__/reasoning-selector.test.tsx`
- Modify: `src/components/agent-local/model-controls.tsx:7`
- Modify: `src/hooks/use-agent-sessions.ts`
- Modify: `src/hooks/use-agent-local-tab.ts:20`
- Modify: `src/hooks/use-session-actions.ts:15`
- Modify: `src/hooks/__tests__/use-session-actions.test.tsx`
- Modify: `src/components/agent-local/agent-local-tab.tsx:35`
- Modify: `src/components/agent-local/agent-chat-detail.tsx`
- Modify: `src/components/agent-local/chat-view-types.ts`
- Modify: `src/components/agent-local/chat-view.tsx`
- Modify: `src/components/agent-local/chat-input-types.ts`
- Modify: `src/components/agent-local/chat-input.tsx`
- Modify: `src/components/agent-local/chat-input-actions-row.tsx`
- Modify: `src/components/agent-local/welcome-view.tsx`
- Create: `src/components/agent-local/__tests__/agent-local-fast-mode.test.tsx`
- Modify: `src/i18n/fr.json`
- Modify: `src/i18n/en.json`
- Modify: `src/i18n/es.json`
- Modify: `src/i18n/de.json`
- Modify: `src/i18n/it.json`
- Modify: `src/i18n/zh.json`
- Modify: `src/i18n/ja.json`
- Create: `src/i18n/openai-fast-translations.test.ts`
- Modify: `THIRD_PARTY_NOTICES.md`

**Interfaces:**
- Consumes: `AvailableModel.supports_fast_mode`, valeur persistée de la session affichée et commande `set_session_fast_mode`.
- Produces: ligne icône/libellé/switch en tête, brouillon d'accueil initialisé à `false`, mutation confirmée sans fermeture du menu.

- [ ] **Step 1: Vérifier la source fournie**

Avant la conversion :

```bash
shasum -a 256 /Users/kevinh/Downloads/typcn--flash-outline.svg
```

Expected: `1c9e53637a6c741a9bcd340d02711e46e8f1931a8bd101b6ad26cac29c4c5ae5`. Lire le `viewBox` et le `d` directement dans ce fichier. Le SVG et le PNG restent hors du dépôt : le dessin sera converti dans la primitive d'icône existante.

- [ ] **Step 2: Étendre le test d'autorité existant et vérifier le rouge**

Ne jamais recréer ni écraser `icon-authority.test.ts`. Ajouter au fichier existant :

```ts
it("ne déclare l'icône Rapide qu'à un seul endroit", () => {
  expect(declarationsOf("FastModeIcon")).toEqual([
    "/src/components/ui/fast-mode-icon.tsx",
  ]);
});
```

Run:

```bash
npx vitest run src/components/ui/__tests__/icon-authority.test.ts
```

Expected: FAIL car `fast-mode-icon.tsx` n'existe pas encore. Les garanties `SessionIcon` et `CopyIcon` déjà présentes restent intactes.

- [ ] **Step 3: Créer `FastModeIcon` avec la primitive existante**

Créer exactement :

```tsx
import { InlineIcon } from "./inline-icon";
import type { InlineIconProps } from "./inline-icon";

/* Éclair du mode Rapide. Typicons/Stephen Hutchings, CC BY-SA 4.0.
   Le tracé reste inchangé; InlineIcon est l'autorité des dessins qui suivent
   la couleur du texte et du thème. Notice dans THIRD_PARTY_NOTICES.md. */
export function FastModeIcon({ size = "var(--icon-xs)", className }: InlineIconProps) {
  return (
    <InlineIcon size={size} className={className} viewBox="0 0 24 24">
      <path fill="currentColor" d="M14.5 4h.005M14.5 4L12 10l5 2.898L9.5 20l2.5-6l-5-2.9zm0-2a2.02 2.02 0 0 0-1.379.551L5.624 9.646a2 2 0 0 0-.61 1.686c.072.626.437 1.182.982 1.498l3.482 2.021l-1.826 4.381a2.003 2.003 0 0 0 1.847 2.77c.498 0 .993-.186 1.375-.548l7.5-7.103a2 2 0 0 0 .61-1.685a2 2 0 0 0-.982-1.498L14.52 9.15l1.789-4.293A2 2 0 0 0 14.5 2" />
    </InlineIcon>
  );
}
```

Run le test d'autorité et un test de rendu qui exige `aria-hidden="true"`, `focusable="false"`, le `viewBox` original et `currentColor`. Expected: PASS. Aucun préfixe `fmi-` n'est créé; les styles de la ligne restent sous le propriétaire `rs-fast-*`.

- [ ] **Step 4: Ajouter la notice de licence**

Suivre le format existant de `THIRD_PARTY_NOTICES.md` avec ces données exactes :

```text
Typicons — flash-outline
Copyright © Stephen Hutchings
Source: https://icon-sets.iconify.design/typcn/flash-outline/
License: Creative Commons Attribution-ShareAlike 4.0 International
License URL: https://creativecommons.org/licenses/by-sa/4.0/
Changes made: the SVG was converted to a React component rendered through InlineIcon; the viewBox and path are unchanged, and currentColor is preserved so the drawing follows the active theme.
```

- [ ] **Step 5: Écrire les tests rouges du sélecteur**

Étendre `reasoning-selector.test.tsx` pour vérifier :

```ts
expect(screen.getAllByRole("switch")).toHaveLength(1);
expect(screen.getByRole("switch", { name: "Rapide" })).not.toBeChecked();
expect(screen.getByText("Rapide").closest("div")?.nextElementSibling).not.toBeNull();
```

Le test doit aussi vérifier l'ordre DOM : la ligne Rapide précède le premier bouton de raisonnement, un modèle incompatible ne la rend pas, le clic appelle `onFastModeChange(true)`, le menu reste ouvert et aucun texte `1.5`, `2.5`, `$`, `€`, `crédit` ou `credit` n'apparaît.

- [ ] **Step 6: Composer la ligne avec les primitives existantes**

Ajouter aux props :

```ts
fastModeEnabled: boolean;
fastModePending: boolean;
onFastModeChange: (enabled: boolean) => void;
```

Rendre avant `options.map` seulement lorsque `model?.supports_fast_mode === true` :

```tsx
<div className="rs-fast-row">
  <span className="rs-fast-label">
    <FastModeIcon className="rs-fast-icon" />
    <span>{t("agentLocal.fastMode")}</span>
  </span>
  <ToggleSwitch
    checked={fastModeEnabled}
    disabled={fastModePending}
    onCheckedChange={onFastModeChange}
    ariaLabel={t("agentLocal.fastMode")}
  />
</div>
```

Ne pas appeler `setOpen(false)` dans ce chemin. Le choix d'un niveau de raisonnement conserve son comportement actuel.

- [ ] **Step 7: Gérer le brouillon d'accueil sans polluer les sessions**

Dans `useAgentLocalTab` :

```ts
const [welcomeFastModeEnabled, setWelcomeFastModeEnabled] = useState(false);
```

`handleWelcomeSend` ne vit pas dans `useAgentLocalTab`. Ajouter `welcomeFastModeEnabled` et `setWelcomeFastModeEnabled` à `SessionActionsDeps` dans `src/hooks/use-session-actions.ts`, étendre `CreateFn` avec un 7e argument `fastModeEnabled?: boolean`, puis transmettre ce booléen dans l'appel `create(...)` de `handleWelcomeSend`. Après création réussie seulement, appeler `setWelcomeFastModeEnabled(false)`. Ajouter ces deux dépendances au tableau du `useCallback`. `useAgentLocalTab` possède le state et le transmet à `useSessionActions`; les autres voies de création omettent le 7e argument et restent donc fausses.

Un changement vers un modèle incompatible ne remet pas le brouillon à `false`; la ligne est seulement masquée, conformément au comportement des sessions.

- [ ] **Step 8: Relier la session réellement affichée**

Dans `AgentLocalTab`, utiliser `displaySessionId` et `displaySession`, pas seulement la session racine :

```tsx
const displayedFastMode = displaySession?.fast_mode_enabled ?? welcomeFastModeEnabled;
const displayedFastPending = displaySessionId
  ? isFastModePending(displaySessionId)
  : false;
```

La callback du chat lie explicitement l'identifiant affiché :

```tsx
onFastModeChange={(enabled) => {
  if (displaySessionId) void setFastMode(displaySessionId, enabled);
}}
```

Propager les trois props Fast à travers `AgentChatDetail`, `ChatView`, `ChatInput`, `ChatInputActionsRow` et `ModelControls`. L'accueil utilise le brouillon local. Cette distinction verrouille l'indépendance d'un onglet clone par rapport à sa racine.

- [ ] **Step 9: Ajouter les sept traductions et leur test**

Ajouter `agentLocal.fastMode` :

```text
fr Rapide
en Fast
es Rápido
de Schnell
it Rapido
zh 快速
ja 高速
```

Le test charge les sept JSON et exige aussi `errors.serviceTierUnavailable` et `errors.sessionSaveFailed` dans chaque langue.

- [ ] **Step 10: Tester l'intégration de deux sessions**

Dans `agent-local-fast-mode.test.tsx`, fournir une session racine active et un clone affiché avec des valeurs opposées. Exiger que le switch reflète le clone et que l'invocation utilise l'identifiant du clone. Changer l'onglet vers la racine doit refléter sa propre valeur sans écraser le clone.

Tester aussi : rechargement des métadonnées, erreur de persistance, modèle incompatible puis retour compatible, activation au clavier et `aria-checked`.

- [ ] **Step 11: Vérifier UI, types et licence**

Run:

```bash
npx vitest run src/components/agent-local/__tests__/reasoning-selector.test.tsx
npx vitest run src/components/agent-local/__tests__/agent-local-fast-mode.test.tsx
npx vitest run src/hooks/__tests__/use-agent-sessions.test.ts
npx vitest run src/hooks/__tests__/use-session-actions.test.tsx
npx vitest run src/i18n/openai-fast-translations.test.ts
npx vitest run src/components/ui/__tests__/icon-authority.test.ts
npx tsc --noEmit
npm run lint
```

- [ ] **Step 12: Commit**

```bash
git add src/components src/hooks src/i18n THIRD_PARTY_NOTICES.md
git commit -m "feat(ui): ajouter Rapide par session"
```

---

### Task 7: Prouver les branchements par mutations ciblées

**Files:**
- Modify: aucun fichier durable; les mutations s'exécutent dans un worktree jetable au commit courant.
- Evidence: sortie des commandes conservée dans la git note finale.

**Interfaces:**
- Consumes: tests des Tasks 1 à 6.
- Produces: preuve qu'un branchement retiré rend bien un test rouge.

- [ ] **Step 1: Créer un worktree jetable exact**

```bash
FAST_MUTATION_DIR="$(mktemp -d)"
git worktree add --detach "$FAST_MUTATION_DIR" HEAD
```

Ne jamais pointer cette variable vers le dépôt principal, `$HOME` ou `~`.

- [ ] **Step 2: Muter la persistance et constater le rouge**

Dans le worktree jetable, appliquer séparément puis annuler chaque mutation :

| Mutation | Test qui doit rougir |
| --- | --- |
| remettre `skip_serializing_if` sur `fast_mode_enabled` | `session_fast_mode` |
| retirer la copie dans `meta_from_session` | `session_index` |
| retirer la préservation dans `save_agent_session` | test de sauvegarde périmée |
| laisser `build_clone` hériter | `clone_session` |
| retirer le verrou partagé d'`update_model` | `session_store_update_race` |
| retirer le verrou partagé de `rename` | course rename/Fast dans `session_store_update_race` |

Après chaque mutation, exécuter uniquement le test indiqué et conserver son échec. Revenir au commit du worktree jetable avant la mutation suivante.

- [ ] **Step 3: Muter les capacités et constater le rouge**

| Mutation | Test qui doit rougir |
| --- | --- |
| faire accepter `starts_with("gpt-5")` | `provider_model_registry` |
| activer Fast dans le fallback OAuth | `model_catalog` |
| autoriser `additional_speed_tiers` sans `service_tiers.priority` | `model_catalog` |
| retirer `supports_fast_mode` du DTO OAuth | `use-available-models-oauth` ou compilation TS |

- [ ] **Step 4: Muter les payloads et constater le rouge**

| Mutation | Test qui doit rougir |
| --- | --- |
| omettre `default` à l'état Standard API | `stream_http` |
| émettre `default` à l'état Standard OAuth | `codex_client::request` |
| retirer le paramètre passé à `retry_stream` | compilation ou test retry |
| relire la session dans le retry | test de capture immuable |
| retirer le champ du payload WebSocket | `websocket` |
| retirer le champ du fallback HTTP | `request_http` |
| faire produire `priority` par l'API | `stream_http` |
| retirer `x-codex-routing-hint` du POST HTTP | `request_http` |
| retirer `x-codex-routing-hint` de la poignée de main WebSocket | `websocket_connect` |
| faire diverger le tier de l'en-tête et celui du corps | `routing_hint` |

- [ ] **Step 5: Muter erreurs, observation et UI**

| Mutation | Test qui doit rougir |
| --- | --- |
| traiter `default` comme Fast servi | `request_measurement` |
| rendre `service_tier_unavailable` retryable | `retry` |
| retirer l'appel `observe_response_metadata` du chemin Codex | `codex_client` |
| fermer le menu au clic du switch | `reasoning-selector` |
| utiliser `activeSessionId` au lieu de `displaySessionId` | `agent-local-fast-mode` |
| supprimer une traduction | `openai-fast-translations` |
| déclarer un second `FastModeIcon` | `icon-authority` |

- [ ] **Step 6: Supprimer uniquement le worktree jetable**

```bash
git worktree remove "$FAST_MUTATION_DIR"
rmdir "$FAST_MUTATION_DIR" 2>/dev/null || true
```

Expected: le dépôt principal est inchangé et chaque mutation annoncée a produit au moins un test rouge.

---

### Task 8: Vérifier globalement, ouvrir l'application et préparer les preuves réelles

**Files:**
- Force-add: `docs/providers/openai-fast/SPEC.md`
- Force-add: `docs/superpowers/plans/2026-08-22-phase-2-openai-fast.md`
- Create after cost approval: `docs/providers/openai-fast/fixtures/openai-api-gpt-5.6-sol-global-2026-08-23.json`
- Create after cost approval: `docs/providers/openai-fast/fixtures/codex-oauth-gpt-5.6-sol-global-2026-08-23.json`
- Modify: aucun autre fichier sauf correction révélée par une preuve rouge.

**Interfaces:**
- Consumes: implémentation et tests des Tasks 1 à 7.
- Produces: branche verte, validation visuelle et statut honnête des appels réels.

- [ ] **Step 1: Vérifier la taille et les doubles autorités**

Run:

```bash
find src src-tauri/src -type f \( -name '*.rs' -o -name '*.ts' -o -name '*.tsx' \) -print0 | xargs -0 wc -l | sort -n | tail -40
rg -n "startsWith\(.*gpt-5|starts_with\(.*gpt-5|ultrafast|service_tier" src src-tauri/src
```

Expected: aucun fichier modifié au-dessus de 230 lignes sans découpage; les seules valeurs Fast **émises** sont choisies dans `fast_mode.rs`, puis transportées par la sérialisation canonique. Les parseurs, diagnostics et journaux peuvent légitimement citer `service_tier` sans devenir une seconde autorité d'émission.

- [ ] **Step 2: Exécuter toute la validation automatique**

Run dans cet ordre :

```bash
npm run contracts:check
npx tsc --noEmit
npm run lint
npm test
cd src-tauri
cargo fmt -- --check
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
cargo test
cd ..
graphify update .
```

Expected: chaque commande est verte. Une seule commande rouge bloque la suite et sa sortie exacte est rapportée.

- [ ] **Step 3: Ouvrir réellement Beaver**

Run:

```bash
npm run tauri dev
```

Vérifier à l'écran :

1. GPT-5.6 API compatible montre Rapide en première ligne;
2. un modèle OpenAI incompatible ne montre aucune ligne;
3. un modèle Codex annoncé compatible montre la ligne;
4. un fallback OAuth sans preuve ne la montre pas;
5. sessions A et B gardent des états différents;
6. un onglet clone n'utilise pas l'état de sa racine;
7. passage incompatible puis retour compatible restaure l'état;
8. fermeture complète et réouverture de Beaver restaure chaque session;
9. clic Fast ne ferme pas le menu, choix de raisonnement le ferme comme avant;
10. clavier, focus visible et VoiceOver fonctionnent;
11. thèmes clair et sombre gardent l'icône et la bascule lisibles;
12. aucun prix, multiplicateur ou crédit n'apparaît.

Ne pas déclarer la validation visuelle terminée sans avoir vu ces douze états.

- [ ] **Step 4: Arrêter avant les appels facturés et demander l'accord**

Présenter à Kevin les deux appels proposés : OpenAI API et Codex OAuth, modèle exact, nombre de scénarios et risque de coût/crédits. Continuer seulement après son accord explicite.

Sans accord, écrire dans la git note : `Validation réelle provider non exécutée — accord de coût non donné.` Ne créer aucune fixture fictive.

- [ ] **Step 5: Après accord, exécuter `docs/providers/plan-de-tests.md` P01–P13 séparément**

Pour API puis OAuth, couvrir et enregistrer séparément :

- catalogue/capacité exacte;
- Standard explicite puis Fast;
- tier demandé et tier retourné;
- streaming texte;
- outil et continuation;
- second tour;
- image;
- compression dans le même tour;
- changement de bascule pendant un flux puis génération suivante;
- refus structuré de tier sans replay;
- 401 OAuth avec un seul refresh;
- 429 sans boucle et sans double effet;
- coût, crédits, confidentialité et sources officielles.

Dans la campagne OAuth, vérifier Fast et Standard sur HTTP puis WebSocket avec `x-codex-routing-hint`. Dans un worktree jetable seulement, retirer l'en-tête et répéter un scénario minimal afin d'établir si le backend route Fast avec le corps seul. Consigner le résultat sans transformer cette expérience en comportement de production.

Chaque fixture contient seulement : provider, modèle, région générale, date, forme du payload expurgée, statut, tier demandé, tier servi, usage borné, identifiant de requête autorisé et liens officiels. Aucun token, clé, email, `chatgpt-account-id`, prompt privé ou réponse complète.

- [ ] **Step 6: Force-add la documentation normative**

Le dépôt ignore `/docs/*`; ajouter intentionnellement :

```bash
git add -f docs/providers/openai-fast/SPEC.md docs/superpowers/plans/2026-08-22-phase-2-openai-fast.md
```

Après accord et appels réels seulement :

```bash
git add -f docs/providers/openai-fast/fixtures
```

- [ ] **Step 7: Commit de clôture documentaire et de preuves**

Si des fixtures réelles existent :

```bash
git commit -m "test(openai): consigner la validation réelle du mode rapide"
```

Sinon :

```bash
git commit -m "docs(openai): versionner le contrat du mode rapide"
```

- [ ] **Step 8: Relire le diff final**

Run:

```bash
git status --short
git diff main...HEAD --stat
git diff main...HEAD --check
git log --oneline main..HEAD
```

Expected: aucun fichier `docs/beaver-site/` dans le diff de la branche Fast, aucune ressource PNG, aucune clé ou token et seulement les responsabilités prévues.

- [ ] **Step 9: Ajouter la git note explicative sur le commit final**

```bash
FINAL_COMMIT="$(git rev-parse HEAD)"
git notes add -m "Phase 2 OpenAI Fast

Autorités:
- préférence durable: AgentSession.fast_mode_enabled
- capacité: ModelInfo.supports_fast_mode
- transport: FastModeRequest capturé une fois par génération

Fil:
- OpenAI API: fast/default
- Codex OAuth: priority/champ absent + x-codex-routing-hint
- OpenAI API non activé ou non annoncé: default
- Codex non compatible/autre provider: champ absent

Preuves:
- tests Rust/TS/frontend, clippy, fmt, lint et contracts:check verts
- mutations ciblées rouges
- validation visuelle clair/sombre et redémarrage: consigner le résultat réel
- appels provider réels: consigner exécuté ou non exécuté

Documentation reviewer:
- docs/providers/openai-fast/SPEC.md
- docs/superpowers/plans/2026-08-22-phase-2-openai-fast.md" "$FINAL_COMMIT"
git notes show "$FINAL_COMMIT"
```

Expected: la note affichée correspond exactement aux preuves réellement obtenues; remplacer aucune ligne par une affirmation non exécutée.

## Review Checklist

- [ ] Une session activée survit à un redémarrage.
- [ ] Deux sessions ont des états indépendants.
- [ ] Clone, sous-agent, heartbeat et gateway commencent désactivés.
- [ ] Un changement de modèle ne modifie pas la préférence.
- [ ] API actif produit `fast`; API inactif ou non annoncé produit `default`.
- [ ] OAuth actif/inactif produit `priority`/champ absent sur HTTP et WebSocket.
- [ ] `x-codex-routing-hint` correspond au modèle et au tier du corps sur HTTP et WebSocket.
- [ ] Modèle OAuth non compatible et autre provider n'ont pas de champ.
- [ ] Retry, outils et compression gardent la capture initiale.
- [ ] Une modification pendant le flux n'agit que sur la génération suivante.
- [ ] Le tier servi est observé sans être déduit du tier demandé.
- [ ] `service_tier_unavailable` n'est pas réessayé et reste traduit après rechargement.
- [ ] La ligne Rapide est première, accessible et sans texte commercial supplémentaire.
- [ ] L'asset est identique et la notice CC BY-SA 4.0 est présente.
- [ ] Aucun prix GPT-5.6 ou OAuth n'est inventé.
- [ ] Les mutations de branchement rendent les tests rouges.
- [ ] Les appels réels restent distingués des preuves locales.
