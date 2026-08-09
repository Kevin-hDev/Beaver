# Jalon 4 — convergence et validation multi-OS

## Autorité et dépendance

Ce document dépend du [contrat de supervision unifiée](./2026-08-09-unified-shutdown-supervision-design.md), de l'[inventaire de reprise](./2026-08-09-shutdown-reference-branch-inventory.md) et des quatre jalons précédents — 1, 1B, 2 et 3 — fusionnés. Sa branche est créée depuis leur `main` validé.

## Objectif fusionnable

Fermer les angles morts restants sans ajouter de nouveau comportement produit, puis prouver que l'ensemble fonctionne sur Windows, macOS et Linux. Ce jalon ne sert pas de rattrapage à des PR précédentes rouges : chaque jalon doit déjà être vert avant sa fusion.

## Nettoyage structurel dans le périmètre

- découper tout fichier de production dépassant 230 lignes ;
- séparer les responsabilités restantes de `scheduler/mod.rs` ;
- centraliser la validation, suppression et reprise des dossiers internes ;
- supprimer les anciens helpers et chemins de fermeture devenus morts ;
- supprimer les champs et types devenus morts, dont `run_when_window_closed` s'il subsiste ;
- conserver `scheduler.notify_config_changed()` après chaque mutation de réveil ;
- centraliser les budgets, codes d'erreur et limites ;
- mettre Graphify à jour.

## Cinq inventaires cumulatifs

1. Tous les processus créés directement ou par une bibliothèque native, classés en possédés, externes, courts ou transférés.
2. Tous les travaux longs ou mutateurs, avec admission, annulation et preuve de fin.
3. Tous les appels synchrones atteignables pendant la fermeture, avec frontière bloquante et borne.
4. Toutes les transitions Ollama, avec l'état durable avant et après chaque mutation.
5. Tous les accès au journal ou aux dossiers Ollama, qui doivent aboutir au gestionnaire unique et au même verrou.

Les recherches couvrent le diff cumulé depuis le `main` antérieur au jalon 1, pas seulement le dernier PR.

La review recoupe aussi les 22 lignes de l'inventaire de reprise. Elle refuse toute ligne encore ouverte ou fermée sans test et revalide spécifiquement le job macOS natif issu de `d3c7011`, déjà introduit au jalon 1B. Ce job doit préparer la source CEF vérifiée, exécuter `cargo check --all-targets` et Clippy strict sur macOS ; sa présence dans le YAML sans exécution verte ne suffit pas.

## Validation native

### Windows

- fermeture par croix et tray ;
- Job Object et descendants ;
- état CEF normal obligatoirement `Ready supervisé`, prouvé nativement avant fusion ;
- helper suivi par table parent privée, boîte sandboxée isolée et Job Object vide propre au slot, affectation imbriquée revalidée avec le sandbox Chromium actif ;
- chaque type CEF réel publie avec ses SIDs de restriction et son niveau MIC sans pouvoir écrire l'autorité ou signaler sa propre admission ; corruption inter-slot et faux handle refusés ;
- publication après 13 secondes refusée par sa génération invalidée, aucun appel CEF, puis disparition du bootstrap constatée sous 5 secondes ;
- helper shell simultané jamais classé comme CEF ; échec local injecté avant initialisation : `Unavailable avant lancement` et aucun helper CEF créé ;
- PTY et handles verrouillés ;
- mise à jour Beaver avec helper survivant ;
- mise à jour Ollama avec renommage temporairement bloqué.

### macOS

- croix rouge qui masque sans fermer ;
- `Cmd+Q` qui ferme tout ;
- Dock masqué pendant le vrai nettoyage ;
- groupes possédés et vérification `proc_pidinfo` ;
- état CEF normal obligatoirement `Ready supervisé`, prouvé nativement avant fusion ;
- arrêt CEF natif normal, objets et groupe préparés avant le sandbox, publication et moniteur après le sandbox, réservation tardive refusée ;
- watchdog général bloqué, reaper parent qui revalide PID/parent/démarrage/exécutable/PGID et auto-terminaison du helper à l'échéance, sans signaler un PGID réutilisé ; disparition constatée sous 5 secondes ;
- échec local injecté avant initialisation : interface utilisable avec capacité navigateur indisponible et aucun helper CEF créé.

### Linux

- fermeture par croix et tray ;
- signal de mort du parent et groupes ;
- zombie non re-signalé ;
- PTY moissonné ;
- `native_browser` désactivé et aucun helper CEF présent.

## Suites obligatoires

- tests Rust ciblés après chaque modification ;
- suite Rust complète séquentielle avec `windows-tests` ;
- suite Rust parallèle pour détecter les états globaux et tests instables ;
- `cargo fmt --check` et Clippy strict ;
- TypeScript, lint et tous les tests frontend ;
- scripts de build, runner E2E, CEF et hôte d'extensions ;
- CI native Windows, Ubuntu et macOS ;
- lancement et fermeture manuels d'un build natif sur les trois OS ;
- tueur ultime précréé, échec de création au démarrage et watchdog général réellement bloqué sans dépassement de l'échéance ;
- contrôle immédiat des services/helpers admis encore exécutables ; si CEF est actif, vérification qu'un candidat non admis reste dans le bootstrap fail-closed, sinon preuve qu'aucun helper n'a été lancé ; puis attente de 5 secondes au plus pour tous les objets résiduels et contrôle des fichiers ;
- mise à jour Beaver interrompue et mise à jour Ollama interrompue ;
- mise à jour Ollama coupée pendant chaque suppression de rebut et entre disparition du rebut, synchronisation du parent et retrait du journal ;
- contrôle final des 22 lignes de reprise et des preuves consignées dans les Git notes des jalons.

## Review finale

La review compare le `main` final au `main` précédant le jalon 1. Elle vérifie les décisions consignées dans le contrat, les cinq inventaires, les erreurs dans les sept langues, les limites de collections, les fichiers sous 230 lignes et l'absence de données sensibles dans les logs.

## Critères de fusion

- aucune nouvelle correction fonctionnelle non couverte par sa propre spec ;
- tous les constats des reviews précédentes reliés à une correction et à un test ;
- les 22 commits de code de la branche de référence tous reliés à une reprise testée ou à un abandon approuvé ;
- aucun service ou helper admis Beaver encore runnable après une vraie fermeture ; tout bootstrap CEF refusé et tout objet noyau résiduel disparaissent dans les 5 secondes de constat ;
- sur Windows et macOS, CEF est `Ready supervisé` avec preuves natives ; le chemin local `Unavailable avant lancement` est testé séparément et n'est pas un état de livraison ;
- aucun impact sur une application ou un démon externe ;
- CI et tests manuels des trois OS verts ;
- Git note finale résumant l'ensemble des décisions et preuves.
