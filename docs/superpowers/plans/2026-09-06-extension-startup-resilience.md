# Résilience du démarrage des extensions — plan de correction

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [x]`) syntax for tracking.

**Goal:** Un registre d’extensions incompatible ou illisible ne bloque plus les conversations en mode agent ; ses données sont conservées et sa cause est expliquée.

**Architecture:** L’index des extensions porte l’autorité de disponibilité. Une initialisation refusée ferme les capacités des extensions et les mutations de leur registre. La préparation des outils conserve uniquement les définitions natives autorisées pour le tour, respecte les limites du fournisseur et publie un avertissement traduit ainsi qu’un diagnostic persistant.

**Tech Stack:** Rust/Tauri, React/TypeScript, Vitest, traductions dans les sept langues.

**Spec:** Demande de Kevin dans cette session, diagnostic comparé entre `1ff5cf3d` et `6b8278a0` : registre v2 incompatible avec le binaire publié v1.2.1, catalogue non initialisé, erreur `request_start (unknown)`.

## Contraintes et décisions

- Pas de rétro-migration ni de réécriture d’un fichier incompatible, abîmé ou inaccessible. Raison : conserver la seule copie des choix utilisateur.
- Un nouveau binaire ne répare pas rétroactivement v1.2.1 ; ce correctif protège les versions qui l’intègrent.
- Le refus reste fermé pour les extensions : aucun outil, skill ou ressource d’extension ne doit être exposé après un échec de registre.
- Les outils natifs sont reconstruits depuis leur catalogue canonique, dans le sous-ensemble déjà autorisé. Aucun outil optionnel ou de sous-agent ne doit être ajouté par le repli.
- Les règles locales de lecture tolérante ne justifient pas de remplacer sur disque le registre refusé : la préservation des données convenue avec Kevin prévaut.
- Une seule branche `codex/extension-startup-resilience` dans le checkout courant, fichiers préexistants hors périmètre exclus du commit. Raison : isolation Git demandée, réutilisation de l’environnement testé sans déplacer le travail utilisateur.
- Un commit final et une note Git détaillée ; la revue indépendante sera lancée par Kevin dans une autre session.

## 1. Registre : compatibilité et disponibilité

Fichiers : `src-tauri/src/services/extensions/{storage,registry,registry_memory,registry_index,startup}.rs`, contrat `src-tauri/resources/extension-host/contract.json`, tests de stockage et d’initialisation.

Interface produite : `extensions::registry_availability() -> Result<(), &'static str>` ; erreurs stables `extensions_registry_version_unsupported`, `extensions_registry_unavailable`, `extensions_state_unavailable` en plus de `extensions_registry_migration_failed`.

- [x] Ajouter des tests : format futur refusé distinctement, JSON abîmé refusé, octets et sauvegardes conservés, ancien format migré normalement.
- [x] Exécuter les tests avant correction et constater les échecs attendus.
- [x] Différencier la version de format avant la désérialisation détaillée ; aucun écrit après refus.
- [x] Enregistrer l’échec de démarrage dans l’index, retirer les capacités, bloquer les mutations et permettre une initialisation valide ultérieure.
- [x] Tester la transition refus → succès avec un catalogue vide valide ; conserver des traces à codes bornés.

## 2. Conversation : repli sûr sur les outils natifs

Fichiers : `src-tauri/src/services/agent_local/extension_tool_set*.rs`, préparation API et Ollama, diagnostics.

Interface consommée : disponibilité du registre ci-dessus. Interface produite : état dégradé du jeu d’outils et code d’avertissement stable.

- [x] Tester le défaut actuel : registre non initialisé avant un message, extension remplaçant un outil natif, outil inconnu, limite fournisseur et sous-ensemble autorisé.
- [x] Avant `configure`, consulter la disponibilité. En cas de refus, retourner les seuls outils natifs correspondants ; retirer découverte/ressources d’extensions.
- [x] Dégrader également une erreur de stockage propre à l’état d’extensions de session, sans masquer une erreur d’admission ou de fournisseur.
- [x] Transmettre une notice par requête et conserver la cause dans les diagnostics ; couvrir API et Ollama.
- [x] Tester que ce jeu dégradé ne recharge aucune extension pendant le tour et que le message peut atteindre le transport simulé.

## 3. Explications utilisateur

Fichiers : `src/lib/agent-error-codes.ts`, `src/i18n/{fr,en,es,de,it,zh,ja}.json`, tests d’erreurs et de notices.

- [x] Traduire les codes de format trop récent, registre illisible/inaccessible et état de session indisponible.
- [x] Réutiliser l’affichage de notices existant ; ne pas montrer de chemins, erreurs système ou identifiants techniques.
- [x] Tester les erreurs persistées et les erreurs reçues en direct, sans `unknown` pour ces codes.
- [x] Vérifier l’affichage des textes dans les thèmes sombre et clair avec les primitives existantes.

## 4. Validation et livraison

- [x] Tests Rust ciblés : stockage/migration, disponibilité, préparation d’outils, diagnostics, contrat des traductions.
- [x] Tests frontend ciblés, `npx tsc --noEmit`, `npm run lint`, `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`.
- [x] Générer et contrôler les artefacts dérivés du contrat d’extensions si modifié.
- [x] Mettre à jour Graphify pour le code et tenter la maintenance documentaire ; signaler la limite ci-dessous.
- [ ] Livraison après gel de ce plan : créer le commit final avec les seuls fichiers contrôlés.
- [ ] Livraison après gel de ce plan : attacher une note Git : causes, décisions, fichiers, tests exécutés et limites, scénarios à examiner par le reviewer. Relire `git notes show HEAD`.

## Résultats et précisions de validation

- Les premiers tests ont reproduit le refus des outils natifs, le diagnostic technique visible et les écrasements de registres refusés. Les corrections ont ensuite fait passer ces cas.
- La fermeture des extensions dure jusqu’à la destruction du jeu d’outils. Un garde borné par session, avec comptage des tours qui se chevauchent, empêche un remplacement natif, un skill ou une ressource d’extension de se réactiver après le repli. Raison : une panne du stockage de session peut laisser d’anciens droits persistés.
- Le repli garde uniquement le sous-ensemble natif déjà admis, avec ses schémas canoniques et la limite du fournisseur ; les outils dynamiques inconnus, y compris MCP, ne sont pas conservés dans ce mode.
- La suite Rust `cargo test --lib extension` passe : **597 réussis, 0 échec, 10 ignorés**, en exécution séquentielle puis en exécution parallèle normale. Les dix échecs intermédiaires venaient de fixtures sans catalogue initialisé ; leur préparation explicite est limitée aux tests, sans nettoyage de dossiers appartenant aux autres fixtures. Le test du refus avant initialisation est isolé dans un processus.
- Le test de transport utilise le chemin Responses réel avec une réponse HTTP simulée : réception de « Bonjour », un seul outil natif transmis, cause retrouvée dans la session relue. La préparation Ollama est également couverte ; aucun appel payant ni essai sur les quatre conversations personnelles.
- Contrats d’extensions régénérés depuis le JSON canonique ; `npm run contracts:check`, TypeScript, lint et Clippy strict passent. Avertissements existants : attribut `deny_unknown_fields` ignoré par ts-rs et taille `__eh_frame` du binaire de tests.
- Notices françaises vues dans les thèmes sombre et clair avec le composant React réel, dans une page temporaire supprimée ensuite. Cette inspection ne constitue pas un test complet du binaire Tauri installé. Les sept traductions sont contrôlées automatiquement.
- `graphify update .` a réussi. La maintenance sémantique `graphify . --update` a été tentée et refusée faute de clé API configurée ; la documentation sémantique n’est donc pas déclarée à jour. Le plan reste un document explicite, même s’il est ignoré par défaut par le dépôt ; ajout ciblé forcé prévu.
- Les fichiers préexistants du site Beaver et les deux captures fast-mode restent hors du commit. La note Git constitue le compte rendu final des deux étapes de livraison ci-dessus ; elles sont encore à exécuter au moment où ce document est figé.
