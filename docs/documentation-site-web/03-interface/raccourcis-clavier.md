# Raccourcis clavier

**Emplacement site** — Référence › Raccourcis clavier (page prévue au sommaire du mockup)
**Répond à** — « Quels raccourcis existent, et quelle touche sur mon système ? »
**Sources** — `src/lib/app-shortcuts.ts:28-49` (la table, autorité unique), `:55-78` (la comparaison des touches), `src/components/settings/shortcuts-settings.tsx:22`, `src/lib/platform.ts:1-8`, `src/hooks/use-agent-local-shortcuts.ts:22-34`, `src/components/layout/use-app-layout-effects.ts:84-97`, `src/hooks/use-conversation-search.ts:34-48`, `src/hooks/use-permission-mode.ts:96-105`, `src/hooks/use-session-tabs.ts:181-192`, `src/components/agent-local/chat-input.tsx:116-137`, `chat-input-editor.tsx:51-60`, `edit-message.tsx:91-101`, `src/i18n/fr.json` (libellés affichés)
**Vérification** — Vérifié dans le code : les vingt raccourcis viennent d'une table unique, et la portée de chacun a été lue dans son gestionnaire

---

## Ce qui a changé, et pourquoi ça compte

Les raccourcis ne sont plus écrits à plusieurs endroits. Ils vivent dans **une table unique**, `APP_SHORTCUTS` (`src/lib/app-shortcuts.ts:28`), que lisent à la fois l'écran des réglages et tous les gestionnaires de touches. Le commentaire qui ouvre la table le dit explicitement (`:26-27`) : c'est une autorité unique, pour qu'un changement ne puisse plus diverger entre l'application et son écran de réglages.

**Conséquence pour la rédaction** : ce que l'écran Réglages › Raccourcis affiche est exact par construction. Une version antérieure de ce brief avertissait d'un raccourci faux dans cet écran ; ce défaut n'existe plus.

---

## Plan de page proposé

1. La touche de commande selon le système
2. Tableau des raccourcis
3. Où chaque raccourci s'applique
4. Les raccourcis ne sont pas personnalisables
5. Raccourcis fournis par le système

---

## Contenu

### 1. La touche de commande selon le système

Beaver emploie deux touches modificatrices, dont le nom change selon le système :

| Rôle | macOS | Windows et Linux |
|---|---|---|
| Commande (« mod ») | ⌘ (Cmd) | Ctrl |
| Alternative | ⌥ (Option) | Alt |

Ces libellés sont calculés à partir du système détecté (`src/lib/platform.ts:5` et `:8`), et l'écran des réglages les substitue à l'affichage (`shortcuts-settings.tsx:9-11`).

Détail à connaître sans forcément l'écrire sur la page : la comparaison accepte **Cmd ou Ctrl indifféremment** sur les trois systèmes (`app-shortcuts.ts:74` : `event.metaKey || event.ctrlKey`). Un utilisateur macOS habitué à Ctrl obtient donc le même résultat.

Dans le reste de la page, écrire les deux formes plutôt qu'une notation abstraite. Un utilisateur Windows ne doit pas avoir à traduire « Mod » dans sa tête.

### 2. Tableau des raccourcis

**Vingt raccourcis au total.** Voir section Tableaux : le tableau reprend la table du code ligne par ligne, dans son ordre.

Deux points de conception à signaler, parce qu'ils expliquent des comportements sinon incompréhensibles :

- **⌘B et ⌥⌘B sont deux raccourcis différents.** Le premier bascule la barre latérale, le second la prévisualisation. Les deux ne se déclenchent jamais ensemble : la comparaison exige que les modificateurs correspondent **exactement**, Alt compris (`app-shortcuts.ts:73-78`). Ce n'est plus un contournement écrit à la main dans un gestionnaire, c'est une propriété de la table.
- **⌘1 à ⌘9 ne servent pas à changer de conversation**, mais à choisir un onglet **dans** la conversation courante — et les onglets n'existent que pour les groupes de clones. Sans clone, ces neuf raccourcis n'ont rien à sélectionner.

### 3. Où chaque raccourci s'applique

Trois portées distinctes, à ne pas mélanger. Le tableau de la section suivante donne la portée de chacun.

**Partout dans l'application** — barre latérale, navigation, recherche de conversations, réglages, taille du texte. Ces raccourcis écoutent la fenêtre entière et ne posent aucune condition.

**Dans la conversation affichée** — terminal, prévisualisation, recherche dans la conversation, curseur dans la zone de message, onglets de clones, changement de mode de permission. Ils ne s'appliquent qu'à la conversation visible, et deux d'entre eux — ⌘J et ⌥⌘B — sont **ignorés** dans deux cas :

- **aucune conversation active** ;
- **le curseur est dans un champ de saisie** — zone de message, champ de recherche, éditeur (`use-agent-local-shortcuts.ts:25` et `:42-46`). Sans cette précaution, taper la lettre J dans un message ouvrirait le terminal.

Maj+Tab applique la même exclusion des champs de saisie (`use-permission-mode.ts:100-102`). Il fonctionne aussi sur l'écran d'accueil d'une nouvelle conversation (`src/components/agent-local/welcome-view.tsx:39`), mais **pas dans une conversation de sous-agent** (`chat-view.tsx:49`, le gestionnaire y est désactivé).

**Dans un champ de saisie** — envoi, retour à la ligne, arrêt de la réponse, validation ou annulation d'une modification de message. Ceux-là ne fonctionnent **que** curseur dans le champ concerné, ce qui est l'inverse exact de la règle précédente.

**Comportement particulier de ⌘J** : si le terminal n'a jamais été ouvert dans cette conversation, le raccourci **crée un premier onglet de terminal** dans le répertoire de travail au lieu de simplement basculer l'affichage.

### 4. Les raccourcis ne sont pas personnalisables

`APP_SHORTCUTS` est une constante figée (`app-shortcuts.ts:28`), et l'écran Réglages › Raccourcis se contente de l'afficher : il ne comporte aucun champ de saisie (`shortcuts-settings.tsx:22-36`).

**Le dire explicitement sur la page.** C'est une question fréquente, et une page qui reste muette laisse croire que l'option est cachée quelque part.

### 5. Raccourcis fournis par le système

À mentionner brièvement, ce sont ceux qu'on cherche en premier :

- **macOS** — `⌘Q` quitte réellement l'application. La pastille rouge se contente de masquer la fenêtre.
- **Windows et Linux** — la croix ferme l'application.

Renvoyer vers *Premier lancement* pour le détail de cette différence.

---

## Tableaux

### Tableau — Tous les raccourcis

Vingt lignes, dans l'ordre de la table du code (`app-shortcuts.ts:29-48`). La colonne « Libellé affiché » reprend le texte français de l'écran des réglages (`src/i18n/fr.json`, clés `settings.shortcuts.*`).

| Libellé affiché | macOS | Windows / Linux | Portée |
|---|---|---|---|
| Ouvrir/fermer le terminal | ⌘J | Ctrl+J | Conversation affichée, hors champ de saisie |
| Ouvrir/fermer la sidebar | ⌘B | Ctrl+B | Partout |
| Précédent | ⌘← | Ctrl+← | Partout |
| Suivant | ⌘→ | Ctrl+→ | Partout |
| Nouvelle session | ⌥⌘N | Alt+Ctrl+N | Partout |
| Rechercher une conversation | ⌘G | Ctrl+G | Partout |
| Ouvrir/fermer le panneau de prévisualisation | ⌥⌘B | Alt+Ctrl+B | Conversation affichée, hors champ de saisie |
| Ouvrir les réglages | ⌘, | Ctrl+, | Partout |
| Rechercher dans la conversation | ⌘F | Ctrl+F | Conversation affichée |
| Placer le curseur dans le champ de discussion | ⌘L | Ctrl+L | Conversation affichée |
| Changer d'onglet de session | ⌘1 à ⌘9 | Ctrl+1 à Ctrl+9 | Conversation affichée, et seulement si elle a des onglets de clones |
| Modifier les permissions | Maj+Tab | Maj+Tab | Conversation affichée et écran d'accueil d'une nouvelle conversation, hors champ de saisie ; pas dans un sous-agent |
| Envoyer le message | Entrée | Entrée | Zone de message |
| Ajouter une nouvelle ligne | Maj+Entrée | Maj+Entrée | Zone de message |
| Arrêter la réponse | Échap Échap | Échap Échap | Pendant que l'agent répond — le premier Échap arme une confirmation, le second arrête |
| Valider et relancer le message modifié | ⌘Entrée | Ctrl+Entrée | Champ de modification d'un message |
| Annuler la modification | Échap | Échap | Champ de modification d'un message, et fermeture de la recherche dans la conversation |
| Agrandir l'interface | ⌘+ | Ctrl++ | Partout |
| Réduire l'interface | ⌘- | Ctrl+- | Partout |
| Rétablir la taille de l'interface | ⌘0 | Ctrl+0 | Partout |

**Attention à deux libellés** : la colonne reproduit fidèlement ce que l'application affiche, et deux entrées y emploient un autre vocabulaire que le reste de la documentation.

- « Ouvrir/fermer la **sidebar** » — anglicisme, là où la documentation dit « barre latérale ».
- « Nouvelle **session** » — là où la documentation dit « nouvelle conversation ».

Sur le site, employer « barre latérale » et « conversation », et signaler les deux libellés pour correction dans l'application — sinon les deux vocabulaires coexisteront sous les yeux du même utilisateur.

**Le double Échap est réel** (vérifié dans le code le 9 septembre 2026, après qu'un rapport d'audit a affirmé le contraire) : le premier Échap arme une confirmation qui reste active 3 secondes (`use-stop-confirmation.ts:3`, `STOP_CONFIRMATION_TIMEOUT_MS`), le second Échap pendant cette fenêtre arrête la réponse (`use-stop-confirmation.ts:21-35`). Passé les 3 secondes, la confirmation retombe et il faut recommencer. Le **bouton stop** du champ de saisie, lui, arrête en **un seul clic**, sans confirmation — viser puis cliquer est déjà un geste délibéré, là où Échap part souvent d'un réflexe (`use-stop-confirmation.ts:37-44`). L'affichage « Échap Échap » de l'écran des réglages (`app-shortcuts.ts:43`) est donc exact.

Le pavé numérique est accepté partout où un chiffre ou un signe est attendu : `Numpad1` à `Numpad9`, `NumpadAdd`, `NumpadSubtract`, `Numpad0` (`app-shortcuts.ts:21-24`, `:46-48`).

### Tableau — Ce qui bloque un raccourci de conversation

| Situation | Effet |
|---|---|
| Aucune conversation ouverte | ⌘J et ⌥⌘B sans effet |
| Curseur dans un champ de saisie | ⌘J, ⌥⌘B et Maj+Tab sans effet |
| Conversation de sous-agent | Maj+Tab sans effet |
| Conversation sans clone | ⌘1 à ⌘9 sans effet : il n'y a pas d'onglet à choisir |
| Terminal jamais ouvert dans cette conversation | ⌘J crée un onglet au lieu de basculer |
| Aucune réponse en cours | Échap dans la zone de message n'a rien à arrêter |

---

## Encadrés

**Encadré « ⌘B et ⌥⌘B »**
> Ajouter la touche Option (ou Alt) à ⌘B change complètement l'action : ⌘B bascule la barre latérale, ⌥⌘B bascule la prévisualisation. Les deux ne se déclenchent jamais ensemble.

**Encadré « La liste n'est pas modifiable »**
> Les raccourcis de Beaver sont fixes. L'écran Réglages › Raccourcis les affiche pour référence, il ne permet pas de les changer.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| ⌘J ne fait rien | Pas de conversation active, ou curseur dans un champ de saisie | Cliquer hors du champ, ou ouvrir une conversation |
| ⌘B ouvre la prévisualisation au lieu de la barre latérale | La touche Option est enfoncée | Relâcher Option |
| ⌘1 à ⌘9 ne changent pas de conversation | Ces raccourcis choisissent un onglet de clone, pas une conversation | Passer d'une conversation à l'autre par la barre latérale |
| Maj+Tab ne change pas le mode de permission | Curseur dans un champ de saisie, ou conversation de sous-agent | Cliquer hors du champ |
| Entrée envoie le message au lieu d'aller à la ligne | Entrée envoie, Maj+Entrée va à la ligne | Utiliser Maj+Entrée |
| Impossible de personnaliser un raccourci | La liste est figée dans le code | Aucune solution : c'est une contrainte assumée |
| Un seul Échap n'arrête pas la réponse | Le premier Échap arme seulement la confirmation, valable 3 secondes | Appuyer une seconde fois sur Échap, ou cliquer sur le bouton stop qui arrête en un clic |
| Un raccourci se déclenche en tapant un message | Ne devrait pas arriver : les champs de saisie sont exclus | Signaler le problème |

---

## Renvois

- *Interface › Vue d'ensemble* — ce que sont la barre latérale et la prévisualisation
- *Interface › Terminal intégré*
- *Interface › Conversations* — la recherche ⌘G et sa portée réelle
- *Interface › Cloner une conversation* — les onglets que ⌘1 à ⌘9 sélectionnent
- *Interface › Thèmes et apparence* — la taille du texte réglée par ⌘+, ⌘- et ⌘0
- *Premier lancement* — le comportement du bouton de fermeture par système

---

## Points à confirmer

- **Le partage de la touche Échap** entre « arrêter la réponse », « annuler la modification » et « fermer la recherche dans la conversation ». Les trois gestionnaires y répondent dans des contextes différents ; l'ordre de priorité quand deux d'entre eux sont actifs n'a pas été établi.
- **Les touches propres à certains composants**, non recensées dans la table : navigateur intégré, terminal, listes déroulantes, sélecteur de mode de permission ouvert, navigation dans le menu des skills (flèches haut et bas, `chat-input.tsx:120-121`). Décider si elles méritent une mention ou relèvent du comportement attendu.
- **Le raccourci de fermeture d'un onglet de clone** n'apparaît pas dans la table. Vérifier s'il existe sous une autre forme, ou si la fermeture passe uniquement par la souris.
- **Le rendu de l'écran Réglages › Raccourcis en français.** Les libellés viennent des clés `settings.shortcuts.*` ; les vérifier à l'écran avant de figer le vocabulaire de la page — le tableau ci-dessus les reprend depuis le fichier de traduction, pas depuis une observation.
