# Jalon 1 — socle de fermeture

## Autorité

Ce document dépend du [contrat de supervision unifiée](./2026-08-09-unified-shutdown-supervision-design.md) et de l'[inventaire de reprise](./2026-08-09-shutdown-reference-branch-inventory.md). En cas de différence, le contrat principal prévaut. Ce jalon ne peut pas modifier les décisions produit, les budgets ou les garanties multi-OS.

## Objectif fusionnable

Installer le socle qui empêche Beaver de rester invisible et impossible à quitter, sans attendre la migration complète de chaque service. La PR conserve les nettoyages existants derrière des adaptateurs jusqu'à ce que le jalon 2 adopte tous les producteurs de processus.

La branche est créée depuis `main`. Elle reprend d'abord uniquement les commits documentaires du contrat, des quatre jalons et de l'inventaire. La grande branche `codex/fix-app-shutdown-lifecycle` sert ensuite uniquement de référence ; aucun ensemble de commits de code n'est repris sans vérification isolée contre `main`.

## Inclus

- état monotone `Running -> Closing -> ReadyToExit` ;
- registre d'admission suivi, borné à 128 opérations ;
- fermeture atomique de l'admission et annulation globale ;
- politique centrale des échéances `8/10/13/15` secondes ;
- calcul de tous les délais à partir d'une échéance absolue ;
- exécuteur bloquant pour les arrêts synchrones ;
- watchdog sur thread système indépendant et tueur ultime précréé avant tout effet de bord ;
- inventaire d'urgence préalloué et interface de signalement ;
- sortie ultime brute `_exit` ou `TerminateProcess` ;
- interception idempotente des demandes de fermeture Tauri ;
- comportement de fermeture Windows, Linux et macOS déjà décidé ;
- retrait de `run_when_window_closed` des modèles actuels, avec lecture tolérante des anciens JSON ;
- conservation de l'ordre natif de fermeture CEF ;
- table d'autorité CEF parent privée à 64 slots, boîtes de publication sandboxées isolées, porte atomique et marqueur de rôle validé avant `cef::execute_process` ;
- terminaison indépendante et continuellement revérifiée de ces helpers entre 13 et 15 secondes, avec Job Object Windows et reaper de groupes macOS séparé, même si `cef::shutdown()` ou le watchdog général ne rend pas la main ;
- rétablissement dès cette PR du job CI macOS natif qui prépare CEF, exécute `cargo check --all-targets` et Clippy strict ;
- fermeture des lignes J1 de l'inventaire de reprise, avec conservation explicite des sous-parties qui restent attribuées au jalon 2 ou 3.

## Exclus

- adoption exhaustive des services et Job Objects hors helpers CEF : jalon 2 ;
- journal et migration Ollama : jalon 3 ;
- nettoyage structurel et validation finale cumulative : jalon 4.

## Compatibilité pendant la transition

Le superviseur appelle les nettoyages existants à travers des fonctions bornées. Aucun ancien chemin n'est supprimé tant que son remplacement n'est pas testé et branché. L'inventaire d'urgence accepte déjà les processus connus, mais un producteur non encore migré reste couvert par le balayage direct existant.

Le watchdog de processus ne signale qu'une identité vérifiable déjà admise dans ses slots et revérifie les terminaisons tant que son échéance n'est pas atteinte. Une publication CEF postérieure à la barrière conserve une génération invalidée et sort dans le bootstrap ; elle n'est jamais transformée en processus possédé. Le tueur ultime est créé et validé au démarrage, puis armé à `Closing` ; il ne parcourt aucun slot et déclenche seul la sortie brute à 15 secondes. Son échec de création refuse le démarrage avant tout effet de bord, tandis qu'un échec tardif du watchdog ne modifie pas l'échéance.

CEF est l'exception adoptée dès ce jalon parce que son arrêt se situe après la boucle Tauri et peut empêcher le balayage final de commencer. Avant l'initialisation, Beaver prépare une table parent privée à 64 slots, une porte atomique et, par réservation, une boîte sandboxée qui ne contient que la publication proposée. États, handles, Jobs et admission restent privés au parent. Le callback injecte un marqueur borné propre au slot ; le helper valide ce rôle, publie son identité dans sa seule boîte puis attend un événement que lui seul ne peut pas signaler. Le parent valide la publication et acquiert la preuve d'arrêt avant de passer le slot privé à `Admitted`, seule autorisation d'entrer dans CEF. `Closing` ferme les nouveaux permis et attend au plus 50 millisecondes ceux déjà pris. Une demande tardive ou une réservation non admise sort avant `cef::execute_process`, sans créer de thread avant le sandbox CEF. Le watchdog peut relire les propositions non fiables à travers la table privée si le traqueur tombe.

Sous Windows, l'exécutable Beaver sert aussi au bac à sable shell. Le filtre CEF exige donc le marqueur réservé, le parent et la génération ; le chemin de l'exécutable seul est insuffisant. Les objets de publication et d'attente emploient les droits minimaux compatibles avec les SIDs activés, les SIDs de restriction et le niveau MIC réel de chaque type CEF sandboxé. Un helper ne peut ni écrire l'autorité parent, ni signaler sa propre admission. Si la capacité est saturée, ambiguë ou incompatible avec le sandbox actif, l'initialisation échoue ou, si CEF tourne déjà, la porte ferme et une fermeture coordonnée commence immédiatement. Aucun helper shell ni navigateur externe n'est adopté.

Avant `Admitted`, chaque helper Windows est placé seul dans un Job Object vide `KILL_ON_JOB_CLOSE`, distinct du job de sandbox Chromium et validé avec le sandbox actif. Cette affectation nécessite aussi `PROCESS_SET_QUOTA`. Les handles processus/Job sont non héritables et possédés uniquement par le slot parent ; tout échec termine le bootstrap au lieu de désactiver le sandbox ou de continuer sans confinement.

Sous macOS, un reaper parent est créé avant CEF et reçoit uniquement les identités privées des groupes déjà admis. De 13 à 15 secondes, il les revalide puis peut les signaler sans dépendre du watchdog général. Le helper ouvre et mappe ses seuls objets puis crée son groupe avant le sandbox ; après le sandbox, il publie et démarre un moniteur minimal qui respecte l'échéance absolue ou s'auto-termine si son parent disparaît. Il ne suppose pas que Seatbelt l'autorise à tuer tout son groupe. L'échec du reaper empêche CEF de démarrer.

## Tests obligatoires

- transitions monotones et demandes répétées ;
- refus d'admission après `Closing` ;
- slot réutilisé protégé par sa génération ;
- registre saturé fermé par défaut ;
- panique, abandon et terminaison normale libèrent leur admission ;
- appel réellement bloquant qui ne peut pas empêcher les échéances du watchdog ;
- appel OS du watchdog de processus lui-même bloqué : le tueur ultime distinct respecte encore l'échéance de 15 secondes ;
- création du tueur ultime refusée au démarrage : aucun effet de bord ; création du watchdog refusée à `Closing` : tueur déjà armé inchangé ;
- aucune seconde enveloppe de délai après interruption ;
- implémentations factices des sorties brutes, sans terminer le processus de test ;
- croix Windows/Linux, croix macOS et Quitter sur les trois OS ;
- anciens JSON avec `run_when_window_closed` lus sans restaurer le comportement ;
- tests de l'ordre CEF existant inchangés ;
- test déterministe avec un processus fixture enregistré comme helper CEF et une frontière CEF volontairement bloquée : le watchdog le termine avant la sortie forcée, sans dépendre du balayage situé après `cef::shutdown()` ;
- test de saturation et d'identité CEF ambiguë : initialisation refusée, aucun PID externe signalé ;
- callback sans ligne de commande exploitable ou marqueur impossible à ajouter : génération invalidée, porte fermée et aucun appel CEF de repli ;
- callback après `Closing`, réservation expirée et helper publié après 13 secondes : génération invalide, aucune admission et sortie avant CEF ;
- candidat CEF créé juste avant 15 secondes mais jamais publié : il n'appelle jamais `cef::execute_process` et disparaît dans la fenêtre de constat ;
- permis d'admission encore bloqué après 50 millisecondes au passage à `Closing` : génération invalidée et admission tardive impossible, même si la preuve locale arrive ensuite ;
- helpers CEF et shell simultanés sous Windows : seul le rôle CEF réservé est adopté ;
- chaque type CEF Windows réel publie sous son jeton sandboxé avec les DACL, SIDs de restriction et niveaux MIC minimaux ; aucun ne peut signaler son admission ou écrire la table parent ;
- boîte CEF corrompue avant validation, réécriture après scellement, faux handle et tentative inter-slot : aucune autorité parent modifiée, aucun processus externe signalé et échec fermé ;
- sandbox Chromium actif + Job Object vide propre au slot : affectation imbriquée réussie ; son échec simulé tue le bootstrap et refuse `Admitted` sans affaiblir le sandbox ;
- helper macOS : objets et groupe préparés avant `sandbox.initialize`, publication et moniteur seulement après, identité revérifiée avant `Admitted`, reaper parent et auto-terminaison exercés avec watchdog général bloqué ;
- PID/PGID macOS réutilisé refusé par le reaper, aucun processus externe signalé et helper réel arrêté à son échéance absolue ;
- course entre retrait normal et terminaison forcée d'un slot CEF Windows : un seul chemin revendique les handles processus/Job, le premier reste ouvert jusqu'au signal de fin et aucun n'est réutilisé ;
- `TerminateProcess` accepté mais processus non encore signalé : le watchdog conserve et revérifie le slot ;
- panne simulée du traqueur : le watchdog part de la table privée et revalide toute proposition de boîte avant de drainer un slot admis ;
- chemin CEF normal : dernier rafraîchissement, handles retirés et thread du traqueur joint avant le balayage final ;
- smoke test CEF natif sur Windows et macOS : initialiser un vrai helper CEF avec le sandbox actif, bloquer la frontière avant l'appel natif avec le hook de test, publier un lancement tardif, laisser agir le filet d'urgence, contrôler qu'aucun helper `Admitted` n'est encore runnable à la sortie, puis attendre au plus les 5 secondes de constat la disparition des états `terminating`, zombies ou bootstrap refusé ;
- test Linux confirmant que `native_browser` reste désactivé et qu'aucun helper CEF n'est créé.

## Critères de fusion

- aucune fenêtre Beaver invisible au-delà de l'échéance absolue dans les tests ;
- aucune nouvelle tâche admise après la fermeture ;
- aucune régression des nettoyages existants ;
- aucun helper CEF possédé ne poursuit d'exécution dans le scénario d'arrêt natif bloqué, et tout objet noyau résiduel disparaît dans la fenêtre de constat ;
- jobs CEF natifs Windows et macOS réellement exécutés et verts ;
- toutes les sous-lignes J1 de l'inventaire sont fermées et référencent leurs tests ;
- fichiers de production sous 230 lignes ;
- tests Rust ciblés et complets, formatage, Clippy, frontend et CI native verts ;
- Git note du jalon avec décisions et preuves de validation.
