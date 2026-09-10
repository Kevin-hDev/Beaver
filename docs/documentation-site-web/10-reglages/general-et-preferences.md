# Préférences : Général, Mascotte, Raccourcis

**Emplacement site** — Réglages › Préférences
**Répond à** — « Comment change-t-on l'apparence, la langue et le comportement au démarrage de Beaver ? »
**Sources** — `src/components/settings/general-settings.tsx`, `general-settings-options.ts`, `theme-selector.tsx`, `font-size-control.tsx`, `code-theme-preview.tsx`, `mascot-settings.tsx`, `shortcuts-settings.tsx` ; `src/lib/app-themes.ts`, `src/lib/app-shortcuts.ts`, `src/hooks/use-settings.ts`, `src/hooks/use-theme.ts`, `src/services/mascot.ts` ; `src-tauri/src/commands/config.rs`, `src-tauri/src/models/config.rs` ; `src/i18n/fr.json`
**Vérification** — Vérifié dans le code le 10 septembre 2026, sur la version **1.2.2**. Aucune vérification à l'écran (voir « Points à confirmer »).

> **Cette page décrit les champs, pas les sujets.** Ce que les thèmes changent visuellement est dans `03-interface/themes-et-apparence.md`, la mascotte dans `03-interface/mascotte.md`, les langues dans `03-interface/langues.md` et le détail des raccourcis dans `03-interface/raccourcis-clavier.md`. Ici : où est le champ, ce qu'il vaut, et où sa valeur est écrite.

---

## Plan de page proposé

1. Ce que contient la section Préférences
2. Onglet Général — l'apparence
3. Onglet Général — les langues
4. Onglet Général — le démarrage
5. Onglet Mascotte
6. Onglet Raccourcis clavier
7. Où chaque réglage est enregistré

---

## Contenu

### 1. Ce que contient la section Préférences

Trois onglets, dans cet ordre (`src/features/extension-ui/core-occupants.tsx:28-33`) :

| Onglet | Ce qu'on y règle |
|---|---|
| **Général** | Thème, texte, langues, aperçu des liens, lancement au démarrage |
| **Mascotte** | Le personnage animé posé sur le bureau |
| **Raccourcis clavier** | La liste des raccourcis — **en consultation seule** |

### 2. Onglet Général — l'apparence

Quatre réglages, dans une première carte (`general-settings.tsx:86-122`).

**Thème** — « Apparence de l'application ». Une grille de vignettes, chacune montrant un aperçu miniature du thème. **Sept choix livrés avec Beaver** (`src/lib/app-themes.ts:1-31`) :

| Libellé affiché | Fond |
|---|---|
| **Clair** | clair |
| **Sombre** | sombre |
| **Émeraude nocturne** | sombre |
| **Cobalt givré** | clair |
| **Brume astrale** | sombre |
| **Éclipse écarlate** | sombre |
| **Système** | suit le réglage du système d'exploitation |

Le choix **Système** est affiché en dernier, avec une vignette coupée en deux — moitié claire, moitié sombre (`theme-selector.tsx:50-57`, `:121-127`). Il ne fige rien : il suit le mode clair ou sombre du système, et bascule avec lui (`src/hooks/use-theme.ts`).

**Une extension installée peut ajouter ses propres thèmes.** Ils apparaissent dans la même grille, après les sept thèmes natifs, avec le nom de l'extension qui les fournit affiché sous le libellé (`theme-selector.tsx:110-120`).

**Taille de police** — « Taille de base de l'interface en pixels ». Un champ numérique avec deux flèches. La valeur est bornée : **de 10 à 24 pixels**, **18 par défaut** (`src/hooks/use-settings.ts:3-5`). Toute valeur hors bornes est ramenée dans l'intervalle plutôt que refusée (`use-settings.ts:44-47`). Les flèches du clavier haut et bas font varier d'un pixel (`font-size-control.tsx:54-61`).

Ce réglage a aussi des raccourcis clavier, décrits plus bas : `Cmd`/`Ctrl` + `+`, `-` et `0`.

**Police d'écriture** — « Police utilisée dans l'application ». **Sept choix** (`use-settings.ts:17-25`) : *System Default*, *JetBrains Mono*, *Helvetica Neue*, *Menlo*, *UI Monospace*, *Pacifico*, *Rancho*. Les deux dernières sont des écritures manuscrites ; les trois du milieu sont des polices à chasse fixe, adaptées au code.

**Thème de code** — « Coloration des blocs de code et previews ». **Cinq choix** (`use-settings.ts:34-40`) : *Défaut*, *GitHub*, *One Dark Pro*, *Tokyo Night*, *Catppuccin*. Chaque thème existe en variante claire et sombre, choisie automatiquement selon le thème de l'application (`use-settings.ts:29-33`).

**Un aperçu en direct** est affiché juste sous la carte : un extrait de code coloré avec le thème sélectionné, qui se met à jour au changement (`general-settings.tsx:124`, `code-theme-preview.tsx`).

### 3. Onglet Général — les langues

Deux réglages distincts, qu'il faut soigneusement séparer sur le site : ils ne concernent pas la même chose.

**Langue** — « Langue de l'interface ». Les **sept langues** de Beaver (`general-settings-options.ts:14-22`) : English, Français, Deutsch, Español, Italiano, 中文, 日本語. Le changement est **immédiat**, sans redémarrage (`general-settings.tsx:79-82`).

**Langue de réponse du LLM** — « Langue dans laquelle le modèle IA répondra ». Les mêmes sept langues, **plus une valeur vide affichée `—`**, qui est le défaut (`general-settings-options.ts:24-33` ; défaut `response_language: ""` dans `src-tauri/src/models/config.rs:53`).

La distinction à écrire clairement : **changer la langue de l'interface ne change pas la langue des réponses de l'agent**, et l'inverse est vrai aussi. Sans valeur ici, le modèle répond comme il l'entend — le plus souvent dans la langue de la question.

**Aperçu des liens** — « Afficher une carte d'aperçu pour les URL dans le chat ». Un interrupteur, **activé par défaut** (`models/config.rs:54`).

### 4. Onglet Général — le démarrage

Deux interrupteurs, dans une dernière carte (`general-settings.tsx:161-184`).

**Lancer au démarrage** — « Ouvrir Beaver automatiquement au démarrage du système ». **Désactivé par défaut** (`models/config.rs:41`).

**Démarrage masqué** — « Démarrer en arrière-plan sans ouvrir la fenêtre ». **Désactivé par défaut**, et **inutilisable tant que le précédent est éteint** : l'interrupteur est alors grisé (`general-settings.tsx:180`).

Ce lien de dépendance est appliqué à trois niveaux, ce qui vaut d'être noté : dans l'affichage, dans l'enregistrement côté interface (`general-settings.tsx:35-37`, `:67`), et **une troisième fois côté moteur**, qui remet la valeur à zéro quoi qu'il arrive si le lancement au démarrage est éteint (`src-tauri/src/commands/config.rs:60-66`). Autrement dit : il n'existe aucun chemin, même par un fichier de configuration édité à la main, qui produise un « démarrage masqué » sans « lancement au démarrage ».

Activer ou désactiver le lancement au démarrage déclenche une **synchronisation avec le système d'exploitation** — c'est là que Beaver s'inscrit ou se retire de la liste des programmes lancés à l'ouverture de session (`config.rs:43-46`, `:105-111`).

### 5. Onglet Mascotte

L'onglet s'ouvre sur un **aperçu animé** du personnage sélectionné, avec son nom et une mention d'état : **« Aperçu animé »** quand l'animation tourne, **« Animation en pause »** sinon (`mascot-settings.tsx:94-111`).

Puis une carte intitulée **« Réglages »** avec deux contrôles (`mascot-settings.tsx:113-143`) :

- **Afficher la mascotte** — « Garder la mascotte visible au-dessus du bureau et des autres applications ». **Désactivée par défaut** (`src/services/mascot.ts:11-16`).
- **Taille** — « Ajuster la taille de la mascotte à l'écran ». Un curseur, gradué **de 70 % à 140 %**, **100 % par défaut** (`mascot.ts:8-9`, `:14`). La valeur en pourcentage est affichée à côté du curseur.

Enfin une section **« Collection »** : une grille de **huit personnages** (`mascot-settings.tsx:19-64`), chacun avec son portrait, son nom et une phrase de description. Le personnage actif porte la mention **« Sélectionnée »**.

| Nom | Description affichée |
|---|---|
| **Beaver** | La mascotte principale de Beaver |
| **Circuit** | Un renne développeur calme et curieux |
| **Kova** | Un jeune combattant low-poly calme et déterminé |
| **Nival** | Un jeune ninja du givre calme, vif et déterminé |
| **Mokai** | Un singe guerrier mystique, agile et concentré |
| **Volt** | Un explorateur cybernétique vif et optimiste |
| **Raku** | Un jeune pirate audacieux et curieux |
| **Pico** | Un petit robot assistant joyeux et expressif |

**Point important pour le site** : la mascotte n'est **pas** une décoration à l'intérieur de la fenêtre. Elle se pose **au-dessus du bureau et des autres applications**, comme le dit le texte du réglage lui-même. Elle se déplace à la souris — l'action porte le libellé « Déplacer la mascotte » (`fr.json`, clé `settings.mascot.moveLabel`) — et sa position est mémorisée (`src/services/mascot.ts:28-33`, champ `position`).

### 6. Onglet Raccourcis clavier

Une simple liste : chaque ligne donne l'action à gauche et la combinaison de touches à droite (`shortcuts-settings.tsx:21-37`).

**Cet écran est en consultation seule.** Aucun raccourci n'est modifiable, et il n'existe aucun bouton pour en changer un. C'est un fait à écrire franchement plutôt qu'à laisser découvrir.

Les touches affichées **s'adaptent au système** : `Cmd` sur Mac, `Ctrl` ailleurs ; `Option` sur Mac, `Alt` ailleurs (`shortcuts-settings.tsx:8-13` ; `src/lib/platform.ts`).

**Vingt raccourcis**, dans l'ordre d'affichage (`src/lib/app-shortcuts.ts:28-49`). La liste complète est reprise dans `03-interface/raccourcis-clavier.md` ; elle est redonnée ici parce que c'est exactement ce que l'écran montre, et que cette page doit permettre de le vérifier.

| Libellé affiché | Touches (notation Mac) |
|---|---|
| Ouvrir/fermer le terminal | `Cmd` + `J` |
| Ouvrir/fermer la sidebar | `Cmd` + `B` |
| Précédent | `Cmd` + `◀` |
| Suivant | `Cmd` + `▶` |
| Nouvelle session | `Option` + `Cmd` + `N` |
| Rechercher une conversation | `Cmd` + `G` |
| Ouvrir/fermer le panneau de prévisualisation | `Option` + `Cmd` + `B` |
| Ouvrir les réglages | `Cmd` + `,` |
| Rechercher dans la conversation | `Cmd` + `F` |
| Placer le curseur dans le champ de discussion | `Cmd` + `L` |
| Changer d'onglet de session | `Cmd` + `1–9` |
| Modifier les permissions | `Shift` + `Tab` |
| Envoyer le message | `Enter` |
| Ajouter une nouvelle ligne | `Shift` + `Enter` |
| Arrêter la réponse | `Esc` `Esc` |
| Valider et relancer le message modifié | `Cmd` + `Enter` |
| Annuler la modification | `Esc` |
| Agrandir l'interface | `Cmd` + `+` |
| Réduire l'interface | `Cmd` + `-` |
| Rétablir la taille de l'interface | `Cmd` + `0` |

**Une remarque à faire figurer** : « Arrêter la réponse » demande **deux appuis sur Échap**, ce que l'écran affiche par deux touches côte à côte (`app-shortcuts.ts:43`). Ce n'est pas une coquille de la documentation.

**Les trois derniers raccourcis pilotent le même réglage que « Taille de police »** de l'onglet Général. Un utilisateur qui agrandit l'interface au clavier verra donc la valeur changer dans les réglages, et réciproquement.

### 7. Où chaque réglage est enregistré

Ce point mérite sa propre section sur le site, parce qu'il a une conséquence concrète : **tous les réglages de cette section ne se sauvegardent pas au même endroit, et certains ne suivent pas quand on copie son dossier de données.**

| Réglage | Enregistré dans | Source |
|---|---|---|
| **Thème** | Le stockage local du navigateur intégré, clés `clgo-theme` et `clgo-theme-base` | `src/hooks/use-theme.ts:27-28`, `:72-73` |
| **Taille de police** | Stockage local, clé `clgo-font-size` | `use-settings.ts:59` |
| **Police d'écriture** | Stockage local, clé `clgo-font-family` | `use-settings.ts:63` |
| **Thème de code** | Stockage local, clé `clgo-code-theme` | `use-settings.ts:69` |
| **Langue de l'interface** | Stockage local, clé `clgo-language` | `general-settings.tsx:81` |
| **Langue de réponse du LLM** | `config.json`, section `advanced` | `commands/config.rs:82` |
| **Aperçu des liens** | `config.json` **et** stockage local (clé `clgo-link-preview`) | `general-settings.tsx:72-74` |
| **Lancer au démarrage** | `config.json` **et** le système d'exploitation | `commands/config.rs:43-46` |
| **Démarrage masqué** | `config.json` | idem |
| **Mascotte** (activation, taille, personnage, position) | `config.json`, section `mascot` | `src-tauri/src/models/config.rs:12` |

Conséquence à écrire simplement : **l'apparence et la langue de l'interface sont propres à l'installation, pas au dossier de données.** Copier `~/.local/share/cl-go-dash/` sur une autre machine y emporte les conversations, les clés, les réglages de l'agent et la mascotte — mais pas le thème ni la langue de l'interface, qui repartiront de leur valeur par défaut.

---

## Encadrés

> **ℹ Deux langues, deux réglages.**
> **Langue** change les menus et les textes de Beaver. **Langue de réponse du LLM** demande au modèle de répondre dans une langue donnée. Les deux sont indépendants : on peut travailler dans une interface en français avec un agent qui répond en anglais, ou l'inverse.

> **⚠ Les raccourcis clavier ne sont pas personnalisables.**
> L'onglet les affiche, il ne les modifie pas. Il n'existe aucun moyen d'en changer un dans l'application.

> **ℹ La mascotte vit en dehors de la fenêtre.**
> Une fois activée, elle se pose au-dessus du bureau et des autres applications, pas à l'intérieur de Beaver. Elle est désactivée par défaut.

> **⚠ Le thème et la langue de l'interface ne sont pas dans votre dossier de données.**
> Ils sont enregistrés côté application, avec l'apparence et la taille du texte. Une réinstallation, ou un dossier de données copié sur une autre machine, les ramène à leur valeur par défaut. Les conversations, les clés et les réglages de l'agent, eux, suivent.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| « J'ai mis l'interface en français mais l'agent répond en anglais » | Ce sont deux réglages distincts | Renseigner **Langue de réponse du LLM** dans Général |
| « Le démarrage masqué est grisé » | Il dépend de **Lancer au démarrage**, éteint | Activer d'abord le lancement au démarrage |
| « J'ai activé le démarrage masqué mais la fenêtre s'ouvre quand même » | Le lancement au démarrage a été désactivé ensuite, ce qui remet le démarrage masqué à zéro, y compris côté moteur | Réactiver les deux |
| « Le texte est illisible, trop petit ou trop grand » | La taille est bornée entre 10 et 24 pixels | `Cmd`/`Ctrl` + `0` rétablit la taille par défaut, 18 pixels |
| « Mes thèmes et ma langue ont disparu après réinstallation » | Ils ne sont pas dans le dossier de données | Les régler à nouveau ; comportement attendu |
| « Un thème que je n'ai pas installé apparaît dans la grille » | Il vient d'une extension ; le nom de sa source est écrit sous la vignette | Le retirer en désactivant l'extension |
| « La mascotte a disparu de mon écran » | Elle est désactivée par défaut, ou déplacée hors du champ visible | Réactiver dans Préférences › Mascotte |
| « Je veux changer un raccourci clavier » | Ce n'est pas prévu dans l'application | Aucune résolution ; à signaler comme demande d'évolution |

---

## Renvois

- `10-reglages/reference-complete.md` — où se trouve tel réglage
- `03-interface/themes-et-apparence.md` — ce que chaque thème change visuellement
- `03-interface/mascotte.md` — le comportement de la mascotte, ses animations
- `03-interface/langues.md` — les sept langues de Beaver
- `03-interface/raccourcis-clavier.md` — chaque raccourci, expliqué
- `10-reglages/agent.md` — le modèle par défaut, l'accès aux fichiers, l'icône de la barre système
- `12-reference/emplacement-des-donnees.md` — ce que contient `~/.local/share/cl-go-dash/`

---

## Points à confirmer

1. **La synchronisation du lancement au démarrage avec le système n'a pas été observée** sur les trois systèmes. Le code appelle une routine dédiée (`commands/config.rs:45`, `autostart_migration::synchronize_for_settings`), mais ce que voit l'utilisateur — une entrée dans les Éléments d'ouverture de session sur macOS, dans le Gestionnaire des tâches sur Windows, un fichier `.desktop` sur Linux — n'a pas été vérifié. À confirmer avant d'écrire quoi que ce soit d'engageant.
2. **La description de la mascotte parle du bureau et des autres applications**, ce qui suppose une fenêtre sans bordure toujours au premier plan. Le comportement exact sur un bureau Linux sans composition, ou sur un Mac en plein écran, n'a pas été vérifié.
3. **Le comportement du choix « Système »** quand l'utilisateur change le mode clair/sombre de son système pendant que Beaver tourne : le code écoute la préférence du système, mais la bascule en direct n'a pas été observée.
4. **Le stockage dans le navigateur intégré peut être vidé** par le système ou par une réinstallation. Ce que voit l'utilisateur quand cela arrive — retour au thème par défaut, retour à l'anglais ? — mérite d'être vérifié avant d'écrire l'encadré correspondant. La valeur de repli du thème et de la langue n'a pas été relue.
5. **Affichage non vérifié — liste de contrôle pour la passe d'interface finale** : la grille des sept vignettes de thème et son rendu en fenêtre étroite ; l'aperçu de coloration de code sous la carte apparence ; l'interrupteur grisé de « Démarrage masqué », et le fait qu'il est visiblement désactivé et non simplement éteint ; l'aperçu animé de la mascotte et sa mention « Aperçu animé » / « Animation en pause » ; la lisibilité des touches affichées dans les deux thèmes.
