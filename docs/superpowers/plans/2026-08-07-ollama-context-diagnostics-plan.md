# Ollama Context Diagnostics Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Afficher un diagnostic chiffré calculé depuis le prompt et les outils réellement injectés quand le contexte Ollama est insuffisant, et porter le palier VRAM intermédiaire à 24 576 tokens.

**Architecture:** Le calcul du budget produit une erreur interne structurée et bornée, sérialisée uniquement pour traverser les couches qui utilisent encore `Result<_, String>`. La frontière Tauri la convertit en code public stable et en nombres sûrs ; le frontend valide ces nombres puis construit le texte via i18n. Les erreurs inconnues restent génériques.

**Tech Stack:** Rust, Tauri 2, React 19, TypeScript, Vitest, i18next.

## Global Constraints

- Ne jamais modifier la sélection des outils choisie par l'utilisateur.
- Compter uniquement le prompt final, les rapports obligatoires et les définitions d'outils réellement envoyés.
- Ne jamais afficher le contenu du prompt, les schémas d'outils, un chemin ou une erreur brute.
- Fournir les sept traductions `fr`, `en`, `es`, `de`, `it`, `zh`, `ja`.
- Conserver les fichiers de production sous 230 lignes.

---

### Task 1: Diagnostic dynamique du budget Rust

**Files:**
- Create: `src-tauri/src/services/agent_local/context_capacity_error.rs`
- Modify: `src-tauri/src/services/agent_local/context_budget_prune.rs`
- Modify: `src-tauri/src/services/agent_local/context_budget_prune_tests.rs`
- Modify: `src-tauri/src/services/agent_local/agent_local_modules_core.rs`

**Interfaces:**
- Produces: `ContextCapacityDetails`, `encode`, `decode`.
- Consumes: estimations réelles déjà calculées par `context_budget` et `token_estimate`.

- [ ] **Step 1: Write failing Rust tests**

Tester que l'erreur contient les tokens système, rapports, outils, total obligatoire, limite d'entrée et fenêtre configurée. Tester séparément l'absence de rapport et le rejet d'une chaîne invalide.

- [ ] **Step 2: Run the focused Rust tests and verify RED**

Run: `cargo test context_capacity --lib`
Expected: FAIL car le type et les fonctions n'existent pas encore.

- [ ] **Step 3: Implement the minimal structured diagnostic**

Créer un format interne stable, numérique et borné. Calculer les valeurs depuis `messages`, `required_reports`, `params.tool_tokens`, `params.max_input` et `params.capsule_context` au point exact où le budget échoue.

- [ ] **Step 4: Run the focused Rust tests and verify GREEN**

Run: `cargo test context_capacity --lib`
Expected: PASS.

### Task 2: Transport public sûr jusqu'au frontend

**Files:**
- Modify: `src-tauri/src/services/agent_local/types_stream.rs`
- Modify: `src-tauri/src/commands/agent_chat.rs`
- Modify: call sites constructing `StreamEvent::Error`
- Modify: `src/types/agent-stream.ts`
- Create: `src/hooks/agent-context-capacity-error.ts`
- Modify: `src/hooks/agent-chat-stream-callbacks.ts`
- Modify: `src/hooks/__tests__/agent-chat-stream-callbacks-error.test.ts`

**Interfaces:**
- Produces Rust event field: `context_capacity: Option<ContextCapacityDetails>`.
- Produces TypeScript helper: validated localized message or `null`.

- [ ] **Step 1: Write failing frontend tests**

Tester le message sans rapport, le message avec rapport, les valeurs dynamiques différentes, et le repli générique si le code ou les nombres sont invalides.

- [ ] **Step 2: Run the focused frontend test and verify RED**

Run: `npm test -- src/hooks/__tests__/agent-chat-stream-callbacks-error.test.ts`
Expected: FAIL sur le nouveau diagnostic.

- [ ] **Step 3: Implement the safe event transport and frontend validator**

Décoder l'erreur interne à la frontière Tauri, envoyer le code `context_capacity_exceeded` et les seuls nombres validés, puis produire le message i18n côté frontend. Les messages inconnus conservent le repli générique.

- [ ] **Step 4: Run Rust and frontend focused tests and verify GREEN**

Run: `cargo test context_capacity --lib`
Run: `npm test -- src/hooks/__tests__/agent-chat-stream-callbacks-error.test.ts`
Expected: PASS.

### Task 3: Traductions et palier 24k

**Files:**
- Modify: `src/i18n/fr.json`
- Modify: `src/i18n/en.json`
- Modify: `src/i18n/es.json`
- Modify: `src/i18n/de.json`
- Modify: `src/i18n/it.json`
- Modify: `src/i18n/zh.json`
- Modify: `src/i18n/ja.json`
- Modify: `src-tauri/src/services/gpu_vram.rs`

**Interfaces:**
- Produces i18n keys: `errors.contextCapacityExceeded` and `errors.contextCapacityExceededWithReports`.
- Produces pure Rust tier resolver for deterministic tests.

- [ ] **Step 1: Write the failing 24k tier test**

Tester 11 999 Mo → 8 192, 12 000 Mo → 24 576, 23 999 Mo → 24 576 et 24 000 Mo → 32 768.

- [ ] **Step 2: Run the focused test and verify RED**

Run: `cargo test vram_context_tiers --lib`
Expected: FAIL car le palier intermédiaire vaut encore 16 384.

- [ ] **Step 3: Change only the intermediate tier and add translations**

Passer `CTX_MID` à `24_576`, faire utiliser le résolveur pur par la détection réelle, et ajouter les deux messages dans les sept langues.

- [ ] **Step 4: Run focused tests and verify GREEN**

Run: `cargo test vram_context_tiers --lib`
Run: `npm test -- src/hooks/__tests__/agent-chat-stream-callbacks-error.test.ts`
Expected: PASS.

### Task 4: Verification and graph maintenance

**Files:**
- Update generated graph: `graphify-out/`

- [ ] **Step 1: Run complete relevant verification**

Run: `npm test -- src/hooks/__tests__/agent-chat-stream-callbacks-error.test.ts`
Run: `npx tsc --noEmit`
Run: `cargo test context_capacity --lib`
Run: `cargo test vram_context_tiers --lib`
Run: `cargo check`

- [ ] **Step 2: Update Graphify**

Run: `graphify update .`

- [ ] **Step 3: Inspect the final diff and commit**

Vérifier que seuls le diagnostic, les traductions, le palier et les documents associés ont changé, puis créer un commit explicite et une note Git pour le reviewer.
