# Phase 1 — nouveaux modèles et xAI OAuth Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Afficher et utiliser GLM-5.3, Grok 4.6 et Gemini 3.7 Flash, garder les modèles encore officiels, actualiser les textes d’accueil et faire passer xAI OAuth par le transport d’abonnement réellement prévu.

**Architecture:** Les registres JSON restent l’autorité des capacités statiques et de l’ordre d’affichage. Le catalogue xAI OAuth devient une autorité dynamique distincte, bornée et validée, qui choisit un backend interne sans fournir d’URL libre. Les descriptions provider restent partagées par l’onboarding et les réglages. Les diagnostics Rust restent l’autorité des échecs persistés.

**Tech Stack:** Rust, Tauri 2, reqwest, serde, React 19, TypeScript, Vitest, API OpenAI-compatible Google/Z.AI/xAI, proxy OAuth xAI.

**Spec:** `docs/providers/releases-2026-08/SPEC.md`

## Global Constraints

- Ne pas commencer la phase 2 avant la validation complète de cette phase.
- Conserver Grok 4.5, Gemini 3.5 et Gemini 3.6 tant que leurs documentations officielles les publient.
- Ne pas afficher de prix pour les nouveaux modèles et ne pas leur attribuer `[Free]` sans gratuité complète explicitement vérifiée.
- Ne jamais envoyer un jeton xAI OAuth vers `api.x.ai`, y compris pour le catalogue.
- Ne jamais accepter une URL, un backend, un en-tête ou un identifiant de modèle non validé depuis le frontend ou le catalogue distant.
- Réponses et collections distantes bornées ; aucun body provider brut ni identité de compte dans les logs, l’IPC ou les fixtures.
- Textes visibles dans les sept langues et erreurs via des codes stables traduits.
- Un fichier de code garde une responsabilité et reste sous 230 lignes ; créer des modules dédiés avant de dépasser le seuil.
- Écrire chaque test de régression avant le changement correspondant.
- Après chaque modification de code, exécuter `graphify update .`.

---

### Task 1: Étendre le registre avec les capacités officielles

**Files:**
- Modify: `src-tauri/src/services/llm/provider_model_registry.rs`
- Modify: `src-tauri/src/services/llm/provider_model_registry_tests.rs`
- Modify: `src-tauri/src/services/llm/openai_compat_models.rs`
- Modify: `src-tauri/src/services/llm/openai_compat_parsing_tests.rs`
- Modify: `src-tauri/resources/provider-models/zai.json`
- Modify: `src-tauri/resources/provider-models/xai.json`
- Modify: `src-tauri/resources/provider-models/google.json`

**Interfaces:**
- `ProviderModelConfig.reasoning_modes: Vec<String>`
- `ProviderModelConfig.default_reasoning_mode: Option<String>`
- `ModelInfo` reçoit ces valeurs sans les recalculer par nom quand le registre les publie.

- [ ] **Step 1: Écrire les tests rouges du schéma**

Ajouter des fixtures qui refusent plus de huit modes, les doublons, un mode inconnu et un défaut absent de la liste. Ajouter les assertions de catalogue :

```rust
assert_eq!(glm_53.reasoning_modes, ["low", "high", "max"]);
assert_eq!(glm_53.default_reasoning_mode.as_deref(), Some("max"));
assert_eq!(grok_46.reasoning_modes, ["low", "medium", "high", "xhigh"]);
assert_eq!(grok_46.default_reasoning_mode.as_deref(), Some("high"));
assert_eq!(gemini_37.reasoning_modes, ["low", "medium", "high"]);
assert_eq!(gemini_37.default_reasoning_mode.as_deref(), Some("medium"));
```

- [ ] **Step 2: Lancer les tests et constater l’échec**

Run: `cd src-tauri && cargo test provider_model_registry --lib && cargo test openai_compat_parsing --lib`

Expected: échec car le registre ne porte pas encore ces métadonnées ou les modèles n’existent pas.

- [ ] **Step 3: Ajouter une métadonnée fermée et validée**

Étendre `ProviderModelConfig`, borner les listes à huit valeurs, accepter seulement les efforts déjà connus par Beaver et vérifier que le défaut appartient à la liste. Faire de cette métadonnée l’autorité prioritaire ; le résolveur par nom ne sert que de repli.

- [ ] **Step 4: Mettre à jour les trois registres**

Ajouter en tête `glm-5.3`, `grok-4.6` et `gemini-3.7-flash` avec les capacités de la SPEC et les URLs officielles dans `source_urls`. Mettre `verified_at` au `2026-08-22`. Garder les entrées antérieures et ne renseigner aucun `is_free` pour les trois nouveautés.

- [ ] **Step 5: Vérifier le registre**

Run: `cd src-tauri && cargo test provider_model_registry --lib && cargo test openai_compat_parsing --lib`

Expected: tous les tests sont verts et l’ordre commence par les trois nouveaux modèles dans leur provider respectif.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/resources/provider-models src-tauri/src/services/llm
git commit -m "feat(models): référencer GLM 5.3, Grok 4.6 et Gemini 3.7"
```

---

### Task 2: Envoyer uniquement les niveaux de raisonnement supportés

**Files:**
- Modify: `src-tauri/src/services/reasoning.rs`
- Modify: `src-tauri/src/services/llm/stream_reasoning.rs`
- Modify: `src-tauri/src/services/llm/stream_http_tests.rs`
- Modify: `src-tauri/src/services/llm/providers/xai.rs`

**Interfaces:**
- `reasoning::supported_modes(provider, model, supports_thinking)` lit d’abord le registre.
- `stream_reasoning::apply(...)` ne transmet qu’une valeur normalisée.

- [ ] **Step 1: Écrire les tests payload rouges**

Tester que GLM-5.3 envoie toujours `thinking.type = enabled` et l’effort choisi, que `off` est normalisé vers `max`, que Grok 4.6 transmet `xhigh`, et que Gemini 3.7 utilise `thinking_level` dans l’enveloppe Google OpenAI-compatible. Tester aussi les défauts `max`, `high` et `medium`.

- [ ] **Step 2: Voir les tests échouer**

Run: `cd src-tauri && cargo test stream_reasoning --lib`

- [ ] **Step 3: Remplacer les exceptions de nom par les métadonnées du registre**

Supprimer la condition limitée à `glm-5.2`. Valider l’effort contre `supported_modes`; si la session contient une ancienne valeur devenue invalide, appliquer le défaut du registre. Conserver les formes de payload propres à chaque provider.

- [ ] **Step 4: Vérifier raisonnement et non-régression**

Run: `cd src-tauri && cargo test reasoning --lib && cargo test stream_http --lib`

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/services/reasoning.rs src-tauri/src/services/llm
git commit -m "feat(reasoning): appliquer les efforts officiels des nouveaux modèles"
```

---

### Task 3: Actualiser les textes partagés de l’onboarding

**Files:**
- Modify: `src/i18n/fr.json`
- Modify: `src/i18n/en.json`
- Modify: `src/i18n/es.json`
- Modify: `src/i18n/de.json`
- Modify: `src/i18n/it.json`
- Modify: `src/i18n/zh.json`
- Modify: `src/i18n/ja.json`
- Modify: `src/lib/provider-copy.test.ts`

**Interfaces:**
- `apiKeys.providers.<provider>.description` reste la seule source pour l’onboarding et les réglages.

- [ ] **Step 1: Écrire le test rouge de cohérence des sept langues**

Faire charger les sept JSON et exiger que les descriptions Google, xAI et Z.AI contiennent respectivement `3.7`, `4.6` et `5.3`. Exiger aussi l’absence de prix, symbole monétaire et multiplicateur dans ces descriptions.

- [ ] **Step 2: Exécuter le test**

Run: `npx vitest run src/lib/provider-copy.test.ts`

Expected: échec sur les versions actuelles.

- [ ] **Step 3: Mettre à jour les traductions**

Modifier seulement les descriptions provider partagées. Ne pas créer de copie sous `onboarding.*` et ne pas retirer les textes `freeTier` existants sans recherche séparée.

- [ ] **Step 4: Vérifier l’onboarding et les réglages**

Run: `npx vitest run src/lib/provider-copy.test.ts src/components/onboarding/__tests__/onboarding-api.test.tsx`

- [ ] **Step 5: Commit**

```bash
git add src/i18n src/lib/provider-copy.test.ts
git commit -m "chore(onboarding): actualiser les modèles mis en avant"
```

---

### Task 4: Créer le catalogue xAI OAuth borné

**Files:**
- Create: `src-tauri/src/services/llm_oauth/xai_identity.rs`
- Create: `src-tauri/src/services/llm_oauth/xai_catalog.rs`
- Create: `src-tauri/src/services/llm_oauth/xai_catalog_wire.rs`
- Create: `src-tauri/src/services/llm_oauth/xai_catalog_tests.rs`
- Modify: `src-tauri/src/services/llm_oauth/mod.rs`
- Modify: `src-tauri/src/services/llm_oauth/headers.rs`
- Modify: `src-tauri/src/services/llm_oauth/lifecycle.rs`
- Modify: `src-tauri/src/services/llm_oauth/store.rs`
- Modify: `src-tauri/src/commands/oauth_provider_models.rs`

**Interfaces:**
- `XaiBackend::{ChatCompletions, Responses}`
- `XaiCatalogModel { id, display_name, backend, context_window, max_output_tokens, reasoning_modes, default_reasoning_mode }`
- Origine constante : `https://cli-chat-proxy.grok.com/v1`; aucun champ URL ne sort du parseur.

- [ ] **Step 1: Écrire les tests rouges du catalogue**

À partir d’une fixture anonymisée `/models-v2`, tester les deux backends, les doublons, 500 modèles maximum, les longueurs, les modes inconnus et une origine non autorisée. Tester que Grok 4.5 et 4.6 sont rendus tels que publiés par le compte.

- [ ] **Step 2: Écrire le test rouge d’identité et d’en-têtes**

Vérifier Bearer, `X-XAI-Token-Auth: xai-grok-cli`, modèle de routage, identité utilisateur privée et `User-Agent` Beaver véridique. Vérifier qu’aucun secret ni `userId` ne traverse la structure IPC.

- [ ] **Step 3: Exécuter les tests**

Run: `cd src-tauri && cargo test xai_catalog --lib && cargo test llm_oauth::headers --lib`

- [ ] **Step 4: Implémenter identité, parsing et cache**

Lire `/user` après login/refresh, conserver l’identité dans le stockage privé Rust, puis lire `/models-v2` avec un body borné. Cache réussi cinq minutes, dernier catalogue valide en repli bref, échec fermé sans catalogue sûr. Un principal différent après refresh invalide la connexion.

- [ ] **Step 5: Brancher la commande de modèles OAuth**

Remplacer pour `xai-oauth` le catalogue générique `/models` par le catalogue dédié. Le frontend reçoit uniquement les champs validés, jamais l’identité, l’origine ni les en-têtes.

- [ ] **Step 6: Vérifier**

Run: `cd src-tauri && cargo test xai_catalog --lib && cargo test oauth_provider_models --lib`

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/services/llm_oauth src-tauri/src/commands/oauth_provider_models.rs
git commit -m "feat(xai): utiliser le catalogue de l'abonnement OAuth"
```

---

### Task 5: Séparer l’inférence xAI API de xAI OAuth

**Files:**
- Modify: `src-tauri/src/services/llm/route.rs`
- Modify: `src-tauri/src/services/llm/route_tests.rs`
- Create: `src-tauri/src/services/llm/xai_oauth_transport.rs`
- Create: `src-tauri/src/services/llm/xai_oauth_transport_tests.rs`
- Modify: `src-tauri/src/services/llm/agent_loop_request.rs`
- Modify: `src-tauri/src/services/llm/stream_http_send.rs`
- Modify: `src-tauri/src/services/llm/retry.rs`
- Modify: `src-tauri/src/services/llm/provider_error.rs`
- Modify: `src-tauri/src/services/llm/provider_error_tests.rs`

**Interfaces:**
- `xai` garde `https://api.x.ai/v1` et une clé API.
- `xai-oauth` utilise le proxy constant et le backend validé du catalogue.
- `resource-exhausted` sans `Retry-After` ne suit pas la boucle générique 2/4/8 secondes.

- [ ] **Step 1: Écrire les tests de route rouges**

Tester qu’un token OAuth ne peut produire aucune requête vers `api.x.ai`, que Chat Completions et Responses utilisent le bon chemin, que les redirections authentifiées sont refusées et que les en-têtes réservés ne sont pas remplaçables.

- [ ] **Step 2: Écrire les tests d’erreur rouges**

Tester 401 + refresh unique, second 401 = reconnexion, 403 = accès abonnement, 429 `resource-exhausted` sans retry aveugle, et `Retry-After` borné/annulable pour un vrai rate limit.

- [ ] **Step 3: Exécuter les tests**

Run: `cd src-tauri && cargo test xai_oauth_transport --lib && cargo test provider_error --lib && cargo test retry --lib`

- [ ] **Step 4: Implémenter le transport dédié**

Construire les requêtes dans `xai_oauth_transport.rs`. Réutiliser les parseurs SSE communs, mais pas l’origine ni les en-têtes Codex OAuth. Le backend est un enum issu du catalogue ; aucune URL distante n’est transmise au constructeur HTTP.

- [ ] **Step 5: Vérifier les routes et les erreurs**

Run: `cd src-tauri && cargo test route --lib && cargo test xai_oauth --lib && cargo test stream_http --lib`

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/services/llm src-tauri/src/services/llm_oauth
git commit -m "fix(xai): isoler le transport OAuth du point API"
```

---

### Task 6: Restaurer durablement un échec sans réponse

**Files:**
- Modify: `src/types/agent-session.ts`
- Create: `src/lib/agent-session-failure.ts`
- Create: `src/lib/agent-session-failure.test.ts`
- Modify: `src/hooks/agent-chat-stream-finalize.ts`
- Modify: `src/hooks/use-agent-chat.ts`
- Modify: `src/components/agent-local/chat-view.tsx`
- Modify: les sept fichiers `src/i18n/*.json`

**Interfaces:**
- Le type TypeScript expose `diagnostic_runs` en lecture.
- `latestTerminalFailure(session)` dérive un code stable sans créer de faux message assistant.

- [ ] **Step 1: Écrire le test rouge avec la session reproduite**

Construire une fixture anonymisée analogue à `e79ff35a-3d3b-4707-bc6b-1242dc68243c` : messages utilisateur, aucun assistant, dernier diagnostic terminal en échec. Exiger que l’erreur soit visible après chargement et qu’une réponse réussie plus récente la rende obsolète.

- [ ] **Step 2: Lancer le test**

Run: `npx vitest run src/lib/agent-session-failure.test.ts`

- [ ] **Step 3: Exposer et dériver le diagnostic**

Aligner les types frontend sur la structure Rust bornée. Utiliser le code d’erreur traduit, pas `safe_summary` comme texte assistant. Conserver le comportement actuel : zéro segment signifie zéro message assistant.

- [ ] **Step 4: Vérifier reprise et streaming**

Run: `npx vitest run src/lib/agent-session-failure.test.ts src/hooks/__tests__/agent-chat-stream-retry.test.ts`

- [ ] **Step 5: Commit**

```bash
git add src/types src/lib src/hooks src/components/agent-local src/i18n
git commit -m "fix(sessions): restaurer les échecs provider persistés"
```

---

### Task 7: Valider la phase 1 sur les chemins réellement exécutés

**Files:**
- Create: `src-tauri/test-fixtures/providers/zai-glm-5.3-global-2026-08-22.json`
- Create: `src-tauri/test-fixtures/providers/google-gemini-3.7-flash-global-2026-08-22.json`
- Create: `src-tauri/test-fixtures/providers/xai-grok-4.6-global-2026-08-22.json`
- Create: `src-tauri/test-fixtures/providers/xai-oauth-grok-4.6-global-2026-08-22.json`
- Modify: aucun fichier produit supplémentaire hors corrections révélées par les validations

- [ ] **Step 1: Exécuter le socle statique**

Run:

```bash
npx tsc --noEmit
npm run lint
npm test
cd src-tauri && cargo check
cd src-tauri && cargo clippy --all-targets -- -D warnings
cd src-tauri && cargo test
graphify update .
```

Expected: toutes les commandes vertes. Toute rouge est rapportée avec sa sortie et bloque la phase 2.

- [ ] **Step 2: Ouvrir l’application et vérifier les deux thèmes**

Vérifier onboarding et réglages dans les sept langues, les trois nouveaux modèles en tête, l’absence de prix ajouté, et le maintien de Grok 4.5/Gemini 3.5/3.6.

- [ ] **Step 3: Faire les appels réels explicitement budgétés**

Avec accord sur le coût : un tour texte et un tool call pour GLM-5.3, Gemini 3.7 Flash et Grok 4.6 API ; pour xAI OAuth, un tour texte, un tool call et un second tour. Capturer route, backend, modèle demandé/servi, codes sûrs et absence de secret.

- [ ] **Step 4: Rejouer l’échec de session**

Produire un échec provider sans segment, fermer et rouvrir la session, puis vérifier visuellement que l’erreur traduite demeure sans faux message assistant.

- [ ] **Step 5: Commit de validation**

```bash
git add src src-tauri
git commit -m "test(providers): valider les modèles d'août 2026"
```
