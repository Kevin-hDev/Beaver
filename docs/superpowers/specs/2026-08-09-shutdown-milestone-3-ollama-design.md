# Jalon 3 — transaction Ollama durable

## Autorité et dépendance

Ce document dépend du [contrat de supervision unifiée](./2026-08-09-unified-shutdown-supervision-design.md), de l'[inventaire de reprise](./2026-08-09-shutdown-reference-branch-inventory.md) et des jalons 1 et 2 fusionnés. Sa branche est créée depuis leur `main` validé et peut avancer pendant que le jalon 1B termine ses preuves CEF natives.

## Objectif fusionnable

Remplacer les suppositions basées sur les dossiers et le résultat booléen du démarrage par un gestionnaire transactionnel unique, récupérable après chaque interruption et incapable de valider le mauvais démon Ollama.

## Périmètre

- verrou unique pour installation, mise à jour, récupération, polling et première installation ;
- journal atomique borné à cinq états ;
- empreintes et versions exactes de la cible et de la version précédente, avec cible rejetée optionnelle uniquement quand la cible a déjà disparu avant le rollback durable ;
- deux stagings modernes distincts ;
- migration des layouts publiés ;
- résolution et confinement du stockage effectif des modèles avant toute mutation de bundle ;
- sonde possédée du binaire cible sur port et dossier de modèles isolés ;
- résultat de démarrage typé ;
- rollback durable et nettoyage non fatal après validation ;
- annulation de première installation avant et après commit ;
- polling fondé sur l'état typé du gestionnaire ;
- codes d'erreur traduits dans les sept langues.

Le champ optionnel `rejected_target` possède une règle fermée : `None` signifie qu'aucune cible rejetée n'est attendue sur disque. Il est réservé à une migration héritée ou à la reprise de `PendingValidation` lorsque la destination a déjà disparu. Si `ollama-bundle-failed` ou son rebut `ollama-bundle-failed-delete` existe malgré cette absence dans `RollbackPending` ou `RollbackCleanupPending`, le gestionnaire classe l'état comme ambigu, ne renomme et ne supprime rien, conserve le journal et renvoie `ollama-update-recovery-required`.

Deux états légaux avec `None` restent récupérables et sont distingués explicitement. `RollbackPending` avec la sauvegarde précédente seule restaure cette sauvegarde en destination, synchronise le parent puis retire le journal. `RollbackCleanupPending` avec la destination précédente seule retire seulement le journal. Lorsqu'une destination cible existe encore, le gestionnaire écrit directement et synchronise `RollbackPending` avec `rejected_target: Some` avant le premier renommage ; le dossier rejeté ne peut donc jamais être créé légitimement avec `None`.

Les suppressions de `ollama-bundle-backup` et `ollama-bundle-failed` passent d'abord par des rebuts internes distincts, renommés atomiquement après validation de l'empreinte. La phase autorise ensuite la reprise d'un rebut partiellement supprimé sans recalculer son empreinte, mais seulement si la source a disparu, la destination active possède encore l'empreinte attendue et la racine du rebut est un dossier interne régulier sans reparse point. Après disparition du rebut, le parent est synchronisé avant le retrait du journal, puis synchronisé encore après ce retrait. Toute autre combinaison ou impossibilité de garantir cet ordre conserve la phase et échoue fermée.

Toutes les lignes J3 de l'inventaire sont fermées dans cette PR, y compris les parties Ollama des lignes partagées avec les jalons précédents.

## Migration obligatoire

Les tests partent des layouts réellement publiés :

- baseline `1.0.2` ;
- `1.1.0` ;
- `1.1.1` ;
- `1.1.2` ;
- branche de référence actuelle.

Ces layouts utilisent `ollama-bundle`, `ollama-bundle-staging`, `ollama-bundle-old` et `ollama-bundle-failed`. La migration applique exactement la table du contrat principal. Une combinaison inconnue reste intacte et retourne une récupération requise ; elle n'est jamais « nettoyée » par intuition.

Les migrations automatisées sont complétées par une vraie mise à niveau depuis chaque version publiée listée, avec un profil produit par cette version et au moins un modèle déjà présent. Après la mise à niveau, la liste des modèles, leurs empreintes de contrôle et un appel local restent valides sans nouveau téléchargement. Ces essais sont exécutés sur chaque OS où la version source a été publiée et leurs versions exactes sont consignées.

## Isolation du stockage des modèles

Avant toute écriture de journal, renommage ou suppression, le gestionnaire calcule le chemin de modèles réellement transmis au sidecar : valeur héritée de `OLLAMA_MODELS` lorsqu'elle existe, sinon valeur par défaut de l'environnement d'exécution. Une valeur relative est résolue depuis le vrai dossier de travail du processus Ollama. La sonde de validation utilise toujours un dossier temporaire explicitement isolé.

Le chemin effectif et chacun des dossiers transactionnels modernes ou hérités sont résolus à partir de leurs ancêtres existants, sans suivre silencieusement un symlink, une junction ou un reparse point. La comparaison emploie l'identité et les règles de casse du système de fichiers, jamais un simple préfixe textuel. Si le stockage des modèles est égal, parent ou enfant de `ollama-bundle`, `ollama-bundle-staging`, `ollama-bundle-old`, d'un staging moderne, d'une sauvegarde, d'une cible rejetée ou d'un rebut, l'opération échoue avant toute mutation avec un code stable traduit dans les sept langues. Un chemin impossible à déterminer échoue également fermé. Aucun chemin complet n'est affiché ou journalisé.

## Validation possédée

La sonde lance l'exécutable canonique du bundle cible, conserve son handle, vérifie qu'il reste vivant avant et après `/api/version`, compare la version normalisée et arrête puis moissonne l'enfant. Elle utilise `OLLAMA_NO_CLOUD=1`, un port local isolé et un dossier de modèles isolé.

Un démon Ollama externe peut continuer à servir l'usage normal, mais il n'est jamais arrêté et sa réponse ne valide jamais le bundle Beaver. Une indisponibilité temporaire conserve `PendingValidation` et suit la politique de reprise bornée du contrat.

## Écritures et annulation

Le journal est synchronisé avant chaque renommage destructif. Les copies, suppressions, hash et synchronisations passent par l'exécuteur bloquant. L'annulation n'est observée qu'aux frontières durables. Une suppression de sauvegarde ou une synchronisation de son parent échouée conserve `CleanupPending`, mais la cible validée reste utilisable et la mise à jour reste annoncée comme réussie.

## Tests obligatoires

- coupure avant et après chaque écriture de journal, renommage et synchronisation du parent ; échec injecté de chacune de ces opérations sans perte des deux versions ;
- chaque combinaison autorisée des cinq états ;
- chaque combinaison ambiguë fermée sans suppression ;
- migration `ollama-bundle-old` seule interrompue juste après l'écriture de `RollbackPending { rejected_target: None }`, puis restaurée automatiquement ;
- seconde coupure après la restauration de cette sauvegarde mais avant le retrait du journal, puis reprise idempotente ;
- `PendingValidation` sans destination et avec sauvegarde seule suit le même chemin ;
- destination cible + sauvegarde : `rejected_target: Some(empreinte cible)` est durable avant la création de `ollama-bundle-failed` ;
- coupure juste après cette écriture `Some` mais avant le renommage : le déplacement et la restauration reprennent automatiquement ;
- coupure après la suppression de la cible rejetée mais avant le retrait de `RollbackCleanupPending { rejected_target: Some }` : le journal est retiré sans toucher à la destination restaurée ;
- coupure et erreur injectée au milieu de chacun des deux rebuts de suppression : le rebut partiel est repris, la destination reste intacte et le journal ne disparaît qu'après le nettoyage ;
- coupure et erreur entre disparition du rebut, synchronisation du parent, retrait du journal et synchronisation finale : reprise idempotente sans rebut orphelin ni perte d'autorité ;
- source + rebut simultanés, rebut hors phase, symlink, junction ou reparse point : aucune suppression ;
- `RollbackCleanupPending { rejected_target: None }` avec destination précédente seule retire le journal sans modifier la destination ;
- `RollbackPending { rejected_target: None }` avec `ollama-bundle-failed` ou `ollama-bundle-failed-delete` présent : récupération requise, aucun renommage ou retrait, journal et contenu des dossiers identiques octet par octet ;
- même preuve pour `RollbackCleanupPending { rejected_target: None }` ;
- migrations de toutes les releases listées ;
- mise à niveau réelle depuis chaque release listée sur ses OS publiés, avec modèle préexistant toujours listé, vérifié et utilisable sans nouveau téléchargement ;
- `OLLAMA_MODELS` absent, externe, relatif, situé dans chaque dossier transactionnel moderne ou hérité, parent ou enfant d'un tel dossier, et aliasé par symlink, junction ou reparse point ; seuls les chemins sans chevauchement sont autorisés selon les règles natives de casse ;
- chemin de modèles impossible à résoudre : aucune écriture de journal, aucun renommage et erreur publique traduite sans chemin ;
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
- aucune transaction ne démarre tant que l'absence de chevauchement avec le stockage effectif des modèles n'est pas prouvée ;
- les mises à niveau réelles de toutes les versions publiées conservent les modèles existants sans redownload ;
- toutes les sous-lignes J3 de l'inventaire sont fermées et référencent leurs tests ;
- tests ciblés, suite complète, CI native et test manuel Ollama verts ;
- Git note détaillée du jalon.
