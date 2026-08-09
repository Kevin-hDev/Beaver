# Jalon 1 — socle de fermeture

## Autorité

Ce document dépend du [contrat de supervision unifiée](./2026-08-09-unified-shutdown-supervision-design.md). En cas de différence, le contrat principal prévaut. Ce jalon ne peut pas modifier les décisions produit, les budgets ou les garanties multi-OS.

## Objectif fusionnable

Installer le socle qui empêche Beaver de rester invisible et impossible à quitter, sans attendre la migration complète de chaque service. La PR conserve les nettoyages existants derrière des adaptateurs jusqu'à ce que le jalon 2 adopte tous les producteurs de processus.

La branche est créée depuis `main`. Elle reprend d'abord uniquement les commits documentaires du contrat et des quatre jalons. La grande branche `codex/fix-app-shutdown-lifecycle` sert ensuite uniquement de référence ; aucun ensemble de commits de code n'est repris sans vérification isolée contre `main`.

## Inclus

- état monotone `Running -> Closing -> ReadyToExit` ;
- registre d'admission suivi, borné à 128 opérations ;
- fermeture atomique de l'admission et annulation globale ;
- politique centrale des échéances `8/10/13/15` secondes ;
- calcul de tous les délais à partir d'une échéance absolue ;
- exécuteur bloquant pour les arrêts synchrones ;
- watchdog sur thread système indépendant ;
- inventaire d'urgence préalloué et interface de signalement ;
- sortie ultime brute `_exit` ou `TerminateProcess` ;
- interception idempotente des demandes de fermeture Tauri ;
- comportement de fermeture Windows, Linux et macOS déjà décidé ;
- retrait de `run_when_window_closed` des modèles actuels, avec lecture tolérante des anciens JSON ;
- conservation de l'ordre natif de fermeture CEF.

## Exclus

- adoption exhaustive des services et Job Objects : jalon 2 ;
- journal et migration Ollama : jalon 3 ;
- nettoyage structurel et validation finale cumulative : jalon 4.

## Compatibilité pendant la transition

Le superviseur appelle les nettoyages existants à travers des fonctions bornées. Aucun ancien chemin n'est supprimé tant que son remplacement n'est pas testé et branché. L'inventaire d'urgence accepte déjà les processus connus, mais un producteur non encore migré reste couvert par le balayage direct existant.

Le watchdog de 13 secondes ne signale qu'une identité vérifiable présente dans ses slots. Il ne transforme jamais un PID découvert tardivement en processus possédé. À 15 secondes, la sortie brute garantit que Beaver lui-même ne reste pas fantôme.

## Tests obligatoires

- transitions monotones et demandes répétées ;
- refus d'admission après `Closing` ;
- slot réutilisé protégé par sa génération ;
- registre saturé fermé par défaut ;
- panique, abandon et terminaison normale libèrent leur admission ;
- appel réellement bloquant qui ne peut pas empêcher les échéances du watchdog ;
- aucune seconde enveloppe de délai après interruption ;
- implémentations factices des sorties brutes, sans terminer le processus de test ;
- croix Windows/Linux, croix macOS et Quitter sur les trois OS ;
- anciens JSON avec `run_when_window_closed` lus sans restaurer le comportement ;
- tests de l'ordre CEF existant inchangés.

## Critères de fusion

- aucune fenêtre Beaver invisible au-delà de l'échéance absolue dans les tests ;
- aucune nouvelle tâche admise après la fermeture ;
- aucune régression des nettoyages existants ;
- fichiers de production sous 230 lignes ;
- tests Rust ciblés et complets, formatage, Clippy, frontend et CI native verts ;
- Git note du jalon avec décisions et preuves de validation.
