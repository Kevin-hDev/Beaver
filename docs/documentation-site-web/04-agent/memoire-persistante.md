# Mémoire persistante

**Emplacement site** — Agent › Mémoire (le mockup la regroupe avec les skills ; elle mérite sa page)
**Répond à** — « Est-ce que l'agent se souvient d'une conversation à l'autre, et qu'est-ce qu'il retient exactement ? »
**Sources** — `src-tauri/src/services/agent_local/memory_types.rs` (lignes 3-8, 12-16), `memory_format.rs` (lignes 8-10), `memory_store.rs`, `memory_paths.rs`, `memory_settings.rs`, `memory_prompt.rs`, `memory_tool.rs`, `memory_project_id.rs`, `memory_overview.rs`, `src/components/settings/memory-settings.tsx`
**Vérification** — Vérifié dans le code : modes, portées, limites, structure d'une note

---

## Plan de page proposé

1. À quoi ça sert
2. Les trois modes
3. Les deux portées
4. Ce que contient une note
5. Ce qui est injecté dans la conversation
6. Les limites
7. Consulter et corriger
8. Les sous-agents et la mémoire

---

## Contenu

### 1. À quoi ça sert

L'agent conserve des notes d'une conversation à l'autre : vos préférences, les conventions d'un projet, des décisions prises, des références utiles.

Sans mémoire, chaque conversation repart de zéro et il faut répéter les mêmes consignes. Avec, l'agent sait déjà comment vous travaillez.

**À distinguer du contexte** : le contexte est ce que le modèle voit dans la conversation en cours ; la mémoire survit à la fermeture de la conversation. C'est la confusion la plus fréquente.

### 2. Les trois modes

| Mode | Comportement |
|---|---|
| **Désactivée** | Aucune mémoire n'est lue ni écrite |
| **Manuelle** | L'agent lit la mémoire, mais n'écrit que si vous le demandez |
| **Automatique** | L'agent lit et écrit de lui-même ce qu'il juge utile |

Le mode par défaut est **Désactivée**. À signaler : quelqu'un qui attend que l'agent se souvienne doit d'abord activer la fonction.

Le choix entre Manuelle et Automatique mérite un conseil plutôt qu'une description neutre :

> En mode Automatique, l'agent décide seul de ce qui mérite d'être retenu. C'est confortable, mais il retiendra parfois des choses que vous n'auriez pas notées. En mode Manuelle, rien n'est écrit sans votre demande — vous gardez la main sur ce qui s'accumule.

### 3. Les deux portées

| Portée | Ce qu'elle contient | Emplacement |
|---|---|---|
| **Globale** | Ce qui vaut partout : vos préférences, votre façon de travailler | `memory/global/` |
| **Par projet** | Ce qui ne vaut que pour un dossier : conventions, commandes, contraintes | `memory/projects/` |

L'interface distingue trois vues : la mémoire globale, celle du **projet actif**, et celle des **autres projets** — utile pour retrouver ce qui a été noté ailleurs.

### 4. Ce que contient une note

Les notes sont rangées par **sujet**, en Markdown. Chacune porte des attributs structurés :

**Type** — quatre valeurs :

| Type | Usage |
|---|---|
| `preference` | Une préférence de l'utilisateur |
| `feedback` | Une consigne sur la façon de travailler |
| `project` | Un élément du projet en cours |
| `reference` | Un pointeur vers une ressource externe |

**Statut** — quatre valeurs : `confirmed` (confirmé par l'utilisateur), `inferred` (déduit par l'agent), `stale` (probablement périmé), `archived`.

**Source** — quatre valeurs : `user`, `parent`, `extractor`, `subagent-suggestion`.

Ces attributs ne sont pas décoratifs. La distinction **confirmé / déduit** est celle qui compte pour l'utilisateur : elle sépare ce qu'il a dit de ce que l'agent a supposé. Une page qui l'explique permet de faire le tri dans sa mémoire.

**Étiquettes** — huit au maximum par note.

### 5. Ce qui est injecté dans la conversation

La mémoire ne rentre pas en entier dans le contexte. Un **budget** limite ce qui est injecté :

| | Valeur |
|---|---|
| Budget par défaut | **3 000 jetons** |
| Budget maximum | **3 000 jetons** |
| Budget minimum | **256 jetons** |

Le budget est réglable dans cet intervalle. Au-delà de ce que le budget permet, un résumé prend le relais.

### 6. Les limites

| Limite | Valeur |
|---|---|
| Sujets par portée | **256** |
| Taille d'un sujet | **48 Ko** |
| Taille d'un résumé de portée | **16 Ko** |
| Étiquettes par note | **8** |
| Jetons injectés | 256 à 3 000 |

### 7. Consulter et corriger

Tout est consultable et modifiable depuis **Réglages › Mémoire** : les notes, leur contenu, leur portée.

C'est le point à mettre en avant. Une mémoire qu'on ne peut pas relire est une boîte noire ; une mémoire qu'on relit, c'est un fichier de notes qu'on corrige quand l'agent a mal compris.

Les fichiers sont en Markdown sur le disque : ils restent lisibles avec n'importe quel éditeur.

### 8. Les sous-agents et la mémoire

Les sous-agents ont un accès **en lecture seule**. Ils bénéficient de ce qui a été retenu, mais n'écrivent pas dans la mémoire commune.

Une source de note s'appelle d'ailleurs `subagent-suggestion` : un sous-agent peut **proposer** une note, sans l'imposer.

---

## Encadrés

**Encadré « Mémoire ou contexte ? »**
> Le contexte est ce que le modèle voit dans la conversation en cours. La mémoire survit à sa fermeture. Une conversation qui sature son contexte ne perd pas la mémoire.

**Encadré « Désactivée par défaut »**
> La mémoire est désactivée à l'installation. Activez-la dans Réglages › Mémoire, en choisissant le mode Manuelle ou Automatique.

**Encadré « Confirmé ou déduit »**
> Chaque note indique si elle vient de vous ou si l'agent l'a supposée. En cas de comportement inattendu, commencez par relire les notes déduites.

**Encadré « Vos notes vous appartiennent »**
> La mémoire est stockée en Markdown dans votre dossier de données. Vous pouvez la relire, la corriger et la supprimer, depuis Beaver ou avec n'importe quel éditeur.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| L'agent ne se souvient de rien | Mémoire désactivée | L'activer dans Réglages › Mémoire |
| L'agent retient des choses non souhaitées | Mode Automatique | Passer en Manuelle, et supprimer les notes concernées |
| Une note est fausse | Note déduite plutôt que confirmée | La corriger ou la supprimer dans les réglages |
| Une note semble ignorée | Budget d'injection atteint | Augmenter le budget, ou alléger la mémoire |
| Impossible d'ajouter un sujet | 256 sujets pour cette portée | Regrouper ou supprimer |
| Un sujet est refusé | Plus de 48 Ko | Le scinder |
| La mémoire d'un projet ne suit pas | Elle est liée au dossier | Vérifier le répertoire de travail de la conversation |

---

## Renvois

- *Agent › Contexte* — la différence, et le budget
- *Agent › Personnalité et AGENTS.md* — l'autre façon de donner des instructions permanentes
- *Agent › Sous-agents* — l'accès en lecture seule
- *Réglages › Agent*
- *Référence › Stockage local*

---

## Points à confirmer

- **Comment un projet est identifié.** Un fichier est dédié au calcul de l'identifiant de projet, et un autre à la migration des mémoires de projet. Comprendre ce qui se passe quand un dossier est déplacé ou renommé : la mémoire suit-elle ?
- **Ce qui déclenche une écriture en mode Automatique.** Un extracteur existe comme source de notes. Savoir à quel moment il s'exécute change la façon de présenter le mode.
- **Le mécanisme de résumé de portée.** Un résumé de 16 Ko existe par portée. Vérifier quand il est produit et ce qu'il remplace.
- **Les notes marquées « périmées »** — qui les marque, et que deviennent-elles ?
- **L'archivage des notes** — accessible depuis l'interface ?
- **Le comportement quand la mémoire est désactivée puis réactivée** : les notes existantes sont-elles conservées ?
- **La suggestion d'un sous-agent** : où apparaît-elle, et comment on l'accepte ou la refuse ?
