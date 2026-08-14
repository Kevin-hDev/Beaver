# Répertoire de travail et accès disque

**Emplacement site** — Agent › Répertoire de travail
**Répond à** — « Où l'agent a-t-il le droit de lire et d'écrire, et comment je restreins ça ? »
**Sources** — `src-tauri/src/services/agent_local/directory_access.rs` (lignes 7-10), `directory_policy.rs`, `directory_access_scope.rs`, `agent_working_dir.rs`, `project_store.rs`, `src/components/settings/file-access-settings.tsx`, `path-list-editor.tsx`, `src-tauri/src/commands/directory_access.rs`
**Vérification** — Vérifié dans le code : limites, messages d'erreur, mécanisme de validation et racines par défaut (`src-tauri/src/models/config.rs`, `default_allowed_paths()`)

---

## Plan de page proposé

1. Le répertoire de travail d'une conversation
2. Les projets
3. La portée d'accès disque
4. Comment un chemin est validé
5. Les limites
6. Le réglage par défaut

---

## Contenu

### 1. Le répertoire de travail d'une conversation

Chaque conversation est rattachée à un dossier. Ce dossier détermine :

- ce que montre l'arbre de fichiers ;
- où démarre le terminal ;
- le dépôt Git sur lequel portent les actions de branche ;
- le contexte que l'agent considère comme « le projet ».

Il est **propre à chaque conversation** : deux conversations ouvertes peuvent travailler sur deux dossiers différents.

### 2. Les projets

Un **projet** est un répertoire de travail enregistré, qu'on retrouve d'une conversation à l'autre sans avoir à le resélectionner. Les projets sont stockés dans `projects.json`.

Point de sécurité à mentionner : certaines opérations — notamment les actions Git liées aux clones — **exigent que le dossier soit un projet enregistré**. Un chemin arbitraire est refusé.

### 3. La portée d'accès disque

Une liste de dossiers autorisés se règle dans **Réglages › Accès aux fichiers**. Elle borne ce que l'agent peut atteindre, indépendamment du répertoire de travail.

Le message affiché en cas de refus est : **« Accès au dossier refusé par les réglages. »**

Ce réglage est le garde-fou principal du produit sur la question de l'accès disque. Il mérite une explication franche : par défaut, l'agent d'une application de bureau a la portée que vous lui donnez, et c'est ici qu'on la donne.

### 4. Comment un chemin est validé

Avant toute lecture ou écriture, un chemin est :

- **normalisé** — les liens et les raccourcis sont résolus jusqu'au chemin réel ;
- **comparé aux racines autorisées** — il doit se trouver à l'intérieur de l'une d'elles.

C'est cette résolution préalable qui empêche de sortir de la zone autorisée par un chemin détourné : un lien symbolique pointant hors des racines est démasqué avant l'accès, pas après.

### 5. Les limites

| Limite | Valeur |
|---|---|
| Dossiers autorisés configurables | **70** |
| Racines d'espace de travail | **73** (les 70 autorisés, plus 3 internes) |
| Longueur d'un chemin | **4 096 caractères** |

Les trois racines supplémentaires correspondent aux emplacements dont l'application a besoin pour fonctionner. Voir *Points à confirmer*.

### 6. Le réglage par défaut

**Par défaut, l'agent a accès à l'intégralité du disque.**

| Système | Racine autorisée par défaut |
|---|---|
| macOS | `/` |
| Linux | `/` |
| Windows | `C:\` |

C'est un **choix assumé du produit**, pas un oubli : l'agent est conçu pour travailler sur n'importe quel dossier de la machine sans configuration préalable, et la protection repose sur les autres mécanismes — la garde de permission, les chemins sensibles toujours soumis à confirmation, et le mode Plan.

**Comment le formuler sur le site.** Ni le cacher, ni en faire un épouvantail. Le ton juste :

> Par défaut, Beaver peut atteindre tous vos fichiers. C'est ce qui lui permet de travailler sur n'importe quel projet sans réglage préalable. Si vous préférez le restreindre à quelques dossiers, la liste se modifie dans Réglages › Accès aux fichiers — et cette restriction s'applique à toutes vos conversations.

Puis renvoyer vers les mécanismes qui restent actifs quelle que soit la portée : chemins sensibles, mode de permission, mode Plan.

Le réglage s'applique dès qu'une liste personnalisée est enregistrée. Une liste vide est ramenée au défaut : on ne peut pas se retrouver sans aucune racine autorisée par accident.

---

## Encadrés

**Encadré « L'accès est complet par défaut »** — à placer en section 6.
> Par défaut, Beaver peut atteindre tous vos fichiers. C'est ce qui lui permet de travailler sur n'importe quel projet sans réglage préalable. Pour le restreindre, modifiez la liste dans Réglages › Accès aux fichiers.

**Encadré « Répertoire de travail et portée d'accès sont deux choses »**
> Le répertoire de travail est le dossier sur lequel porte une conversation. La portée d'accès est la liste des dossiers que l'agent peut atteindre, toutes conversations confondues.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| « Accès au dossier refusé par les réglages » | Chemin hors des dossiers autorisés | Ajouter le dossier dans Réglages › Accès aux fichiers |
| Une action Git est refusée sur un dossier valide | L'opération exige un projet enregistré | Enregistrer le dossier comme projet |
| Un lien symbolique ne donne pas accès à sa cible | Le chemin réel est résolu avant vérification | Ajouter la cible aux dossiers autorisés |
| Impossible d'ajouter un dossier de plus | Soixante-dix au maximum | Regrouper en dossiers parents |
| L'arbre de fichiers est vide | Répertoire de travail non défini ou inaccessible | Le définir dans la conversation |

---

## Renvois

- *Agent › Permissions* — les chemins sensibles refusés même dans la zone autorisée
- *Sécurité › Accès aux fichiers* — le détail du modèle
- *Interface › Arbre de fichiers*
- *Interface › Conversations* — ce qui est propre à chaque conversation
- *Réglages › Application*

---

## Points à confirmer

- ~~La valeur par défaut de la liste de dossiers autorisés.~~ **Tranché** : `/` sur macOS et Linux, `C:\` sur Windows — accès complet au disque, choix assumé du produit.
- **Les trois racines internes** ajoutées aux soixante-dix configurables : lesquelles, et pourquoi.
- **Comment se définit le répertoire de travail d'une conversation** : hérité du projet, choisi à la création, modifiable en cours ? Non relevé.
- **Ce qui se passe quand on change le répertoire de travail** d'une conversation en cours : terminal, arbre de fichiers, branche Git suivent-ils ?
- **L'interaction avec les chemins sensibles.** Un fichier `.env` situé dans un dossier autorisé reste soumis à confirmation via la garde de permission. Vérifier que les deux mécanismes se cumulent bien, et l'expliquer.
- **La portée d'accès des sous-agents** — même liste que le parent, ou restreinte à leur copie de travail ?
