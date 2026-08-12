# Jalon 2 — supervision des services et processus

> Exécute ce plan par lots courts. Pour chaque comportement, écris le test, observe son échec attendu, applique le correctif minimal, puis relance le test et ses voisins avant de committer.

**Objectif :** Fais passer tous les travaux longs et tous les processus possédés par Beaver par l'unique superviseur de fermeture. Après une vraie fermeture, aucun service Beaver ne doit rester exécutable et chaque ressource terminée doit disparaître dans la fenêtre native de cinq secondes.

**Autorité :** `AppExitCoordinator` et son `AdmissionRegistry` restent l'unique barrière globale. N'introduis ni `TaskRegistry`, ni `TaskControl`, ni second jeton global. Expose seulement un handle d'admission clonable qui pointe vers ce registre, puis injecte-le dans les services. Chaque service possède uniquement son registre local borné, ses handles et ses compteurs locaux.

**Raison :** le jalon 1 a déjà fermé la porte globale. Le jalon 2 doit faire passer les producteurs existants par cette porte sans recréer une autorité parallèle.

## Préconditions vérifiées

- Branche : `codex/shutdown-milestone-2` à partir de `d0c7d6b`.
- Frontend : `414` fichiers et `1889` tests verts.
- Rust séquentiel : `3082` tests verts, `4` ignorés.
- Rust parallèle : deux exécutions ont échoué dans des tests différents avec `private-store-unavailable`, toujours pendant une écriture de session Windows ; les mêmes tests réussissent seuls.
- Les deux tests imposés par la spécification réussissent seuls : correction `64/65` et lecture directe d'un résultat persistant.

## Règles d'exécution

1. Garde les fichiers de production sous 230 lignes et une responsabilité par fichier.
2. Ne tiens aucun verrou de registre pendant un `await` ou pendant l'attente d'un handle.
3. Dérive tous les délais de l'échéance absolue du jalon 1 ; ne crée aucun budget relatif concurrent.
4. Libère une admission sur succès, erreur, annulation, panique et abandon.
5. Retourne des codes publics stables et traduis les messages visibles dans les sept langues.
6. Ne change pas la transaction d'installation ou de mise à jour d'Ollama ; adopte uniquement la possession de son processus.
7. Ne réimplémente pas la supervision CEF du jalon 1B ; classe seulement les descendants WebView Tauri indirects.
8. Après chaque lot, mets à jour Graphify et ajoute une git note avec la preuve rouge puis verte.

## Lot 0 — rendre la suite parallèle hermétique

### Tâche 0.1 — prouver la course du stockage privé Windows

**Fichiers :**

- Modifier : `src-tauri/src/services/private_store_tests.rs`
- Modifier si le diagnostic l'exige : `src-tauri/src/services/private_store/private_store_windows.rs`
- Modifier si le diagnostic l'exige : `src-tauri/src/services/private_store/windows_acl.rs`

1. Ajoute un test Windows qui lance plusieurs `atomic_write` vers des fichiers distincts sous un même parent privé.
2. Synchronise le départ des écritures pour exercer simultanément `create_private_dirs` et `secure_acl`.
3. Vérifie les octets de chaque fichier et l'absence de `private-store-unavailable`.
4. Exécute ce test avec assez de répétitions pour observer l'échec avant toute correction.
5. Conserve les erreurs visibles génériques ; si une trace de diagnostic est nécessaire, limite-la à `cfg(test)` et retire-la après identification de l'étape fautive.

**Mutation protégée :** deux écritures sur le même parent ne doivent jamais se faire échouer mutuellement pendant l'application et la vérification de l'ACL.

### Tâche 0.2 — corriger l'autorité ACL partagée sans sérialiser la suite

**Fichiers :**

- Modifier : `src-tauri/src/services/private_store.rs`
- Créer au besoin : `src-tauri/src/services/private_store/path_lock.rs`
- Modifier : `src-tauri/src/services/private_store_tests.rs`

1. Protège uniquement la mutation ACL d'un même chemin par un verrou interne borné ou par une initialisation idempotente qui ne réécrit pas l'ACL déjà conforme.
2. Ne crée pas de collection non bornée : si des verrous par chemin sont requis, borne-la et évince uniquement les entrées inactives.
3. Ne garde pas ce verrou pendant l'écriture, `sync_all` ou le remplacement atomique du fichier.
4. Relance le test de concurrence jusqu'à stabilité.

### Tâche 0.3 — démontrer séparément les deux interférences imposées

**Fichiers :**

- Modifier seulement si nécessaire : `src-tauri/src/services/agent_local/subagent_correction_capacity_tests.rs`
- Modifier seulement si nécessaire : `src-tauri/src/services/agent_local/tool_result_truncate.rs`
- Créer au besoin : `src-tauri/src/services/agent_local/test_storage.rs`

1. Ajoute à chaque test une barrière locale qui rend sa ressource et son nettoyage explicites sans verrou global de suite.
2. Pour le test `64/65`, prouve que les 64 empreintes appartiennent à sa propre exécution et que son nettoyage n'agit que sur ses deux UUID.
3. Pour le résultat persistant, prouve que la lecture vise le répertoire UUID créé par ce test et que le nettoyage ne retire que ce répertoire.
4. Si la correction de l'ACL suffit, n'ajoute aucune sérialisation artificielle à ces tests.
5. Exécute les deux tests seuls, le sous-ensemble agent, puis `cargo test --lib --features windows-tests` avec le parallélisme par défaut et à 32 threads.

**Commit attendu :** `test(storage): make private-store tests parallel-safe`

## Lot 1 — exposer l'admission globale et les diagnostics locaux

### Tâche 1.1 — publier un handle étroit vers l'unique registre

**Fichiers :**

- Modifier : `src-tauri/src/app_exit.rs`
- Modifier : `src-tauri/src/app_exit/registry.rs`
- Modifier : `src-tauri/src/app_exit/registry_admission.rs`
- Modifier : `src-tauri/src/app_exit/registry_tests.rs`
- Créer : `src-tauri/src/app_exit/work_supervisor.rs`

1. Écris les tests de refus après `Closing`, de saturation à 128 et de réutilisation bien au-delà de 128 admissions cumulées.
2. Expose `AppWorkSupervisor`, clonable mais sans méthode de fermeture publique.
3. Expose `AppWorkAdmission`, non clonable, avec son jeton enfant et une libération RAII.
4. Fais retourner `AppWorkAdmissionError::{Closing, Capacity}` et les codes `app-shutting-down` / `app-work-capacity-reached`.
5. Fournis le handle depuis `AppExitCoordinator::work_supervisor()` ; ne crée aucun singleton de repli.

### Tâche 1.2 — centraliser un registre local borné réutilisable

**Fichiers :**

- Créer : `src-tauri/src/services/work_registry.rs`
- Créer : `src-tauri/src/services/work_registry_tests.rs`
- Modifier : `src-tauri/src/services/mod.rs`

1. Écris les tests des états `Open`, `Closing`, `Closed`, des générations et de la libération conditionnelle.
2. Stocke un tableau fixe de slots, jamais un `HashMap` alimenté par une entrée externe.
3. Expose les compteurs fixes `active`, `high_water`, `saturation_refusals`, `closing_refusals`.
4. N'enregistre aucun identifiant, contenu, chemin ou donnée utilisateur dans les diagnostics.
5. Fais de `stop_and_wait(deadline)` une opération idempotente : ferme l'admission locale, annule, extrait les handles, relâche le verrou, puis attend le budget restant.
6. Vérifie succès, erreur, annulation, panique et abandon avec des tests qui auraient échoué si un slot restait occupé.

**Commit attendu :** `feat(shutdown): expose tracked service admission`

## Lot 2 — travaux sans processus et dettes fonctionnelles

### Tâche 2.1 — adopter flux agents, sous-agents et commandes shell

**Fichiers principaux :**

- Modifier : `src-tauri/src/services/agent_local/agent_stream*.rs`
- Modifier : `src-tauri/src/services/agent_local/subagent_task*.rs`
- Modifier : `src-tauri/src/services/agent_local/subagent_explorer_process.rs`
- Modifier : `src-tauri/src/services/agent_local/tool_bash_process_run.rs`
- Modifier : `src-tauri/src/commands/agent_chat.rs`
- Ajouter des tests ciblés dans les modules voisins existants.

1. Refuse chaque nouveau flux ou sous-agent après `Closing`.
2. Enveloppe chaque tâche longue dans une admission globale puis locale.
3. Propage le jeton d'annulation jusqu'aux boucles de lecture et aux enfants.
4. Vérifie qu'une fermeture au démarrage, en exécution et pendant l'arrêt libère chaque slot.

### Tâche 2.2 — stabiliser les codes d'admission des flux

**Fichiers :**

- Modifier : `src-tauri/src/services/agent_local/types_stream.rs`
- Modifier : `src/types/agent-stream.ts`
- Modifier : `src/hooks/agent-chat-stream-callbacks.ts`
- Modifier : `src/hooks/__tests__/agent-chat-stream-callbacks-error.test.ts`
- Modifier les sept catalogues sous `src/i18n/`.

1. Écris d'abord un test frontend pour chaque code fermé : fermeture globale et saturation locale.
2. Fais transporter le code structuré par Rust sans message technique.
3. Garde un mapping frontend exhaustif et fermé.
4. Ajoute les traductions françaises, anglaises, espagnoles, allemandes, italiennes, chinoises et japonaises.

### Tâche 2.3 — corriger les réveils ponctuels

**Fichiers :**

- Modifier : `src-tauri/src/services/scheduler/fire.rs`
- Créer au besoin : `src-tauri/src/services/scheduler/fire_once.rs`
- Modifier : `src-tauri/src/services/scheduler/due.rs`
- Modifier : `src-tauri/src/services/scheduler/log.rs`
- Modifier : `src-tauri/src/services/scheduler/*tests.rs`

1. Écris les tests `Inactive` silencieux, annulation après revendication journalisée et erreur `claim_once` journalisée.
2. Retourne un résultat typé pour un réveil ponctuel ; n'infère pas son état depuis un texte.
3. Appelle `scheduler.notify_config_changed()` après chaque mutation.
4. Borne et assainis chaque trace.

**Commit attendu :** `feat(shutdown): supervise agent work and wakeups`

## Lot 3 — téléchargements et handoff de mise à jour

### Tâche 3.1 — adopter les téléchargements de modèles et Beaver

**Fichiers :**

- Modifier : `src-tauri/src/services/model_downloads.rs`
- Modifier : `src-tauri/src/services/model_downloads_store.rs`
- Modifier : `src-tauri/src/commands/app_update*.rs`
- Modifier : `src-tauri/src/commands/model_downloads.rs`
- Ajouter les tests ciblés voisins.

1. Vérifie l'annulation à chaque étape réseau, écriture, validation et renommage.
2. Supprime uniquement le fichier partiel possédé par l'admission annulée.
3. Refuse un nouveau téléchargement après `Closing` avec le code public du domaine.
4. Prouve la libération sur erreur réseau, annulation et validation refusée.

### Tâche 3.2 — préserver uniquement le helper transféré

**Fichiers :**

- Modifier : `src-tauri/src/commands/app_update_helper.rs`
- Modifier : `src-tauri/src/commands/app_update_install.rs`
- Modifier : `src-tauri/src/updater_worker/*.rs`
- Modifier : `src-tauri/src/services/update_handoff.rs` si présent, sinon créer ce domaine sous `services/update_handoff/`.

1. Écris les tests helper non transféré arrêté / helper validé et transféré préservé.
2. Rends le transfert atomique et irréversible seulement après validation complète.
3. N'autorise aucun autre processus à survivre.

**Commit attendu :** `feat(shutdown): cancel downloads and secure update handoff`

## Lot 4 — gateway et extensions

### Tâche 4.1 — fermer le gateway sur les trois canaux

**Fichiers :**

- Modifier : `src-tauri/src/services/gateway/service.rs`
- Modifier : `src-tauri/src/services/gateway/service_runtime.rs`
- Modifier : `src-tauri/src/services/gateway/supervisor.rs`
- Modifier : `src-tauri/src/services/gateway/channels/{telegram,discord,slack}.rs`
- Modifier : `src-tauri/src/services/gateway/service_tests.rs`

1. Borne la file à 256 messages et les traitements à 64.
2. Ferme la file avant d'annuler les lecteurs.
3. Attends les handles hors du verrou du service.
4. Prouve l'arrêt et l'absence de redémarrage sur Telegram, Discord et Slack.
5. Prouve que la croix rouge macOS ne ferme pas le gateway.

### Tâche 4.2 — arrêter l'hôte d'extensions et ses opérations

**Fichiers :**

- Modifier : `src-tauri/src/services/extensions/runtime.rs`
- Modifier : `src-tauri/src/services/extensions/runtime_restart.rs`
- Modifier : `src-tauri/src/services/extensions/host_process.rs`
- Modifier : `src-tauri/src/services/extensions/host_reader.rs`
- Modifier : `src-tauri/src/services/extensions/process_runner.rs`
- Modifier : `src-tauri/src/services/extensions/installer.rs`
- Ajouter les tests ciblés voisins.

1. Ferme l'admission avant de tuer l'hôte.
2. Annule lecteurs, installation, démarrage, redémarrage et exécution.
3. Prouve qu'aucun watchdog ne relance l'hôte après l'arrêt.
4. Fais échouer fermement tout démarrage dont l'affectation native échoue.

**Commit attendu :** `feat(shutdown): stop gateway and extension services`

## Lot 5 — processus de service natifs

### Tâche 5.1 — adopter SearXNG

**Fichiers :**

- Modifier : `src-tauri/src/services/searxng/{lifecycle,runtime,process}.rs`
- Ajouter : `src-tauri/src/services/searxng/lifecycle_tests.rs`

1. Supervise installation, serveur et lecteurs.
2. Prouve un vrai lancement puis la disparition du processus dans le profil natif.

### Tâche 5.2 — adopter MCP stdio et installations

**Fichiers :**

- Modifier : `src-tauri/src/services/mcp_bridge/{process_manager,process_spawn,stdio}.rs`
- Modifier : `src-tauri/src/services/mcp_bridge/stdio_integration_tests.rs`

1. Affecte le processus au confinement natif avant de rendre le handle.
2. Ferme stdin, annule les lecteurs, termine le groupe, puis moissonne.
3. Prouve un vrai processus stdio lancé et récolté.

### Tâche 5.3 — adopter Forecast

**Fichiers :**

- Modifier : `src-tauri/src/services/forecast/sidecar*.rs`
- Modifier : `src-tauri/src/services/forecast/model_manager/*.rs`
- Modifier : `src-tauri/src/services/forecast/evaluation/*.rs`
- Modifier : `src-tauri/src/services/forecast/sidecar_runtime_tests.rs`

1. Supervise sidecar, installation, évaluations et commandes longues.
2. Prouve arrêt pendant démarrage, exécution et installation.
3. Prouve le moissonnage du vrai sidecar du profil de test.

### Tâche 5.4 — adopter PTY et shell

**Fichiers :**

- Modifier : `src-tauri/src/services/terminal/mod.rs`
- Modifier : `src-tauri/src/services/terminal/pty_session.rs`
- Modifier : `src-tauri/src/services/terminal/tests.rs`
- Modifier : `src-tauri/src/services/background_command.rs`

1. Garde la limite existante de 16 sessions.
2. Annule le lecteur, ferme le PTY, termine le groupe et attend les threads hors verrou.
3. Prouve un vrai shell lancé puis absent après fermeture.

**Commit attendu :** `feat(shutdown): supervise native service processes`

## Lot 6 — confinement natif unique

### Tâche 6.1 — unifier la possession des processus

**Fichiers :**

- Modifier : `src-tauri/src/services/process_tree.rs`
- Créer : `src-tauri/src/services/owned_process.rs`
- Créer : `src-tauri/src/services/owned_process_tests.rs`
- Modifier : `src-tauri/src/services/ollama_lifecycle.rs`

1. Fais de `OwnedProcess` l'unique chemin de lancement pour un processus possédé.
2. Sur Windows, affecte immédiatement au Job Object global ; si l'affectation échoue, tue, balaie les descendants et moissonne avant de retourner une erreur.
3. Sur Linux, crée un groupe dédié et arme le signal de mort parent avant `exec` ; signale le groupe avant l'individu et ignore/moissonne les zombies.
4. Sur macOS, crée un groupe dédié et enregistre PID, groupe et heure de départ dans un tableau fixe ; revérifie l'identité avec `proc_pidinfo` avant tout signal.
5. Pour Ollama, remplace seulement la possession du processus ; ne change ni téléchargement, ni extraction, ni validation, ni transaction d'installation.

### Tâche 6.2 — classer les descendants WebView Tauri

**Fichiers :**

- Modifier : `src-tauri/src/services/browser/process_role.rs`
- Modifier : `src-tauri/src/services/browser/process_role_tests.rs`
- Ajouter les tests natifs E2E dans le profil navigateur existant.

1. Identifie uniquement les descendants dédiés à Beaver.
2. Ne signale jamais un service système partagé.
3. Active une vraie WebView sur Windows, Linux et macOS, puis vérifie l'absence de descendants après fermeture.

**Commit attendu :** `feat(shutdown): confine every owned child process`

## Lot 7 — inventaire, nettoyage final et preuves

### Tâche 7.1 — fermer l'inventaire exhaustif

**Fichiers :**

- Créer : `docs/superpowers/specs/2026-08-12-shutdown-milestone-2-spawn-inventory.md`
- Modifier : `docs/superpowers/specs/2026-08-09-shutdown-reference-branch-inventory.md`

1. Recherche tous les `Command::new`, `tokio::process::Command`, `portable-pty`, `tokio::spawn`, `tauri::async_runtime::spawn` et `std::thread::spawn`.
2. Pour chaque résultat de production, inscris propriétaire, admission, annulation, attente, confinement et test ; marque explicitement les opérations courtes ou externes exemptées avec leur raison.
3. Ferme chaque sous-ligne J2 de l'inventaire historique avec le commit et la preuve.
4. Vérifie que CEF renvoie au jalon 1B au lieu d'être dupliqué.

### Tâche 7.2 — simplifier le nettoyage central

**Fichiers :**

- Modifier : `src-tauri/src/app_exit/cleanup.rs`
- Modifier : `src-tauri/src/app_exit/cleanup_tests.rs`

1. Fais fermer les services par leurs `stop_and_wait` idempotents.
2. Garde une seule échéance absolue.
3. Retire les arrêts directs devenus doubles.
4. Vérifie que le registre global est vide et que le dépassement d'un service est tracé sans tuer prématurément le nettoyage.

### Tâche 7.3 — validation complète et git notes

1. Exécute `npm run lint`, `npx tsc --noEmit`, `npm test`.
2. Exécute `cargo fmt --check`, `cargo check`, `cargo clippy --all-targets -- -D warnings`, puis les suites Rust parallèles et séquentielles.
3. Exécute les profils natifs Windows localement ; exige les profils macOS et Linux dans la CI.
4. Exécute `graphify update .` après le code et `/graphify --update` après les documents indexés.
5. Ajoute à chaque commit J2 une git note avec : hash de référence, comportement, raison, test rouge, test vert, tests voisins et commit remplaçant.
6. Pousse la branche seulement quand toutes les preuves locales accessibles sont vertes ; ouvre ensuite la PR et attends tous les contrôles.

## Critères de fin

- Aucun producteur long ou processus possédé ne contourne l'admission globale.
- Chaque service refuse les nouveaux départs après `Closing` et expose des compteurs bornés sans données utilisateur.
- Une vraie fermeture ne laisse aucun service Beaver exécutable ni processus enfant possédé.
- Aucun processus externe ou service système partagé n'est tué.
- Le helper de mise à jour ne survit qu'après transfert validé.
- Les deux tests d'interférence imposés passent seuls et dans la suite parallèle sans sérialisation globale.
- Les réveils ponctuels et les erreurs de flux respectent leurs contrats typés et leurs sept traductions.
- L'inventaire de spawn est complet, les lignes J2 historiques sont fermées et les git notes correspondent au code exécuté.
- Les tests frontend, Rust, natifs et CI sont verts avec leurs sorties lues.
