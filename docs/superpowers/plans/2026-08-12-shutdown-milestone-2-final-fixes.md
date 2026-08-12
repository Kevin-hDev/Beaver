# Shutdown Milestone 2 Final Fixes Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Tu corriges les quatre régressions de la re-review avec une autorité unique pour l’audit différé, la fermeture et la mesure GPU.

**Architecture:** Tu sors l’audit gateway du fil réseau au moyen d’une file bornée et d’un worker possédé. Tu centralises la combinaison des résultats de fermeture, tu rends l’arrêt d’extensions fail-closed et tu remplaces les sondes GPU synchrones par un instantané publié uniquement par la sonde possédée.

**Tech Stack:** Rust, Tokio, Tauri 2, tests unitaires Cargo, Git notes.

## Global Constraints

- Tu écris le test avant le correctif et tu observes son échec attendu.
- Tu ne lances aucun processus ni aucune tâche longue hors d’un registre possédé.
- Tu bornes toute file alimentée par un canal externe.
- Tu utilises une seule autorité par ressource et tu écris la raison près de la décision.
- Tu gardes chaque fichier de code sous 230 lignes et tu ne compactes pas le code pour contourner la limite.
- Tu ne rends aucun détail technique visible à l’utilisateur.

---

### Task 1: Tu sors l’audit gateway du fil réseau

**Files:**
- Create: `src-tauri/src/services/gateway/refusal_audit.rs`
- Modify: `src-tauri/src/services/gateway/mod.rs`
- Modify: `src-tauri/src/services/gateway/channels/mod.rs`
- Modify: `src-tauri/src/services/gateway/channels/backpressure.rs`
- Modify: `src-tauri/src/services/gateway/channels/backpressure_tests.rs`
- Modify: `src-tauri/src/services/gateway/channels/discord.rs`
- Modify: `src-tauri/src/services/gateway/channels/slack.rs`
- Modify: `src-tauri/src/services/gateway/channels/telegram.rs`
- Modify: `src-tauri/src/services/gateway/service.rs`
- Modify: `src-tauri/src/services/gateway/service_channels.rs`
- Modify: `src-tauri/src/services/gateway/service_consumer.rs`
- Modify: `src-tauri/src/services/gateway/work_supervision.rs`

**Interfaces:**
- Produces: `RefusalAudit::try_record(ChannelKey, &'static str) -> RefusalAuditOutcome`.
- Produces: un worker `run_refusal_audit` annulable, enregistré dans `GatewayWorkServices`.
- Consumes: `service_audit::work_refused` uniquement depuis le worker bloquant.

- [ ] **Step 1: Tu écris le test rouge**

Tu construis une file d’audit réelle avec un écrivain qui bloque sur une barrière. Tu attends que l’écrivain soit bloqué, tu remplis la file de messages gateway, puis tu exécutes `try_enqueue` dans un thread séparé. Tu exiges son retour avant 50 ms et tu vérifies que la file d’audit reste bornée à 64.

- [ ] **Step 2: Tu observes l’échec attendu**

Run: `cargo test gateway::channels::backpressure_tests -- --nocapture`

Expected: le test échoue parce que `try_enqueue` appelle encore l’audit synchrone ou parce que `RefusalAudit` n’existe pas.

- [ ] **Step 3: Tu écris le correctif minimal**

Tu ajoutes la file bornée, son compteur saturant et son worker possédé. Tu passes son expéditeur aux canaux et au consommateur. Tu retires tout appel direct à `work_refused` des chemins chauds.

- [ ] **Step 4: Tu vérifies le vert ciblé**

Run: `cargo test gateway::channels::backpressure_tests -- --nocapture`

Run: `cargo test gateway::refusal_audit -- --nocapture`

Expected: tous les tests ciblés passent sans accès au coffre dans le thread réseau.

- [ ] **Step 5: Tu commits la correction**

Tu commits uniquement les fichiers de la tâche avec le message `fix(gateway): move refusal audit off network loops` et tu ajoutes une git note qui contient la preuve rouge/verte.

### Task 2: Tu attends toujours les registres extensions et MCP

**Files:**
- Create: `src-tauri/src/services/shutdown_completion.rs`
- Modify: `src-tauri/src/services/mod.rs`
- Modify: `src-tauri/src/services/extensions/runtime_lifecycle.rs`
- Modify: `src-tauri/src/services/mcp_bridge/process_pool.rs`

**Interfaces:**
- Produces: `combine_with_work(processes_stopped: bool, work: impl Future<Output = bool>) -> bool`.

- [ ] **Step 1: Tu écris le test rouge**

Tu passes `false` comme résultat processus et un futur qui marque un `AtomicBool`. Tu exiges que le booléen soit marqué et que le résultat final soit `false`.

- [ ] **Step 2: Tu observes l’échec attendu**

Run: `cargo test shutdown_completion -- --nocapture`

Expected: échec parce que l’autorité de combinaison n’existe pas.

- [ ] **Step 3: Tu écris le correctif minimal**

Tu attends d’abord le futur du registre, puis tu combines les deux booléens. Extensions et MCP appellent cette fonction au lieu d’un `&&` qui contient un `.await` à droite.

- [ ] **Step 4: Tu vérifies le vert ciblé**

Run: `cargo test shutdown_completion -- --nocapture`

Expected: le test passe et la marque prouve que le futur est attendu malgré l’échec processus.

- [ ] **Step 5: Tu commits la correction**

Tu commits avec `fix(shutdown): await domain work after process failures` et tu ajoutes la git note de preuve.

### Task 3: Tu bloques toute relance après un arrêt d’extension incomplet

**Files:**
- Modify: `src-tauri/src/services/extensions/runtime_lifecycle.rs`
- Modify: `src-tauri/src/services/extensions/installer.rs`
- Modify: `src-tauri/src/services/extensions/runtime_dispatch.rs`
- Test: `src-tauri/src/services/extensions/runtime_lifecycle.rs`
- Test: `src-tauri/src/services/extensions/installer_tests.rs`

**Interfaces:**
- Produces: une fonction d’arrêt qui conserve le slot tant que la mort du processus n’est pas confirmée.
- Produces: une frontière de mutation qui retourne `OperationFailure::HostUnavailable` avant toute modification si l’arrêt échoue.

- [ ] **Step 1: Tu écris les tests rouges**

Tu simules un arrêt `false` et tu comptes les appels de mutation et de redémarrage. Tu exiges zéro appel. Tu simules aussi un slot dont l’arrêt expire et tu exiges que le slot reste occupé pour empêcher un second spawn.

- [ ] **Step 2: Tu observes les échecs attendus**

Run: `cargo test services::extensions -- --nocapture`

Expected: les tests échouent parce que les résultats sont actuellement ignorés et que le slot est retiré avant confirmation.

- [ ] **Step 3: Tu écris le correctif minimal**

Tu conserves ou réinsères le processus non confirmé. Tu fais retourner immédiatement l’erreur générique existante dans la désinstallation, la mise à jour, le redémarrage et l’invalidation. Tu n’appelles jamais `start_untracked` après un `false`.

- [ ] **Step 4: Tu vérifies le vert ciblé**

Run: `cargo test services::extensions -- --nocapture`

Expected: tous les tests extensions passent et aucun chemin testé ne crée un second hôte.

- [ ] **Step 5: Tu commits la correction**

Tu commits avec `fix(extensions): fail closed when host stop is incomplete` et tu ajoutes la git note de preuve.

### Task 4: Tu fais de la sonde possédée l’unique autorité GPU

**Files:**
- Modify: `src-tauri/src/services/gpu_vram.rs`
- Modify: `src-tauri/src/services/gpu_vram/linux.rs`
- Modify: `src-tauri/src/services/gpu_vram/macos.rs`
- Modify: `src-tauri/src/services/gpu_vram/windows.rs`
- Modify: `src-tauri/src/services/gpu_detect.rs`
- Modify: `src-tauri/src/runtime_startup.rs`
- Modify: `src-tauri/src/ollama_polling.rs`
- Modify: `src-tauri/src/commands/ollama_setup.rs`
- Modify: `src-tauri/src/services/compress/context_resolve.rs`
- Modify: `src-tauri/src/services/forecast/hardware_profile.rs`
- Modify: `src-tauri/src/services/ollama_env.rs`
- Modify: `src-tauri/src/commands/config.rs`

**Interfaces:**
- Produces: `refresh_owned(cancel) -> Option<GpuVramSnapshot>` qui publie le cache.
- Produces: `cached_snapshot()`, `cached_total_mb()`, `cached_used_mb()` et `compute_default_num_ctx()` sans E/S ni processus.

- [ ] **Step 1: Tu écris les tests rouges**

Tu injectes un instantané dans l’autorité GPU et tu vérifies que tous les calculs synchrones lisent cette valeur. Tu ajoutes un contrôle de frontière qui refuse les anciennes fonctions `detect_total`/`detect_used` et tout `.output()` dans les modules de plateforme GPU.

- [ ] **Step 2: Tu observes les échecs attendus**

Run: `cargo test gpu_vram -- --nocapture`

Expected: échec parce que le cache n’existe pas et que les anciennes sondes synchrones sont encore compilées.

- [ ] **Step 3: Tu écris le correctif minimal**

Tu publies chaque résultat de `detect_owned` dans le cache. Tu fais lire ce cache à tous les consommateurs synchrones, tu amorces la mesure dans `runtime_startup` avant Ollama et tu supprimes les sondes synchrones des trois plateformes.

- [ ] **Step 4: Tu vérifies le vert ciblé**

Run: `cargo test gpu_vram -- --nocapture`

Expected: les tests passent et la recherche `rg "detect_total|detect_used|\.output\(\)" src-tauri/src/services/gpu_vram` ne trouve aucun ancien chemin de sonde.

- [ ] **Step 5: Tu commits la correction**

Tu commits avec `fix(gpu): centralize measurement in owned probes` et tu ajoutes la git note de preuve.

### Task 5: Tu vérifies le lot complet

**Files:**
- Modify if required by formatting only: les fichiers déjà touchés.

- [ ] **Step 1: Tu mets le graphe à jour**

Run: `graphify update .`

- [ ] **Step 2: Tu vérifies les frontières et le format**

Run: `cargo fmt --check`

Run: `cargo clippy --all-targets -- -D warnings`

- [ ] **Step 3: Tu exécutes les suites ciblées et complètes**

Run: `cargo test gateway::`

Run: `cargo test services::extensions`

Run: `cargo test mcp_bridge`

Run: `cargo test gpu_vram`

Run: `cargo test`

- [ ] **Step 4: Tu contrôles la portée**

Tu relis `git diff --check`, `git status --short`, les fichiers dépassant 230 lignes et l’inventaire des appels GPU/audit. Tu ne déclares vert que ce que les sorties viennent de prouver.

- [ ] **Step 5: Tu commits les ajustements de vérification**

Si le graphe ou un ajustement strictement nécessaire a changé, tu le commits séparément avec sa git note. Sinon, tu ne crées pas de commit vide.
