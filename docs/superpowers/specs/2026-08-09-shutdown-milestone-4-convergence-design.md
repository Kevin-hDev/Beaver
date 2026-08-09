# Jalon 4 — convergence et validation multi-OS

## Autorité et dépendance

Ce document dépend du [contrat de supervision unifiée](./2026-08-09-unified-shutdown-supervision-design.md) et des trois jalons précédents fusionnés. Sa branche est créée depuis leur `main` validé.

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

1. Tous les processus créés, classés en possédés, externes, courts ou transférés.
2. Tous les travaux longs ou mutateurs, avec admission, annulation et preuve de fin.
3. Tous les appels synchrones atteignables pendant la fermeture, avec frontière bloquante et borne.
4. Toutes les transitions Ollama, avec l'état durable avant et après chaque mutation.
5. Tous les accès au journal ou aux dossiers Ollama, qui doivent aboutir au gestionnaire unique et au même verrou.

Les recherches couvrent le diff cumulé depuis le `main` antérieur au jalon 1, pas seulement le dernier PR.

## Validation native

### Windows

- fermeture par croix et tray ;
- Job Object et descendants ;
- PTY et handles verrouillés ;
- mise à jour Beaver avec helper survivant ;
- mise à jour Ollama avec renommage temporairement bloqué.

### macOS

- croix rouge qui masque sans fermer ;
- `Cmd+Q` qui ferme tout ;
- Dock masqué pendant le vrai nettoyage ;
- groupes possédés et vérification `proc_pidinfo` ;
- arrêt CEF natif.

### Linux

- fermeture par croix et tray ;
- signal de mort du parent et groupes ;
- zombie non re-signalé ;
- PTY moissonné ;
- arrêt CEF natif.

## Suites obligatoires

- tests Rust ciblés après chaque modification ;
- suite Rust complète séquentielle avec `windows-tests` ;
- suite Rust parallèle pour détecter les états globaux et tests instables ;
- `cargo fmt --check` et Clippy strict ;
- TypeScript, lint et tous les tests frontend ;
- scripts de build, runner E2E, CEF et hôte d'extensions ;
- CI native Windows, Ubuntu et macOS ;
- lancement et fermeture manuels d'un build natif sur les trois OS ;
- contrôles de processus et de fichiers après fermeture ;
- mise à jour Beaver interrompue et mise à jour Ollama interrompue.

## Review finale

La review compare le `main` final au `main` précédant le jalon 1. Elle vérifie les décisions consignées dans le contrat, les cinq inventaires, les erreurs dans les sept langues, les limites de collections, les fichiers sous 230 lignes et l'absence de données sensibles dans les logs.

## Critères de fusion

- aucune nouvelle correction fonctionnelle non couverte par sa propre spec ;
- tous les constats des reviews précédentes reliés à une correction et à un test ;
- aucun processus Beaver possédé après une vraie fermeture ;
- aucun impact sur une application ou un démon externe ;
- CI et tests manuels des trois OS verts ;
- Git note finale résumant l'ensemble des décisions et preuves.
