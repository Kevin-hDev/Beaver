# macOS CEF Early Loading Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Charger CEF au tout début du processus macOS afin de supprimer le `SIGTRAP` intermittent, tout en conservant les chemins de démarrage Windows et Linux.

**Architecture:** Un petit coordinateur de démarrage testable impose l'ordre « charger CEF, préparer AppKit, puis capturer l'environnement shell » et renvoie un garde opaque. Le point d'entrée macOS transmet ce garde au cycle de vie Tauri. Le moteur CEF exige une référence à ce garde et réutilise les chemins validés qu'il contient, ce qui empêche par construction tout chargement tardif ou toute seconde résolution divergente. À la fermeture, un second coordinateur arrête CEF avant de détruire le garde.

**Tech Stack:** Rust, Tauri 2, cef-rs, tests unitaires comportementaux.

**Global constraints:** Limiter le changement à macOS avec `cfg(target_os = "macos")`, ne pas modifier `windows_entry.rs`, ne pas inclure CEF dans Linux, garder chaque fichier source sous 230 lignes, ne révéler aucun chemin interne dans les erreurs, puis mettre à jour Graphify après les changements de code.

---

### Task 1: Écrire les tests rouges du coordinateur macOS

**Files:**
- Create: `src-tauri/src/startup_tests.rs`
- Modify: `src-tauri/src/lib.rs`

**Step 1: Write a failing order test**

Créer un test qui appelle l'API souhaitée `prepare_macos_browser` avec deux opérations contrôlées. La première ajoute `load` à une liste, la seconde ajoute `prepare`. Le résultat doit contenir le garde et la liste doit être exactement `load`, puis `prepare`.

Le changement de production que ce test protège est l'inversion accidentelle de l'ordre, qui recréerait le crash intermittent.

```rust
#[test]
fn macos_browser_library_loads_before_native_application() {
    let events = RefCell::new(Vec::new());
    let guard = prepare_macos_browser(
        || {
            events.borrow_mut().push("load");
            Ok(TestGuard::new(&events))
        },
        || {
            events.borrow_mut().push("prepare");
            true
        },
    );

    assert!(guard.is_some());
    assert_eq!(*events.borrow(), ["load", "prepare"]);
}
```

**Step 2: Add failure behavior tests**

Ajouter deux tests séparés :

- si le chargement échoue, la préparation native n'est jamais appelée et le résultat vaut `None` ;
- si la préparation native échoue, le garde est immédiatement détruit et le résultat vaut `None`.

Ces tests protègent le démarrage dégradé de Beaver sans navigateur, sans laisser une bibliothèque chargée inutilement.

Ajouter aussi un test à trois opérations contrôlées qui exige l'ordre exact
`load`, `prepare`, `shell`. Il protège l'invariant principal : aucun thread de
capture du shell ne peut exister au moment du chargement de CEF.

Ajouter un test avec un descendant sorti du groupe de processus et conservant
le tube ouvert. La capture doit respecter son échéance sans attendre ce lecteur
indéfiniment. CEF étant déjà chargé, un lecteur tardif ne peut plus recréer la
fenêtre du crash.

**Step 3: Add shutdown ordering test**

Tester `shutdown_before_library_unload` avec un garde dont `Drop` ajoute `unload` à la liste et une fermeture qui ajoute `shutdown`. L'ordre attendu est exactement `shutdown`, puis `unload`.

**Step 4: Register and run the tests to verify RED**

Déclarer dans `lib.rs` :

```rust
#[cfg(test)]
mod startup_tests;
```

Run: `cd src-tauri && cargo test startup_tests --lib -- --nocapture`

Expected: FAIL à la compilation parce que `prepare_macos_browser` et `shutdown_before_library_unload` n'existent pas encore. Cet échec correspond exactement aux deux comportements manquants.

**Step 5: Commit the red tests**

```bash
git add src-tauri/src/lib.rs src-tauri/src/startup_tests.rs
git commit -m "test(browser): define macOS CEF bootstrap behavior"
```

### Task 2: Implémenter le coordinateur et le garde CEF

**Files:**
- Modify: `src-tauri/src/startup.rs`
- Modify: `src-tauri/src/services/browser/cef_library.rs`
- Modify: `src-tauri/src/services/browser/mod.rs`
- Modify: `src-tauri/src/lib.rs`

**Step 1: Implement the minimal coordinator**

Dans `startup.rs`, ajouter deux fonctions génériques compilées pour les tests et macOS :

```rust
#[cfg(any(test, target_os = "macos"))]
pub(crate) fn prepare_macos_browser<Guard>(
    load_library: impl FnOnce() -> Result<Guard, ()>,
    prepare_native: impl FnOnce() -> bool,
) -> Option<Guard> {
    let library = load_library().ok()?;
    prepare_native().then_some(library)
}

#[cfg(any(test, target_os = "macos"))]
pub(crate) fn shutdown_before_library_unload<Guard>(
    library: Option<Guard>,
    shutdown: impl FnOnce(),
) {
    shutdown();
    drop(library);
}
```

**Step 2: Run coordinator tests to verify GREEN**

Run: `cd src-tauri && cargo test startup_tests --lib -- --nocapture`

Expected: PASS pour l'ordre nominal, les deux replis et l'ordre de fermeture.

**Step 3: Expose an opaque macOS library guard**

Dans `cef_library.rs`, remplacer `CefLibrary` par `BrowserLibraryGuard`. Ajouter `load_for_current_process`, qui obtient l'exécutable courant, récupère le répertoire CEF et réutilise `native_paths::resolve_runtime_files` avant d'appeler la logique `cef::load_library` existante. Le garde conserve le `RuntimeFiles` validé pour le transmettre au moteur. Conserver le `Drop` existant qui appelle `cef::unload_library`.

Dans `browser/mod.rs`, réexporter le garde uniquement sur macOS. Dans `startup.rs`, ajouter `load_macos_browser_library()`, puis réexporter la fonction et le type depuis `lib.rs` sous la même garde macOS.

**Step 4: Run focused tests and compilation**

Run: `cd src-tauri && cargo test startup_tests --lib -- --nocapture`

Expected: PASS.

Run: `cd src-tauri && cargo check`

Expected: le nouveau garde compile avant son branchement dans `main.rs`.

**Step 5: Commit the coordinator and loader**

```bash
git add src-tauri/src/startup.rs src-tauri/src/services/browser/cef_library.rs src-tauri/src/services/browser/mod.rs src-tauri/src/lib.rs
git commit -m "feat(browser): add early macOS CEF bootstrap"
```

### Task 3: Brancher le chargement précoce et transférer la propriété

**Files:**
- Modify: `src-tauri/src/main.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/services/browser/runtime_integration.rs`
- Modify: `src-tauri/src/services/browser/cef_engine.rs`

**Step 1: Load CEF before AppKit and shell capture in the macOS entry point**

Après le helper éventuel et la politique réseau Git, appeler le coordinateur macOS. Il charge CEF, prépare l'application native puis capture l'environnement shell. Si le garde vaut `None`, écrire uniquement le message générique existant. Sur macOS, appeler ensuite `cl_go_dash_lib::run(browser_library)`. Sur Linux, conserver la capture du shell, la préparation native et `cl_go_dash_lib::run()` sans argument.

**Step 2: Keep the guard through the Tauri lifecycle**

Donner deux façades à `run` autour d'un même `run_inner` : une version macOS prenant `Option<BrowserLibraryGuard>` et une version non-macOS sans argument.

Lors de chaque événement Tauri, transmettre `browser_library.as_ref()` à l'intégration macOS. À la fin, appeler `shutdown_before_library_unload(browser_library, || services::browser::shutdown(&app_handle))`, puis `process::exit`. Le garde reste ainsi vivant pendant toute l'utilisation de CEF.

**Step 3: Make engine initialization require the guard**

Sur macOS, ajouter un paramètre `&BrowserLibraryGuard` à `runtime_integration::setup_on_run_event`, `cef_engine::initialize` et `initialize_inner`. Supprimer l'appel tardif `CefLibrary::load`, puis faire consommer au moteur le `RuntimeFiles` déjà validé par le garde au lieu de résoudre les chemins une seconde fois. Sur Windows, la résolution et les signatures existantes continuent de compiler grâce à `cfg`.

Cette dépendance typée empêche de réintroduire un démarrage du moteur macOS sans bibliothèque préchargée.

**Step 4: Run focused tests and checks**

Run: `cd src-tauri && cargo test startup_tests --lib -- --nocapture`

Expected: PASS.

Run: `cd src-tauri && cargo check`

Expected: PASS sur macOS avec le nouveau flux complet.

**Step 5: Commit the behavior change**

```bash
git add src-tauri/src/main.rs src-tauri/src/lib.rs src-tauri/src/services/browser/runtime_integration.rs src-tauri/src/services/browser/cef_engine.rs
git commit -m "fix(browser): load CEF before macOS threads start"
```

### Task 4: Vérifier les frontières de plateformes et le repli

**Files:**
- Test: `src-tauri/src/startup_tests.rs`
- Test: `src-tauri/src/services/browser/runtime_handle_tests.rs`
- Test: `src-tauri/src/services/browser/build_policy_tests.rs`

**Step 1: Run all browser and startup tests**

Run: `cd src-tauri && cargo test startup_tests --lib -- --nocapture`

Expected: PASS ; le chargeur manquant et la préparation native en échec aboutissent tous deux à `None`, donc Beaver peut continuer sans navigateur.

Run: `cd src-tauri && cargo test services::browser --lib -- --nocapture`

Expected: PASS pour le cycle de vie, la politique Linux et le runtime navigateur.

Les tests doivent aussi vérifier que seul l'événement Tauri `Ready` autorise
l'initialisation, que `app_lifecycle::run` utilise le coordinateur de fermeture
dans le bon ordre, que `app_exit.rs` ne dépend jamais du navigateur et que le
point d'entrée macOS capture le shell via le coordinateur CEF unique.

**Step 2: Run platform policy tests**

Run: `cd src-tauri && cargo test build_policy_tests --lib -- --nocapture`

Expected: PASS ; Linux continue d'exclure les modules natifs. `windows_entry.rs` n'est pas modifié et continue d'appeler la version sans argument de `run` lors d'une compilation Windows.

**Step 3: Review platform-scoped diff**

Run: `git diff HEAD~1 -- src-tauri/src/main.rs src-tauri/src/lib.rs src-tauri/src/startup.rs src-tauri/src/services/browser`

Expected: tous les symboles du nouveau garde et du coordinateur de production sont protégés par `cfg(target_os = "macos")` ou `cfg(any(test, target_os = "macos"))`.

### Task 5: Validation complète et stress de lancement macOS

**Files:**
- Update: `graphify-out/` via l'outil Graphify

**Step 1: Run formatting and static checks**

Run: `cd src-tauri && cargo fmt --check`

Expected: PASS.

Run: `cd src-tauri && cargo check`

Expected: PASS.

Run: `cd src-tauri && cargo clippy --all-targets -- -D warnings`

Expected: PASS sans avertissement.

**Step 2: Run complete tests**

Run: `cd src-tauri && cargo test`

Expected: PASS pour toute la suite Rust.

Run: `npm run lint`

Expected: PASS.

Run: `npm test -- --run`

Expected: PASS.

Run: `npx tsc --noEmit`

Expected: PASS.

**Step 3: Update the knowledge graph**

Run: `graphify update .`

Expected: PASS pour la mise à jour AST locale.

**Step 4: Repeat macOS launches**

Construire ou lancer l'application dans le mode local disponible, puis effectuer au minimum 20 démarrages/fermetures consécutifs. À chaque passage, vérifier que Beaver atteint son écran principal et qu'aucun nouveau rapport `SIGTRAP` n'apparaît dans `~/Library/Logs/DiagnosticReports/`.

Si le runtime CEF n'est pas disponible dans l'environnement de test, documenter cette limite et s'appuyer sur les tests comportementaux, la suite complète et la CI multi-plateforme pour le reste.

**Step 5: Final review**

Run: `git diff --check && git status --short`

Expected: aucune erreur d'espace, uniquement les changements prévus.
