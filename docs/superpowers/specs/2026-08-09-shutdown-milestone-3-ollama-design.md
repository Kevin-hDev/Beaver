# Jalon 3 — transaction Ollama durable

## Autorité et dépendance

Ce document dépend du [contrat de supervision unifiée](./2026-08-09-unified-shutdown-supervision-design.md) et des jalons 1 et 2 fusionnés. Sa branche est créée depuis leur `main` validé.

## Objectif fusionnable

Remplacer les suppositions basées sur les dossiers et le résultat booléen du démarrage par un gestionnaire transactionnel unique, récupérable après chaque interruption et incapable de valider le mauvais démon Ollama.

## Périmètre

- verrou unique pour installation, mise à jour, récupération, polling et première installation ;
- journal atomique borné à cinq états ;
- empreintes et versions exactes de la cible et de la version précédente, avec cible rejetée optionnelle uniquement pour un rollback hérité où elle a déjà disparu ;
- deux stagings modernes distincts ;
- migration des layouts publiés ;
- sonde possédée du binaire cible sur port et dossier de modèles isolés ;
- résultat de démarrage typé ;
- rollback durable et nettoyage non fatal après validation ;
- annulation de première installation avant et après commit ;
- polling fondé sur l'état typé du gestionnaire ;
- codes d'erreur traduits dans les sept langues.

## Migration obligatoire

Les tests partent des layouts réellement publiés :

- baseline `1.0.2` ;
- `1.1.0` ;
- `1.1.1` ;
- `1.1.2` ;
- branche de référence actuelle.

Ces layouts utilisent `ollama-bundle`, `ollama-bundle-staging`, `ollama-bundle-old` et `ollama-bundle-failed`. La migration applique exactement la table du contrat principal. Une combinaison inconnue reste intacte et retourne une récupération requise ; elle n'est jamais « nettoyée » par intuition.

## Validation possédée

La sonde lance l'exécutable canonique du bundle cible, conserve son handle, vérifie qu'il reste vivant avant et après `/api/version`, compare la version normalisée et arrête puis moissonne l'enfant. Elle utilise `OLLAMA_NO_CLOUD=1`, un port local isolé et un dossier de modèles isolé.

Un démon Ollama externe peut continuer à servir l'usage normal, mais il n'est jamais arrêté et sa réponse ne valide jamais le bundle Beaver. Une indisponibilité temporaire conserve `PendingValidation` et suit la politique de reprise bornée du contrat.

## Écritures et annulation

Le journal est synchronisé avant chaque renommage destructif. Les copies, suppressions, hash et synchronisations passent par l'exécuteur bloquant. L'annulation n'est observée qu'aux frontières durables. Une suppression de sauvegarde échouée conserve `CleanupPending`, mais la cible validée reste utilisable et la mise à jour reste annoncée comme réussie.

## Tests obligatoires

- coupure avant et après chaque écriture de journal et chaque renommage ;
- chaque combinaison autorisée des cinq états ;
- chaque combinaison ambiguë fermée sans suppression ;
- migrations de toutes les releases listées ;
- démon externe sur la route globale avec version identique ou différente ;
- sonde cible réussie, version différente, port occupé et enfant mort ;
- échec de nettoyage après validation ;
- rollback interrompu puis repris ;
- récupération et mise à jour concurrentes sérialisées ;
- première installation annulée avant commit et pendant le premier démarrage ;
- frontend actualisé après installation conservée ;
- erreurs et statuts traduits dans les sept langues.

## Critères de fusion

- aucun accès direct aux dossiers transactionnels hors gestionnaire ;
- aucun booléen ne confond sidecar possédé et démon externe ;
- toute interruption testée possède une reprise automatique ou un état fermé sans perte ;
- tests ciblés, suite complète, CI native et test manuel Ollama verts ;
- Git note détaillée du jalon.
