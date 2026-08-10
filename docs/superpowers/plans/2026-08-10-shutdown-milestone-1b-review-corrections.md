# Jalon 1B — plan correctif après review cumulative

## Autorité

Ce plan applique l'amendement du [jalon 1B](../specs/2026-08-09-shutdown-milestone-1b-cef-design.md). Il corrige uniquement les écarts confirmés par la review cumulative ; il ne remplace pas le protocole d'admission, les ACL minimales, le sandbox ou les Jobs déjà validés.

## Règles d'exécution

- chaque comportement commence par un test qui échoue pour la cause attendue ;
- chaque correction est vérifiée seule avant de passer à la suivante ;
- aucune erreur OS n'est classée à partir de son texte ;
- aucun retry n'existe après le premier appel à `cef::initialize` ;
- les fichiers de production restent sous 230 lignes et les valeurs de délai restent centralisées ;
- Windows et macOS sont validés nativement ; Linux reste sans `native_browser` ;
- chaque commit de sécurité ou de cycle de vie reçoit une Git note expliquant la décision et ses preuves.

## Tâche 1 — fermer CEF sans court-circuiter le nettoyage

**Fichiers principaux :**

- `src-tauri/src/app_exit.rs`
- `src-tauri/src/app_exit/policy.rs`
- `src-tauri/src/app_exit/coordinator_tests.rs`
- `src-tauri/src/services/browser/cef_runtime_policy.rs`
- `src-tauri/src/services/browser/cef_supervision/emergency.rs`
- tests voisins de la porte et du coordinateur

1. Écrire un test rouge où un permis CEF dépasse la barrière, mais où le résultat du coordinateur reste `Started`, l'ultime garde est armée et le nettoyage des autres services peut commencer.
2. Centraliser l'échéance CEF dans `ShutdownTimeline` et la transmettre à la fermeture de porte ; supprimer les `50 ms` locaux sur ce chemin.
3. Distinguer `barrière vidée` de `barrière dépassée`. Les deux poursuivent la fermeture ; seule une corruption réelle de l'autorité produit une violation fatale.
4. Vérifier les tests `app_exit`, table CEF et startup, puis répéter les courses en parallèle et en séquentiel.

## Tâche 2 — restaurer les rôles Windows et accepter la vraie ligne Chromium

**Fichiers principaux :**

- `src-tauri/src/windows_entry_plan.rs`
- `src-tauri/src/windows_entry.rs`
- `src-tauri/src/windows_entry_tests.rs`
- `src-tauri/src/services/agent_local/shell_sandbox/*`
- tests du bootstrap empaqueté

1. Écrire des tests rouges pour les quatre résultats exclusifs : parent, CEF réservé, shell isolé et invalide. Couvrir la combinaison shell + CEF et plus de 64 arguments Chromium opaques.
2. Remplacer le filtre global par un parseur en flux : seuls les switches privés Beaver sont décodés et bornés. Les arguments non Unicode ou longs que Beaver n'interprète pas ne sont ni copiés ni rejetés.
3. Router `ShellSandbox` vers le helper existant avant toute branche CEF dans le bootstrap empaqueté.
4. Conserver la borne réelle de `CreateProcess` pour le lanceur de développement et le refus de toute substitution de module.
5. Exécuter un smoke empaqueté avec une commande shell inoffensive sous profil restreint, puis les tests Windows natifs CEF.

## Tâche 3 — rendre le reaper macOS réellement indépendant

**Fichiers principaux :**

- `src-tauri/src/services/browser/cef_supervision/macos/tracker.rs`
- `src-tauri/src/services/browser/cef_supervision/macos/tracker_loop.rs`
- nouveau module de reaper dédié si nécessaire
- `src-tauri/src/services/browser/cef_supervision/emergency.rs`
- `src-tauri/src/services/browser/cef_supervision/macos/tracker_tests.rs`
- `src-tauri/src/services/browser/cef_supervision/macos/bootstrap.rs`

1. Écrire un test rouge qui arrête le traqueur normal, maintient un helper admis et prouve que l'appel d'urgence distinct le rend non exécutable.
2. Créer et valider le reaper avant CEF. Il reçoit son propre contrôle atomique et une vue parent privée fixe ; il ne lit pas un drapeau destiné à la boucle du traqueur.
3. Brancher le vrai passage à `Closing` sur les pages de contrôle et événements de production. Le moniteur du helper reste la seconde coupure à 14 secondes ou à la disparition du parent.
4. Revalider l'identité complète avant chaque signal et ne jamais signaler un PGID réutilisé.
5. Exécuter les tests natifs macOS : traqueur paniqué, watchdog bloqué, reaper absent, helper réel sandboxé et fermeture complète.

## Tâche 4 — gérer proprement l'indisponibilité et le redémarrage

**Fichiers principaux :**

- `src-tauri/src/services/browser/cef_engine.rs`
- `src-tauri/src/services/browser/runtime_integration.rs`
- `src-tauri/src/services/browser/runtime_handle.rs`
- nouveaux modules de prévalidation/retry si nécessaires
- `src-tauri/src/app_exit.rs` et commande Tauri dédiée au redémarrage coordonné
- `src/components/ui/toast.tsx`, `src/components/ui/toast.css`
- `src/lib/toast-emitter.ts`
- `src/hooks/use-browser-capability.ts`
- sept fichiers `src/i18n/*.json` et tests associés

1. Écrire les tests rouges pour une erreur transitoire retentée exactement une fois après 200 ms, une erreur déterministe non retentée et un `cef::initialize` faux suivi d'aucun autre appel CEF.
2. Classer les erreurs transitoires par type/code OS. Une seconde tentative recrée tout l'état natif ; elle ne réutilise aucun objet de la première.
3. Si `cef::initialize` retourne faux, marquer l'échec, ne jamais appeler `cef::shutdown` et demander une fermeture coordonnée.
4. Ajouter un intent immuable `Exit | Restart` au coordinateur. Le redémarrage suit le nettoyage normal ; après `ReadyToExit`, il appelle seulement alors le mécanisme Tauri. Le tueur ultime ne relance rien.
5. Étendre le toast existant avec une action générique optionnelle et une fermeture accessible, sans nouveau système visuel. La notification CEF est traduite dans les sept langues, non modale et disparaît entièrement — bouton compris — après la constante centrale de 10 secondes.
6. Vérifier les tests backend de cycle de vie, les tests du toast, du hook de capacité, les traductions, TypeScript et lint.

## Tâche 5 — transformer les preuves CI en preuves exécutées

**Fichiers principaux :**

- `scripts/cef/cef-supervision-contracts.mjs`
- `scripts/cef/cef-supervision-contracts.test.mjs`
- `.github/workflows/ci.yml`
- scripts de smoke CEF empaqueté à créer ou étendre

1. Écrire une fixture invalide et constater que l'exécution directe du contrat retourne aujourd'hui zéro sur macOS/Linux ; corriger l'entrée directe puis observer le code non nul.
2. Faire énumérer à chaque filtre Rust ses tests attendus et échouer si le nombre est nul avant de lancer la suite.
3. Ajouter un smoke empaqueté qui lance un vrai enfant CEF avec sa ligne de commande réelle, ouvre une page et ferme Beaver pendant son activité. Les petits processus factices restent uniquement des tests de protocole.
4. Rejouer localement les contrats, puis la CI Windows/macOS/Linux.

## Tâche 6 — résoudre le marqueur de bundle Tauri à sa vraie frontière

**Fichiers principaux :**

- `src-tauri/scripts/prepare-cef-windows.ps1`
- scripts/tests d'empaquetage Windows voisins
- bootstrap ou module d'intégration choisi après preuve
- `src-tauri/src/lib.rs`

1. Reproduire l'avertissement sur un paquet NSIS et localiser le marqueur dans le bootstrap final et dans le module qui exécute Tauri.
2. Vérifier dans la source officielle Tauri comment le type de bundle est injecté. Refuser toute solution qui masque seulement l'avertissement ou invente une valeur à l'exécution.
3. Écrire d'abord un test d'empaquetage qui échoue si le bootstrap ou le module Tauri ne reçoit pas le type réellement construit.
4. Implémenter une seule autorité compatible avec le mécanisme Tauri officiel, puis valider NSIS et tout autre bundle Windows effectivement demandé par la commande.
5. Retirer le `black_box` devenu inutile seulement si le test prouve que le bon module porte la valeur attendue.

## Tâche 7 — revue cumulative et clôture

1. Exécuter les tests ciblés après chaque tâche, puis les suites complètes Rust séquentielle et parallèle, format, Clippy, frontend, TypeScript, lint et `npm run test:cef`.
2. Construire et fermer réellement Beaver sur Windows et macOS avec une page CEF active ; vérifier qu'aucun processus possédé ne reste runnable.
3. Vérifier l'absence de régression sur Ollama, SearXNG, extensions, shell isolé, navigateur normal, croix macOS et fermeture Windows/Linux.
4. Mettre à jour Graphify, l'inventaire et l'état factuel des preuves seulement après résultats réels.
5. Attacher une Git note détaillée à chaque commit concerné, puis une note finale reliant causes racines, décisions, alternatives rejetées et validations multi-OS.
6. La PR reste brouillon tant que le vrai smoke CEF, le reaper macOS et le marqueur de bundle ne sont pas prouvés.
