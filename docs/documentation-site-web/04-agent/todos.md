# Liste de tâches

**Emplacement site** — Agent › Liste de tâches
**Répond à** — « C'est quoi cette liste qui apparaît quand l'agent travaille, et est-ce que je peux la piloter ? »
**Sources** — `src-tauri/src/services/agent_local/types_todo.rs`, `tool_todo.rs`, `tool_todo_state.rs` (ligne 9), `tool_todo_neglect.rs` (lignes 7-9), `tool_todo_summary.rs`, `tool_todo_delete.rs`, `tool_todo_parse.rs`
**Vérification** — Vérifié dans le code : statuts, limites et mécanisme de relance

---

## Plan de page proposé

1. À quoi ça sert
2. Ce que contient une tâche
3. Les séries de tâches
4. Ce que vous pouvez faire
5. Quand l'agent néglige sa liste
6. Les limites

---

## Contenu

### 1. À quoi ça sert

Sur un travail long, l'agent découpe lui-même la tâche en étapes et tient une liste visible en direct. Vous voyez ce qui est fait, ce qui est en cours et ce qui reste.

Double intérêt : vous suivez l'avancement sans relire toute la conversation, et l'agent garde le fil de ce qu'il doit faire au lieu de dériver.

L'outil de liste de tâches est **optionnel et désactivé par défaut**. À signaler : quelqu'un qui a vu cette fonction ailleurs ne la trouvera pas tant qu'il ne l'aura pas activée dans les réglages.

### 2. Ce que contient une tâche

Chaque tâche porte :

- un **intitulé** ;
- une **formulation en cours d'action**, employée pendant l'exécution ;
- un **statut** parmi trois : à faire, en cours, terminée.

### 3. Les séries de tâches

Les tâches sont groupées en **séries**. Une série correspond à un travail donné, avec son propre titre.

Une série a **trois états possibles** :

| État | Signification |
|---|---|
| Active | En cours de traitement |
| En pause | Suspendue, avec une raison enregistrée |
| Terminée | Toutes les tâches sont achevées |

Une série passe automatiquement en « terminée » quand toutes ses tâches le sont — et une série vide n'est jamais considérée comme terminée.

### 4. Ce que vous pouvez faire

Cinq outils composent la fonction :

| Outil | Rôle |
|---|---|
| `todo_write` | L'agent crée ou met à jour ses tâches |
| `todo_history` | Consulter les séries passées |
| `todo_pause` | Suspendre la série en cours, avec une raison |
| `todo_resume` | Reprendre une série suspendue |
| `todo_delete` | Supprimer une série |

Quatre d'entre eux — tous sauf `todo_write` — restent **disponibles en mode Plan**, ce qui permet de consulter et de piloter les listes pendant une phase d'exploration.

### 5. Quand l'agent néglige sa liste

Mécanisme peu visible mais utile à documenter, parce qu'il explique un comportement qu'on observe.

Quand une série est active et que l'agent n'avance pas dessus au fil des échanges :

- après **deux tours** sans progression, une relance lui est adressée ;
- après **quatre tours**, la série est **mise en pause automatiquement**, avec la mention d'une pause pour négligence.

L'intérêt : une liste abandonnée en plein milieu ne reste pas indéfiniment marquée comme active, ce qui fausserait la lecture de l'avancement.

### 6. Les limites

| Limite | Valeur |
|---|---|
| Séries conservées par conversation | **20** |
| Relance après | 2 tours sans progression |
| Mise en pause automatique après | 4 tours sans progression |

Au-delà de vingt séries, les plus anciennes sont écartées.

---

## Encadrés

**Encadré « Fonction à activer »**
> La liste de tâches est un outil optionnel, désactivé par défaut. Activez le groupe correspondant dans Réglages › Outils.

**Encadré « Pause automatique »**
> Si l'agent cesse d'avancer sur une série pendant quatre tours, Beaver la met en pause plutôt que de la laisser affichée comme active.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| Aucune liste n'apparaît | Outil désactivé par défaut | L'activer dans Réglages › Outils |
| Une série passe en pause seule | Quatre tours sans progression | La reprendre avec `todo_resume` |
| L'agent ne crée pas de liste | Tâche jugée trop courte, ou outil désactivé | Le lui demander explicitement |
| Une ancienne série a disparu | Vingt séries conservées par conversation | Normal |
| L'agent ne peut pas écrire ses tâches | Mode Plan actif : `todo_write` y est interdit | Approuver le plan d'abord |

---

## Renvois

- *Agent › Mode Plan* — ce qui reste permis
- *Outils › Vue d'ensemble* — activer le groupe
- *Agent › Sous-agents* — suivre un travail délégué

---

## Points à confirmer

- **Ce que voit l'utilisateur.** Emplacement de la liste dans l'interface, aspect, mise à jour en direct. Non relevé — c'est pourtant l'essentiel d'une page sur un affichage.
- **Les cinq outils sont-ils pilotables par l'utilisateur**, ou uniquement appelés par l'agent ? Le vocabulaire de la page en dépend : « vous pouvez suspendre » n'a de sens que s'il existe un bouton.
- **La longueur maximale d'un intitulé de tâche.** Une limite existe pour la raison de pause ; celle des intitulés n'a pas été relevée.
- **Ce qu'est exactement un « tour »** dans le compteur de négligence — un message de l'utilisateur, ou un cycle complet de l'agent ?
- **La consultation de l'historique** depuis l'interface, et non seulement par l'outil.
