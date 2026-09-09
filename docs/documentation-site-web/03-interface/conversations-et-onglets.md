# Conversations et navigation

**Emplacement site** — Interface › Conversations
**Répond à** — « Comment je passe d'une conversation à l'autre, et comment je les organise ? »
**Sources** — `src-tauri/src/commands/agent_sessions.rs:13-212`, `src-tauri/src/services/agent_local/session_order.rs:40`, `session_limits.rs:8-10`, `conversation_admission.rs:151-153`, `session_tabs.rs`, `session_tabs_state.rs:6`, `types_session.rs`, `src-tauri/src/services/llm/fast_mode.rs`, `src/hooks/use-session-tabs.ts`, `use-archived-agent-sessions.ts`, `src/components/layout/search-dialog.tsx:36` et `:41-51`, `CLAUDE.md`
**Vérification** — Vérifié dans le code pour la navigation, l'ordre manuel, l'épinglage, le renommage, la suppression, le désarchivage, l'export Markdown, le mode rapide, la portée de la recherche et toutes les limites ; plusieurs gestes d'interface restent à relever à l'écran

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
4. Archiver, désarchiver, renommer, supprimer
5. La recherche
6. Exporter une conversation en Markdown
7. Ce qu'une conversation retient

---

## Contenu

### 1. Une conversation, ce que c'est

Chaque conversation possède **ses propres réglages**, et non ceux de l'application :

- son modèle et son fournisseur ;
- son mode de permission ;
- son **mode rapide** — un réglage par conversation (`set_session_fast_mode`, `agent_sessions.rs:96`), commandé depuis la barre de saisie, qui demande au fournisseur un traitement prioritaire lorsqu'il le propose (`src-tauri/src/services/llm/fast_mode.rs`) et reste sans effet chez les autres ;
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
- **L'ordre de la liste est manuel**, pas calculé à partir d'une date. Il est décidé par l'utilisateur (`reorder_agent_sessions`, `agent_sessions.rs:13`) et conservé dans `session-order.json`, qui en est l'**autorité unique** (`src-tauri/src/services/agent_local/session_order.rs:40`) : les conversations elles-mêmes ne portent pas leur rang.
- **Une conversation peut être épinglée** pour rester en tête de liste — `pin_agent_session` et `unpin_agent_session` (`agent_sessions.rs:197` et `:202`) — avec un ordre propre aux conversations épinglées (`reorder_pinned_agent_sessions`, `:23`). C'est la fonction que le README appelle « favoris ».

### 3. Les onglets de clones

Voir *Cloner une conversation* pour le détail. Ce qu'il faut retenir ici :

- Une conversation et ses clones forment un **groupe de trois onglets au maximum**.
- Un onglet peut être **renommé** et **fermé**.
- Fermer un onglet peut aussi nettoyer la branche Git qui lui était liée.

### 4. Archiver, désarchiver, renommer, supprimer

- Une conversation terminée s'archive plutôt que de se supprimer : elle quitte la liste principale sans être perdue (`archive_agent_session`, `agent_sessions.rs:177`).
- Les conversations archivées se consultent dans **Réglages › Conversations archivées**, et **se désarchivent** (`restore_agent_session`, `:192`).
- **La suppression définitive existe aussi** (`delete_agent_session`, `:170`) : archiver n'est pas la seule sortie.
- **Renommer une conversation est possible** (`rename_agent_session`, `:102`), mais refusé sur une session enfant.
- **Les sous-agents suivent un chemin d'archivage distinct** : le code emploie un mécanisme dédié et refuse l'archivage dans certains états.
- Cloner une conversation archivée produit un clone **non archivé**.

### 5. La recherche

- Raccourci **⌘G** (Ctrl+G).
- Ouvre une boîte de recherche permettant de retrouver une conversation et de s'y rendre.
- **Elle porte sur le nom de la conversation et le nom du projet, jamais sur le contenu des messages** (`src/components/layout/search-dialog.tsx:41-51`). Les sous-agents et les clones sont exclus de la liste (`:36`).

**Point de rédaction décisif** : ne pas laisser croire à une recherche plein texte. Un utilisateur qui cherche une phrase prononcée dans une conversation ne la trouvera pas ici.

### 6. Exporter une conversation en Markdown

Une conversation s'exporte en Markdown (`export_agent_session_markdown`, `agent_sessions.rs:207`). C'est la voie pour sortir un échange de l'application — archivage personnel, partage, relecture hors ligne.

### 7. Ce qu'une conversation retient

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
| Mode rapide | Conversation |
| Répertoire de travail | Conversation |
| Outils actifs | Conversation |
| Connecteurs MCP activés | Conversation |
| Branche Git | Conversation |
| Onglets de terminal et de navigateur | Conversation |
| Thème, langue, police | Application |
| Mémoire globale | Application |

### Tableau — Les limites

| | Valeur | Source |
|---|---|---|
| Messages par conversation | **2 000** | `agent_local/session_limits.rs:10` |
| Fichiers de conversation | **4 096** | `session_limits.rs:9` |
| Taille d'un fichier de conversation | **32 Mo** | `session_limits.rs:8` |
| Onglets par groupe de clones | **3** | `agent_local/session_tabs_state.rs:6` |
| Onglets de navigateur par conversation | **10** | `services/browser/session_types.rs:3` |
| Terminaux simultanés | **16**, globalement | `services/terminal/manager.rs:35` |

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

- ~~Ce sur quoi porte la recherche.~~ **Tranché** : le nom de la conversation et le nom du projet, jamais le contenu des messages ; sous-agents et clones exclus (`search-dialog.tsx:36` et `:41-51`).
- ~~Renommer une conversation.~~ **Tranché** : possible, sauf sur une session enfant (`agent_sessions.rs:102`).
- ~~Supprimer définitivement une conversation.~~ **Tranché** : possible (`agent_sessions.rs:170`), en plus de l'archivage (`:177`).
- ~~Désarchiver.~~ **Tranché** : possible (`restore_agent_session`, `agent_sessions.rs:192`).
- ~~Les favoris.~~ **Tranché** : la fonction existe sous le nom d'**épinglage** (`pin_agent_session` / `unpin_agent_session`, `agent_sessions.rs:197` et `:202`).
- ~~Le comportement au-delà de 2 000 messages.~~ **Tranché** : **refus d'envoi**, ni troncature ni compression (`agent_local/conversation_admission.rs:151-153`).
- ~~Le tri de la barre latérale.~~ **Tranché** : manuel, autorité unique `session-order.json` (`agent_local/session_order.rs:40`).
- **La restauration au lancement** — quelle conversation est rouverte, et dans quel état. Reste ouvert.
- **Ce que change exactement le mode rapide côté utilisateur.** Le code demande un niveau de service prioritaire aux fournisseurs qui l'acceptent (`services/llm/fast_mode.rs`) ; établir la liste des fournisseurs concernés et l'effet observable avant d'en faire une promesse sur le site.
- **La présentation à l'écran de l'épinglage et de l'export Markdown** — où sont les commandes, et sous quel libellé. À relever pendant la passe d'interface.
