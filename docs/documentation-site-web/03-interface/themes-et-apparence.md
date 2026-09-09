# Thèmes et apparence

**Emplacement site** — Référence › Thèmes et langues (page prévue au sommaire du mockup), ou Interface › Apparence
**Répond à** — « Comment je change l'allure de l'application, la taille du texte et la coloration du code ? »
**Sources** — `src/lib/app-themes.ts`, `src/hooks/use-settings.ts` (lignes 3-42), `src/components/settings/theme-selector.tsx`, `font-size-control.tsx`, `code-theme-preview.tsx`, `src/styles/themes/`
**Vérification** — Vérifié dans le code : les listes et les bornes sont lues dans les fichiers de définition

---

## Plan de page proposé

1. Les thèmes
2. Le mode Système
3. La taille du texte
4. La police
5. La coloration du code
6. Où se règle quoi

---

## Contenu

### 1. Les thèmes

**Six thèmes**, plus une option Système. Deux sont clairs, quatre sont sombres.

Tableau complet en section Tableaux.

À expliquer : chaque thème est une palette complète, pas seulement une couleur d'accentuation. Le passage de l'un à l'autre est immédiat, sans redémarrage.

### 2. Le mode Système

- L'option **Système** suit le réglage clair/sombre du système d'exploitation.
- Elle bascule entre **Clair** et **Sombre** uniquement. Les quatre thèmes colorés ne sont jamais choisis automatiquement.

C'est le point qui surprend : quelqu'un qui aime Emerald Night et met « Système » ne retrouvera pas Emerald Night la nuit, mais le thème Sombre standard.

### 3. La taille du texte

- Réglable de **10 à 24 pixels**.
- Valeur par défaut : **18 pixels**.
- Réglage par saisie directe ou par incréments d'un pixel.
- Toute valeur hors bornes est ramenée dans l'intervalle ; une saisie invalide revient à 18.

### 4. La police

**Sept familles** proposées, du classique au fantaisiste. Liste complète en section Tableaux.

À signaler : quatre d'entre elles sont à chasse fixe — utile pour lire du code dans les réponses — et deux sont des polices manuscrites, à réserver à un usage décoratif.

### 5. La coloration du code

- **Cinq thèmes de coloration** pour les blocs de code dans les réponses.
- **Chacun possède une variante claire et une variante sombre**, choisie automatiquement selon le thème de l'application. On ne règle donc pas séparément « coloration claire » et « coloration sombre ».
- L'écran de réglage affiche un aperçu côte à côte des deux variantes sur un extrait de code.

### 6. Où se règle quoi

Tout est dans **Réglages › Général**, à l'exception de ce qui touche à la mascotte, qui a son propre onglet.

Le thème et la langue sont également proposés pendant le parcours d'accueil.

---

## Tableaux

### Tableau — Les thèmes

| Thème | Type | Identifiant |
|---|---|---|
| Clair | Clair | `light` |
| Sombre | Sombre | `dark` |
| Emerald Night | Sombre | `emerald-night` |
| Cobalt Frost | Clair | `cobalt-frost` |
| Astral Mist | Sombre | `astral-mist` |
| Crimson Eclipse | Sombre | `crimson-eclipse` |
| Système | Suit le système, entre Clair et Sombre | `system` |

### Tableau — Les polices

| Police | Nature |
|---|---|
| System Default | Police du système |
| JetBrains Mono | Chasse fixe |
| Helvetica Neue | Proportionnelle |
| Menlo | Chasse fixe |
| UI Monospace | Chasse fixe |
| Pacifico | Manuscrite |
| Rancho | Manuscrite |

### Tableau — La coloration du code

| Thème de coloration |
|---|
| Défaut |
| GitHub |
| One Dark Pro |
| Tokyo Night |
| Catppuccin |

### Tableau — Bornes de la taille du texte

| | Valeur |
|---|---|
| Minimum | 10 px |
| Maximum | 24 px |
| Défaut | 18 px |
| Pas | 1 px |

---

## Encadrés

**Encadré « Le mode Système ne couvre que deux thèmes »**
> L'option Système alterne entre Clair et Sombre selon le réglage de votre système d'exploitation. Les thèmes colorés — Emerald Night, Cobalt Frost, Astral Mist, Crimson Eclipse — se choisissent explicitement.

**Encadré « Une seule coloration à régler »**
> Chaque thème de coloration du code existe en version claire et sombre. Beaver choisit la bonne selon le thème de l'application : vous n'avez qu'un réglage à faire.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| Le thème coloré disparaît la nuit | L'option Système est active et ne connaît que Clair et Sombre | Choisir le thème explicitement |
| La taille du texte ne descend pas plus bas | Bornée à 10 px | Aucune : c'est la limite basse |
| Une valeur saisie est remplacée | Valeur hors bornes ou non numérique | La valeur est ramenée dans l'intervalle, ou remise à 18 px |
| Le code est illisible dans un thème | La coloration ne convient pas à la palette | Essayer une autre coloration ; l'aperçu montre les deux variantes |

---

## Renvois

- *Parcours d'accueil* — thème et langue au premier lancement
- *Interface › Langues*
- *Interface › Mascotte*
- *Réglages › Général et préférences*

---

## Points à confirmer

- **Les noms des thèmes sont-ils traduits ?** Ils sont définis avec des clés de traduction. Vérifier ce qui s'affiche réellement en français : « Emerald Night » ou une traduction. La page doit employer les mêmes mots que l'interface.
- **L'effet de la police sur les blocs de code.** Vérifier si le réglage de police s'applique aussi au code, ou seulement au texte courant — deux des sept polices sont manuscrites, ce qui rendrait le code illisible.
- **La persistance des réglages.** Ils semblent stockés côté navigateur plutôt que dans le dossier de données. Confirmer, et en déduire ce qui survit à une réinstallation.
- **Les thèmes sont-ils personnalisables ?** Six fichiers de palette existent dans le dépôt. Confirmer qu'il n'existe aucun moyen d'en ajouter un depuis l'application, et le dire.
- **L'accessibilité.** Aucun réglage de contraste renforcé ni de réduction des animations n'a été repéré. Vérifier avant d'écrire quoi que ce soit sur le sujet.
