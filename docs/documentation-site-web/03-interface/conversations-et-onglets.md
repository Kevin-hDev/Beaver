# Conversations et navigation

**Emplacement site** — Interface › Conversations
**Répond à** — « Comment je passe d'une conversation à l'autre, et comment je les organise ? »
**Sources** — `src-tauri/src/commands/agent_sessions.rs`, `src-tauri/src/services/agent_local/session_tabs.rs`, `session_tabs_state.rs` (ligne 6), `types_session.rs`, `src/hooks/use-session-tabs.ts`, `use-archived-agent-sessions.ts`, `src/components/layout/search-dialog.tsx`, `CLAUDE.md`
**Vérification** — Vérifié dans le code pour la navigation, les onglets, l'archivage et les limites ; plusieurs gestes d'interface restent à relever

---

## Correction importante par rapport au mockup

**Les conversations ne s'ouvrent plus en onglets.** Le multi-onglet de conversations a été retiré du produit.

La navigation se fait **uniquement par la barre latérale**. Les seuls onglets qui subsistent dans une conversation sont ceux créés par le **clonage** : une conversation et ses clones forment un groupe de **trois onglets au maximum**.

Toute formulation évoquant « ouvrir plusieurs conversations en onglets » est fausse. Ne pas la reprendre du mockup ni d'une capture ancienne.

---

## Plan de page proposé

1. Une conversation, ce que c'est
2. Naviguer entre les conversations
3. Les onglets de clones
4. Archiver
5. La recherche
6. Ce qu'une conversation retient

---

## Contenu

### 1. Une conversation, ce que c'est

Chaque conversation possède **ses propres réglages**, et non ceux de l'application :

- son modèle et son fournisseur ;
- son mode de permission ;
- son répertoire de travail ;
- ses outils actifs ;
- ses connecteurs activés ;
- sa branche Git ;
- ses onglets de terminal et de navigateur.

C'est le point structurant : changer de conversation change l'environnement de travail, pas seulement le fil de discussion.

### 2. Naviguer entre les conversations

- La **barre latérale** liste les conversations. C'est le seul moyen de passer de l'une à l'autre.
- Nouvelle conversation : **⌥⌘N** (Alt+Ctrl+N).
- La barre latérale se masque avec **⌘B** (Ctrl+B).

### 3. Les onglets de clones

Voir *Cloner une conversation* pour le détail. Ce qu'il faut retenir ici :

- Une conversation et ses clones forment un **groupe de trois onglets au maximum**.
- Un onglet peut être **renommé** et **fermé**.
- Fermer un onglet peut aussi nettoyer la branche Git qui lui était liée.

### 4. Archiver

- Une conversation terminée s'archive plutôt que de se supprimer : elle quitte la liste principale sans être perdue.
- Les conversations archivées se consultent dans **Réglages › Conversations archivées**.
- **Les sous-agents suivent un chemin d'archivage distinct** : le code emploie un mécanisme dédié et refuse l'archivage dans certains états.
- Cloner une conversation archivée produit un clone **non archivé**.

### 5. La recherche

- Raccourci **⌘G** (Ctrl+G).
- Ouvre une boîte de recherche permettant de retrouver une conversation et de s'y rendre.

Voir *Points à confirmer* : la portée exacte de la recherche n'est pas établie.

### 6. Ce qu'une conversation retient

- **2 000 messages au maximum** par conversation.
- Une conversation par fichier dans `agent-sessions/`.
- Les conversations créées automatiquement par un réveil portent un indicateur qui les distingue.
- Les gros résultats d'outils sont stockés à part, dans `tool-results/`, pour ne pas alourdir le fichier de conversation.
- Chaque conversation garde la trace des échecs de flux et des diagnostics — ces historiques ne sont **pas** repris dans un clone.

---

## Tableaux

### Tableau — Ce qui est propre à chaque conversation

| Élément | Portée |
|---|---|
| Modèle et fournisseur | Conversation |
| Mode de permission | Conversation |
| Répertoire de travail | Conversation |
| Outils actifs | Conversation |
| Connecteurs MCP activés | Conversation |
| Branche Git | Conversation |
| Onglets de terminal et de navigateur | Conversation |
| Thème, langue, police | Application |
| Mémoire globale | Application |

### Tableau — Les limites

| | Valeur |
|---|---|
| Messages par conversation | 2 000 |
| Onglets par groupe de clones | 3 |
| Onglets de navigateur par conversation | 10 |
| Terminaux simultanés | 16 (global) |

---

## Encadrés

**Encadré « Chaque conversation a son environnement »**
> Modèle, permissions, répertoire de travail et outils sont propres à chaque conversation. Changer de conversation change le contexte de travail.

**Encadré « Archiver plutôt que supprimer »**
> Une conversation archivée quitte la liste principale sans être effacée. Vous la retrouvez dans Réglages › Conversations archivées.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| Impossible d'ouvrir deux conversations côte à côte | Le multi-onglet de conversations n'existe plus | Naviguer par la barre latérale |
| Le modèle change en changeant de conversation | Le modèle est propre à chaque conversation | Comportement voulu |
| Une conversation n'accepte plus de messages | Plafond de 2 000 messages | Cloner pour repartir d'un point antérieur |
| Un sous-agent refuse d'être archivé | Chemin d'archivage distinct, refus dans certains états | Vérifier son état |
| La barre latérale a disparu | Masquée, ou repliée automatiquement sur fenêtre étroite | ⌘B, ou élargir la fenêtre |

---

## Renvois

- *Interface › Cloner une conversation*
- *Interface › Vue d'ensemble*
- *Agent › Répertoire de travail*
- *Agent › Contexte et compression*
- *Réglages › Application* — les conversations archivées

---

## Points à confirmer

- **Ce sur quoi porte la recherche** — titres seulement, ou contenu des messages ? Détermine entièrement l'utilité de la fonction et la façon de la présenter.
- **Renommer une conversation.** Les onglets sont renommables ; pour les conversations elles-mêmes, à vérifier. Le nom semble attribué automatiquement.
- **Supprimer définitivement** une conversation — possible depuis l'interface, ou seulement archivage ?
- **Désarchiver** — possible depuis les réglages ?
- **Les favoris.** Le README mentionne des conversations favorites ; aucun mécanisme correspondant n'a été relevé dans le code lu. Vérifier si la fonction existe encore, comme le multi-onglet qui a disparu.
- **Le comportement au-delà de 2 000 messages** — refus, troncature, ou compression ?
- **La restauration au lancement** — quelle conversation est rouverte, et dans quel état.
- **Le tri de la barre latérale** — par date de modification, de création, ou manuel ?
