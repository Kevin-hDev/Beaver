# Shutdown Milestone 1B CEF Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Superviser tous les helpers CEF natifs sur Windows et macOS avant leur entrée dans CEF, puis garantir qu'ils ne peuvent plus exécuter de travail après la sortie forcée de Beaver, sans désactiver le sandbox ni adopter un processus externe.

**Architecture:** Ajouter une autorité CEF séparée du registre général du jalon 1 : porte atomique, table parent privée de 64 slots, boîtes de publication non fiables isolées par réservation et traqueur natif. Windows confine chaque helper admis dans un Job Object propre ; macOS combine groupe de processus, reaper parent et moniteur d'auto-terminaison du helper. Le watchdog existant ne reçoit qu'une vue CEF fixe et non bloquante pour la phase 13–15 secondes. Toute impossibilité détectée avant lancement rend le navigateur indisponible localement ; toute ambiguïté après lancement ferme Beaver de manière coordonnée.

**Tech Stack:** Rust, Tauri 2, CEF 150, atomiques standard, `rand::rngs::OsRng`, `zeroize`, `windows-sys`, `libc`, GitHub Actions Windows/macOS natives.

---

## Contraintes d'exécution

- Commencer chaque comportement par un test rouge observé, puis implémenter le minimum.
- Aucun fichier de production ne dépasse 230 lignes ; les tests restent regroupés par unité testée.
- Les tables sont fixes : 64 slots CEF, sans `Vec`, `Map` ou file alimentée par un helper.
- Les nonces font 256 bits, viennent de `OsRng`, ne sont jamais loggés et sont zéroïsés après usage.
- Le parent conserve exclusivement l'état d'autorité et les handles natifs ; toute donnée écrite par un helper est non fiable.
- Une identité n'est jamais fondée sur le PID, le nom ou le chemin seuls.
- Aucun appel CEF n'est permis avant admission parent explicite.
- Le sandbox reste actif ; aucune ACL, capability ou exception antivirus large n'est autorisée.
- Les erreurs visibles restent génériques ; les logs locaux n'acceptent que cinq catégories bornées et sans détail système.
- Linux reste sans `native_browser` : ce jalon ne commence pas l'intégration CEF Linux.
- Après chaque tâche : tests ciblés, `cargo fmt --check`, `git diff --check`, puis commit local cohérent.

### Task 1: Figer le protocole borné, les rôles et les diagnostics

**Files:**
- Create: `src-tauri/src/services/browser/cef_supervision.rs`
- Create: `src-tauri/src/services/browser/cef_supervision/constants.rs`
- Create: `src-tauri/src/services/browser/cef_supervision/role_marker.rs`
- Create: `src-tauri/src/services/browser/cef_supervision/diagnostics.rs`
- Create: `src-tauri/src/services/browser/cef_supervision/protocol_tests.rs`
- Modify: `src-tauri/src/services/browser/mod.rs`
- Modify: `src-tauri/src/services/browser/process_role.rs`

**Step 1: Écrire les tests rouges**

Tester :

- seuls les rôles CEF autorisés sont encodés ; le rôle shell est refusé ;
- marqueur borné, versionné et strict, sans valeur dupliquée ni argument inconnu ;
- nonce exactement 32 octets, jamais formaté par `Debug` ou `Display`, zéroïsé à la destruction ;
- cinq catégories seulement : `object`, `permission`, `admission`, `reaper`, `sandbox` ;
- toute chaîne issue de l'OS est rejetée par le contrat de diagnostic.

Run:

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib --features windows-tests services::browser::cef_supervision::protocol_tests -- --nocapture
```

Expected: échec de compilation avant création des modules.

**Step 2: Implémenter le noyau sans I/O natif**

- `CefProcessRole` est un enum fermé et sérialisé par un identifiant numérique borné.
- `CefLaunchMarker` contient version, slot, génération et nonce ; son parseur rejette toute entrée trop longue ou ambiguë.
- Le helper retire le marqueur privé de sa ligne de commande immédiatement après validation puis zéroïse sa copie ; le parent zéroïse sa copie à la libération de la réservation.
- `CefUnavailableCategory` n'expose qu'un code local constant et n'implémente aucun rendu de détail système.
- Les constantes 64 slots, 50 ms, 10 ms et 5 s vivent dans `constants.rs` ; les échéances 13/15 s continuent de venir de `app_exit::policy`.

**Step 3: Vérifier et commit**

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib --features windows-tests services::browser::cef_supervision::protocol_tests
cargo fmt --manifest-path src-tauri/Cargo.toml --check
git diff --check
git add src-tauri/src/services/browser
git commit -m "feat(browser): define bounded CEF supervision protocol"
```

### Task 2: Construire la porte atomique et la table parent privée

**Files:**
- Create: `src-tauri/src/services/browser/cef_supervision/gate.rs`
- Create: `src-tauri/src/services/browser/cef_supervision/slots.rs`
- Create: `src-tauri/src/services/browser/cef_supervision/mailbox.rs`
- Create: `src-tauri/src/services/browser/cef_supervision/table_tests.rs`
- Modify: `src-tauri/src/services/browser/cef_supervision.rs`

**Step 1: Écrire les tests rouges de course**

Tester :

- 64 réservations réussissent, la 65e échoue fermée ;
- chaque réutilisation change la génération et le nonce ;
- `Closing` ferme atomiquement la porte et invalide toute réservation non admise ;
- un permis acquis avant `Closing` doit finir en moins de 50 ms, sinon sa génération est invalidée ;
- une publication tardive, réécrite ou provenant d'un autre slot ne peut pas modifier l'autorité ;
- `Drop`, panique et refus libèrent exactement la bonne génération ;
- saturation ou poison simulé ne déclenche jamais de lancement CEF.

**Step 2: Implémenter les états monotones**

- `CefLaunchGate` combine bit fermé et nombre de permis transitoires.
- `CefAuthorityTable` possède un tableau fixe de 64 `CefAuthoritySlot` parent-only.
- La boîte enfant-écrivable ne contient que PID déclaré, rôle, génération, nonce et indicateur de publication ; aucun handle, état admis ou donnée d'un autre slot.
- Le parent copie puis valide une publication une seule fois, scelle sa vue privée et ignore toute réécriture ultérieure.
- `CefReservation` et `CefAdmission` sont non `Clone` et libèrent seulement leur génération dans `Drop`.

**Step 3: Vérifier les courses plusieurs fois**

```powershell
1..10 | ForEach-Object { cargo test --manifest-path src-tauri/Cargo.toml --lib --features windows-tests services::browser::cef_supervision::table_tests -- --test-threads=8 }
cargo test --manifest-path src-tauri/Cargo.toml --lib --features windows-tests services::browser::cef_supervision::table_tests -- --test-threads=1
```

Commit: `feat(browser): add private CEF admission authority`.

### Task 3: Préparer les objets de publication interprocessus de manière sûre

**Files:**
- Create: `src-tauri/src/services/browser/cef_supervision/ipc.rs`
- Create: `src-tauri/src/services/browser/cef_supervision/ipc_tests.rs`
- Create: `src-tauri/src/services/browser/cef_supervision/windows/security.rs`
- Create: `src-tauri/src/services/browser/cef_supervision/windows/security_tests.rs`
- Create: `src-tauri/src/services/browser/cef_supervision/macos/objects.rs`
- Modify: `src-tauri/src/services/browser/cef_supervision.rs`
- Modify: `src-tauri/src/services/browser/mod.rs`

**Step 1: Écrire les tests rouges**

Tester le layout fixe et les droits minimaux :

- une boîte, une page de contrôle et deux événements uniques par réservation ;
- handles non héritables ; parent seul modifiable sur contrôle/événements ; helper limité à lecture contrôle, écriture boîte et synchronisation ;
- Windows : DACL avec SID activé et SID de restriction réels, SACL MIC au niveau requis par le token sandboxé ;
- macOS : tous les descripteurs nécessaires sont ouverts avant `sandbox.initialize` ;
- corruption, faux handle et nom inter-slot sont refusés sans accès hors réservation.

**Step 2: Implémenter derrière des traits testables**

- `PublicationObjects` possède et ferme tous ses objets dans `Drop`.
- Le nom natif dérive du nonce sans exposer celui-ci dans les logs.
- Sous Windows, construire explicitement le descripteur de sécurité avec `windows-sys`, vérifier chaque retour et libérer chaque allocation native.
- Sous macOS, ouvrir/maper les objets avant Seatbelt et ne conserver après sandbox que les descripteurs déjà ouverts.
- Tout échec avant lancement retourne une `CefUnavailableCategory` ; aucun helper n'est créé.

**Step 3: Vérifier sur la plateforme disponible**

Run les tests purs sur tous les OS, les tests ACL natifs sous Windows, puis compilation macOS via CI. Commit: `feat(browser): secure CEF publication objects`.

### Task 4: Confiner et identifier les helpers Windows

**Files:**
- Create: `src-tauri/src/services/browser/cef_supervision/windows.rs`
- Create: `src-tauri/src/services/browser/cef_supervision/windows/identity.rs`
- Create: `src-tauri/src/services/browser/cef_supervision/windows/job.rs`
- Create: `src-tauri/src/services/browser/cef_supervision/windows/tracker.rs`
- Create: `src-tauri/src/services/browser/cef_supervision/windows/tests.rs`
- Modify: `src-tauri/src/services/browser/windows_sandbox.rs`

**Step 1: Écrire les tests rouges natifs/injectés**

Tester :

- identité stable = PID + parent + heure de création + exécutable canonique + rôle/réservation ;
- PID réutilisé, parent incorrect, exécutable différent ou marqueur shell : refus ;
- `OpenProcess` demande seulement `SYNCHRONIZE | PROCESS_TERMINATE | PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_SET_QUOTA` ;
- Job Object vide, `KILL_ON_JOB_CLOSE`, sans UI limits, puis assignation avant `Admitted` ;
- échec d'assignation ferme les handles et arrête uniquement le bootstrap identifié ;
- `TerminateProcess` accepté mais handle non signalé conserve le slot `Terminating` pour revérification ;
- fermeture du dernier handle Job tue le helper même si le tracker et le watchdog général sont bloqués.

**Step 2: Implémenter l'autorité Windows**

- `WindowsProcessIdentity` est construite depuis le handle ouvert, jamais depuis la seule publication.
- `WindowsJobGuard` reste dans le slot parent privé ; aucun handle n'est dupliqué vers le helper.
- Le tracker précréé revendique les publications, valide l'identité, crée/assigne le Job puis seulement accorde l'admission.
- Toutes les fonctions natives renvoient des erreurs typées internes ; les logs ne reçoivent qu'une catégorie bornée.

**Step 3: Vérifier**

Run les tests injectés avec `windows-tests`, puis les tests natifs dans le profil CEF réel. Commit: `feat(browser): confine Windows CEF helpers`.

### Task 5: Brancher le bootstrap et les callbacks de lancement Windows

**Files:**
- Modify: `src-tauri/src/windows_entry.rs`
- Modify: `src-tauri/src/windows_entry_plan.rs`
- Modify: `src-tauri/src/services/browser/cef_app.rs`
- Create: `src-tauri/src/services/browser/cef_supervision/bootstrap.rs`
- Create: `src-tauri/src/services/browser/cef_supervision/bootstrap_tests.rs`
- Modify: `src-tauri/src/services/browser/cef_engine.rs`

**Step 1: Écrire les tests rouges d'ordre**

Tester :

- le parent réserve et ajoute le marqueur dans `on_before_child_process_launch` ;
- le point d'entrée distingue strictement helper CEF, helper shell et processus navigateur parent ;
- un helper ouvre ses objets, publie, attend l'admission puis revalide la génération avant `execute_process` ;
- callback sans ligne exploitable, marqueur dupliqué, réservation expirée, publication après `Closing` ou après 13 s : sortie avant CEF ;
- candidat jamais publié ne charge ni CEF, ni profil, ni VRAM et ne crée aucun descendant ;
- une ambiguïté après lancement possible appelle la vraie fermeture coordonnée.

**Step 2: Modifier l'ordre sans repli permissif**

- Insérer le marqueur uniquement dans le callback CEF dédié ; ne jamais l'ajouter au helper shell.
- Dans `RunWinMain`, analyser le rôle avant tout `execute_process` de helper.
- Le processus navigateur parent conserve son chemin actuel après validation des prérequis.
- `cef_engine::initialize` ne passe à `Ready` qu'après création du tracker et de tous les objets.
- Avant tout helper potentiel, une erreur marque `BrowserCapability::Unavailable`; après ce point, elle déclenche `app_exit::request`.

**Step 3: Vérifier**

Run tests d'ordre, test simultané CEF/shell, puis un smoke natif avec sandbox actif. Commit: `feat(browser): admit Windows CEF helpers before execution`.

### Task 6: Superviser les helpers macOS sans dépendre de Seatbelt pour les tuer

**Files:**
- Create: `src-tauri/src/services/browser/cef_supervision/macos.rs`
- Create: `src-tauri/src/services/browser/cef_supervision/macos/identity.rs`
- Create: `src-tauri/src/services/browser/cef_supervision/macos/reaper.rs`
- Create: `src-tauri/src/services/browser/cef_supervision/macos/monitor.rs`
- Create: `src-tauri/src/services/browser/cef_supervision/macos/tests.rs`
- Modify: `src-tauri/src/bin/cl-go-dash-helper.rs`
- Modify: `src-tauri/src/services/browser/cef_engine.rs`
- Modify: `src-tauri/src/services/browser/process_role.rs`

**Step 1: Écrire les tests rouges d'ordre et d'identité**

Tester :

- boîte, contrôle, événements et groupe créés avant `sandbox.initialize` ; publication et moniteur démarrés après sandbox ;
- validation PID + parent + heure de départ + exécutable + PGID ; PGID réutilisé refusé ;
- reaper parent précréé avant CEF, rescannant les générations jusqu'à 15 s et revalidant avant chaque signal ;
- moniteur helper s'auto-termine si le parent disparaît ou à l'échéance 14 s ;
- le moniteur tue seulement son propre helper, sans supposer `kill(-pgid)` autorisé par Seatbelt ;
- reaper absent avant lancement => indisponible sans helper ; reaper perdu après lancement => fermeture coordonnée ;
- tracker ou watchdog général bloqué => reaper/moniteur restent efficaces.

**Step 2: Implémenter les trois autorités indépendantes**

- `MacProcessIdentity` utilise `proc_pidinfo`, `proc_pidpath` et `getpgid` dans des buffers fixes.
- Le reaper parent possède seulement des identités revalidables et ne signale jamais un PGID stale.
- Le moniteur post-sandbox surveille la page de contrôle et l'existence du parent ; il appelle `_exit` sur lui-même.
- Aucun descendant non suivi ne peut être créé avant admission ; les tests bootstrap l'imposent.

**Step 3: Vérifier sur macOS natif**

Run tests unitaires, smoke helper réel sandboxé et scénario `cef::shutdown` bloqué. Commit: `feat(browser): supervise macOS CEF helpers`.

### Task 7: Raccorder la vue d'urgence CEF au jalon 1

**Files:**
- Create: `src-tauri/src/services/browser/cef_supervision/emergency.rs`
- Create: `src-tauri/src/services/browser/cef_supervision/emergency_tests.rs`
- Modify: `src-tauri/src/app_exit/watchdog.rs`
- Modify: `src-tauri/src/app_exit.rs`
- Modify: `src-tauri/src/startup.rs`
- Modify: `src-tauri/src/startup_tests.rs`
- Modify: `src-tauri/src/services/browser/runtime_integration.rs`

**Step 1: Écrire les tests rouges du double défaut**

Tester :

- la vue CEF est distincte des 128 slots généraux ;
- `Closing` ferme la porte CEF avant masquage des fenêtres et invalide les réservations tardives ;
- entre 13 et 15 s, la passe fixe revalide et signale continuellement les 64 slots sans attente bloquante ;
- Windows ferme/revérifie les Jobs ; macOS réveille le reaper ;
- watchdog CEF bloqué, `cef::shutdown` bloqué ou tracker paniqué : le tueur ultime reste indépendant ;
- à 15 s aucun helper admis n'est runnable ; objets terminating/zombies/bootstrap refusés disparaissent dans les 5 s du test ;
- chemin normal conserve l'ordre `services -> boucle Tauri -> cef::shutdown -> constat final`.

**Step 2: Ajouter une interface étroite**

- `AppExitCoordinator` reçoit une `CefEmergencyView` fixe initialisée avant tout lancement CEF.
- Le watchdog appelle seulement `close_gate`, `force_once` et `has_runnable`; aucun mutex, allocation ou callback CEF.
- Le post-loop constate les slots restants sans créer une nouvelle échéance.
- `cef::shutdown()` n'est jamais la preuve de disparition des helpers.

**Step 3: Vérifier les échéances absolues**

Run tests accélérés avec horloge injectée, puis les tests startup/app_exit complets. Commit: `feat(exit): enforce CEF emergency shutdown`.

### Task 8: Prouver les capacités et l'échec fermé sans casser Beaver

**Files:**
- Modify: `src-tauri/src/services/browser/runtime_handle.rs`
- Modify: `src-tauri/src/services/browser/runtime_integration.rs`
- Modify: `src-tauri/src/services/browser/cef_diagnostics.rs`
- Create: `src-tauri/src/services/browser/cef_supervision/capability_tests.rs`
- Modify closest existing frontend/browser capability tests found by `rg`.

**Step 1: Écrire les tests rouges**

Pour chaque prérequis injecté (`object`, `permission`, `admission`, `reaper`, `sandbox`) :

- `BrowserCapability::Unavailable` avant lancement ;
- aucun helper créé et Beaver reste utilisable ;
- aucune bascule silencieuse vers `Unavailable` après création potentielle ;
- aucun chemin, PID, token, handle ou texte OS dans l'événement UI/log public ;
- `Ready` signifie réellement tracker actif et confinement disponible.

**Step 2: Centraliser la décision**

- Ajouter un état interne `ReadySupervised` distinct sans changer le contrat frontend public si inutile.
- La capacité publique `Ready` n'est émise qu'après preuve locale des prérequis.
- Les erreurs avant lancement utilisent le message traduit existant ; les erreurs post-lancement demandent une fermeture coordonnée.

**Step 3: Vérifier les tests voisins**

Run les tests browser/runtime, `npm test`, `npx tsc --noEmit`. Commit: `fix(browser): expose only supervised CEF readiness`.

### Task 9: Étendre les preuves CI Windows/macOS et garder Linux hors périmètre

**Files:**
- Modify: `.github/workflows/ci.yml`
- Create: `scripts/cef/cef-supervision-contracts.mjs`
- Create: `scripts/cef/cef-supervision-contracts.test.mjs`
- Modify: `package.json`
- Modify: `scripts/e2e/run.mjs` only if a bounded native test entry is required.

**Step 1: Écrire les contrats rouges**

Tester que la CI :

- prépare CEF vérifié sur Windows et macOS ;
- compile et Clippy les chemins natifs ;
- exécute le smoke `Ready supervisé`, le helper réel sandboxé et le scénario d'échec injecté ;
- vérifie Linux sans `native_browser` ni helper CEF ;
- ne désactive jamais le sandbox et ne passe aucune exception antivirus.

**Step 2: Étendre les jobs existants**

- Réutiliser `backend-windows-native` et `backend-macos-native`, sans nouveau téléchargement non vérifié.
- Ajouter des tests natifs nommés et bornés, avec nettoyage systématique de leurs helpers.
- Laisser les essais Windows renforcé et macOS Gatekeeper/quarantaine comme validation manuelle obligatoire documentée, car les runners GitHub ne les représentent pas.

**Step 3: Vérifier localement les contrats puis pousser pour preuve native**

```powershell
node --test scripts/cef/cef-supervision-contracts.test.mjs
npm run test:cef
```

Commit: `ci: prove native CEF supervision`.

### Task 10: Fermer l'inventaire J1B et exécuter la revue cumulative

**Files:**
- Modify: `docs/superpowers/specs/2026-08-09-shutdown-reference-branch-inventory.md`
- Modify only for factual implementation clarifications: `docs/superpowers/specs/2026-08-09-shutdown-milestone-1b-cef-design.md`

**Step 1: Fermer précisément les sous-lignes J1B**

- Fermer seulement les portions J1B des lignes 1, 2 et 6 de l'inventaire.
- Référencer chaque commit, test rouge/vert, smoke natif et preuve de sandbox.
- Conserver exactement les 22 entrées et laisser J2/J3/J4 ouverts.
- Consigner séparément les essais build empaqueté : Windows protections Microsoft, Windows renforcé, macOS Gatekeeper/quarantaine.

**Step 2: Contrôles structurels et sécurité**

```powershell
git diff --check
node scripts/build/check-rust-format.mjs
cargo fmt --manifest-path src-tauri/Cargo.toml --check
```

- Aucun fichier de production modifié au-dessus de 230 lignes.
- Aucun tableau dynamique alimenté par helper.
- Aucun nonce/handle/PID/chemin dans les logs publics.
- Aucun `execute_process` helper avant admission.
- Aucun `no_sandbox`, permission large ou identification PID/nom seule.
- Aucun code CEF Linux ajouté.

**Step 3: Validation complète**

```powershell
cargo test --manifest-path src-tauri/Cargo.toml --lib --features windows-tests -- --test-threads=1
cargo test --manifest-path src-tauri/Cargo.toml --lib --features windows-tests
cargo check --manifest-path src-tauri/Cargo.toml --all-targets
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
npm test
npx tsc --noEmit
npm run lint
```

Tout échec est expliqué et corrigé ; aucune relance opportuniste ne remplace l'analyse.

**Step 4: Maintenance Graphify**

```powershell
graphify update .
```

Puis exécuter la mise à jour documentaire Graphify prévue par les règles du projet.

**Step 5: Review cumulative avant PR**

Relire tout le diff depuis `main` sous huit angles indépendants : admission, identité, confinement, sandbox, arrêt 13–15 s, erreurs/diagnostics, autres OS et non-régression du navigateur normal. Rejouer explicitement tous les scénarios obligatoires de la spec, y compris les doubles défauts.

**Step 6: Git note finale**

Attacher au dernier commit une note détaillée : problème, objectif, protocole, décisions Windows/macOS, raisons du fail-closed avant lancement, raison de la fermeture coordonnée après lancement, alternatives rejetées, preuves sandbox/antivirus/Gatekeeper, tests et CI, inventaire fermé et limites restantes jusqu'au jalon 4.

**Step 7: PR et fusion**

- Pousser la branche et ouvrir une PR non brouillon seulement après les preuves natives.
- Surveiller tous les jobs jusqu'au verdict final.
- Ne fusionner que si Windows et macOS sont `Ready supervisé`; une plateforme désactivée n'est pas un résultat acceptable.
- Ne publier aucune release depuis le `main` transitoire avant la validation du jalon 4 ; la branche `maintenance/v1.1.2-pre-shutdown` reste l'autorité des correctifs urgents publiables.
