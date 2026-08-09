# Jalon 2 — processus et services possédés

## Autorité et dépendance

Ce document dépend du [contrat de supervision unifiée](./2026-08-09-unified-shutdown-supervision-design.md), de l'[inventaire de reprise](./2026-08-09-shutdown-reference-branch-inventory.md), du [jalon 1](./2026-08-09-shutdown-milestone-1-core-design.md) et du [jalon 1B](./2026-08-09-shutdown-milestone-1b-cef-design.md). Sa branche est créée depuis le `main` où ces deux jalons ont été fusionnés.

## Objectif fusionnable

Faire passer chaque service capable de lancer un processus ou un travail long par le superviseur, puis garantir son arrêt et son moissonnage. À la fusion, une vraie fermeture ne laisse aucun service Beaver possédé encore runnable ; les objets déjà terminés disparaissent ensuite dans la fenêtre native de constat de 5 secondes.

## Inventaire obligatoire

- gateway Telegram, Discord et Slack, consommateur et traitements de messages ;
- hôte d'extensions, installation, démarrage, redémarrage et exécution ;
- SearXNG, installation Python, serveur et lecteurs ;
- serveurs MCP stdio et commandes d'installation ;
- Forecast, runtime Python, sidecar, évaluations et commandes longues ;
- terminaux PTY, shells et threads lecteurs ;
- sidecar Ollama existant pour sa propriété de processus seulement ;
- téléchargements de modèles ;
- scheduler et réveils ;
- flux agentiques, sous-agents et commandes shell ;
- téléchargement de mise à jour Beaver et helper `UpdateHandoff` ;
- OAuth et autres serveurs locaux temporaires du périmètre de fermeture ;
- sous-processus WebView créés indirectement par Tauri ou le runtime OS, classés comme descendants dédiés possédés ou services système partagés externes à partir d'une observation native.

Le jalon reprend aussi explicitement les deux dettes fonctionnelles que la grande branche avait corrigées mais que `main` possède encore :

- résultats typés des réveils ponctuels, annulation après consommation journalisée, désactivation bénigne silencieuse, erreur de revendication manquée journalisée et notification après chaque mutation ;
- codes d'admission de flux stables, liste publique fermée côté interface et traductions dans les sept langues.

Toutes les lignes J2 de l'inventaire sont fermées dans cette PR. Une ligne partagée conserve la preuve de ses parties J1 et J1B déjà fusionnées ; toute sous-partie J3 reste explicitement ouverte jusqu'au jalon 3.

La review du jalon recherche aussi tous les `Command::new`, `tokio::process::Command`, `portable-pty`, `tokio::spawn`, `tauri::async_runtime::spawn` et `std::thread::spawn`. Elle compare en plus l'arbre de processus natif avant et après l'ouverture des WebViews Tauri et CEF afin de couvrir les créations cachées dans les bibliothèques. Les helpers CEF restent la propriété du jalon 1B et sont revalidés ici sans réimplémentation ; chaque autre résultat reçoit un propriétaire, une borne et un chemin d'arrêt, ou une exemption documentée.

## Confinement par plateforme

### Windows

Les enfants possédés sont affectés immédiatement au Job Object global après spawn et avant restitution du handle. Les terminaux utilisent le handle natif fourni par `portable-pty`. Un échec d'affectation arrête et moissonne l'enfant, balaie les descendants détectés dans la fenêtre, puis échoue sans continuer.

Le helper de mise à jour utilise exclusivement le chemin `UpdateHandoff` du contrat principal et n'entre pas dans le Job Object destructeur.

### Linux

Les enfants directs possédés reçoivent un groupe dédié et un signal de mort du parent avant `exec`. Le signal de groupe précède les signaux individuels. Les zombies et processus morts sont ignorés puis moissonnés par leur propriétaire.

### macOS

Les enfants possédés reçoivent un groupe dédié. Les slots du watchdog stockent PID, groupe et heure de démarrage ; `proc_pidinfo` revérifie l'identité avec une structure de pile fixe avant le signal d'urgence.

## Arrêt des services

Chaque service expose un `stop_and_wait` idempotent qui ferme son admission locale, annule, attend dans le budget restant puis abandonne seulement les tâches encore actives. Les handles ne sont jamais attendus sous le verrou qui protège leur registre.

Le gateway borne sa file à 256 messages et ses traitements simultanés à 64. Une vraie fermeture l'arrête ; la croix rouge macOS ne déclenche pas la fermeture et le laisse actif.

Le téléchargement Beaver écoute l'annulation à chaque étape et supprime son partiel. Le helper validé reste l'unique survivant autorisé. Le menu et le clic du tray partagent `show`, `unminimize`, `focus` ; un vrai Quitter macOS masque le Dock avant le nettoyage.

## Tests obligatoires

- démarrage refusé après `Closing` pour chaque service ;
- fermeture pendant `starting`, `running` et `stopping` ;
- saturation de chaque registre borné ;
- aucun redémarrage d'extension après son arrêt ;
- gateway simulé sur les trois canaux, file fermée et tâches attendues ;
- MCP, Forecast, SearXNG et PTY réellement lancés puis moissonnés dans les profils de test natifs ;
- WebView Tauri activée sur chaque OS, descendants dédiés identifiés puis absents après fermeture, sans signal envoyé aux services système partagés ;
- Job Object réussi et échec d'affectation Windows ;
- groupe et zombie Linux ;
- groupe et identité macOS ;
- fermeture pendant le téléchargement et le handoff Beaver ;
- helper non transféré arrêté, helper transféré préservé ;
- réveil ponctuel inactif sans appel provider, puis annulation après revendication enregistrée comme annulée ;
- erreur de `claim_once` pendant la réconciliation d'un réveil manqué : entrée d'erreur bornée et traduite ; état `Inactive` bénin sans fausse erreur ;
- notification du scheduler après chaque mutation de réveil ;
- admission de flux saturée ou remplacée affichée dans chacune des sept langues, erreur inconnue masquée par le fallback générique ;
- tray et Dock sur leurs plateformes natives.

## Critères de fusion

- inventaire des spawns complet et joint à la review ;
- aucun service possédé encore runnable après fermeture, puis aucun objet processus possédé vérifiable après la fenêtre native de constat ;
- aucune application externe adoptée ou tuée ;
- aucun changement de transaction Ollama hors intégration de son processus au superviseur ;
- toutes les sous-lignes J2 de l'inventaire sont fermées et référencent leurs tests ;
- suites, CI native et tests manuels du jalon verts ;
- Git note détaillée du jalon.
