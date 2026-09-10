# Terminal intégré

**Emplacement site** — Interface › Terminal intégré (ou Outils › Fichiers et terminal dans le regroupement du mockup)
**Répond à** — « Comment j'ouvre un terminal dans Beaver, quel shell, et quelles limites ? »
**Sources** — `src-tauri/src/services/terminal/manager.rs:35`, `terminal/limits.rs:1-7`, `terminal/shell_environment.rs:22` et `:28-35`, `terminal/shell_environment_tests.rs:50-55`, `terminal/shell_helper.rs:72-74`, `terminal/pty_session_windows.rs:161-164`, `terminal/tab_store.rs:181`, `terminal/mod.rs:92-95`, `src-tauri/src/commands/terminal.rs`, `src/components/terminal/`, `src/hooks/use-agent-local-shortcuts.ts`
**Vérification** — Vérifié dans le code : les deux limites de seize, le choix du shell par système, les variables d'environnement imposées et retirées, et le mécanisme d'authentification

---

## Plan de page proposé

1. À quoi il sert
2. Ouvrir un terminal
3. Quel shell est lancé
4. Les onglets
5. Les limites
6. Terminal de l'utilisateur et commandes de l'agent

---

## Contenu

### 1. À quoi il sert

Un vrai terminal, dans l'application, rattaché au répertoire de travail de la conversation. Pas une console de sortie : un shell interactif complet.

Il évite l'aller-retour vers une application externe quand on veut vérifier soi-même ce que l'agent vient de faire.

### 2. Ouvrir un terminal

- Raccourci **⌘J** sur macOS, **Ctrl+J** sur Windows et Linux.
- **Si aucun terminal n'a jamais été ouvert dans cette conversation**, le raccourci en crée un dans le répertoire de travail. Sinon, il bascule simplement l'affichage.
- Le raccourci est **sans effet** hors d'une conversation, ou quand le curseur est dans un champ de saisie.

### 3. Quel shell est lancé

| Système | Shell |
|---|---|
| macOS et Linux | Le shell défini par la variable `SHELL`, ou `/bin/bash` à défaut |
| Windows | `powershell.exe` |

Sur macOS et Linux, le chemin du shell est validé avant lancement : un chemin invalide est refusé plutôt que d'être exécuté.

**Ce que Beaver change à l'environnement du shell** — une autorité unique s'en charge (`src-tauri/src/services/terminal/shell_environment.rs`), et c'est le point qui mérite le paragraphe :

- Elle **impose** `TERM=xterm-256color` et `COLORTERM=truecolor` (`:22`).
- Elle **retire** six variables posées par le lanceur de l'application : `NO_COLOR`, `NODE_DISABLE_COLORS`, `FORCE_COLOR`, `COLOR`, `CLICOLOR`, `CLICOLOR_FORCE` (`:28-35`).

Le motif est écrit dans le fichier : sans cela, un lanceur qui désactive la couleur rendait la sortie de `vite`, `cargo` et de la ligne de commande Tauri uniformément grise dans le terminal intégré.

`EDITOR`, en revanche, est **transmise telle quelle** : Beaver n'y touche pas, et un test le verrouille (`src-tauri/src/services/terminal/shell_environment_tests.rs:50-55`).

### 4. Les onglets

- Le terminal est **multi-onglets**.
- Les onglets ouverts sont conservés entre deux lancements, dans `terminal-tabs.json`.

### 5. Les limites

**Deux limites différentes portent la valeur 16** : ne pas les confondre.

| Limite | Valeur | Portée | Source |
|---|---|---|---|
| Processus de terminal vivants | **16** | **Globale**, toutes conversations confondues | `terminal/manager.rs:35` (`MAX_PTY_SESSIONS`) |
| Onglets enregistrés par groupe | **16** | Par conversation | `terminal/limits.rs:6` (`MAX_TABS_PER_GROUP`) |
| Groupes d'onglets | **128** | Globale | `terminal/limits.rs:5` |
| Onglets enregistrés au total | **256** | Globale | `terminal/limits.rs:7` |
| Taille d'une écriture | **65 536 octets** | Par écriture | `terminal/limits.rs:1` |

Au-delà de seize **processus**, l'ouverture est refusée avec un message explicite. La limite est volontaire : chaque terminal vivant est un processus système, et une application qui en ouvre sans compter finit par épuiser les ressources de la machine. Un onglet enregistré mais dont le processus n'est pas lancé ne consomme pas ce quota.

### 6. Terminal de l'utilisateur et commandes de l'agent

Distinction à faire nettement, parce qu'elle est source de confusion :

- **Le terminal intégré** est le vôtre. Ce que vous y tapez n'est pas vu par l'agent.
- **L'outil `bash`** est celui de l'agent. Ses commandes s'exécutent séparément, avec le contrôle des permissions.

Les deux partagent le répertoire de travail de la conversation, mais ce sont deux mécanismes distincts.

---

## Tableaux

### Tableau — Récapitulatif

| | Valeur |
|---|---|
| Raccourci | ⌘J / Ctrl+J |
| Shell sur macOS et Linux | `$SHELL`, sinon `/bin/bash` |
| Shell sur Windows | `powershell.exe` |
| Processus de terminal simultanés | **16**, globalement |
| Onglets par conversation | **16**, et **256** au total |
| Écriture maximale | **65 536 octets** |
| Onglets conservés | Oui, dans `terminal-tabs.json` |
| Répertoire initial | Répertoire de travail de la conversation |

---

## Encadrés

**Encadré « Deux terminaux différents »**
> Le terminal intégré est le vôtre : l'agent ne voit pas ce que vous y tapez. Les commandes que l'agent exécute passent par son propre outil, soumis aux permissions.

**Encadré « Seize au maximum »**
> Beaver n'ouvre pas plus de seize terminaux à la fois, toutes conversations confondues. Chacun est un processus système ; la limite protège les ressources de votre machine.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| ⌘J ne fait rien | Pas de conversation active, ou curseur dans un champ de saisie | Cliquer hors du champ, ou ouvrir une conversation |
| « La limite de terminaux actifs est atteinte. Fermez un onglet pour réessayer. » (`terminal.liveLimitReached`, fr.json:1237 — corrigé le 10 septembre 2026 : l'ancienne paraphrase « Trop de terminaux ouverts » n'existe pas dans l'application) | Seize processus de terminal déjà actifs, toutes conversations confondues | Fermer des onglets, y compris dans d'autres conversations |
| Le terminal démarre dans le mauvais dossier | Il suit le répertoire de travail de la conversation | Changer le répertoire de travail de la conversation |
| Le shell n'est pas celui attendu | La variable `SHELL` n'est pas celle du terminal habituel | Vérifier `SHELL` dans l'environnement d'où l'application est lancée |
| Un collage volumineux est tronqué | Écriture plafonnée à 65 536 octets | Passer par un fichier |

---

## Renvois

- *Interface › Raccourcis clavier*
- *Agent › Répertoire de travail*
- *Outils › Terminal et shell* — l'outil `bash` de l'agent
- *Référence › Limites et quotas*

---

## Points à confirmer

- **Le comportement au changement de répertoire de travail** alors qu'un terminal est déjà ouvert : suit-il, ou reste-t-il où il était ?
- **La restauration des onglets au lancement.** `terminal-tabs.json` conserve les onglets, mais les processus ne survivent évidemment pas à la fermeture. Vérifier ce qui est réellement restauré : les onglets vides, le répertoire, l'historique ?
- ~~Le shell sous Windows.~~ **Tranché** : `powershell.exe`, résolu par un chemin système validé (`terminal/pty_session_windows.rs:161-164`). **Aucun réglage** ne permet de préférer `cmd.exe` ou PowerShell 7 ; le dire franchement sur le site.
- ~~La limite de seize est-elle globale ou par conversation ?~~ **Tranché** : **seize processus au total** (`terminal/manager.rs:35`), seize onglets par conversation et deux cent cinquante-six onglets au total (`terminal/limits.rs:5-7`).
- **Le jeton d'authentification des sessions.** Chaque session reçoit un jeton vérifié à chaque écriture. Détail interne, sans intérêt pour l'utilisateur, mais à mentionner dans la page *Sécurité › Durcissement*.
