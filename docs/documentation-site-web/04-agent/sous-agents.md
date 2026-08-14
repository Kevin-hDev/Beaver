# Sous-agents

**Emplacement site** — Agent › Sous-agents
**Répond à** — « L'agent peut-il déléguer, et comment je garde le contrôle de ce que font ses enfants ? »
**Sources** — `src-tauri/src/services/agent_local/tool_catalog.rs` (lignes 17-27), `subagent_instruction_delivery.rs` (lignes 5-7), `types_subagent_change.rs` (lignes 4-22), `subagent_change_store.rs` (lignes 6-7), `subagent_directory_limits.rs` (lignes 4-6), `subagent_archive.rs`, `subagent_cancellation.rs`, `subagent_directory_git.rs`, `permission_policy.rs`
**Vérification** — Vérifié dans le code : les neuf outils, les limites, les états et les deux types d'espace de travail

---

## Le point de sécurité à ne pas manquer

**Les sous-agents ne demandent pas de confirmation.** Ils s'exécutent dans un mode interne qui contourne la garde de permission, exactement comme le mode Accès complet.

Quelqu'un qui travaille en Demande d'approbation et délègue une tâche doit le savoir : ce qu'il délègue échappe à ses confirmations.

Le contrôle existe, mais il se situe ailleurs — **en amont**, au moment de décider quoi déléguer, et **en aval**, en inspectant les changements avant de les appliquer. C'est ce déplacement qu'il faut expliquer, pas le masquer.

---

## Plan de page proposé

1. À quoi ça sert
2. Les neuf outils
3. L'isolation : deux types d'espace de travail
4. Suivre et corriger un sous-agent en cours
5. Inspecter et appliquer les changements
6. Les limites
7. Ce qu'un sous-agent ne peut pas faire

---

## Contenu

### 1. À quoi ça sert

L'agent principal confie une partie du travail à une session enfant, qui dispose de son propre contexte et travaille de son côté.

Deux bénéfices :

- **le contexte du parent reste léger** — l'exploration fastidieuse a lieu chez l'enfant, seul le résultat remonte ;
- **le travail est isolé** — l'enfant modifie des fichiers dans un espace séparé, pas directement dans votre dossier.

### 2. Les neuf outils

| Outil | Rôle |
|---|---|
| `delegate_task` | Lancer un sous-agent sur une tâche |
| `list_subagents` | Lister les sous-agents et leur état |
| `get_subagent` | Consulter le détail de l'un d'eux |
| `message_subagent` | Lui envoyer une consigne en cours de route |
| `cancel_subagent` | L'interrompre |
| `archive_subagent` | L'archiver une fois terminé |
| `inspect_subagent_changes` | Voir ce qu'il a modifié |
| `apply_subagent_changes` | Appliquer ses changements |
| `discard_subagent_changes` | Les jeter |

Le groupe est **actif par défaut**.

**`apply_subagent_changes` est soumis à confirmation** en mode Demande d'approbation. C'est le point de contrôle central : l'enfant travaille librement, mais l'intégration de son travail passe par vous.

### 3. L'isolation : deux types d'espace de travail

Un sous-agent travaille dans un espace séparé, de deux natures possibles :

| Type | Mécanisme |
|---|---|
| **Git** | Une copie de travail Git séparée, dans `subagent-worktrees/` |
| **Dossier** | Une copie du dossier, quand le projet n'est pas un dépôt Git |

Le second cas est borné, parce que copier un dossier peut coûter cher :

| Limite de l'instantané | Valeur |
|---|---|
| Fichiers | **20 000** |
| Taille totale | **512 Mo** |
| Taille d'un fichier | **64 Mo** |

Au-delà, l'instantané est refusé. À documenter : quelqu'un dont le projet contient un dossier de dépendances volumineux comprendra pourquoi la délégation échoue.

### 4. Suivre et corriger un sous-agent en cours

- L'état de chaque sous-agent est visible en direct.
- Une **consigne peut lui être envoyée pendant qu'il travaille** — pour le corriger, le réorienter, lui donner une précision.
- Les consignes sont mises en file : **huit au maximum** en attente.
- Une consigne fait **50 000 caractères** au plus.
- Un sous-agent peut être **interrompu** à tout moment.

Un onglet de conversation peut **réclamer l'attention** quand un sous-agent attend une réponse.

### 5. Inspecter et appliquer les changements

Chaque lot de changements passe par **quatre états** :

| État | Signification |
|---|---|
| En attente | Le sous-agent a terminé, rien n'est appliqué |
| Conflit | Les changements entrent en conflit avec l'état actuel |
| Appliqué | Intégrés à votre dossier |
| Écarté | Jetés |

Avant d'appliquer, vous voyez **la liste des fichiers touchés**, avec la nature de chaque modification.

| Limite | Valeur |
|---|---|
| Chemins modifiés par lot | **128** |
| Longueur d'un chemin | **512 caractères** |
| Lots conservés | **256** |
| Taille d'un descriptif de lot | **128 Ko** |

L'état « conflit » mérite une explication : il survient quand votre dossier a changé pendant que le sous-agent travaillait. Beaver refuse alors d'appliquer aveuglément plutôt que d'écraser votre travail.

### 6. Les limites

Récapitulatif en section Tableaux.

### 7. Combien de sous-agents à la fois

Deux plafonds, tous deux vérifiés dans le code :

| Plafond | Valeur | Message affiché |
|---|---|---|
| Sous-agents actifs **par session** | **4** | « Limite de 4 sous-agents par session atteinte » |
| Sous-agents actifs **au total**, toutes sessions confondues | **8** | « Limite de 8 sous-agents actifs atteinte » |

Le second est le plus surprenant : deux conversations qui délèguent en même temps se partagent le même budget global de huit.

### 8. Ce qu'un sous-agent ne peut pas faire

**Déléguer à son tour.** La délégation en cascade est explicitement interdite : un sous-agent qui tente de lancer un sous-agent reçoit le refus « Les sous-agents ne peuvent pas lancer d'autres sous-agents. »

L'arborescence est donc **plate, sur un seul niveau** : votre conversation, et ses enfants. Jamais de petits-enfants.

C'est un choix de conception à expliquer plutôt qu'à subir : la délégation en cascade rend le suivi impossible et la consommation de jetons imprévisible.

**Autres restrictions :**

- **Être cloné** — le clonage est refusé sur une conversation de sous-agent.
- **Écrire dans la mémoire** — un sous-agent peut la lire, pas la modifier. Le contrôle est explicite dans le code : sur les fichiers de mémoire, seule la lecture passe.
- **Disposer de tous les outils** — chaque sous-agent reçoit un **profil d'outils restreint** selon son type. Un outil hors profil est refusé.
- **Être archivé dans n'importe quel état** — l'archivage passe par un mécanisme dédié qui refuse certains cas.

**Un type particulier** : le sous-agent de type `coder` exige d'être lancé depuis un dossier valide, sans quoi la délégation est refusée avec « Un sous-agent code doit être lancé depuis un dossier valide. »

---

## Tableaux

### Tableau — Toutes les limites

| Limite | Valeur |
|---|---|
| Taille d'une consigne | 50 000 caractères |
| Consignes en file d'attente | 8 |
| Chemins modifiés par lot | 128 |
| Longueur d'un chemin | 512 caractères |
| Lots de changements conservés | 256 |
| Descriptif d'un lot | 128 Ko |
| Fichiers dans un instantané de dossier | 20 000 |
| Taille d'un instantané | 512 Mo |
| Taille d'un fichier dans l'instantané | 64 Mo |
| **Sous-agents actifs par session** | **4** |
| **Sous-agents actifs au total** | **8** |
| Profondeur de délégation | 1 niveau — la cascade est interdite |

### Tableau — Sous-agent et clone

| | Sous-agent | Clone |
|---|---|---|
| Qui le lance | L'agent, ou vous | Vous |
| Point de départ | Des instructions neuves | Un message précis d'une conversation |
| Espace de travail | Isolé, copie séparée | Le même que l'originale |
| Résultat | Rendu à l'agent parent | Une conversation que vous menez |
| Demande de confirmation | Non | Selon votre mode |
| Clonable | Non | Oui |

---

## Encadrés

**Encadré « Les sous-agents ne demandent pas de confirmation »** — avertissement, en tête de page.
> Une session déléguée s'exécute sans demander d'approbation, même si votre conversation est en Demande d'approbation. Le contrôle se fait autrement : ses changements sont isolés, et vous les inspectez avant de les appliquer.

**Encadré « Rien n'est appliqué sans vous »**
> Un sous-agent modifie des fichiers dans un espace séparé. Ses changements n'atteignent votre dossier que lorsque vous les appliquez.

**Encadré « Conflit »**
> Si votre dossier a changé pendant que le sous-agent travaillait, ses changements passent en conflit. Beaver refuse de les appliquer aveuglément plutôt que d'écraser votre travail.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| Un sous-agent modifie des fichiers sans rien demander | Les sous-agents contournent la garde de permission | Inspecter ses changements avant de les appliquer |
| La délégation échoue sur un gros projet | Instantané au-delà de 20 000 fichiers ou 512 Mo | Exclure les dossiers volumineux, ou travailler dans un dépôt Git |
| Les changements sont en conflit | Le dossier a changé pendant le travail du sous-agent | Inspecter, puis appliquer ou jeter |
| Impossible d'envoyer une consigne de plus | Huit consignes déjà en file | Attendre qu'il en traite |
| Impossible de cloner un sous-agent | Volontaire | Cloner la conversation parente |
| « Limite de 4 sous-agents par session atteinte » | Quatre enfants actifs pour cette conversation | Attendre qu'un termine, ou en interrompre un |
| « Limite de 8 sous-agents actifs atteinte » | Huit enfants actifs toutes conversations confondues | Vérifier les autres conversations : le plafond est global |
| « Les sous-agents ne peuvent pas lancer d'autres sous-agents » | La délégation en cascade est interdite | Faire déléguer par la conversation principale |
| « Un sous-agent code doit être lancé depuis un dossier valide » | Type `coder` sans répertoire de travail utilisable | Définir un répertoire de travail valide |
| Un sous-agent ne peut pas écrire en mémoire | Accès en lecture seule | Normal : il peut proposer une note, pas l'écrire |
| Un sous-agent refuse d'être archivé | État incompatible | Attendre qu'il termine, ou l'interrompre |
| Un ancien lot de changements a disparu | 256 lots conservés | Normal |

---

## Renvois

- *Agent › Permissions* — pourquoi les sous-agents ne demandent pas
- *Interface › Cloner une conversation* — l'autre façon de paralléliser
- *Outils › Sous-agents* — le détail des neuf outils
- *Automatisation › Workflow Git* — les copies de travail
- *Référence › Stockage local* — où vivent les espaces isolés

---

## Points à confirmer

- ~~Un sous-agent peut-il lui-même déléguer ?~~ **Tranché** : non, la cascade est interdite explicitement (`tool_delegate.rs:55-60`).
- ~~Combien de sous-agents simultanément ?~~ **Tranché** : 4 par session, 8 au total.
- **La portée d'accès disque d'un sous-agent** — celle du parent, ou restreinte à son espace isolé ? Question de sécurité, à trancher avant publication.
- **Le modèle employé par un sous-agent** — celui du parent, ou configurable à la délégation ?
- **La liste complète des types de sous-agents et de leurs profils d'outils.** Un type `coder` est identifié, avec sa contrainte propre. Les autres types et le contenu exact de chaque profil restent à établir — c'est une information utile : savoir qu'un sous-agent n'a pas accès à tel outil évite de lui confier une tâche qu'il ne peut pas faire.
- **Ce que voit l'utilisateur** : où s'affiche la liste des sous-agents, leur état, l'écran d'inspection des changements. Non relevé.
- **Le nettoyage des espaces isolés** après application ou abandon — automatique, différé, manuel ?
