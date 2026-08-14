# Le panneau latéral

**Emplacement site** — Interface › Panneau latéral
**Répond à** — « À quoi sert cette zone à droite, et pourquoi mon aperçu disparaît quand j'ouvre le navigateur ? »
**Sources** — `src/hooks/use-forecast-panel.ts` (lignes 9, 22), `src/components/agent-local/mode-selector.tsx`, `src/components/layout/panel-slots.tsx`, `app-layout.tsx`, `sidebar-compact-state.ts`, `src/hooks/use-agent-local-shortcuts.ts`
**Vérification** — Vérifié dans le code : les trois modes et le mode par défaut

---

## Le point à comprendre

**Le panneau latéral n'affiche qu'une chose à la fois.** Prévisualisation, navigateur et Forecast se partagent le même espace : ouvrir l'un remplace l'autre.

Ce n'est pas un défaut, c'est une contrainte de place assumée. Mais quelqu'un qui voit son aperçu de fichier disparaître en ouvrant le navigateur croit à un bug. La page doit le dire d'emblée.

---

## Plan de page proposé

1. Ce que contient le panneau
2. Les trois modes
3. Basculer d'un mode à l'autre
4. Redimensionner et masquer
5. Le comportement automatique

---

## Contenu

### 1. Ce que contient le panneau

La zone à droite de la conversation. Elle sert à consulter sans quitter le fil : un fichier, une page web, un graphique de prévision.

### 2. Les trois modes

| Mode | Contenu | Disponibilité |
|---|---|---|
| **Prévisualisation** | Fichiers, images, tableurs, documents | Partout — mode par défaut |
| **Navigateur** | Navigateur web intégré | macOS et Windows |
| **Forecast** | Espace de prévision de séries temporelles | Partout |

Le mode **Prévisualisation** est actif par défaut.

### 3. Basculer d'un mode à l'autre

- Un sélecteur dédié dans l'interface de la conversation.
- Raccourci **⌥⌘B** (Alt+Ctrl+B) pour afficher ou masquer le panneau.

Attention lors de la rédaction : l'écran Réglages › Raccourcis de l'application annonce un raccourci erroné pour cette action. Voir *Interface › Raccourcis clavier*.

### 4. Redimensionner et masquer

- La largeur du panneau se règle par glissement, avec une largeur minimale imposée.
- La barre latérale gauche se masque avec **⌘B** (Ctrl+B), ce qui laisse plus de place au reste.

### 5. Le comportement automatique

L'application ajuste la disposition selon la largeur disponible : quand la fenêtre devient trop étroite pour afficher confortablement la barre latérale, la conversation et le panneau, la barre latérale se replie d'elle-même et la conversation passe en affichage compact.

C'est utile à documenter : un utilisateur qui réduit sa fenêtre et voit la barre latérale disparaître doit savoir que c'est voulu et réversible.

---

## Encadrés

**Encadré « Un seul contenu à la fois »**
> Prévisualisation, navigateur et Forecast partagent le même espace. Ouvrir l'un remplace l'autre — rien n'est perdu, il suffit de rebasculer.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| L'aperçu disparaît en ouvrant le navigateur | Les trois modes partagent le panneau | Rebasculer sur Prévisualisation |
| Le mode Navigateur est absent | Système Linux | Non disponible |
| La barre latérale se replie seule | Fenêtre trop étroite | Élargir la fenêtre, ou ⌘B |
| Le panneau ne se réduit pas davantage | Largeur minimale imposée | Masquer le panneau |
| ⌥⌘B ne fonctionne pas | Pas de conversation active, ou curseur dans un champ | Cliquer hors du champ |

---

## Renvois

- *Interface › Arbre de fichiers et prévisualisations*
- *Interface › Navigateur intégré*
- *Forecast › Vue d'ensemble*
- *Interface › Raccourcis clavier*

---

## Points à confirmer

- **La position exacte du sélecteur de mode** dans l'interface, et son aspect. Non relevé.
- **Le mode est-il mémorisé par conversation** ou global ? Le hook est rattaché à la conversation, ce qui suggère un réglage par conversation. À confirmer.
- **Les largeurs minimales** en pixels — deux constantes existent, l'une pour la conversation, l'autre pour l'affichage compact. Valeurs non relevées.
- **Le seuil de repli automatique** de la barre latérale. Non relevé.
- **Le panneau peut-il être détaché** dans une fenêtre séparée, comme la mascotte ? Rien ne le suggère, à confirmer.
