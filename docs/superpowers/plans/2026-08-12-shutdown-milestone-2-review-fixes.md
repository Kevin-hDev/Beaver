# Shutdown Milestone 2 Review Fixes Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fermer les deux bloquants et les vingt défauts de la revue complète du jalon 2 avec des tests de régression non aveugles.

**Architecture:** Les autorités existantes restent uniques : superviseur global, superviseurs métier, état de chaque service et échéance absolue descendante. Les corrections sont livrées en sept commits fonctionnels ; chaque commit commence par une preuve rouge et finit par ses tests ciblés verts et une git note.

**Tech Stack:** Rust, Tokio, Tauri 2, React 19, TypeScript, Vitest, i18next, tests natifs Windows/Linux/macOS.

## Global Constraints

- Aucun code de production n'est écrit avant l'échec observé de son test de régression.
- Toute attente de fermeture consomme l'échéance absolue fournie par l'appelant ; aucun sous-service ne recrée un budget complet.
- Toute collection alimentée de l'extérieur reste bornée et toute erreur visible est générique et traduite.
- Les fichiers de code source restent sous 230 lignes ; les tests conservent une responsabilité unique.
- Toute commande système reçoit ses arguments séparément et tout enfant possédé passe par `OwnedProcess` ou l'autorité native déjà établie.
- Chaque commit reçoit une git note avec la commande rouge, la cause observée, la commande verte et son résultat.
- Après chaque modification de code, exécuter `graphify update .`. Après les documents, tenter l'actualisation sémantique Graphify et consigner honnêtement son résultat.

---

### Task 1: Préserver le helper transféré sous Linux

**Files:**
- Modify: `src-tauri/src/services/process_tree.rs`
- Modify: `src-tauri/src/services/process_tree_unix.rs`
- Create: `src-tauri/src/services/process_tree_unix_tests.rs`
- Modify: `src-tauri/src/commands/app_update_helper_process.rs`
- Modify: `src-tauri/src/commands/app_update_helper_process_tests.rs`
- Modify: `src-tauri/src/commands/app_update_install.rs`
- Test: `src-tauri/src/updater_worker/process_tests.rs`

**Interfaces:**
- Produces: `process_tree::configure_update_helper(&mut tokio::process::Command) -> std::io::Result<()>`, qui crée seulement le groupe Unix et n'arme jamais `PR_SET_PDEATHSIG`.
- Produces: `spawn_update_helper(...) -> impl Future<Output = Result<SpawnedUpdateHelper, String>>`, dont les opérations fichier et identité s'exécutent dans `spawn_blocking`.
- Preserves: `process_tree::configure` pour tous les enfants possédés.

- [ ] **Step 1: Écrire le test Linux du vrai décès du parent**

Créer un test natif qui lance un intermédiaire, lui fait démarrer un enfant avec `configure_update_helper`, termine l'intermédiaire et attend un fichier témoin écrit par l'enfant. Ajouter le contrôle inverse avec `configure` : l'enfant possédé ne doit pas produire ce témoin.

```rust
#[cfg(target_os = "linux")]
#[test]
fn transferred_helper_survives_real_parent_death() {
    let outcome = run_parent_death_probe(ProbeMode::Transferred);
    assert_eq!(outcome, ProbeOutcome::WitnessWritten);
}

#[cfg(target_os = "linux")]
#[test]
fn owned_child_dies_with_real_parent() {
    let outcome = run_parent_death_probe(ProbeMode::Owned);
    assert_eq!(outcome, ProbeOutcome::NoWitness);
}
```

- [ ] **Step 2: Observer l'échec rouge**

Run: `cd src-tauri; cargo test transferred_helper_survives_real_parent_death -- --nocapture`

Expected: FAIL parce que le helper reçoit encore `SIGKILL` à la mort du parent et le témoin n'existe pas.

- [ ] **Step 3: Séparer explicitement la configuration transférée**

Ajouter l'API dédiée et garder la raison à son point d'appel :

```rust
pub(crate) fn configure_update_helper(command: &mut Command) -> io::Result<()> {
    // Le helper applique la mise à jour après la mort de Beaver ; PDEATHSIG
    // annulerait précisément le travail transféré.
    unix::configure_process_group(command)
}
```

`configure` continue à appeler le groupe puis `configure_linux_parent_death` pour les enfants possédés.

- [ ] **Step 4: Écrire le test rouge du worker Tokio non bloqué**

Dans `app_update_helper_process_tests.rs`, injecter une barrière dans l'opération bloquante et vérifier qu'une tâche sentinelle Tokio progresse avant sa libération. Le test doit appeler l'API publique asynchrone, pas une copie de son algorithme.

- [ ] **Step 5: Déporter copie, synchronisation, spawn et validation**

Rendre `spawn_update_helper` asynchrone, déplacer son bloc synchrone complet dans `tokio::task::spawn_blocking`, appeler `configure_update_helper`, puis adapter `app_update_install.rs` avec `.await?`. Propager l'échec du join comme erreur générique sans lancer l'installation.

- [ ] **Step 6: Vérifier le lot et committer**

Run: `cd src-tauri; cargo test app_update_helper_process -- --nocapture`

Run on Linux: `cd src-tauri; cargo test transferred_helper_survives_real_parent_death -- --nocapture`

Run: `cd src-tauri; cargo fmt --check`

Expected: tous verts, puis commit `fix(update): preserve transferred helper on Linux` avec git note rouge/verte.

### Task 2: Traduire tous les refus d'admission

**Files:**
- Create: `src/lib/admission-error.ts`
- Create: `src/lib/admission-error.test.ts`
- Modify: `src/hooks/use-agent-stream.ts`
- Modify: `src/hooks/agent-stream-failure.ts`
- Modify: `src/hooks/agent-stream-manager.ts`
- Modify: `src/hooks/__tests__/use-agent-stream.test.ts`
- Modify: `src/lib/tool-error-message.ts`
- Modify: `src/lib/tool-error-message.test.ts`
- Modify: `src/components/agent-local/tool-detail-row.tsx`
- Create: `src/components/agent-local/tool-detail-row-admission.test.tsx`
- Modify: `src-tauri/src/services/agent_local/subagent_spawn_channel.rs`
- Modify: `src-tauri/src/services/agent_local/subagent_spawn_channel_tests.rs`
- Modify: `src/i18n/en.json`, `fr.json`, `es.json`, `de.json`, `it.json`, `zh.json`, `ja.json`

**Interfaces:**
- Produces: `admissionErrorKey(error: unknown): AdmissionErrorKey | null`.
- Produces: `admissionErrorMessage(error: unknown, t: TFunction, fallbackKey?: string): string`.
- Changes: `agentStreamManager.failSession(sessionId, message)` reçoit déjà le texte traduit et ne choisit plus son propre fallback.
- Produces: `isAdmissionError(error: unknown): boolean`, utilisé pour ne jamais recopier le code dans les détails du panneau d'outils.

- [ ] **Step 1: Écrire la table de tests fermée**

```ts
it.each([
  ["app-shutting-down", "errors.admission.appShuttingDown"],
  ["app-work-capacity-reached", "errors.admission.appCapacity"],
  ["service-shutting-down", "errors.admission.serviceShuttingDown"],
  ["service-work-capacity-reached", "errors.admission.serviceCapacity"],
  ["gateway-shutting-down", "errors.admission.gatewayShuttingDown"],
  ["gateway-busy", "errors.admission.gatewayBusy"],
])("mappe %s", (code, key) => expect(admissionErrorKey(code)).toBe(key));

it("masque un code inconnu", () => {
  expect(admissionErrorMessage("/private/path", t)).toBe("errors.operationFailed");
});
```

Charger ensuite chacun des sept JSON et vérifier que les six chemins existent et contiennent une chaîne non vide.

- [ ] **Step 2: Observer le rouge frontend**

Run: `npx vitest run src/lib/admission-error.test.ts`

Expected: FAIL car le module et les clés n'existent pas.

- [ ] **Step 3: Implémenter le traducteur central et les catalogues**

Utiliser un objet `as const`, un `Set` fermé et une extraction qui accepte uniquement une chaîne exacte ou une erreur Tauri sérialisée contenant exactement le code. Ne jamais inclure le texte d'une erreur inconnue dans le résultat.

- [ ] **Step 4: Prouver les trois consommateurs**

Ajouter des tests où `invoke("chat_stream")` rejette `app-shutting-down`, où la mise en file rejette `service-work-capacity-reached`, et où le panneau d'outils reçoit `gateway-busy`. Les trois assertions portent sur la traduction et vérifient que le code brut est absent du document rendu.

- [ ] **Step 5: Brancher les consommateurs et stabiliser le backend**

Passer le message traduit à `failSession`. Lors d'un refus de mise en file, afficher un toast traduit avant de retourner `false`. Consulter `admissionErrorKey` en premier dans `toolErrorMessage`, puis supprimer le détail brut dans `ToolDetailRow` lorsque `isAdmissionError` est vrai. Remplacer les deux textes français du canal de sous-agents par `service-shutting-down` et `service-work-capacity-reached` selon la branche.

- [ ] **Step 6: Vérifier le lot et committer**

Run: `npx vitest run src/lib/admission-error.test.ts src/hooks/__tests__/use-agent-stream.test.ts src/lib/tool-error-message.test.ts src/components/agent-local/tool-detail-row-admission.test.tsx`

Run: `npx tsc --noEmit`

Run: `cd src-tauri; cargo test subagent_spawn_channel -- --nocapture`

Expected: tous verts, puis commit `fix(admission): translate lifecycle refusals` avec git note rouge/verte.

### Task 3: Rendre le scheduler durable et localisable

**Files:**
- Modify: `src-tauri/src/models/config.rs`
- Modify: `src-tauri/src/services/scheduler/runtime.rs`
- Create: `src-tauri/src/services/scheduler/runtime_tests.rs`
- Modify: `src-tauri/src/services/scheduler/state.rs`
- Modify: `src-tauri/src/services/scheduler/log.rs`
- Modify: `src-tauri/src/services/scheduler/log_tests.rs`
- Modify: `src/types/wakeup.ts`
- Modify: `src/components/heartbeat/wakeup-history.tsx`
- Create: `src/lib/wakeup-run-error.ts`
- Create: `src/lib/wakeup-run-error.test.ts`
- Modify: the seven `src/i18n/*.json` catalogs

**Interfaces:**
- Produces: `WakeupRun.error_code: Option<WakeupRunErrorCode>` avec `serde(default)` ; `error` reste seulement compatible en lecture.
- Produces: `log::log_refused(id, scheduled_for, code) -> Result<(), String>` et toutes les fonctions de journal retournent leur résultat.
- Produces: `wakeupRunErrorMessage(run, t) -> string` qui ne rend jamais `run.error` brut.

- [ ] **Step 1: Écrire les tests rouges d'admission et de curseur**

Injecter un `SchedulerWakeupWork` saturé puis fermé et appeler la logique extraite `handle_due_wakeup`. Vérifier une entrée `error_code` pour chaque refus. Injecter ensuite un store de curseur dont le premier write échoue : deux passages doivent produire une seule décision pour la paire littérale `wakeup-1/2026-08-12T10:00:00+02:00`.

- [ ] **Step 2: Observer le rouge scheduler**

Run: `cd src-tauri; cargo test services::scheduler::runtime_tests -- --nocapture`

Expected: FAIL car les refus ne sont pas persistés et les erreurs `write_last_checked` sont ignorées.

- [ ] **Step 3: Écrire les tests rouges du journal atomique**

Faire tourner un lecteur pendant une rotation et vérifier que chaque ligne lue se désérialise. Ajouter un hook fichier qui échoue avant le renommage et vérifier que l'ancien journal reste identique. Les fixtures utilisent un répertoire temporaire réel.

- [ ] **Step 4: Implémenter une autorité de journal unique**

Faire prendre `WRITE_LOCK` par `list_runs`, `append` et la rotation. Écrire la rotation dans un temporaire du même dossier, `flush`, `sync_all`, puis `rename`. Dédupliquer sous le verrou par `(wakeup_id, scheduled_for)` avant l'ajout d'un résultat de refus ou de rejeu.

- [ ] **Step 5: Propager les erreurs et les codes**

Ne faire avancer `last_checked` qu'après les décisions journalisées. Sur échec de checkpoint, conserver la frontière précédente et tracer sans donnée utilisateur. Remplacer les textes français persistés par l'enum fermé ; garder la désérialisation des anciens journaux.

- [ ] **Step 6: Traduire l'historique et vérifier**

Ajouter les clés aux sept catalogues, utiliser `wakeupRunErrorMessage` dans `wakeup-history.tsx`, puis lancer :

Run: `cd src-tauri; cargo test services::scheduler -- --nocapture`

Run: `npx vitest run src/lib/wakeup-run-error.test.ts src/hooks/__tests__/use-wakeups.test.ts`

Expected: verts, puis commit `fix(scheduler): persist every wakeup outcome` avec git note rouge/verte.

### Task 4: Protéger le gateway contre la saturation et terminer à Off

**Files:**
- Modify: `src-tauri/src/services/gateway/channels/discord.rs`
- Modify: `src-tauri/src/services/gateway/channels/slack.rs`
- Modify: `src-tauri/src/services/gateway/channels/telegram.rs`
- Create: `src-tauri/src/services/gateway/channels/backpressure.rs`
- Create: `src-tauri/src/services/gateway/channels/backpressure_tests.rs`
- Modify: `src-tauri/src/services/gateway/service.rs`
- Modify: `src-tauri/src/services/gateway/service_audit.rs`
- Modify: `src-tauri/src/services/gateway/service_tests.rs`
- Modify: `src-tauri/src/services/gateway/service_runtime.rs`

**Interfaces:**
- Produces: `try_enqueue(&mpsc::Sender<InboundMessage>, InboundMessage, &ChannelKey) -> EnqueueOutcome` utilisant uniquement `try_send`.
- Changes: `GatewayService` conserve un `CancellationToken` de run accessible sans le mutex `run`.
- Preserves: `stop_and_wait(deadline: Instant) -> bool`, qui publie `ChannelStatus::Off` avant son retour.

- [ ] **Step 1: Écrire les tests rouges de heartbeat et des trois files**

Saturer une vraie file bornée, injecter plusieurs ticks Discord et vérifier que le pong ou le compteur heartbeat progresse sans consommation de message. Vérifier que les boucles Slack et Telegram observent leur annulation avec la file pleine.

- [ ] **Step 2: Observer le rouge backpressure**

Run: `cd src-tauri; cargo test services::gateway::channels::backpressure_tests -- --nocapture`

Expected: FAIL car `sender.send(...).await` bloque les boucles.

- [ ] **Step 3: Implémenter l'envoi non bloquant commun**

Centraliser `try_send`. Mapper `Full` vers un audit `gateway_busy`, `Closed` vers `gateway_shutting_down`, sans contenu de message ni identifiant externe. Le heartbeat ne passe jamais par cette file.

- [ ] **Step 4: Écrire les tests rouges d'arrêt**

Maintenir `run.lock()` au-delà de l'échéance et vérifier que le token actif est annulé. Bloquer l'audit et vérifier qu'une lecture de `state` reste possible. Après arrêt manuel, vérifier littéralement `ChannelStatus::Off` et l'événement final.

- [ ] **Step 5: Réordonner l'arrêt**

Annuler via le token hors mutex, écrire seulement `Stopping` et capturer les clés sous `state.write`, exécuter l'audit hors verrou dans `spawn_blocking` borné par l'échéance, attendre le work supervisor, puis publier `Off`. Ne jamais attendre un worker en tenant `state` ou `run`.

- [ ] **Step 6: Vérifier le lot et committer**

Run: `cd src-tauri; cargo test services::gateway -- --nocapture`

Expected: vert, puis commit `fix(gateway): keep channels responsive during shutdown` avec git note rouge/verte.

### Task 5: Borner MCP et l'hôte d'extensions

**Files:**
- Modify: `src-tauri/src/services/mcp_bridge/process_pool.rs`
- Modify: `src-tauri/src/services/mcp_bridge/process_manager.rs`
- Modify: `src-tauri/src/services/mcp_bridge/stdio_transport.rs`
- Create: `src-tauri/src/services/mcp_bridge/process_pool_tests.rs`
- Modify: `src-tauri/src/services/extensions/host_process.rs`
- Modify: `src-tauri/src/services/extensions/host_process_tests.rs`
- Modify: `src-tauri/src/services/extensions/runtime_lifecycle.rs`
- Create: `src-tauri/src/services/extensions/runtime_lifecycle_tests.rs`

**Interfaces:**
- Changes: MCP extrait le pool avant d'attendre `spawn_owner`; `terminate_entry(entry, deadline)` borne stdin, signal et wait.
- Changes: `HostProcess::kill(&mut self, deadline: Instant) -> bool`.
- Changes: `ExtensionRuntime::stop_host(deadline: Instant) -> bool` et `stop_and_wait` propage la même échéance.

- [ ] **Step 1: Écrire les tests rouges MCP**

Bloquer `spawn_owner`, publier un vrai faux-processus dans le pool et vérifier qu'il est extrait et moissonné. Bloquer ensuite le mutex stdin d'une entrée et lancer plusieurs terminaisons : le temps total doit rester inférieur à l'échéance commune plus une marge d'une seconde.

- [ ] **Step 2: Observer le rouge MCP**

Run: `cd src-tauri; cargo test services::mcp_bridge::process_pool_tests -- --nocapture`

Expected: FAIL car le pool n'est pas drainé après timeout du propriétaire et stdin/join sont sans borne.

- [ ] **Step 3: Corriger le propriétaire MCP**

Fermer l'admission, vider le pool sous son mutex court, relâcher le mutex puis terminer chaque entrée avec `timeout_at(deadline, ...)`. Le verrou `spawn_owner` ne décide plus si le pool peut être vidé. Après l'échéance, utiliser le chemin forcé existant et retourner `false`.

- [ ] **Step 4: Écrire les tests rouges extensions**

Faire expirer une première attente de `reader_done`, signaler le receiver ensuite, puis rappeler `kill` et vérifier qu'il observe la fin. Bloquer aussi le mutex du processus et vérifier que `stop_and_wait` respecte l'échéance absolue.

- [ ] **Step 5: Propager l'échéance extensions**

Attendre `reader_done` par emprunt mutable et ne faire `take()` qu'après succès. Extraire le handle sous mutex avec `timeout_at`, le tuer hors mutex, et faire partager la même échéance à `stop_host` et au superviseur de travail.

- [ ] **Step 6: Vérifier le lot et committer**

Run: `cd src-tauri; cargo test services::mcp_bridge -- --nocapture`

Run: `cd src-tauri; cargo test services::extensions::host_process -- --nocapture`

Run: `cd src-tauri; cargo test services::extensions::runtime_lifecycle -- --nocapture`

Expected: verts, puis commit `fix(services): bound MCP and extension teardown` avec git note rouge/verte.

### Task 6: Raccourcir les verrous SearXNG et Forecast

**Files:**
- Modify: `src-tauri/src/services/searxng/lifecycle.rs`
- Modify: `src-tauri/src/services/searxng/process.rs`
- Modify: `src-tauri/src/services/searxng/lifecycle_tests.rs`
- Modify: `src-tauri/src/services/searxng/runtime_tests.rs`
- Modify: `src-tauri/src/services/forecast/sidecar.rs`
- Modify: `src-tauri/src/services/forecast/sidecar_spawn.rs`
- Modify: `src-tauri/src/services/forecast/sidecar_stop.rs`
- Modify: `src-tauri/src/services/forecast/sidecar_lifecycle_tests.rs`
- Modify: `src-tauri/src/services/forecast/model_manager/uninstall.rs`
- Modify: `src-tauri/src/services/forecast/model_manager/tests.rs`

**Interfaces:**
- SearXNG adds un `start_gate` séparé et une génération de publication ; le mutex `process` ne contient que le handle publié.
- Forecast ajoute une identité de processus comparable avant/après `health_info` et `stop_state(deadline)`.
- `uninstall_from_roots` supprime le modèle avant le runtime familial devenu inutilisé.

- [ ] **Step 1: Écrire les tests rouges des verrous**

Suspendre chaque phase lente SearXNG avant publication et appeler `stop_and_wait` avec une échéance courte : il doit retourner et l'enfant `kill_on_drop` doit disparaître. Suspendre `health_info` Forecast et vérifier que `stop_state` extrait et tue le processus sans attendre l'appel réseau.

- [ ] **Step 2: Observer le rouge lifecycle**

Run: `cd src-tauri; cargo test services::searxng::lifecycle_tests -- --nocapture`

Run: `cd src-tauri; cargo test services::forecast::sidecar_lifecycle_tests -- --nocapture`

Expected: FAIL car les mutex de processus couvrent encore les opérations lentes.

- [ ] **Step 3: Réduire les sections critiques**

Protéger l'unicité du démarrage SearXNG par `start_gate`, effectuer préparation et readiness hors `process`, puis publier seulement si génération et annulation concordent. Pour Forecast, copier port, modèle et PID ainsi qu'un `Zeroizing<String>` temporaire pour le token sous verrou, exécuter `health_info` dans `spawn_blocking` borné, puis incrémenter l'usage seulement si l'identité est inchangée. Faire recevoir l'échéance par `stop_state` et laisser le token temporaire se zéroïser sur tous les retours.

- [ ] **Step 4: Écrire le test rouge de transaction Forecast**

Dans des répertoires temporaires réels, injecter un échec après chaque frontière. Pour chaque cas où le dossier modèle existe encore, vérifier que son runtime familial existe aussi. Le cas d'échec du nettoyage runtime doit laisser le modèle absent et seulement un runtime inutilisé.

- [ ] **Step 5: Réordonner la désinstallation et borner le diagnostic SearXNG**

Supprimer d'abord le modèle, recalculer si la famille est encore utilisée, puis nettoyer le runtime. Remplacer la lecture complète du log SearXNG par une queue bornée en octets destinée uniquement aux logs techniques ; retourner un code public générique à l'appelant.

- [ ] **Step 6: Vérifier le lot et committer**

Run: `cd src-tauri; cargo test services::searxng -- --nocapture`

Run: `cd src-tauri; cargo test services::forecast::sidecar -- --nocapture`

Run: `cd src-tauri; cargo test services::forecast::model_manager -- --nocapture`

Expected: verts, puis commit `fix(sidecars): make shutdown and uninstall interruptible` avec git note rouge/verte.

### Task 7: Annuler les sondes GPU et éviter le scan Git de fermeture

**Files:**
- Modify: `src-tauri/src/ollama_polling.rs`
- Modify: `src-tauri/src/services/gpu_vram.rs`
- Modify: `src-tauri/src/services/gpu_vram/windows.rs`
- Modify: `src-tauri/src/services/gpu_vram/linux.rs`
- Modify: `src-tauri/src/services/gpu_vram/macos.rs`
- Create: `src-tauri/src/services/gpu_vram/owned_probe.rs`
- Create: `src-tauri/src/services/gpu_vram/owned_probe_tests.rs`
- Modify: `src-tauri/src/services/agent_local/tool_bash_process_run.rs`
- Modify: `src-tauri/src/services/agent_local/tool_bash_changes.rs`
- Modify: `src-tauri/src/services/agent_local/tool_bash_changes_tests.rs`
- Create: `src-tauri/src/services/agent_local/tool_bash_process_run_tests.rs`

**Interfaces:**
- Produces: `gpu_vram::get_vram_info_owned(cancel: ServiceWorkCancellation) -> Future<Output = Option<(u64, u64)>>`.
- Produces: un runner de sonde qui utilise `OwnedProcess`, borne stdout et sélectionne entre sortie, annulation et timeout opérationnel.
- Changes: `settle_changes(session, tracker, completion, shutdown_cancelled)` évite `finish_changes` seulement pour l'annulation globale.

- [ ] **Step 1: Écrire les tests rouges GPU**

Lancer comme sonde un exécutable de test qui attend indéfiniment, annuler le service et vérifier la disparition réelle de son PID. Lancer un producteur de sortie supérieur à la borne et vérifier la troncature. Sous Windows, vérifier par le handle de test que PowerShell est enregistré dans l'autorité `OwnedProcess`.

- [ ] **Step 2: Observer le rouge GPU**

Run: `cd src-tauri; cargo test services::gpu_vram::owned_probe_tests -- --nocapture`

Expected: FAIL car le polling utilise encore un `spawn_blocking` non annulable et les commandes Windows ne sont pas possédées.

- [ ] **Step 3: Implémenter le runner possédé**

Créer la commande avec arguments séparés, la lancer via `OwnedProcess`, lire au maximum la constante bornée, puis `tokio::select!` entre `wait`, annulation et délai opérationnel. Sur les deux derniers chemins, terminer et moissonner avant le retour. Faire utiliser ce runner par le polling sur les trois OS.

- [ ] **Step 4: Écrire le test rouge du bilan Git**

Injecter un compteur au niveau de l'opération de scan, déclencher l'annulation globale après le lancement d'une commande shell, puis vérifier `scan_count == 0` et `changes_incomplete == true`. Une fin normale doit vérifier `scan_count == 1`.

- [ ] **Step 5: Distinguer les causes de fin**

Conserver un booléen `shutdown_cancelled` au moment du `select`. Pour ce seul chemin, vider les événements watcher déjà disponibles via une méthode non bloquante de `ChangeTracker`, ne pas appeler `finish_changes`, puis marquer le résultat incomplet. Les autres causes gardent le scan final.

- [ ] **Step 6: Vérifier le lot et committer**

Run: `cd src-tauri; cargo test services::gpu_vram -- --nocapture`

Run: `cd src-tauri; cargo test tool_bash_process_run -- --nocapture`

Run: `cd src-tauri; cargo test tool_bash_changes -- --nocapture`

Expected: verts, puis commit `fix(runtime): cancel probes and shutdown bookkeeping` avec git note rouge/verte.

### Task 8: Validation complète et remise à la re-review

**Files:**
- Modify only if required by verified failures: files owned by the failing lot
- Update: `graphify-out/` through the mandated Graphify command
- No product feature is added in this task

**Interfaces:**
- Consumes: les sept commits précédents et leurs git notes.
- Produces: une plage de commits reproductible et les résultats de validation locaux et CI.

- [ ] **Step 1: Vérifier les tailles et la structure**

Run: `npm run test:brand-boundaries`

Run: PowerShell count checking every changed Rust/TS/TSX source file is at most 230 lines; test files are reported separately but not rejected by the project exception.

- [ ] **Step 2: Exécuter les validations frontend**

Run: `npm run lint`

Run: `npx tsc --noEmit`

Run: `npm test`

Expected: zéro échec et aucun filtre vide.

- [ ] **Step 3: Exécuter les validations Rust**

Run: `cd src-tauri; cargo fmt --check`

Run: `cd src-tauri; cargo check`

Run: `cd src-tauri; cargo clippy --all-targets -- -D warnings`

Run: `cd src-tauri; cargo test`

Expected: zéro échec ; les tests ignorés existants sont énumérés sans être présentés comme exécutés.

- [ ] **Step 4: Actualiser Graphify et contrôler l'historique**

Run: `graphify update .`

Run: `git status --short`

Run: `git log --oneline 47e48be..HEAD`

Run: `git notes list` puis `git notes show <commit>` pour chaque commit de correction.

Expected: graphe actualisé, arbre propre, une note par commit.

- [ ] **Step 5: Pousser et attendre les contrôles natifs**

Push la branche `codex/shutdown-milestone-2`, suivre la PR 14 jusqu'à la fin des contrôles Linux, Windows et macOS, et lire tout log rouge avant de corriger. Toute correction CI suit elle aussi rouge/vert et reçoit son propre commit et sa note.

- [ ] **Step 6: Fournir la plage de re-review**

Donner `90309c0..HEAD` si le reviewer inclut les documents, et `<premier-commit-code>^..HEAD` pour la revue du code uniquement. Joindre les résultats exacts des commandes locales et l'URL de l'exécution CI verte.
