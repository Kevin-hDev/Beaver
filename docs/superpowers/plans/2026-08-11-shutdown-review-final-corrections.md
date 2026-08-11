# Shutdown Review Final Corrections Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Corriger N1, N2, N12, N3 et N4 afin que la publication Windows soit réellement vérifiée, que le redémarrage survive au watchdog et que la supervision CEF reste bornée, cohérente et polie.

**Architecture:** `AppExitCoordinator` conserve l'intention et la chronologie uniques. Les trackers CEF ajoutent une échéance de publication dérivée de `CEF_ADMISSION_TIMEOUT`; Windows ajoute une table d'urgence fixe qui conserve les objets de signalisation indépendamment du thread normal. Les transitions concurrentes utilisent les générations et les comparaisons atomiques déjà présentes.

**Tech Stack:** GitHub Actions YAML, Node.js 24 avec `js-yaml`, Rust stable, Tauri 2, `windows-sys` 0.61, tests `node:test` et `cargo test`.

## Global Constraints

- Corriger uniquement N1, N2, N12, N3 et N4; laisser N5 à N11 et N13 à N15 hors périmètre.
- Écrire et exécuter chaque test rouge avant son correctif.
- Conserver `ExitIntent`, `ShutdownTimeline`, `CEF_ADMISSION_TIMEOUT` et les tables de 64 slots comme autorités uniques.
- Utiliser uniquement des ticks monotones inter-processus pour l'échéance Windows; ne jamais les comparer à l'heure murale.
- Une expiration sans publication est récupérable; une publication incohérente reste fatale.
- Les traces restent génériques et n'exposent ni chemin, nonce, ligne de commande, handle ou jeton.
- Aucun fichier de production ou de test mêlant plusieurs responsabilités ne doit dépasser 230 lignes; extraire de nouveaux modules ciblés avant ce seuil.
- Sous Windows, toute commande `cargo test` de ce plan doit ajouter `--lib --features windows-tests`, définir `CARGO_TARGET_DIR` sur un chemin court et mettre Ninja dans `PATH`. Cette feature est l'autorité du projet pour les tests sans chargement de la DLL CEF de distribution.

---

### Task 1: N1 — Vérification structurelle et effective du paquet Windows

**Files:**
- Modify: `package.json`
- Modify: `package-lock.json`
- Modify: `.github/workflows/release.yml`
- Modify: `scripts/release/release-workflow.test.mjs`

**Interfaces:**
- Consumes: `jobs.build.steps` du workflow GitHub Actions.
- Produces: une étape `Inspect and install Windows package` dont `env` contient `CARGO_BUILD_TARGET` et `BEAVER_TAURI_BUNDLE_TYPE`, et dont le script propage immédiatement l'échec de `tauri-bundle-marker.mjs verify`.

- [ ] **Step 1: Déclarer le parseur YAML direct**

Run: `npm install --save-dev --save-exact js-yaml@4.3.1`

Expected: `package.json` et `package-lock.json` déclarent directement `js-yaml` 4.3.1; l'override existant reste identique.

- [ ] **Step 2: Remplacer le contrôle textuel N1 par un test structurel rouge**

Dans `scripts/release/release-workflow.test.mjs`, charger le YAML et sélectionner l'étape par son nom réel :

```js
import { load as loadYaml } from "js-yaml";

const workflowDocument = loadYaml(workflow);

test("la vérification Windows possède ses variables et propage son échec natif", () => {
  const step = workflowDocument.jobs.build.steps.find(
    ({ name }) => name === "Inspect and install Windows package",
  );
  assert.ok(step);
  assert.equal(step.env.CARGO_BUILD_TARGET, "${{ matrix.target }}");
  assert.equal(step.env.BEAVER_TAURI_BUNDLE_TYPE, "${{ matrix.bundles }}");
  assert.match(
    step.run,
    /tauri-bundle-marker\.mjs verify[\s\S]*?if \(\$LASTEXITCODE -ne 0\) \{[\s\S]*?throw/,
  );
});
```

Supprimer de `inspecte chaque bundle avec son outil natif` les deux assertions globales N1 qui ne prouvent pas la portée de l'étape.

- [ ] **Step 3: Exécuter le test et observer l'échec attendu**

Run: `node --test --test-name-pattern="vérification Windows possède" scripts/release/release-workflow.test.mjs`

Expected: FAIL parce que `CARGO_BUILD_TARGET` et `BEAVER_TAURI_BUNDLE_TYPE` sont absents de l'environnement de l'étape consommatrice.

- [ ] **Step 4: Fournir les variables et propager l'erreur native**

Dans l'étape `Inspect and install Windows package` :

```yaml
env:
  BEAVER_INSTALLER: ${{ steps.paths.outputs.asset }}
  CARGO_BUILD_TARGET: ${{ matrix.target }}
  BEAVER_TAURI_BUNDLE_TYPE: ${{ matrix.bundles }}
```

Juste après `tauri-bundle-marker.mjs verify` :

```powershell
if ($LASTEXITCODE -ne 0) {
  throw "Windows package validation failed."
}
```

- [ ] **Step 5: Vérifier N1**

Run: `npm run test:release-workflow`

Expected: tous les tests du workflow passent.

- [ ] **Step 6: Commit N1**

```powershell
git add package.json package-lock.json .github/workflows/release.yml scripts/release/release-workflow.test.mjs
git commit -m "fix: enforce Windows bundle verification"
```

### Task 2: N2 — Action finale de fermeture unique et fidèle à l'intention

**Files:**
- Create: `src-tauri/src/app_exit/final_action.rs`
- Create: `src-tauri/src/app_exit/final_action_tests.rs`
- Modify: `src-tauri/src/app_exit.rs`
- Modify: `src-tauri/src/app_exit/request_flow.rs`
- Modify: `src-tauri/src/app_exit/watchdog.rs`
- Modify: `src-tauri/src/app_exit/watchdog_tests.rs`

**Interfaces:**
- Consumes: `ShutdownState`, `ExitIntent`, code de sortie et origine `Cleanup | Watchdog`.
- Produces: `final_action::run(state, intent, exit_code, source, dispatch) -> bool`; seul le gagnant de `Closing -> ReadyToExit` appelle `dispatch(ExitIntent, i32)`.

- [ ] **Step 1: Écrire les tests rouges de l'action finale**

Créer `final_action_tests.rs` avec deux tests :

```rust
#[test]
fn watchdog_restart_dispatches_restart_once() {
    let state = ShutdownState::new();
    assert_eq!(state.begin_closing(), BeginClosing::Started);
    let calls = AtomicUsize::new(0);

    assert!(run(&state, ExitIntent::Restart, 0, FinalActionSource::Watchdog, |intent, _| {
        assert_eq!(intent, ExitIntent::Restart);
        calls.fetch_add(1, Ordering::AcqRel);
    }));
    assert_eq!(calls.load(Ordering::Acquire), 1);
}

#[test]
fn cleanup_after_watchdog_cannot_dispatch_a_second_action() {
    let state = ShutdownState::new();
    assert_eq!(state.begin_closing(), BeginClosing::Started);
    let calls = AtomicUsize::new(0);
    let dispatch = |_, _| { calls.fetch_add(1, Ordering::AcqRel); };

    assert!(run(&state, ExitIntent::Restart, 0, FinalActionSource::Watchdog, dispatch));
    assert!(!run(&state, ExitIntent::Restart, 0, FinalActionSource::Cleanup, dispatch));
    assert_eq!(calls.load(Ordering::Acquire), 1);
}
```

- [ ] **Step 2: Exécuter les tests et observer l'échec attendu**

Run: `cargo test app_exit::final_action_tests --manifest-path src-tauri/Cargo.toml`

Expected: FAIL parce que `final_action` et son API n'existent pas.

- [ ] **Step 3: Implémenter l'unique transition finale**

Dans `final_action.rs` :

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum FinalActionSource { Cleanup, Watchdog }

pub(super) fn run(
    state: &ShutdownState,
    intent: ExitIntent,
    exit_code: i32,
    source: FinalActionSource,
    dispatch: impl FnOnce(ExitIntent, i32),
) -> bool {
    if !state.mark_ready() {
        ::log::info!("[exit] final action already claimed");
        return false;
    }
    if source == FinalActionSource::Watchdog && intent == ExitIntent::Restart {
        ::log::warn!("[exit] restart triggered by watchdog");
    }
    dispatch(intent, exit_code);
    true
}
```

La raison est inscrite près du CAS : cette transition est l'autorité unique afin que le chemin normal et le watchdog ne puissent pas agir deux fois.

- [ ] **Step 4: Brancher le nettoyage et le watchdog sur la même fonction**

- Ajouter les modules dans `app_exit.rs` sans dépasser 230 lignes.
- Transmettre `ExitIntent` à `spawn_watchdog` puis `WatchdogThread::spawn`.
- Dans `request_flow.rs`, remplacer le `mark_ready` suivi du `match` par `final_action::run(..., Cleanup, ...)`.
- Dans `watchdog.rs`, appeler `final_action::run(..., Watchdog, ...)` à l'échéance de 10 secondes.
- La fonction de dispatch Tauri garde exactement : `Exit => app.exit(code)`, `Restart => app.request_restart()`.

- [ ] **Step 5: Vérifier N2 et la non-régression du watchdog**

Run: `cargo test app_exit::final_action_tests --manifest-path src-tauri/Cargo.toml`

Run: `cargo test app_exit::watchdog_tests --manifest-path src-tauri/Cargo.toml`

Expected: les tests de l'action finale et du watchdog passent; le compteur reste à un lorsque nettoyage et watchdog se succèdent.

- [ ] **Step 6: Commit N2**

```powershell
git add src-tauri/src/app_exit.rs src-tauri/src/app_exit/final_action.rs src-tauri/src/app_exit/final_action_tests.rs src-tauri/src/app_exit/request_flow.rs src-tauri/src/app_exit/watchdog.rs src-tauri/src/app_exit/watchdog_tests.rs
git commit -m "fix: preserve restart through shutdown watchdog"
```

### Task 3: N12 — Une échéance CEF absolue à 13 secondes

**Files:**
- Modify: `src-tauri/src/app_exit/policy.rs`
- Modify: `src-tauri/src/app_exit/policy_tests.rs`

**Interfaces:**
- Consumes: `ShutdownTimeline::emergency_deadline()`.
- Produces: `ShutdownTimeline::cef_helper_exit_deadline()` strictement égal à l'échéance d'urgence.

- [ ] **Step 1: Modifier les attentes pour obtenir un test rouge**

Dans les deux tests de chronologie, attendre littéralement 13 secondes ou 13 millisecondes :

```rust
assert_eq!(timeline.cef_helper_exit_deadline(), timeline.emergency_deadline());
```

- [ ] **Step 2: Exécuter le test et observer l'échec attendu**

Run: `cargo test app_exit::policy_tests --manifest-path src-tauri/Cargo.toml`

Expected: FAIL; l'implémentation renvoie encore 14 secondes.

- [ ] **Step 3: Supprimer le budget concurrent**

Supprimer `CEF_HELPER_EXIT_MARGIN` et remplacer l'implémentation par :

```rust
pub(super) fn cef_helper_exit_deadline(self) -> Instant {
    self.emergency_deadline()
}
```

- [ ] **Step 4: Vérifier et commit N12**

Run: `cargo test app_exit::policy_tests --manifest-path src-tauri/Cargo.toml`

Expected: tous les tests de politique passent.

```powershell
git add src-tauri/src/app_exit/policy.rs src-tauri/src/app_exit/policy_tests.rs
git commit -m "fix: align CEF shutdown deadline"
```

### Task 4: N3 — Expiration récupérable des publications absentes

**Files:**
- Modify: `src-tauri/src/services/browser/cef_supervision/constants.rs`
- Modify: `src-tauri/src/services/browser/cef_supervision/table_tests.rs`
- Create: `src-tauri/src/services/browser/cef_supervision/windows/tracker_reservation.rs`
- Modify: `src-tauri/src/services/browser/cef_supervision/windows.rs`
- Modify: `src-tauri/src/services/browser/cef_supervision/windows/tracker.rs`
- Modify: `src-tauri/src/services/browser/cef_supervision/windows/tracker_pending.rs`
- Modify: `src-tauri/src/services/browser/cef_supervision/windows/tracker_loop.rs`
- Modify: `src-tauri/src/services/browser/cef_supervision/windows/tracker_tests.rs`
- Create: `src-tauri/src/services/browser/cef_supervision/macos/tracker_reservation.rs`
- Modify: `src-tauri/src/services/browser/cef_supervision/macos.rs`
- Modify: `src-tauri/src/services/browser/cef_supervision/macos/tracker.rs`
- Modify: `src-tauri/src/services/browser/cef_supervision/macos/pending.rs`
- Modify: `src-tauri/src/services/browser/cef_supervision/macos/tracker_loop.rs`
- Modify: `src-tauri/src/services/browser/cef_supervision/macos/tracker_tests.rs`

**Interfaces:**
- Consumes: `CEF_ADMISSION_TIMEOUT`, `CefAuthoritySlot::clear_if(generation, SLOT_RESERVED)`.
- Produces: `publication_deadline(now: Instant) -> Instant`, une échéance dans chaque `WindowsPendingLaunch` et `MacPendingLaunch`, et un retrait atomique `take_if_expired(slot, now)`.

- [ ] **Step 1: Écrire le test rouge de la course atomique**

Dans `table_tests.rs`, faire courir deux threads synchronisés : l'un abandonne la réservation, l'autre appelle `claim`. Vérifier sur plusieurs itérations que soit le claim gagne, soit l'abandon gagne, mais qu'une réservation d'une génération suivante reste toujours utilisable.

Mutation visée : remplacer `clear_if(generation, SLOT_RESERVED)` par un effacement non conditionnel doit faire échouer ce test.

- [ ] **Step 2: Écrire les tests rouges des 64 expirations**

Ajouter sur chaque plateforme un test qui :

```rust
for _ in 0..CEF_SLOT_CAPACITY {
    tracker.handle().reserve_expired_for_test().expect("expired reservation");
}
wait_until_all_expired(&tracker);
assert!(tracker.handle().reserve().is_ok());
assert_eq!(tracker.failure_for_test(), None);
```

Le helper de test fixe l'échéance dans le passé; il ne modifie pas la constante de production.

- [ ] **Step 3: Exécuter les tests et observer les échecs attendus**

Windows :

`cargo test cef_supervision::table_tests --manifest-path src-tauri/Cargo.toml`

`cargo test cef_supervision::windows_tracker_tests --manifest-path src-tauri/Cargo.toml`

Expected: FAIL car les réservations sans publication ne sont jamais retirées et la 65e réservation est refusée.

macOS, sur runner macOS :

`cargo test cef_supervision::table_tests --manifest-path src-tauri/Cargo.toml`

`cargo test cef_supervision::macos_tracker_tests --manifest-path src-tauri/Cargo.toml`

Expected: même échec de capacité.

- [ ] **Step 4: Dériver et stocker l'échéance unique**

Dans `constants.rs` :

```rust
pub(super) fn publication_deadline(now: Instant) -> Instant {
    now.checked_add(CEF_ADMISSION_TIMEOUT).unwrap_or(now)
}
```

Extraire les implémentations de réservation dans `tracker_reservation.rs` pour garder `tracker.rs` sous 230 lignes. Les chemins de production utilisent `publication_deadline(Instant::now())`; les méthodes `reserve_expired_for_test` appellent la même implémentation avec une échéance passée.

- [ ] **Step 5: Expirer sans fermer Beaver**

Sur les deux plateformes :

- `take_if_expired` retire uniquement le pointeur ou l'option encore présent dans le slot concerné;
- après le retrait, relire une dernière fois la mailbox;
- si elle est publiée, poursuivre l'admission normale;
- si elle reste `Unpublished`, écrire `[browser] CEF helper publication expired`, laisser tomber la réservation et continuer sans `fail`;
- toute autre erreur ou incohérence conserve le chemin fatal existant.

- [ ] **Step 6: Vérifier la course et la récupération Windows**

Run: `cargo test cef_supervision::table_tests --manifest-path src-tauri/Cargo.toml`

Run: `cargo test cef_supervision::windows_tracker_tests --manifest-path src-tauri/Cargo.toml`

Expected: les 64 slots sont recyclés, la 65e réservation réussit et aucune panne du superviseur n'est enregistrée.

- [ ] **Step 7: Commit N3**

```powershell
git add src-tauri/src/services/browser/cef_supervision
git commit -m "fix: expire unpublished CEF reservations"
```

### Task 5: N4 — Signal poli Windows indépendant du tracker normal

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Create: `src-tauri/src/services/browser/cef_supervision/windows/clock.rs`
- Create: `src-tauri/src/services/browser/cef_supervision/windows/clock_tests.rs`
- Create: `src-tauri/src/services/browser/cef_supervision/windows/emergency_slots.rs`
- Create: `src-tauri/src/services/browser/cef_supervision/windows/emergency_slots_tests.rs`
- Create: `src-tauri/src/services/browser/cef_supervision/windows/helper_monitor.rs`
- Create: `src-tauri/src/services/browser/cef_supervision/windows/helper_monitor_tests.rs`
- Modify: `src-tauri/src/services/browser/cef_supervision/windows.rs`
- Modify: `src-tauri/src/services/browser/cef_supervision/windows/objects.rs`
- Modify: `src-tauri/src/services/browser/cef_supervision/windows/tracker.rs`
- Modify: `src-tauri/src/services/browser/cef_supervision/windows/tracker_reservation.rs`
- Modify: `src-tauri/src/services/browser/cef_supervision/windows/tracker_pending.rs`
- Modify: `src-tauri/src/services/browser/cef_supervision/windows/tracker_loop.rs`
- Modify: `src-tauri/src/services/browser/cef_supervision/windows/tracker_tests.rs`
- Modify: `src-tauri/src/services/browser/cef_supervision/bootstrap.rs`

**Interfaces:**
- Consumes: l'échéance absolue de 13 secondes, `WindowsPublicationObjects::begin_closing`, `WindowsNativeAuthority` et le Job Object existant.
- Produces: `clock::ticks_at(Instant)`, `clock::reached(u64)`, `WindowsEmergencySlots` de 64 entrées et un moniteur enfant arrêté/rejoint par `WindowsHelperAdmission`.

- [ ] **Step 1: Écrire les tests rouges de l'horloge et des objets d'urgence**

Tests requis :

```rust
#[test]
fn monotonic_deadline_reaches_without_wall_clock() {
    let ticks = ticks_at(Instant::now() + Duration::from_millis(20)).expect("ticks");
    assert!(!reached(ticks).expect("before"));
    while !reached(ticks).expect("after") { std::thread::yield_now(); }
}

#[test]
fn emergency_slots_signal_pending_and_admitted_objects() {
    // Installer deux générations dans deux slots, appeler begin_closing,
    // puis vérifier les événements et le même deadline_ticks dans chaque page.
}
```

Mutation visée : retirer l'appel à `begin_closing` d'un slot doit laisser son événement non signalé et faire échouer le test.

- [ ] **Step 2: Écrire le test rouge du moniteur enfant**

Avec une action de test qui incrémente un compteur, démarrer le moniteur avant l'admission, signaler une fermeture dont l'échéance est atteinte, puis attendre que le compteur vaille un. Ajouter un second cas où la génération attendue est différente.

- [ ] **Step 3: Exécuter les tests et observer l'échec attendu**

Run: `cargo test cef_supervision::windows_clock_tests --manifest-path src-tauri/Cargo.toml`

Run: `cargo test cef_supervision::windows_emergency_slots_tests --manifest-path src-tauri/Cargo.toml`

Run: `cargo test cef_supervision::windows_helper_monitor_tests --manifest-path src-tauri/Cargo.toml`

Expected: FAIL parce que les trois modules n'existent pas et `begin_closing` est encore réservé aux tests.

- [ ] **Step 4: Ajouter l'horloge monotone Windows**

Ajouter la feature `Win32_System_WindowsProgramming` à `windows-sys`. `clock.rs` appelle `QueryUnbiasedInterruptTimePrecise`, dont les ticks de 100 ns sont partagés entre processus. Convertir la durée restante par arrondi supérieur et utiliser des additions vérifiées; zéro ou dépassement renvoie `CefUnavailableCategory::Object`.

- [ ] **Step 5: Ajouter la table d'urgence fixe**

`WindowsEmergencySlots` contient exactement `[Mutex<Option<Entry>>; CEF_SLOT_CAPACITY]`. `Entry` conserve génération et `Arc<WindowsPublicationObjects>`. `install` renvoie une inscription RAII; sa destruction appelle `clear(slot, generation)` afin qu'une ancienne génération ne puisse pas effacer la suivante. `begin_closing` parcourt les 64 cases, tente tous les signaux et ne masque aucun échec.

- [ ] **Step 6: Brancher la table avant le thread normal**

- Construire `WindowsEmergencySlots` dans `WindowsTrackerShared` avant le spawn du tracker.
- Installer l'objet juste après sa création et avant `pending.install`.
- Déplacer l'inscription RAII de `WindowsPendingLaunch` vers `ActiveHelper` après admission.
- À la sortie du helper ou sur toute erreur, la destruction de l'inscription libère seulement la génération correspondante.
- Dans `emergency_close`, fermer les portes, calculer les ticks à partir de `helper_exit_deadline`, puis appeler `emergency.begin_closing`; un échec appelle `fail` mais le Job Object reste armé.

- [ ] **Step 7: Démarrer le moniteur enfant avant l'attente d'admission**

Retirer `#[cfg(test)]` de `WindowsPublicationObjects::begin_closing`. Convertir les objets helper en `Arc`, démarrer `helper_monitor` immédiatement après `publish`, puis appeler `wait_for_parent`. Le moniteur :

- échoue fermé si la page de contrôle est invalide ou sa génération diffère;
- attend l'événement de fermeture par tranches bornées;
- appelle `TerminateProcess(GetCurrentProcess(), 1)` lorsque `clock::reached(deadline_ticks)`;
- est arrêté et rejoint dans `Drop for WindowsHelperAdmission` si `cef::execute_process` revient.

- [ ] **Step 8: Vérifier N4 sur Windows**

Run: `cargo test cef_supervision::windows_clock_tests --manifest-path src-tauri/Cargo.toml`

Run: `cargo test cef_supervision::windows_emergency_slots_tests --manifest-path src-tauri/Cargo.toml`

Run: `cargo test cef_supervision::windows_helper_monitor_tests --manifest-path src-tauri/Cargo.toml`

Run: `cargo test cef_supervision::windows_tracker_tests --manifest-path src-tauri/Cargo.toml`

Expected: le signal atteint les objets en attente et admis, le moniteur réagit à l'échéance, et le Job Object force toujours le processus lors de la phase d'urgence.

- [ ] **Step 9: Commit N4**

```powershell
git add src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src/services/browser/cef_supervision
git commit -m "fix: signal Windows CEF helpers before force"
```

### Task 6: Vérifications intégrales et maintenance du graphe

**Files:**
- Modify if needed: files already listed above only.
- Update generated knowledge graph: `graphify-out/` (ignored by Git).

**Interfaces:**
- Consumes: les cinq corrections commitées.
- Produces: preuves fraîches de formatage, compilation, lint et tests.

- [ ] **Step 1: Contrôler les tailles et les changements**

Run: `git diff 4e9d212..HEAD --check`

Run: compter les lignes de tous les fichiers Rust/MJS modifiés et découper toute responsabilité de production dépassant 230 lignes.

- [ ] **Step 2: Vérifier Node et le workflow**

Run: `npm run test:release-workflow`

Expected: PASS sans test vide.

- [ ] **Step 3: Vérifier Rust complètement**

Run: `cargo fmt --check --manifest-path src-tauri/Cargo.toml`

Run: `cargo check --manifest-path src-tauri/Cargo.toml`

Run: `cargo clippy --all-targets --manifest-path src-tauri/Cargo.toml -- -D warnings`

Run (Windows): `cargo test --lib --features windows-tests --manifest-path src-tauri/Cargo.toml -- --test-threads=1`

Run (macOS): `cargo test --manifest-path src-tauri/Cargo.toml`

Expected: chaque commande sort avec le code 0.

- [ ] **Step 4: Maintenir et diagnostiquer Graphify**

Run: `graphify update .`

Run: `graphify diagnose multigraph --graph graphify-out/graph.json --json`

Expected: zéro endpoint manquant, zéro relation pendante, zéro doublon exact et zéro effondrement de relation.

- [ ] **Step 5: Vérifier les critères d'acceptation**

- Le YAML est analysé structurellement et l'étape Windows propage `$LASTEXITCODE`.
- L'action finale n'est exécutée qu'une fois et conserve `Restart` au watchdog.
- Toutes les échéances enfant CEF valent l'échéance absolue de 13 secondes.
- Les 64 réservations sans publication expirent sans panne et la génération suivante survit.
- Les helpers Windows en attente et admis reçoivent le signal avant le Job Object.
- Aucun secret ou détail interne n'apparaît dans les traces ajoutées.

- [ ] **Step 6: Traiter toute régression avant clôture**

Si une vérification échoue, revenir à la tâche propriétaire du comportement, ajouter d'abord un test rouge qui reproduit la régression, appliquer le correctif minimal, puis répéter l'intégralité de cette tâche de vérification. Ne créer aucun commit générique qui mélangerait plusieurs propriétaires.
