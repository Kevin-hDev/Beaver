# Jalon 1 — socle de fermeture

## Autorité

Ce document dépend du [contrat de supervision unifiée](./2026-08-09-unified-shutdown-supervision-design.md) et de l'[inventaire de reprise](./2026-08-09-shutdown-reference-branch-inventory.md). En cas de différence, le contrat principal prévaut. Ce jalon ne peut pas modifier les décisions produit, les budgets ou les garanties multi-OS.

## Objectif fusionnable

Installer le socle qui empêche le processus Beaver de rester invisible et impossible à quitter, sans attendre la migration complète de chaque service ni la supervision native de CEF. La PR conserve les nettoyages existants derrière des adaptateurs jusqu'à ce que les jalons suivants adoptent leurs producteurs.

La branche est créée depuis `main`. Elle reprend d'abord uniquement les commits documentaires du contrat, des cinq jalons et de l'inventaire. La grande branche `codex/fix-app-shutdown-lifecycle` sert ensuite uniquement de référence ; aucun ensemble de commits de code n'est repris sans vérification isolée contre `main`.

Ce jalon est volontairement limité : il garantit la sortie du processus parent Beaver à l'échéance absolue, mais ne prétend pas encore prouver la disparition forcée de chaque helper CEF si l'arrêt natif se bloque. Cette preuve appartient au [jalon 1B](./2026-08-09-shutdown-milestone-1b-cef-design.md), obligatoire avant le jalon 2 et avant toute nouvelle release publique.

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
- conservation de l'ordre natif existant services → sortie Tauri → arrêt CEF → balayage ;
- fermeture des lignes J1 de l'inventaire de reprise, avec conservation explicite des sous-parties attribuées aux jalons 1B, 2 ou 3.

## Exclus

- admission, identification, confinement et terminaison forcée des helpers CEF : jalon 1B ;
- CI native et smoke tests du protocole CEF sandboxé : jalon 1B ;
- adoption exhaustive des autres services et Job Objects : jalon 2 ;
- journal et migration Ollama : jalon 3 ;
- nettoyage structurel et validation finale cumulative : jalon 4.

## Compatibilité pendant la transition

Le superviseur appelle les nettoyages existants à travers des fonctions bornées. Aucun ancien chemin n'est supprimé tant que son remplacement n'est pas testé et branché. L'inventaire d'urgence accepte déjà les processus connus, mais un producteur non encore migré reste couvert par le balayage direct existant.

Le watchdog de processus ne signale qu'une identité vérifiable déjà admise dans ses slots et revérifie les terminaisons tant que son échéance n'est pas atteinte. Le tueur ultime est créé et validé au démarrage, puis armé à `Closing` ; il ne parcourt aucun slot et déclenche seul la sortie brute à 15 secondes. Son échec de création refuse le démarrage avant tout effet de bord, tandis qu'un échec tardif du watchdog ne modifie pas l'échéance.

Le chemin CEF reste inchangé dans ce jalon : même sandbox, même initialisation et même ordre d'arrêt. Si `cef::shutdown()` se bloque, le tueur ultime garantit que Beaver lui-même sort à 15 secondes, mais le jalon 1 ne transforme pas l'auto-terminaison habituelle des helpers Chromium en garantie contractuelle. L'absence de preuve forcée pour ces helpers est une exception temporaire, explicitement ouverte dans l'inventaire jusqu'au jalon 1B.

Aucune release publique ne peut être créée dans cet état transitoire. Le jalon 1B doit être fusionné et validé nativement avant la reprise des jalons produit suivants ou la publication d'une version.

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
- ordre CEF existant inchangé ;
- frontière CEF factice volontairement bloquée : le processus parent atteint tout de même la sortie ultime ;
- smoke test du chemin CEF normal existant sur Windows et macOS, sans modification du sandbox ni du lancement des helpers ;
- test Linux confirmant que `native_browser` reste désactivé et qu'aucun helper CEF n'est créé.

## Critères de fusion

- aucune fenêtre Beaver invisible au-delà de l'échéance absolue dans les tests ;
- aucune nouvelle tâche admise après la fermeture ;
- aucune régression des nettoyages ni de l'ordre CEF existants ;
- l'exception CEF temporaire est visible et reste ouverte pour le jalon 1B, sans fausse déclaration « aucun helper survivant » ;
- toutes les sous-lignes J1 de l'inventaire sont fermées et référencent leurs tests ;
- fichiers de production sous 230 lignes ;
- tests Rust ciblés et complets, formatage, Clippy, frontend et CI native verts ;
- aucune release publique avant fusion du jalon 1B ;
- Git note du jalon avec décisions et preuves de validation.
