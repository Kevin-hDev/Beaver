# Raccourcis clavier

**Emplacement site** — Référence › Raccourcis clavier (page prévue au sommaire du mockup)
**Répond à** — « Quels raccourcis existent, et quelle touche sur mon système ? »
**Sources** — `src/components/layout/use-app-layout-effects.ts` (lignes 60-92), `src/hooks/use-agent-local-shortcuts.ts` (lignes 25-45), `src/components/settings/shortcuts-settings.tsx`, `src/lib/platform.ts`
**Vérification** — Vérifié dans le code : chaque raccourci a été lu dans son gestionnaire d'événement

---

## Avertissement au rédacteur

**L'écran Réglages › Raccourcis de l'application affiche un raccourci faux.**

Il annonce **⌥⌘J** (Alt+Ctrl+J) pour basculer la prévisualisation. Le code implémente **⌥⌘B** (`use-agent-local-shortcuts.ts:29` : `mod && event.altKey && event.code === "KeyB"`).

**Publier le raccourci du code, pas celui de l'écran des réglages.** Et signaler le problème pour correction dans l'application — voir *Points à confirmer*.

---

## Plan de page proposé

1. La touche de commande selon le système
2. Tableau des raccourcis
3. Quand les raccourcis ne s'appliquent pas
4. Raccourcis fournis par le système

---

## Contenu

### 1. La touche de commande selon le système

Beaver emploie deux touches modificatrices, dont le nom change selon le système :

| Rôle | macOS | Windows et Linux |
|---|---|---|
| Commande | ⌘ (Cmd) | Ctrl |
| Alternative | ⌥ (Option) | Alt |

Dans le reste de la page, écrire les deux formes plutôt qu'une notation abstraite. Un utilisateur Windows ne doit pas avoir à traduire « Mod » dans sa tête.

### 2. Tableau des raccourcis

Sept raccourcis au total. Voir section Tableaux.

Deux points de conception à signaler, parce qu'ils expliquent des comportements sinon incompréhensibles :

- **⌘B et ⌥⌘B sont deux raccourcis différents.** Le premier bascule la barre latérale, le second la prévisualisation. Le gestionnaire ignore volontairement ⌘B quand Alt est enfoncé, pour que les deux ne se déclenchent pas ensemble.
- **⌘J ne fonctionne que dans une conversation ouverte.** C'est un raccourci de session, pas un raccourci global.

### 3. Quand les raccourcis ne s'appliquent pas

Les raccourcis de session — terminal et prévisualisation — sont **ignorés** dans deux cas :

- **aucune conversation active** ;
- **le curseur est dans un champ de saisie** — zone de message, champ de recherche, éditeur. Sans cette précaution, taper la lettre J dans un message ouvrirait le terminal.

Les raccourcis de navigation, eux, restent actifs partout.

**Comportement particulier de ⌘J** : si le terminal n'a jamais été ouvert dans cette conversation, le raccourci **crée un premier onglet de terminal** dans le répertoire de travail au lieu de simplement basculer l'affichage.

### 4. Raccourcis fournis par le système

À mentionner brièvement, ce sont ceux qu'on cherche en premier :

- **macOS** — `⌘Q` quitte réellement l'application. La pastille rouge se contente de masquer la fenêtre.
- **Windows et Linux** — la croix ferme l'application.

Renvoyer vers *Premier lancement* pour le détail de cette différence.

---

## Tableaux

### Tableau — Tous les raccourcis

| Action | macOS | Windows / Linux | Portée |
|---|---|---|---|
| Afficher ou masquer le terminal | ⌘J | Ctrl+J | Conversation active uniquement |
| Afficher ou masquer la barre latérale | ⌘B | Ctrl+B | Partout |
| Afficher ou masquer la prévisualisation | ⌥⌘B | Alt+Ctrl+B | Conversation active uniquement |
| Ouvrir la recherche | ⌘G | Ctrl+G | Partout |
| Revenir en arrière | ⌘← | Ctrl+← | Partout |
| Aller en avant | ⌘→ | Ctrl+→ | Partout |
| Nouvelle conversation | ⌥⌘N | Alt+Ctrl+N | Partout |

### Tableau — Ce qui bloque un raccourci de session

| Situation | Effet |
|---|---|
| Aucune conversation ouverte | ⌘J et ⌥⌘B sans effet |
| Curseur dans un champ de saisie | ⌘J et ⌥⌘B sans effet |
| Terminal jamais ouvert dans cette conversation | ⌘J crée un onglet au lieu de basculer |

---

## Encadrés

**Encadré « ⌘B et ⌥⌘B »**
> Ajouter la touche Option (ou Alt) à ⌘B change complètement l'action : ⌘B bascule la barre latérale, ⌥⌘B bascule la prévisualisation. Les deux ne se déclenchent jamais ensemble.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| ⌘J ne fait rien | Pas de conversation active, ou curseur dans un champ de saisie | Cliquer hors du champ, ou ouvrir une conversation |
| Le raccourci de prévisualisation ne marche pas | L'écran des réglages affiche ⌥⌘J, le vrai raccourci est ⌥⌘B | Utiliser ⌥⌘B |
| ⌘B ouvre la prévisualisation au lieu de la barre latérale | La touche Option est enfoncée | Relâcher Option |
| Un raccourci se déclenche en tapant un message | Ne devrait pas arriver : les champs de saisie sont exclus | Signaler le problème |

---

## Renvois

- *Interface › Vue d'ensemble* — ce que sont la barre latérale et la prévisualisation
- *Interface › Terminal intégré*
- *Premier lancement* — le comportement du bouton de fermeture par système

---

## Points à confirmer

- **Corriger l'écran Réglages › Raccourcis dans l'application.** Il annonce ⌥⌘J pour la prévisualisation, le code fait ⌥⌘B. Tant que ce n'est pas corrigé, le site et l'application se contrediront — et c'est l'application qu'on croira.
- **Les raccourcis sont-ils personnalisables ?** Rien dans le code lu ne le suggère : la liste est figée. Confirmer, et le dire explicitement — c'est une question fréquente.
- **Existe-t-il d'autres raccourcis non recensés ?** Deux gestionnaires ont été lus. Vérifier les composants qui gèrent leurs propres touches : navigateur intégré, terminal, éditeur de prévisualisation, menus déroulants. Le sélecteur de mode de permission, par exemple, réagit aux touches 1, 2 et 3 quand il est ouvert.
- **Le raccourci de fermeture d'onglet de conversation** n'apparaît nulle part. Vérifier s'il existe.
- **Les touches de navigation dans les listes** (flèches, Échap) sont gérées par un mécanisme partagé. Décider si elles méritent d'être documentées ou si elles relèvent du comportement attendu.
