# Cloner une conversation

**Emplacement site** — Agent › Cloner une conversation
**Répond à** — « Je veux repartir d'un point précis de la conversation sans perdre ce qui a été appris. Comment ? »
**Sources** — `src-tauri/src/services/agent_local/clone_session.rs` (lignes 13, 55-160), `clone_session_build.rs`, `clone_summary.rs` (lignes 4-8), `clone_roots.rs` (ligne 4), `types_session.rs` (lignes 35-38, 121-133), `session_tabs_state.rs` (ligne 6), `src-tauri/src/commands/agent_clone.rs`
**Vérification** — Vérifié dans le code : les deux modes, ce qui est copié, ce qui est réinitialisé, les limites et le comportement en cas d'échec

---

## Le mécanisme en une phrase

Cloner crée **une copie de la conversation arrêtée à un message précis**, et l'ouvre dans un onglet à côté de l'originale.

Ce n'est ni une copie complète, ni un simple résumé : c'est une **reprise à un point choisi**.

---

## Plan de page proposé

1. À quoi ça sert
2. Où se trouve le bouton
3. Les deux modes
4. Ce que contient le clone
5. Les onglets
6. Le lien avec Git
7. Les limites
8. Cloner un clone

---

## Contenu

### 1. À quoi ça sert

Reprendre une conversation à partir d'un moment précis, quand la suite n'a pas donné ce qu'on voulait.

Cas typique : l'agent part sur une mauvaise piste au dixième message et s'y enfonce pendant vingt messages. Plutôt que de tout recommencer, on repart du dixième — en gardant, si on veut, la mémoire de ce qui n'a pas marché.

### 2. Où se trouve le bouton

**Sous chaque message de la conversation, sauf le dernier.**

Le dernier message est exclu parce qu'il n'y a rien après lui : cloner à cet endroit reviendrait à dupliquer la conversation entière.

### 3. Les deux modes

C'est le cœur de la page. Les deux modes copient la conversation jusqu'au message choisi ; ils diffèrent sur le traitement de **ce qui suivait**.

**Mode « couper »**
- Tout ce qui suit le message choisi est **abandonné**.
- Le clone démarre exactement dans l'état où était la conversation à ce moment-là.
- Aucun modèle n'est appelé : la création est immédiate.
- À utiliser quand la suite n'a rien apporté.

**Mode « résumer »**
- Tout ce qui suit le message choisi est **résumé par un modèle**, et le résumé est ajouté au clone sous forme de contexte caché.
- Le résumé retient les **erreurs rencontrées et à ne pas reproduire**, les **décisions prises** et l'**état d'avancement**.
- Les **fichiers lus** et les **fichiers modifiés** pendant la partie résumée sont relevés et attachés au clone.
- Un **axe de résumé** peut être précisé pour orienter ce sur quoi le modèle doit se concentrer.
- La génération peut être **annulée** en cours.
- À utiliser quand la suite a échoué mais a appris quelque chose.

Le mode « résumer » est **refusé s'il n'y a rien après le message choisi** — le résumé serait vide.

### 4. Ce que contient le clone

**Repris de l'originale** : tous les messages jusqu'au message choisi inclus, le modèle, le fournisseur, les réglages de la conversation.

**Réinitialisé** :
- le nom devient « Clone - » suivi du nom de l'originale ;
- les dates de création et de modification ;
- l'état d'archivage ;
- l'historique des échecs de flux et des diagnostics ;
- **la branche Git** — un clone démarre sans branche associée ;
- tous les attributs de sous-agent.

**Ajouté en mode « résumer »** : un message de contexte caché contenant le résumé.

**Ce qui n'est pas clonable** : les conversations de sous-agents. Le clonage est refusé sur elles.

### 5. Les onglets

Point important, car l'interface a changé : **les conversations ne s'ouvrent pas en onglets.** On navigue d'une conversation à l'autre par la barre latérale.

**Les onglets n'existent que pour les clones.** Une conversation et ses clones forment un groupe d'onglets.

- **Trois onglets au maximum par groupe** — l'originale plus deux clones.
- Un onglet peut être **renommé**.
- Un onglet peut être **fermé**, avec une option de nettoyage de la branche Git associée.

Chaque clone garde deux références : son **parent immédiat** et la **racine du groupe**. C'est ce qui permet de cloner un clone tout en gardant le groupe cohérent.

### 6. Le lien avec Git

Un clone démarre **sans branche Git**. Trois actions sont possibles ensuite :

- **créer** une branche liée au clone ;
- **lier** une branche existante ;
- **délier** la branche.

À la fermeture d'un onglet, une commande dédiée permet de nettoyer la branche associée et de revenir à une branche de repli.

Le dépôt doit être un **projet enregistré** : les chemins non autorisés sont refusés.

### 7. Les limites

| Limite | Valeur |
|---|---|
| Onglets par groupe | **3** (l'originale + 2 clones) |
| Jetons du résumé | **3 072** |
| Contenu analysé pour le résumé | **120 000 caractères** |
| Résultat d'outil retenu | **2 000 caractères** |
| Fichiers suivis | **200** |
| Ancêtres dans une chaîne de clones | **64** |
| Délai de génération du résumé (modèle local) | **180 secondes** |

Le résumé est généré par **le modèle de la conversation clonée**, pas par un modèle dédié. Un modèle local lent allonge donc l'opération.

**En cas d'échec du résumé**, le clone est supprimé et son onglet retiré : on ne se retrouve pas avec une conversation à moitié construite.

### 8. Cloner un clone

C'est autorisé, et c'est même l'usage prévu : on peut réitérer sur une branche en **accumulant la mémoire des erreurs évitées**. Chaque clone en mode « résumer » ajoute son propre contexte caché, ce qui donne un résumé cumulatif au fil des tentatives.

La chaîne est bornée à 64 ancêtres.

---

## Tableaux

### Tableau — Les deux modes

| | Couper | Résumer |
|---|---|---|
| Ce qui suit le message | Abandonné | Résumé par un modèle |
| Appel à un modèle | Non | Oui |
| Durée | Immédiat | Dépend du modèle |
| Erreurs passées conservées | Non | Oui |
| Fichiers touchés relevés | Non | Oui |
| Axe de résumé personnalisable | — | Oui |
| Possible sur le dernier message | — | Non, refusé |
| Annulable | — | Oui |

### Tableau — Clone et sous-agent

| | Cloner | Déléguer à un sous-agent |
|---|---|---|
| Qui décide | Vous | L'agent, ou vous |
| Point de départ | Un message précis de la conversation | Des instructions neuves |
| Résultat | Une conversation que vous menez, dans un onglet | Un travail rendu à l'agent parent |
| Clonable | Oui | Non |

---

## Encadrés

**Encadré « L'originale n'est pas modifiée »**
> Cloner crée une nouvelle conversation. L'originale reste intacte, dans son propre onglet.

**Encadré « Deux modes, deux usages »**
> Utilisez « couper » quand la suite n'a rien apporté. Utilisez « résumer » quand elle a échoué mais que l'agent y a appris quelque chose — les erreurs rencontrées seront transmises pour ne pas être répétées.

**Encadré « Trois onglets au maximum »**
> Une conversation et ses clones forment un groupe de trois onglets au plus. Fermez-en un pour en créer un nouveau.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| Pas de bouton de clonage sous le dernier message | Volontaire : rien à cloner après lui | Cloner depuis un message antérieur |
| Le mode « résumer » est refusé | Aucun message après celui choisi | Utiliser « couper », ou choisir un message antérieur |
| Impossible de cloner une conversation | C'est une conversation de sous-agent | Non clonable |
| Impossible de créer un onglet de plus | Trois onglets déjà ouverts dans le groupe | Fermer un onglet |
| Le clonage prend du temps | Le résumé est généré par le modèle de la conversation | Annuler, ou utiliser le mode « couper » |
| Le clone a disparu après une erreur | Nettoyage volontaire en cas d'échec du résumé | Réessayer |
| Le clone n'est sur aucune branche Git | Un clone démarre sans branche | Créer ou lier une branche |

---

## Renvois

- *Interface › Conversations et sessions*
- *Agent › Sous-agents*
- *Agent › Contexte et compression*
- *Automatisation › Workflow Git*

---

## Points à confirmer

- **Le vocabulaire.** Trois termes circulent : *clone* dans le code, *fork* à l'oral, *brancher* dans le mockup. Fixer un seul mot pour l'interface et la documentation. « Cloner » a l'avantage de coller au code et aux noms de commandes ; « forker » est plus parlant pour qui vient de Git, mais entre en collision avec les branches Git déjà présentes dans le produit.
- **Les libellés des deux modes dans l'interface.** Le code les nomme `Cut` et `Summary`. Les libellés français affichés n'ont pas été relevés.
- **Le résumé est-il consultable ?** Il est stocké dans la conversation et injecté comme message caché. Vérifier si l'utilisateur peut le lire — il oriente toute la suite de la conversation.
- **Le champ d'axe de résumé** — comment il est présenté, et ce qu'on est censé y écrire.
- **Le nom automatique « Clone - … ».** Vérifier ce que ça donne après plusieurs clones successifs, et si l'onglet peut être renommé dès la création.
- **Le comportement du nettoyage de branche Git** à la fermeture d'un onglet : ce qui est supprimé, et le rôle de la branche de repli.
