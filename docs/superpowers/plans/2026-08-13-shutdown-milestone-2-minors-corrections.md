# Shutdown Milestone 2 Minor Review Corrections Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Corriger les sept écarts bloquant la fusion du lot mineur du jalon 2, les prouver par des tests de régression, pousser la branche et attendre une CI verte sans fusionner.

**Architecture:** Le scheduler ajoute un registre borné d'occurrences en vol qui bloque durablement son curseur jusqu'à la décision terminale, puis sa rotation conserve une marge de 250 lignes. Le gateway sépare les issues Discord, partage le classement des trames WebSocket et centralise le comptage des refus. OAuth MCP reprend le serveur concurrent borné de Codex, tandis que Forecast sépare ses étapes réelles de toute injection compilée uniquement en test.

**Tech Stack:** Rust 2021, Tokio, Tauri 2, tokio-tungstenite, Cargo tests, Git notes, Graphify.

## Global Constraints

- Ne jamais fusionner dans `main` sans validation explicite de l'utilisateur.
- Écrire et exécuter chaque test rouge avant son correctif ; le test doit échouer pour le défaut nommé.
- Conserver une seule autorité pour chaque capacité, délai, état et classification.
- Toute collection alimentée par des événements reste bornée ; aucun secret ou détail interne n'entre dans les traces.
- Toute comparaison du `state` OAuth reste en temps constant et les buffers de requête restent zéroïsés.
- Aucun fichier de production touché ne dépasse 230 lignes ; les tests peuvent dépasser ce seuil s'ils couvrent une seule unité.
- Les délais OAuth restent centralisés : 300 secondes globales, 5 secondes par connexion, 50 connexions acceptées au maximum.
- Le journal scheduler expose au maximum 500 résultats et revient à 250 lignes après rotation.
- Mettre à jour Graphify après le code ; utiliser la mise à jour documentaire complète quand elle est disponible.

---

### Task 1: Rendre les occurrences scheduler en vol visibles à la réconciliation

**Files:**
- Create: `src-tauri/src/services/scheduler/in_flight.rs`
- Modify: `src-tauri/src/services/scheduler/mod.rs`
- Modify: `src-tauri/src/services/scheduler/runtime.rs`
- Modify: `src-tauri/src/services/scheduler/runtime_tests.rs`
- Modify: `src-tauri/src/services/scheduler/work_supervision.rs`

**Interfaces:**
- Consumes: `SCHEDULED_WAKEUPS_CAPACITY`, `ScheduledWakeup`, `DateTime<Local>`, et la garde RAII du registre de travail.
- Produces: `InFlightWakeups::reserve(wakeup_id, scheduled_for) -> Result<InFlightWakeupGuard, InFlightReservationError>`, `InFlightWakeups::partition(candidates) -> ReconciliationCandidates { decidable, has_in_flight }`.

- [ ] **Step 1: Écrire le test rouge du réveil long**

Ajouter un test qui réserve `daily@08:00`, présente cette occurrence à la réconciliation et vérifie qu'elle n'est pas décidable et que le curseur doit rester bloqué. Libérer la garde puis vérifier que la même occurrence devient décidable.

```rust
#[test]
fn in_flight_occurrence_blocks_missed_decision_until_terminal_result() {
    let registry = InFlightWakeups::default();
    let guard = registry.reserve("daily", scheduled_for).unwrap();
    let blocked = registry.partition(vec![(daily_wakeup("daily", scheduled_for), scheduled_for)]);
    assert!(blocked.decidable.is_empty());
    assert!(blocked.has_in_flight);
    drop(guard);
    let released = registry.partition(vec![(daily_wakeup("daily", scheduled_for), scheduled_for)]);
    assert_eq!(released.decidable.len(), 1);
    assert!(!released.has_in_flight);
}
```

Le changement de production qui doit faire échouer ce test est l'absence du registre ou le retrait de l'occurrence avant la décision terminale.

- [ ] **Step 2: Exécuter le test et lire l'échec attendu**

Run: `cargo test --lib --features windows-tests in_flight_occurrence_blocks_missed_decision_until_terminal_result -- --nocapture --test-threads=1`

Expected: FAIL parce que `InFlightWakeups` et sa partition n'existent pas.

- [ ] **Step 3: Implémenter le registre borné et sa garde**

Déplacer la capacité vers `pub(super) const SCHEDULED_WAKEUPS_CAPACITY: usize = 64`. Dans `in_flight.rs`, utiliser un `Arc<Mutex<HashSet<OccurrenceKey>>>`; la garde retire exactement sa clé au `Drop`. `reserve` refuse un doublon et refuse au plafond partagé.

```rust
#[derive(Clone, Default)]
pub(super) struct InFlightWakeups(Arc<Mutex<HashSet<OccurrenceKey>>>);

pub(super) struct InFlightWakeupGuard {
    registry: InFlightWakeups,
    key: Option<OccurrenceKey>,
}

impl Drop for InFlightWakeupGuard {
    fn drop(&mut self) {
        if let Some(key) = self.key.take() {
            self.registry.remove(&key);
        }
    }
}
```

- [ ] **Step 4: Brancher le runtime sans perdre la durabilité**

Créer le registre une seule fois dans `run_loop`, le passer à `reconcile_missed` et `handle_due`, réserver avant `work.spawn`, puis déplacer la garde dans la tâche. Un doublon signifie que le travail original possède déjà la décision et est ignoré ; un plafond atteint est traité comme le refus de capacité existant. Si une occurrence candidate reste en vol, journaliser les autres décisions mais retourner `false` avant `checkpoint(cutoff)`.

```rust
let candidates = in_flight.partition(missed_occurrences(wakeups, last_checked, cutoff));
// journaliser candidates.decidable
if candidates.has_in_flight || !decisions_persisted {
    return false;
}
checkpoint(cutoff).await
```

La raison du gel est écrite près du retour : avancer le curseur rendrait un crash pendant le réveil impossible à réconcilier au démarrage suivant.

- [ ] **Step 5: Exécuter les tests scheduler ciblés**

Run: `cargo test --lib --features windows-tests services::scheduler:: -- --nocapture --test-threads=1`

Expected: PASS, y compris le nouveau test et tous les tests de refus/arrêt existants.

---

### Task 2: Rendre la rotation du journal réellement amortie

**Files:**
- Modify: `src-tauri/src/services/scheduler/log_store.rs`
- Modify: `src-tauri/src/services/scheduler/log_tests.rs`

**Interfaces:**
- Consumes: `MAX_LINES = 500` et la métadonnée incrémentale existante.
- Produces: `ROTATED_LINES = MAX_LINES / 2` et `rotated_content` qui rend exactement 250 lignes, nouvelle entrée comprise.

- [ ] **Step 1: Étendre le test de lecture jusqu'au 502e ajout**

Après le premier ajout qui provoque la rotation, ajouter une deuxième entrée avec le même observateur et exiger que le nombre de lectures reste à deux.

```rust
append_at_with_read_observer(&path, indexed_entry(MAX_LINES + 1), move || {
    reads_after_rotation.fetch_add(1, Ordering::Relaxed);
}).await.unwrap();
assert_eq!(reads.load(Ordering::Relaxed), 2);
```

Le changement de production qui doit faire échouer ce test est une rotation qui revient exactement au seuil de 500.

- [ ] **Step 2: Exécuter le test rouge**

Run: `cargo test --lib --features windows-tests appends_do_not_reread_the_log_before_the_trim_boundary -- --nocapture --test-threads=1`

Expected: FAIL avec trois lectures au lieu de deux.

- [ ] **Step 3: Conserver une marge de 250 lignes**

Ajouter `pub(super) const ROTATED_LINES: usize = MAX_LINES / 2` et faire garder `ROTATED_LINES - 1` anciennes lignes avant la nouvelle. Adapter le test de rotation atomique pour attendre `ROTATED_LINES` résultats, sans changer le plafond de lecture `MAX_LINES`.

- [ ] **Step 4: Exécuter les tests du journal puis committer le lot scheduler**

Run: `cargo test --lib --features windows-tests services::scheduler::log:: -- --nocapture --test-threads=1`

Run: `cargo test --lib --features windows-tests services::scheduler:: -- --nocapture --test-threads=1`

Expected: PASS pour les deux commandes.

Commit: `fix(scheduler): preserve in-flight wakeup decisions`

Git note: raison du gel du curseur, marge 250/500, tests rouges observés puis comptes verts exacts.

---

### Task 3: Séparer les issues du traitement Discord

**Files:**
- Modify: `src-tauri/src/services/gateway/channels/discord.rs`
- Modify: `src-tauri/src/services/gateway/channels/discord_events.rs`
- Create: `src-tauri/src/services/gateway/channels/discord_events_tests.rs`
- Modify: `src-tauri/src/services/gateway/channels/mod.rs`

**Interfaces:**
- Produces: `DiscordEventOutcome::{Continue, Reconnect, ConsumerClosed}`.
- Consumes: un sink générique `Sink<WsMessage> + Unpin` afin de tester les vrais effets d'envoi sans serveur Discord.

- [ ] **Step 1: Écrire trois tests rouges**

Tester avec des sinks bornés : échec d'envoi d'IDENTIFY → `Reconnect`; envoi réussi dont le sink conserve la trame et empêche la zéroïsation → `Continue`; message entrant avec consommateur fermé → `ConsumerClosed`.

Le changement de production qui doit faire échouer ces tests est le booléen qui donne la même issue aux trois cas.

- [ ] **Step 2: Exécuter les tests rouges**

Run: `cargo test --lib --features windows-tests discord_events_tests -- --nocapture --test-threads=1`

Expected: FAIL parce que l'issue fermée n'existe pas et que l'échec transitoire termine encore le gateway.

- [ ] **Step 3: Implémenter l'issue fermée**

```rust
pub(super) enum DiscordEventOutcome {
    Continue,
    Reconnect,
    ConsumerClosed,
}
```

Dans `handle_hello`, calculer séparément `sent` et `zeroized`. Un échec de zéroïsation écrit seulement `warn!("[gateway] nettoyage du payload Discord incomplet")`, sans jeton ni payload. Dans `discord.rs`, `Reconnect` casse uniquement la boucle de connexion et `ConsumerClosed` casse `'gateway`.

- [ ] **Step 4: Exécuter les tests Discord ciblés**

Run: `cargo test --lib --features windows-tests discord -- --nocapture --test-threads=1`

Expected: PASS.

---

### Task 4: Classer les trames WebSocket dans une autorité commune

**Files:**
- Create: `src-tauri/src/services/gateway/channels/websocket_message.rs`
- Modify: `src-tauri/src/services/gateway/channels/mod.rs`
- Modify: `src-tauri/src/services/gateway/channels/discord.rs`
- Modify: `src-tauri/src/services/gateway/channels/slack.rs`

**Interfaces:**
- Produces: `IncomingWebSocket::{Text(Utf8Bytes), Ignore, Disconnect}` et `classify_incoming(Option<Result<WsMessage, WsError>>) -> IncomingWebSocket`.
- Consumes: `tokio_tungstenite::tungstenite::Message` des deux canaux.

- [ ] **Step 1: Écrire le tableau de tests rouge**

Exiger `Text → Text`, `Ping/Pong/Binary → Ignore`, `Close/Err/None → Disconnect`. Les valeurs attendues sont littérales et indépendantes du classificateur.

- [ ] **Step 2: Exécuter le test rouge**

Run: `cargo test --lib --features windows-tests websocket_message -- --nocapture --test-threads=1`

Expected: FAIL parce que le module n'existe pas.

- [ ] **Step 3: Implémenter et brancher le classificateur**

Les branches `Ignore` font `continue`; `Disconnect` casse uniquement la connexion. Les deux canaux conservent leur traitement `Text` actuel. Ne modifier ni les délais ni `ReconnectPolicy`.

- [ ] **Step 4: Exécuter les tests gateway ciblés**

Run: `cargo test --lib --features windows-tests services::gateway:: -- --nocapture --test-threads=1`

Expected: PASS.

---

### Task 5: Compter le refus d'admission du consommateur

**Files:**
- Modify: `src-tauri/src/services/gateway/service_consumer.rs`

**Interfaces:**
- Produces: `record_work_refusal(&RefusalAudit, ChannelKey, ServiceWorkAdmissionError)` utilisé par l'unique branche de refus du consommateur.
- Consumes: `RefusalAudit::record_refusal`, autorité compteur puis audit.

- [ ] **Step 1: Écrire le test rouge**

Créer un audit sans lancer son écrivain, appeler la fonction de refus du consommateur et vérifier `audit.counter().total() == 1`.

Le changement de production qui doit faire échouer le test est un retour à `try_record`.

- [ ] **Step 2: Exécuter le test rouge**

Run: `cargo test --lib --features windows-tests consumer_work_refusal_increments_counter_without_writer -- --nocapture --test-threads=1`

Expected: FAIL avec un compteur à zéro.

- [ ] **Step 3: Utiliser l'autorité de refus puis vérifier et committer le gateway**

Remplacer l'appel direct par `let _ = refusal_audit.record_refusal(channel_key, error.audit_code());` via la fonction testée.

Run: `cargo test --lib --features windows-tests services::gateway:: -- --nocapture --test-threads=1`

Expected: PASS.

Commit: `fix(gateway): keep transient channels reconnectable`

Git note: trois issues Discord, classificateur partagé, compteur local, tests rouges et compte gateway vert exact.

---

### Task 6: Empêcher une socket muette de bloquer OAuth MCP

**Files:**
- Modify: `src-tauri/src/services/mcp_oauth/callback_server.rs`
- Modify: `src-tauri/src/services/mcp_oauth/callback_server_tests.rs`

**Interfaces:**
- Consumes: `verify_state_constant_time`, `MAX_REQUEST_LEN`, annulation et délai global existants.
- Produces: `CONNECTION_TIMEOUT = 5s`, `JoinSet<Option<CallbackResult>>`, 50 handlers acceptés au maximum.

- [ ] **Step 1: Écrire le test réseau rouge**

Ouvrir une première `TcpStream` sans envoyer un octet, garder cette socket vivante, puis envoyer un vrai callback valide sur une seconde connexion. Borner le test à une seconde et exiger le code attendu.

Le changement de production qui doit faire échouer le test est le retour à une lecture séquentielle sans délai par connexion.

- [ ] **Step 2: Exécuter le test rouge**

Run: `cargo test --lib --features windows-tests silent_connection_does_not_block_valid_mcp_callback -- --nocapture --test-threads=1`

Expected: FAIL par expiration du délai du test.

- [ ] **Step 3: Aligner le serveur sur le propriétaire Codex**

Accepter les sockets dans un `JoinSet`, copier le `state` attendu dans `Zeroizing<String>` pour chaque handler, entourer `handle_connection` de `timeout(CONNECTION_TIMEOUT, ...)`, et continuer après les résultats invalides. Ne jamais dépasser `MAX_ATTEMPTS` handlers acceptés.

- [ ] **Step 4: Exécuter les tests MCP OAuth ciblés**

Run: `cargo test --lib --features windows-tests services::mcp_oauth:: -- --nocapture --test-threads=1`

Expected: PASS, y compris mauvais state, annulation et socket muette.

---

### Task 7: Retirer l'injection Forecast de la compilation publiée

**Files:**
- Modify: `src-tauri/src/services/forecast/model_manager/uninstall.rs`
- Modify: `src-tauri/src/services/forecast/model_manager/tests.rs`
- Amend note metadata: git note de `bb2ec62` après création du commit correctif.

**Interfaces:**
- Produces: `UninstallTransaction` avec `remove_staging`, `remove_model`, `remove_unused_runtime`; `UninstallBoundary` et `uninstall_with_failure_after` uniquement sous `#[cfg(test)]`.
- Consumes: `fs_safety::remove_path`, `family_has_other_installed_model`, `remove_runtime`.

- [ ] **Step 1: Refactoriser les étapes sans modifier leur ordre**

La fonction de production construit la transaction puis appelle exactement les trois étapes. Aucune étape n'accepte de paramètre d'injection.

```rust
let transaction = UninstallTransaction::prepare(model_id, models, sidecar)?;
transaction.remove_staging().await?;
transaction.remove_model().await?;
transaction.remove_unused_runtime().await
```

Le changement est un contrat de compilation, pas un nouveau comportement runtime : les tests d'invariant existants appellent les mêmes étapes réelles via une orchestration `#[cfg(test)]`.

- [ ] **Step 2: Placer toute injection sous `#[cfg(test)]`**

Annoter l'enum et le helper de scénario ; supprimer `fail_after` et `fail_if_requested` de tout chemin de production. L'orchestrateur de test appelle une étape réelle, compare la frontière demandée, puis poursuit.

- [ ] **Step 3: Vérifier les invariants et la compilation de production**

Run: `cargo test --lib --features windows-tests every_uninstall_boundary_preserves_the_model_runtime_invariant -- --nocapture --test-threads=1`

Run: `cargo test --lib --features windows-tests services::forecast::model_manager:: -- --nocapture --test-threads=1`

Run: `cargo check --lib`

Expected: PASS ; `cargo check` compile le chemin non-test, puis la relecture du diff confirme que `UninstallBoundary` et l'orchestration d'injection sont entièrement derrière `#[cfg(test)]`.

- [ ] **Step 4: Corriger l'audit et committer le lot OAuth/Forecast**

Commit: `fix(runtime): close OAuth and Forecast review gaps`

Ajouter sa git note avec le test socket rouge/vert, les comptes MCP/Forecast et `cargo check`. Réécrire la git note de `bb2ec62` en conservant son historique, mais remplacer l'attribution erronée de l'obligation 28 par le hash du nouveau commit.

---

### Task 8: Vérification finale, Graphify, push et CI

**Files:**
- Modify if needed: `docs/superpowers/specs/2026-08-13-shutdown-milestone-2-minors-review.md` uniquement pour ajouter le statut des sept constats, sans effacer la review d'origine.
- Update generated graph: `graphify-out/` (ignoré par Git).

**Interfaces:**
- Consumes: les trois commits de correction et leurs git notes.
- Produces: branche poussée, notes poussées, CI observée, plage de commits prête pour une nouvelle review.

- [ ] **Step 1: Contrôler le périmètre et la structure**

Run: `git diff 5915271..HEAD --check`

Run: script PowerShell qui compte chaque fichier Rust de production touché et échoue au-dessus de 230 lignes.

Expected: aucun espace fautif, aucun fichier de production au-dessus de 230 lignes.

- [ ] **Step 2: Exécuter les vérifications Rust fraîches**

Run: `cargo fmt --check`

Run: `cargo test --lib --features windows-tests -- --test-threads=1`

Run: `cargo check`

Run: `cargo clippy --all-targets -- -D warnings`

Le profil `windows-tests` reste réservé à `cargo test` : il désactive le navigateur natif pour ne pas charger CEF dans le binaire de tests, tandis que Clippy doit vérifier le profil Windows natif utilisé par la CI.

Expected: toutes les commandes sortent avec le code 0 ; consigner les comptes exacts.

- [ ] **Step 3: Exécuter les vérifications frontend fraîches**

Run: `npm test`

Run: `npx tsc --noEmit`

Run: `npm run lint`

Expected: toutes les commandes sortent avec le code 0 ; aucune erreur ou avertissement nouveau.

- [ ] **Step 4: Maintenir le graphe et relire les preuves**

Run: `graphify update .`

Relire `git log --show-notes`, `git status --short`, les sept critères de la review et les trois diffs de correction. Ne déclarer vert que ce qui possède une sortie fraîche.

- [ ] **Step 5: Pousser sans fusionner et surveiller la CI**

Run: `git push origin codex/shutdown-milestone-2-minors`

Run: `git push origin refs/notes/commits`

Mettre à jour la PR existante, attendre tous les contrôles GitHub et lire chaque résultat. Si un contrôle échoue, diagnostiquer sa cause avant tout nouveau correctif. Si tout est vert, donner la plage `f88e6db..HEAD` au reviewer et attendre sa validation ; ne pas fusionner.
