# Terminal intégré

**Emplacement site** — Interface › Terminal intégré (ou Outils › Fichiers et terminal dans le regroupement du mockup)
**Répond à** — « Comment j'ouvre un terminal dans Beaver, quel shell, et quelles limites ? »
**Sources** — `src-tauri/src/services/terminal/mod.rs` (lignes 39-75), `pty_session.rs` (lignes 30-124), `src-tauri/src/commands/terminal.rs`, `src/components/terminal/`, `src/hooks/use-agent-local-shortcuts.ts`
**Vérification** — Vérifié dans le code : limites, choix du shell et mécanisme d'authentification

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

Détail à mentionner pour les utilisateurs de zsh : Beaver neutralise le passage en mode vi que déclenche une variable `EDITOR` contenant « vi ». Sans cela, le terminal se retrouverait dans un mode d'édition déroutant.

### 4. Les onglets

- Le terminal est **multi-onglets**.
- Les onglets ouverts sont conservés entre deux lancements, dans `terminal-tabs.json`.

### 5. Les limites

| Limite | Valeur |
|---|---|
| Terminaux ouverts simultanément | **16** |
| Taille d'une écriture | **65 536 octets** |

Au-delà de seize terminaux, l'ouverture est refusée avec un message explicite. La limite est volontaire : chaque terminal est un processus système, et une application qui en ouvre sans compter finit par épuiser les ressources de la machine.

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
| Terminaux simultanés | 16 |
| Écriture maximale | 65 536 octets |
| Onglets conservés | Oui, dans `terminal-tabs.json` |
| Répertoire initial | Répertoire de travail de la conversation |

---

## Encadrés

**Encadré « Deux terminaux différents »**
> Le terminal intégré est le vôtre : l'agent ne voit pas ce que vous y tapez. Les commandes que l'agent exécute passent par son propre outil, soumis aux permissions.

**Encadré « Seize au maximum »**
> Beaver n'ouvre pas plus de seize terminaux à la fois. Chacun est un processus système ; la limite protège les ressources de votre machine.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| ⌘J ne fait rien | Pas de conversation active, ou curseur dans un champ de saisie | Cliquer hors du champ, ou ouvrir une conversation |
| « Trop de terminaux ouverts » | Seize terminaux déjà actifs | Fermer des onglets |
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
- **Le shell sous Windows.** `powershell.exe` est lancé en dur. Vérifier s'il existe un moyen de préférer `cmd.exe` ou PowerShell 7, et sinon le dire.
- **La limite de seize est-elle globale ou par conversation ?** Le gestionnaire semble global. À confirmer, la formulation en dépend.
- **Le jeton d'authentification des sessions.** Chaque session reçoit un jeton vérifié à chaque écriture. Détail interne, sans intérêt pour l'utilisateur, mais à mentionner dans la page *Sécurité › Durcissement*.
