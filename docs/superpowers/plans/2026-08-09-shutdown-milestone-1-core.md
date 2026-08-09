# Shutdown Milestone 1 Core Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Garantir que le processus Beaver quitte au plus tard à l’échéance absolue, fermer toute nouvelle admission de travail dès le début de la fermeture et préserver les nettoyages actuels ainsi que l’ordre CEF existant.

**Architecture:** Remplacer le coordinateur minimal par une autorité unique composée d’un état monotone, d’un registre borné avec générations, d’une chronologie absolue et de deux filets indépendants. Le chemin normal garde les nettoyages existants derrière une frontière bloquante ; le watchdog demande la sortie Tauri à 10 secondes et le tueur ultime, créé avant tout effet de bord, force uniquement la mort du processus parent à 15 secondes. Le jalon crée l’interface de l’inventaire d’urgence sans migrer les producteurs, réservés au jalon 2, et ne modifie aucun protocole CEF, réservé au jalon 1B.

**Tech Stack:** Rust, Tauri 2, Tokio, `tokio_util::sync::CancellationToken`, atomiques standard, `windows-sys` sous Windows, `libc` sous Unix, Vitest/TypeScript pour les contrats frontend.

---

## Contraintes d’exécution

- Chaque modification de comportement commence par un test rouge observé.
- Aucun fichier Rust ou TypeScript de production ne dépasse 230 lignes.
- Les durées de production vivent uniquement dans `app_exit/policy.rs`.
- Les tests utilisent des durées injectées et des sorties factices ; aucun test ne peut tuer son propre processus.
- Une attente consomme le temps restant jusqu’à une échéance absolue ; elle ne recrée jamais un délai complet.
- Le code de ce jalon ne revendique pas la supervision forcée des helpers CEF.
- Aucun producteur de service n’est migré dans le registre général avant le jalon 2.
- `run_when_window_closed` disparaît des modèles actuels, mais Serde continue d’ignorer ce champ dans les anciens JSON.
- Après chaque lot : test ciblé, `cargo fmt --check`, puis commit local explicite.

### Task 1: Stabiliser la ligne de base sans reprendre le vieux lot processus

**Files:**
- Modify: `src-tauri/src/services/agent_local/subagent_completion.rs`
- Modify: `src-tauri/src/services/agent_local/subagent_completion_events.rs`
- Modify: `src-tauri/src/services/agent_local/subagent_terminal_event_order_tests.rs`
- Modify: `src-tauri/src/services/agent_local/subagent_completion_boundary_tests.rs`

**Step 1: Observer le test rouge existant**

Run:

```powershell
$env:CARGO_TARGET_DIR='C:\Users\huynh\projects\Beaver\src-tauri\target'
cargo test --manifest-path src-tauri/Cargo.toml --lib --features windows-tests message_between_terminal_save_and_registry_completion_is_never_stranded -- --exact --nocapture
```

Expected: échec au plafond de deux secondes, car le test tente de relire une session pendant que le verrou de cette session est encore détenu.

**Step 2: Ajouter un point de synchronisation uniquement destiné aux tests**

- Ajouter un callback `after_child_saved` à `persist_terminal_completion_inner` juste après la sauvegarde réussie et avant la publication de fin.
- Faire passer un callback vide dans tous les chemins de production.
- Étendre `persist_terminal_completion_with_hooks` au lieu de multiplier les wrappers de test.
- Remplacer le polling du test par un `oneshot` envoyé par ce callback.

**Step 3: Vérifier la correction isolée**

Run le test exact trois fois, puis :

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib --features windows-tests subagent_completion_boundary_tests -- --nocapture
```

Expected: vert et déterministe.

**Step 4: Commit de stabilisation**

```powershell
git add src-tauri/src/services/agent_local
git commit -m "test: stabilize subagent completion boundary"
```

La note finale précisera que la ligne 14 de l’inventaire reste ouverte : aucun parcours de processus de `c266c9c` n’est repris ici.

### Task 2: Centraliser la chronologie et l’état monotone

**Files:**
- Create: `src-tauri/src/app_exit/policy.rs`
- Create: `src-tauri/src/app_exit/state.rs`
- Create: `src-tauri/src/app_exit/policy_tests.rs`
- Create: `src-tauri/src/app_exit/state_tests.rs`
- Modify: `src-tauri/src/app_exit.rs`

**Step 1: Écrire les tests rouges**

Tester :

- `Running -> Closing -> ReadyToExit` seulement ;
- deux demandes concurrentes ne démarrent qu’une fermeture ;
- les échéances 8/10/13/15 secondes partagent exactement le même instant d’origine ;
- `remaining_until` renvoie zéro après l’échéance et ne recrée pas un budget.

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib --features windows-tests app_exit::state_tests app_exit::policy_tests
```

Expected: échec de compilation avant création des modules.

**Step 2: Implémenter le minimum**

- `ShutdownPhase` encodé par `AtomicU8` avec transitions comparées-échangées.
- `ShutdownPolicy` contient les quatre durées de production et peut être raccourci dans les tests.
- `ShutdownTimeline` capture une seule origine monotone et calcule toutes les échéances absolues.
- Aucun délai métier n’est laissé dans `app_exit.rs`.

**Step 3: Vérifier puis refactorer**

Run les deux modules de tests et `cargo fmt --check`.

### Task 3: Construire le registre d’admission borné et prouvable

**Files:**
- Create: `src-tauri/src/app_exit/registry.rs`
- Create: `src-tauri/src/app_exit/registry_tests.rs`
- Modify: `src-tauri/src/app_exit.rs`

**Step 1: Écrire les tests rouges**

Tester :

- 128 admissions réussissent, la 129e échoue fermée ;
- une fermeture atomique refuse toute admission suivante ;
- une admission déjà linéarisée reçoit l’annulation globale ;
- fin normale, panique et abandon d’une tâche libèrent exactement un slot ;
- une clé de génération périmée ne peut pas libérer un slot réutilisé ;
- `wait_empty_until` réussit à la dernière libération et expire à l’échéance absolue.

**Step 2: Implémenter sans collection extensible**

- Porte atomique combinant bit fermé et compteur actif.
- Tableau fixe de 128 slots atomiques ; chaque réutilisation incrémente sa génération.
- `TrackedAdmission` non `Clone`, possédant un jeton enfant et libérant la génération exacte dans `Drop`.
- Méthode consommante qui enveloppe une future afin que panique ou abort détruisent la garde.
- `Notify` préalloué uniquement pour réveiller l’attente normale ; le watchdog ne le consulte jamais.
- Code public stable `app-shutting-down` pour admission fermée, sans détail interne.

**Step 3: Vérifier les courses**

Run le module au moins dix fois avec plusieurs threads de test, puis une fois avec `--test-threads=1`.

### Task 4: Précréer le tueur ultime indépendant

**Files:**
- Create: `src-tauri/src/app_exit/raw_exit.rs`
- Create: `src-tauri/src/app_exit/ultimate.rs`
- Create: `src-tauri/src/app_exit/ultimate_tests.rs`
- Modify: `src-tauri/src/app_exit.rs`
- Modify: `src-tauri/src/lib.rs`

**Step 1: Écrire les tests rouges avec une sortie factice**

Tester :

- thread créé avant l’appel d’un callback représentant le premier effet de bord ;
- échec injecté de création : le callback d’effet de bord n’est jamais appelé ;
- armement unique à une échéance absolue ;
- appel factice exactement une fois à l’échéance même si le watchdog est bloqué ;
- désarmement de test propre sans thread abandonné ;
- panique de l’action primaire déclenche l’action de dernier recours factice.

**Step 2: Implémenter le chemin de production**

- Thread nommé, créé par `AppExitCoordinator::initialize()` avant `tauri::Builder`.
- État et échéance en atomiques préalloués ; réveil par `Thread::unpark`, sans canal créé à la fermeture.
- Windows : `TerminateProcess(GetCurrentProcess(), code)`.
- macOS/Linux : `libc::_exit(code)`.
- Le thread ne parcourt aucun registre et n’appelle aucun nettoyage.
- Une erreur de création est renvoyée ; `run_inner` refuse le démarrage avant la construction de Tauri.

**Step 3: Vérifier la plateforme compilée**

Run tests ciblés, `cargo check --features windows-tests`, puis vérifier par inspection `cfg` que les deux branches natives ont la même interface.

### Task 5: Ajouter l’inventaire d’urgence fixe et le watchdog séparé

**Files:**
- Create: `src-tauri/src/app_exit/emergency.rs`
- Create: `src-tauri/src/app_exit/watchdog.rs`
- Create: `src-tauri/src/app_exit/watchdog_tests.rs`
- Modify: `src-tauri/src/app_exit.rs`

**Step 1: Écrire les tests rouges**

Tester :

- inventaire fixe de 128 slots et refus fermé à saturation ;
- identité/génération périmée ignorée ;
- demande de sortie Tauri à 10 secondes à partir de l’origine commune ;
- début du drainage d’urgence à 13 secondes sans déplacer l’échéance 15 secondes ;
- échec injecté de création du watchdog sans modification du tueur déjà armé ;
- fonction de signalement factice bloquée : le tueur ultime s’exécute quand même.

**Step 2: Implémenter uniquement le socle J1**

- Slots atomiques préalloués et génération, sans balayage machine global.
- Le watchdog reçoit une vue fixe, marque `ReadyToExit` et demande `AppHandle::exit` à 10 secondes.
- Entre 13 et 15 secondes, il tente seulement les identités déjà publiées par l’interface.
- Aucun service n’est branché dans ces slots dans ce jalon ; cette adoption reste explicitement ouverte au jalon 2.
- Le tueur ultime ne dépend ni du résultat de spawn du watchdog ni de ses appels OS.

**Step 3: Vérifier les deux défaillances indépendantes**

Run les tests accélérés plusieurs fois et vérifier qu’aucun test n’attend les durées de production.

### Task 6: Encadrer les nettoyages actuels par l’échéance absolue

**Files:**
- Create: `src-tauri/src/app_exit/cleanup.rs`
- Create: `src-tauri/src/app_exit/blocking.rs`
- Create: `src-tauri/src/app_exit/cleanup_tests.rs`
- Modify: `src-tauri/src/app_exit.rs`

**Step 1: Écrire les tests rouges**

Tester :

- un arrêt synchrone réellement bloqué n’empêche pas le watchdog ni le tueur ultime ;
- tous les nettoyages existants sont lancés derrière la même échéance gracieuse ;
- une expiration abandonne l’attente une seule fois, sans deuxième enveloppe ;
- Ollama reste la dernière phase du chemin gracieux ;
- une panique de nettoyage est capturée et ne réinitialise pas la fermeture.

**Step 2: Déplacer sans changer la responsabilité des services**

- Déplacer le contenu actuel de `cleanup_services` dans `cleanup.rs`.
- Conserver les annulations et arrêts existants.
- Placer chaque appel synchrone atteignable dans `spawn_blocking` via un helper commun.
- Utiliser `timeout_at(timeline.graceful_deadline())` autour du chemin complet.
- À l’expiration, ne pas attendre de nouveau les threads bloquants ; le filet 10/13/15 reste l’autorité.

**Step 3: Vérifier les tests voisins**

Run les tests gateway, terminal, Ollama lifecycle, SearXNG, extensions et app exit concernés.

### Task 7: Brancher le coordinateur dans Tauri sans modifier CEF

**Files:**
- Modify: `src-tauri/src/app_exit.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/app_lifecycle.rs`
- Modify: `src-tauri/src/startup.rs`
- Modify: `src-tauri/src/startup_tests.rs`

**Step 1: Écrire/adapter les tests rouges d’ordre**

Tester :

- première demande : fermeture admission, armement ultime, lancement watchdog, masquage, nettoyage ;
- demandes suivantes pendant `Closing` : interceptées sans nouveau nettoyage ;
- `ReadyToExit` : demande Tauri laissée passer ;
- ordre `services -> sortie boucle Tauri -> cef::shutdown -> inventaire post-boucle` ;
- CEF factice bloqué : le callback de sortie ultime est quand même observé ;
- le chemin CEF normal garde exactement la préparation, le sandbox et l’arrêt actuels.

**Step 2: Intégrer**

- Initialiser le coordinateur au tout début de `run_inner`, avant `Builder`.
- Gérer une erreur d’initialisation par un arrêt de démarrage non interactif et générique.
- À la première `ExitRequested`, fermer admission/annuler, armer l’ultime, tenter le watchdog, puis masquer les fenêtres et lancer le nettoyage.
- Le nettoyage réussi marque `ReadyToExit` et appelle `AppHandle::exit` immédiatement ; le watchdog le fait au plus tard à 10 secondes.
- Après retour de la boucle et après l’arrêt CEF, appeler la passe post-boucle de l’inventaire déjà publié, bornée par la même chronologie.
- Ne toucher à aucun code de lancement, sandbox, admission ou confinement CEF.

**Step 3: Vérifier l’ordre**

Run `startup_tests`, `app_lifecycle` et `app_exit` avec et sans `windows-tests` lorsque compilable localement.

### Task 8: Appliquer la décision produit de fermeture par OS

**Files:**
- Modify: `src-tauri/src/app_events.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/models/gateway_config.rs`
- Modify: `src/types/channels.ts`
- Modify: `src/components/settings/test-utils/settings-tab-test-data.ts`
- Add or modify the closest existing config/frontend tests found by `rg`.

**Step 1: Écrire les tests rouges de politique**

Tester par fonction pure :

- croix Windows/Linux => `Quit` ;
- croix rouge macOS => `Hide` ;
- menu Quitter sur les trois OS => `Quit` ;
- un ancien JSON contenant `run_when_window_closed` se désérialise, mais une sérialisation moderne ne recrée pas le champ.

**Step 2: Implémenter la politique unique**

- Supprimer la lecture du gateway dans la décision de croix Windows/Linux.
- Conserver le comportement natif macOS : masquer la fenêtre principale sans fermer l’app.
- Toute vraie sortie ferme aussi le gateway via le nettoyage existant.
- Retirer `run_when_window_closed` du modèle Rust, du type TypeScript et des fixtures.
- Ne pas ajouter de migration destructive : Serde ignore le champ historique inconnu.

**Step 3: Vérifier frontend et backend**

Run les tests config ciblés, `npx tsc --noEmit` et les tests Settings concernés.

### Task 9: Fermer précisément le périmètre J1 et documenter les preuves

**Files:**
- Modify: `docs/superpowers/specs/2026-08-09-shutdown-reference-branch-inventory.md`
- Modify only if implementation reveals a factual mismatch: `docs/superpowers/specs/2026-08-09-shutdown-milestone-1-core-design.md`

**Step 1: Mettre à jour l’inventaire**

- Fermer uniquement les sous-parties J1 des lignes 1, 3, 6, 7, 8, 10, 19 et 21.
- Laisser explicitement ouvertes les sous-parties J1B, J2 et J3.
- Pour chaque fermeture, ajouter le commit de remplacement et les tests rouges/verts correspondants.
- Mentionner séparément la stabilisation du test de la ligne 14 sans fermer cette ligne.

**Step 2: Contrôler les documents**

- Vérifier les liens locaux.
- Vérifier que les 22 lignes sont toujours présentes exactement une fois.
- Vérifier que Linux CEF reste hors périmètre et que Windows/macOS CEF ne sont pas désactivés.

### Task 10: Revue globale et validation avant commit final

**Step 1: Contrôles structurels**

```powershell
git diff --check
node scripts/check-rust-format.mjs
cargo fmt --manifest-path src-tauri/Cargo.toml --check
```

- Rechercher tout fichier de production modifié dépassant 230 lignes.
- Rechercher les durées 8/10/13/15 dupliquées hors `policy.rs`.
- Rechercher tout usage restant de `run_when_window_closed` hors test de compatibilité historique.
- Rechercher tout nouveau spawn de processus ou nouveau code CEF : il ne doit y en avoir aucun.

**Step 2: Validations Rust**

```powershell
$env:CARGO_TARGET_DIR='C:\Users\huynh\projects\Beaver\src-tauri\target'
cargo test --manifest-path src-tauri/Cargo.toml --lib --features windows-tests -- --test-threads=1
cargo test --manifest-path src-tauri/Cargo.toml --lib --features windows-tests
cargo check --manifest-path src-tauri/Cargo.toml --features windows-tests
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --features windows-tests -- -D warnings
```

Tout échec est investigué ; aucun résultat n’est déclaré vert par simple relance.

**Step 3: Validations frontend**

```powershell
npm test
npx tsc --noEmit
npm run lint
```

**Step 4: Maintenance Graphify**

Comme le code et la documentation indexée ont changé :

```powershell
graphify update .
```

Puis exécuter la mise à jour documentaire Graphify prévue par les règles du projet et vérifier qu’aucun lien n’a été cassé.

**Step 5: Review cumulative du diff**

Comparer tout le diff à `main` sous cinq angles :

1. aucune fenêtre invisible au-delà du plafond ;
2. aucune admission après `Closing` et aucune fausse preuve de fin ;
3. mêmes nettoyages et même ordre CEF ;
4. comportement Windows/macOS/Linux conforme au contrat ;
5. aucun périmètre J1B/J2/J3 implémenté ou déclaré terminé par erreur.

**Step 6: Commit et Git note**

Créer des commits cohérents par lot, puis rattacher au dernier commit une note détaillée contenant : problème, objectif, architecture, décisions rejetées, lignes d’inventaire fermées ou laissées ouvertes, tests rouges/verts, validations multi-OS disponibles et limites restantes du jalon 1B.
