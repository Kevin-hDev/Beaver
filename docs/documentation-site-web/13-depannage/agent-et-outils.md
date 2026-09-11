# Quand l'agent s'arrête ou refuse d'agir

**Emplacement site** — Dépannage › L'agent et ses outils
**Répond à** — « L'agent a refusé une action, s'est arrêté tout seul, ou un outil a échoué : qu'est-ce qui s'est passé et que puis-je faire ? »
**Sources** — `src-tauri/src/services/agent_local/` : `permission_gate.rs`, `permission_policy.rs`, `permission_bash.rs`, `permission_request.rs`, `security.rs`, `directory_access.rs`, `write_guard.rs`, `tool_executor_write.rs`, `tool_executor_errors.rs`, `tool_dispatcher_shell_error.rs`, `tool_result_contract.rs`, `tool_bash.rs`, `tool_definitions_chat.rs`, `circuit_breaker.rs`, `agent_loop_limits.rs`, `agent_loop_errors.rs`, `agent_loop_support.rs`, `subagent_turn_limit.rs`, `subagent_completion.rs`, `subagent_panic_supervisor.rs`, `subagent_task_failure.rs`, `subagent_tool_guard.rs`, `context_capacity_error.rs`, `stream_diagnostics.rs`, `stream_diagnostics_failure.rs`, `stream_diagnostics_support.rs`, `agent_settings.rs` ; `src-tauri/src/commands/agent_chat_run_spawn.rs` ; `src/lib/agent-error-codes.ts`, `src/lib/tool-error-message.ts`, `src/lib/agent-session-failure.ts`, `src/lib/app-error.ts` ; `src/hooks/agent-chat-stream-callbacks.ts`, `src/hooks/agent-chat-restored-failure.ts`, `src/hooks/agent-chat-stream-finalize.ts`, `src/hooks/agent-context-capacity-error.ts` ; `src/components/agent-local/permission-dialog.tsx` ; `src/i18n/fr.json`
**Vérification** — Vérifié dans le code, ligne par ligne, le 10 septembre 2026. Aucun état n'a été provoqué à l'écran : les messages sont documentés d'après le code, avec leur source, conformément à la convention de ce dossier.

> **Ce qui n'est pas ici.** Les erreurs de clés API et de fournisseurs sont dans `13-depannage/providers-et-cles.md` ; les pannes du moteur Ollama dans `13-depannage/ollama.md` ; le mode Plan et la compression du contexte ont leurs propres pages. Cette page-ci couvre ce qui se passe entre la demande et l'outil.

---

## Plan de page proposé

1. Les trois endroits où lire un échec
2. Pourquoi Beaver vous demande l'autorisation
3. « Autoriser à … ? » — la fenêtre de demande
4. Une action refusée : ce que voit l'agent, ce que vous voyez
5. Une commande shell bloquée d'office
6. « Écriture bloquée : fichier non lu avant modification »
7. Un fichier hors des zones autorisées
8. Le contexte obligatoire dépasse la capacité du modèle
9. L'agent a atteint sa limite de tours
10. L'agent tournait en rond
11. Un sous-agent en échec
12. Où sont écrits les diagnostics

---

## Contenu

### 1. Les trois endroits où lire un échec

C'est la première chose à écrire sur la page, parce que la suite en dépend : **Beaver ne met pas tous les échecs au même endroit.**

| Où | Ce qu'on y trouve | Durée de vie |
|---|---|---|
| **Sous le bloc d'un outil**, dans la conversation | L'échec d'**un outil** : une phrase par catégorie d'erreur, plus un conseil quand le code en fournit un | Reste dans la conversation |
| **Sous la réponse**, quand la génération s'arrête | L'échec du **flux entier** : une phrase, plus un résumé technique court | Le résumé technique **disparaît** au rechargement de la conversation (`agent-chat-restored-failure.ts:19-20`) |
| **`~/.local/share/cl-go-dash/logs/permission-diagnostics.jsonl`** | Chaque demande d'autorisation et chaque réponse | Fichier tourné à **2 Mio** (`permission_gate.rs:105`, `:117-123`) |

Beaver garde en mémoire de session les **20 derniers échecs de flux** et les **20 derniers diagnostics d'exécution** (`stream_diagnostics_support.rs:8-9`), enregistrés dans le fichier de la conversation.

### 2. Pourquoi Beaver vous demande l'autorisation

Trois modes, choisis par conversation, changés au clavier avec **Shift+Tab** (`permissionMode.toggleHint`, `fr.json:552`) :

| Libellé affiché | Ce que ça change |
|---|---|
| **Chatbot** | L'agent ne dispose que de **deux outils** : `web_search` et `web_fetch` (`tool_definitions_chat.rs:3-5`, test `:12-20`). Aucun accès aux fichiers |
| **Accès complet** | Aucune demande d'autorisation n'est posée (`permission_policy.rs:43-45`) |
| **Demander l'autorisation** | Une fenêtre s'ouvre avant chaque action classée sensible |

Un quatrième mode, `subagent`, existe uniquement en interne : il s'applique aux sessions enfants et contourne la garde de permission exactement comme **Accès complet** (`permission_policy.rs:43-45`). Il n'est jamais proposé à l'utilisateur.

**Ce qui déclenche une demande en mode « Demander l'autorisation »** (`permission_gate.rs:68-81`, `:83-102`) :

| Outil | Condition |
|---|---|
| `write_file`, `edit_file`, `web_fetch`, `write_spreadsheet`, `write_document`, `create_branch`, `checkout_branch`, `apply_subagent_changes`, `manage_automation`, `forecast_data_audit`, `forecast_run`, `forecast_backtest` | **Toujours** |
| `bash` | Seulement si la commande n'est pas reconnue comme sûre |
| `bash_control` | Si des caractères sont envoyés au processus |
| `transform_image` | Sauf liste d'opérations vide |
| `search_mcp_tools` | Seulement en mode `call`, c'est-à-dire quand l'agent exécute réellement un outil MCP |
| Outil d'extension | Selon l'effet déclaré — voir le tableau des effets |

**Les vingt commandes considérées comme sûres** (`permission_bash.rs:4-30`) : `ls`, `cat`, `head`, `tail`, `wc`, `grep`, `find`, `git status|log|diff|show|remote|tag`, `git branch` seul, `pwd`, `echo`, `which`, `cargo check|test|clippy|build`, `npx tsc`, `npm run|test`, `tree`, `file`, `stat`, `du`, `df`.

**Une commande sûre redevient sensible dès qu'elle contient un opérateur de shell** — `;`, `&&`, `||`, `|`, `` ` ``, `$(`, un retour à la ligne, `<(`, `>(`, `<<`, `>`, `$'`, `&`, `<` (`permission_bash.rs:42-58`). C'est la raison la plus fréquente d'une demande sur un `grep` en apparence anodin : le tube en fait une commande composée.

**Les effets d'extension et ce qu'ils autorisent** (`permission_policy.rs:13-41`) :

| Effet | Libellé affiché | Demande une confirmation | Bouton « Toujours autoriser » |
|---|---|---|---|
| `read-only` | Lecture seule | Non | — |
| `external-read` | Lecture externe | Oui | Oui |
| `local-write` | Écriture locale | Oui | Oui |
| `external-write` | Écriture externe | Oui | Oui |
| `process` | Exécution d'un processus | Oui | **Non** |
| `secret` | Données sensibles | Oui | **Non** |
| `unknown` | Action non classée | Oui | **Non** |

Libellés : `permissionDialog.effects.*` (`fr.json:572-578`). Les trois derniers effets ne proposent jamais « Toujours autoriser » : chaque appel repose la question.

### 3. « Autoriser à … ? » — la fenêtre de demande

Titre exact : **« Autoriser à {{action}} ? »** (`permissionDialog.title`, `fr.json:569`). Trois boutons : **« Refuser »** (avec l'indication `esc`), **« Autoriser une fois »**, **« Toujours autoriser »** (`fr.json:580-582` ; `permission-dialog.tsx:59-87`).

La touche **Échap refuse** (`permission-dialog.tsx:30-32`).

Quatre outils seulement ont un libellé traduit dans le titre : « écrire un fichier », « modifier un fichier », « exécuter une commande », « récupérer une URL » (`permissionDialog.tools.*`, `fr.json:584-587`). Pour les autres, le titre affiche le **nom technique de l'outil** — voir « Anomalies relevées ».

**Deux cas où aucune fenêtre n'apparaît alors qu'on l'attendrait :**

- **Plus de 64 demandes en attente** : la 65ᵉ est refusée sans fenêtre (`permission_gate.rs:104`, `:157-159`).
- **La réponse est annulée pendant l'attente** : la demande est retirée et vaut refus (`permission_gate.rs:181-184`).

### 4. Une action refusée : ce que voit l'agent, ce que vous voyez

Quand vous refusez, l'agent reçoit un résultat d'erreur portant le message **« L'utilisateur a refusé cette action. »**, le code `user_denied_tool` et la catégorie *permission* (`tool_executor_errors.rs:10-16`).

Vous, vous lisez la phrase de la catégorie : **« Cette opération n'est pas autorisée. »** (`fr.json:467`, via `tool-error-message.ts:10`, `:50-51`).

**Les dix catégories d'erreur d'outil** et leur phrase (`tool_result_contract.rs:39-51` ; `fr.json:466-475`) :

| Catégorie | Phrase affichée |
|---|---|
| `validation` | La demande transmise à l'outil est invalide. |
| `permission` | Cette opération n'est pas autorisée. |
| `not_found` | La ressource demandée est introuvable. |
| `conflict` | L'état actuel empêche cette opération. |
| `timeout` | L'outil n'a pas répondu à temps. |
| `cancelled` | L'opération a été annulée. |
| `unavailable` | L'outil est temporairement indisponible. |
| `external` | Un service externe a empêché l'opération. |
| `execution` | L'outil n'a pas pu terminer l'opération. |
| `internal` | Une erreur interne a empêché l'opération. |

**Le conseil sous l'erreur.** Certains échecs ajoutent une phrase d'orientation. La plus fréquente : **« Vérifiez l'état obtenu avant de relancer : l'action a peut-être déjà été exécutée. »** (`fr.json:464`). Côté commandes shell, trois cas la portent explicitement, avec une formulation propre au code : délai dépassé, sortie indisponible, échec non classé (`tool_dispatcher_shell_error.rs`, respectivement `:22-24`, `:88-90`, `:120-122`).

**Un résultat d'outil manquant.** Si le flux se termine alors qu'un outil n'a pas rendu son résultat, Beaver écrit **« Le résultat de l'outil manque après la fin du flux. »** avec le code `tool_result_missing` (`agent-chat-stream-finalize.ts:134-146` ; `fr.json:462`). Ce n'est pas une panne de l'outil : c'est le flux qui s'est arrêté avant lui.

### 5. Une commande shell bloquée d'office

Certaines commandes ne sont **pas soumises à votre autorisation : elles sont refusées** (`security.rs:43-62`, appelé depuis `tool_bash.rs:39` et `:94`). Le refus vaut dans tous les modes, y compris **Accès complet**.

**Les quinze motifs interdits, mot pour mot** (`security.rs:5-21`) : `sudo rm`, `chmod 777`, `dd if=`, `mkfs.`, `> /dev/sd`, `fdisk`, `shutdown`, `reboot`, `init 0`, `init 6`, `:(){:|:&};:`, `del /f /s /q`, `rd /s /q`, `format c:`, `format d:`.

**Quatre formes supplémentaires détectées par motif** (`security.rs:23-29`, `:53-58`) : `eval "$…`, un `find` avec `-delete`, un `rsync` avec `--delete`, un `dd` visant `of=/dev/`, et `mkfs ` isolé.

Messages exacts : **« Commande bloquée : pattern dangereux « … » »** (`security.rs:48-51`) ou **« Commande bloquée : pattern dangereux détecté »** (`security.rs:59`).

Ce sont des messages en dur, en français, écrits pour l'agent — ils ne passent pas par le système de traduction (voir « Anomalies relevées »).

### 6. « Écriture bloquée : fichier non lu avant modification »

Message exact : **« Écriture bloquée : fichier non lu avant modification. Utilise read_file sur ce chemin d'abord. »** (`write_guard.rs:48`), code `write_guard_rejected`, catégorie *permission* (`tool_executor_write.rs:113-119`).

C'est une protection contre l'écrasement à l'aveugle : Beaver n'autorise la modification d'un fichier **existant** que si l'agent l'a lu pendant la conversation. Un fichier qui n'existe pas encore n'est pas concerné (`write_guard.rs:44-46`).

**La mémoire des fichiers lus est bornée à 1 000 chemins**, et l'ajout du 1 001ᵉ efface les **100 plus anciens** (`write_guard.rs:3-4`, `:29-32`). Sur une très longue conversation touchant beaucoup de fichiers, un fichier lu au début peut donc redemander une lecture. C'est le comportement attendu, pas une panne.

**Ce que l'utilisateur a à faire : rien.** L'agent relit le fichier et repart. La seule situation à signaler est la répétition en boucle, traitée à la section 10.

### 7. Un fichier hors des zones autorisées

Deux messages, selon le sens de l'opération (`security.rs:133`, `:192`) :

- **« Lecture interdite hors des zones autorisées »**
- **« Écriture interdite hors des zones autorisées »**

Les zones autorisées sont, dans l'ordre où le code les assemble (`security.rs:31-41`, `:64-89`) : les dossiers configurés dans les réglages, le dossier de sortie de session s'il est configuré, le dossier temporaire du système, le dossier de travail de la conversation, et les ressources propres à l'agent — `memory/`, `skills/` et les fichiers d'instructions (`agent_resource_access.rs:33`, `:44-47`).

**Côté interface**, le refus d'un dossier de session porte un autre libellé : **« Dossier non autorisé »**, suivi de « Les sessions sont limitées aux dossiers suivants : » et de **« Pour utiliser un autre dossier, modifiez Réglages > Avancé > Accès fichiers. »** (`fr.json:562-564`). C'est là que se règle la liste.

**Deux bornes** (`directory_access.rs:7`, `:9`) : **70 chemins autorisés** au maximum, **4 096 caractères** par chemin. Au-delà, le réglage est refusé (`directory_access.rs:7-10`). Corrigé le 10 septembre 2026 : ne plus citer « Accès au dossier refusé par les réglages. » — cette chaîne interne n'a jamais été tracée jusqu'à une ligne de rendu (l'écran de session passe par `directoryAccess.*`), on énonce le refus sans citer de message.

**Un lien symbolique n'est jamais suivi** pour les ressources de l'agent : un `memory/` qui pointe ailleurs est simplement ignoré (`agent_resource_access.rs:63-80`, test `:126-139`).

### 8. Le contexte obligatoire dépasse la capacité du modèle

C'est le seul message d'erreur de Beaver qui **donne ses chiffres**. Quatre variantes, choisies selon deux questions : la fenêtre du modèle est-elle connue, et des rapports de sous-agents sont-ils obligatoires (`agent-context-capacity-error.ts:12-19`).

La plus complète (`fr.json:1375`) :

> « Le contexte obligatoire dépasse la capacité du modèle : prompt système {{systemTokens}} tokens + rapports obligatoires {{reportTokens}} tokens + outils actifs {{toolTokens}} tokens = {{requiredTokens}} tokens, pour une limite d'entrée de {{maxInputTokens}} tokens sur une fenêtre de {{contextWindow}} tokens. »

Quand la fenêtre réelle du modèle n'a pas pu être déterminée, la phrase se termine par « La fenêtre réelle du modèle n'a pas pu être déterminée. » et parle d'une **cible de réduction** plutôt que d'une capacité (`fr.json:1372-1373`).

**Ce que dit exactement le calcul.** Ce qui dépasse n'est pas la conversation : c'est ce que Beaver doit envoyer **avant même le premier message** — les instructions système, les rapports de sous-agents à remettre, et la description de tous les outils actifs. La conversation n'entre pas dans la somme.

**Les trois termes sont les trois leviers**, et c'est tout ce que le calcul permet d'affirmer : moins d'outils actifs (connecteurs MCP et extensions en portent l'essentiel), pas de rapport de sous-agent en attente, ou un modèle à plus grande fenêtre. **L'application ne propose aucune action dans ce message** ; elle affiche les chiffres et s'arrête.

**Les chiffres affichés sont vérifiés avant affichage**, des deux côtés : la somme doit être exacte, chaque valeur inférieure à **16 777 216**, et la limite d'entrée inférieure ou égale à la fenêtre (`context_capacity_error.rs:41-58` ; `agent-context-capacity-error.ts:30-45`). Si un compteur est incohérent, Beaver n'affiche **aucun chiffre** plutôt qu'un chiffre faux (`context_capacity_error.rs:96-101`).

### 9. L'agent a atteint sa limite de tours

**Un tour = un aller-retour modèle puis outils.** La limite est de **200 tours** par réponse (`agent_loop_limits.rs:1`).

Message technique produit : **« Limite de tours agent atteinte (200). »** (`agent_loop_errors.rs:1-6`). L'arrêt est décidé au tour **199**, et Beaver en profite pour décharger le modèle de la mémoire graphique avant de rendre la main (`agent_loop_support.rs:138-145`).

**Le même message sert dans un second cas**, sans rapport avec les 200 tours : quand un sous-agent est encore actif, qu'un rapport n'a pas été remis, ou qu'une correction reste en file (`subagent_turn_limit.rs:14-27`). Le tour est refusé pour laisser l'enfant finir.

**La phrase prévue pour l'utilisateur** est « L'agent a atteint sa limite de tours. Continue dans un nouveau message. » (`errors.maxTurns`, `fr.json:1422`) — mais elle ne s'affiche pas au moment de l'arrêt. Voir « Anomalies relevées », point 1.

### 10. L'agent tournait en rond

Beaver compte les appels d'outils **strictement identiques** qui se suivent : même nom d'outil, mêmes arguments, une fois les objets JSON remis dans un ordre stable (`circuit_breaker.rs:35-60`). Au **sixième d'affilée**, il arrête la réponse (`circuit_breaker.rs:1`, `:21-25`).

Message technique produit : **« Circuit breaker : 6 appels identiques consécutifs détectés. Boucle probable, arrêt. »**

**Un seul argument qui change remet le compteur à un** (`circuit_breaker.rs:27-30`). Une boucle où l'agent relit le même fichier avec un décalage différent à chaque fois ne sera donc pas coupée — c'est la limite des 200 tours qui s'en charge.

**La phrase prévue pour l'utilisateur** est « L'agent s'est arrêté après plusieurs échecs. Vérifie la dernière action puis réessaye. » (`errors.circuitBreaker`, `fr.json:1423`) — elle ne s'affiche jamais. Voir « Anomalies relevées », point 2.

### 11. Un sous-agent en échec

Quatre états affichés sous la bulle des sous-agents (`fr.json:2015-2018`) : **« terminé »**, **« échoué »**, **« annulé »**, **« interrompu »**.

**Trois messages internes distinguent trois pannes**, tous en français en dur :

| Situation | Message | Source |
|---|---|---|
| Le sous-agent s'est effondré en cours de route | « Le sous-agent n'a pas pu terminer correctement. » | `subagent_panic_supervisor.rs:5` |
| Sa préparation a échoué avant même de démarrer | La même phrase | `subagent_task_failure.rs:11` |
| Il a travaillé mais n'a pas pu remettre son rapport | « Le sous-agent n'a pas pu finaliser son rapport. » | `subagent_completion.rs:7-8` |
| Un outil lui est interdit | « Outil indisponible pour ce sous-agent. » | `subagent_tool_guard.rs:7` |

**Un effondrement est rattrapé, pas propagé** : la panique est capturée dans la tâche qui possède le sous-agent, et une routine de reprise marque l'état terminal (`subagent_panic_supervisor.rs:7-18`, `:20-28`). Pour un sous-agent de type *coder*, Beaver tente d'abord de récupérer les modifications déjà faites dans son espace de travail avant de conclure (`subagent_panic_supervisor.rs:33-52`).

**Ce que vous ne pouvez pas faire** : écrire dans la conversation d'un sous-agent. Toute tentative renvoie **« Cette session de sous-agent est en lecture seule. »** (`fr.json:1349`).

**À dire clairement sur le site** : un sous-agent hérite du mode `subagent`, qui **contourne la garde de permission** (`permission_policy.rs:43-45`). En mode « Demander l'autorisation », les actions du sous-agent ne vous sont donc pas soumises. C'est un choix, pas un défaut, mais il doit être écrit.

### 12. Où sont écrits les diagnostics

**Le journal des autorisations** : `~/.local/share/cl-go-dash/logs/permission-diagnostics.jsonl` (`permission_gate.rs:114-116`). Une ligne JSON par événement, quatre champs — horodatage, événement, outil, détail (`permission_gate.rs:139-144`). Le fichier est renommé en `.jsonl.1` au-delà de **2 Mio** (`permission_gate.rs:105`, `:117-123`) : une seule génération est conservée.

Les événements écrits :

| Événement | Ce qu'il signifie |
|---|---|
| `request` + détail `permission_prompt_sent` | Une fenêtre a été envoyée à l'interface (`permission_gate.rs:177`) |
| `respond_found` + `allow` / `allow_session` / `deny` | Votre réponse a été reçue (`permission_gate.rs:190-196`) |
| `respond_missing` + `stale_or_unknown_permission` | Une réponse est arrivée pour une demande qui n'existait plus (`permission_gate.rs:198`) |
| `auto_bypass` | Le mode ne demande pas d'autorisation (`tool_executor_write.rs:93`) |
| `memory_write_authorized` | Une écriture en mémoire autorisée par la politique dédiée (`tool_executor_write.rs:87-91`) |

**Le résumé technique sous une réponse interrompue** — par exemple « Interruption pendant le tool read_file (stream_error). » ou « Limite de tours agent atteinte après le dernier tool bash. » — est construit dans `stream_diagnostics_failure.rs:59-88`. Il est en **français en dur**, il n'est jamais traduit, et il **disparaît quand vous rouvrez la conversation** (`agent-chat-restored-failure.ts:19-20`). Si vous voulez le garder, il faut le lire ou le copier avant de changer d'onglet.

**Pas de journal Ollama.** Le moteur n'écrit aucune trace — voir `13-depannage/ollama.md`.

---

## Encadrés

> **ℹ Un tube change tout.**
> `grep -r motif .` ne demande aucune autorisation ; `grep -r motif . | head` en demande une. Dès qu'une commande contient `|`, `&&`, `;`, `>` ou une substitution, elle sort de la liste des commandes sûres et repasse par vous.

> **⚠ Certaines commandes ne vous sont jamais proposées.**
> Quinze motifs — `sudo rm`, `mkfs.`, `format c:`, `shutdown`… — sont refusés avant même la question, dans tous les modes, y compris **Accès complet**. Ce n'est pas un réglage : c'est un refus.

> **⚠ Un sous-agent ne vous demande rien.**
> Les sessions enfants tournent en mode interne `subagent`, qui contourne la garde de permission. En mode « Demander l'autorisation », les actions du parent vous sont soumises ; celles de ses sous-agents, non.

> **ℹ « Écriture bloquée : fichier non lu » n'est pas une erreur.**
> C'est une protection contre l'écrasement à l'aveugle. L'agent relit le fichier et repart tout seul. Vous n'avez rien à faire.

> **ℹ Le résumé technique d'une interruption ne survit pas au rechargement.**
> Il s'affiche sous la réponse au moment de l'échec, puis disparaît quand vous rouvrez la conversation. Copiez-le si vous voulez le signaler.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause vérifiée | Ce qu'on peut faire |
|---|---|---|
| « Beaver me demande l'autorisation pour un simple `ls` » | La commande contient un opérateur de shell (`permission_bash.rs:42-58`) | Comportement attendu. Rien à corriger |
| « Rien ne se passe, aucune fenêtre n'apparaît » | Plus de **64 demandes** en attente : les suivantes sont refusées sans fenêtre (`permission_gate.rs:104`, `:157-159`) | Arrêter la réponse, relancer |
| « La fenêtre affiche un nom bizarre : `forecast_run` » | Quatre outils seulement ont un libellé traduit (`fr.json:584-587`) | Cosmétique. Voir « Anomalies relevées », point 3 |
| « Cette opération n'est pas autorisée. » | Refus de votre part, ou garde d'écriture, ou outil interdit en mode Plan (`tool_executor_write.rs:107-135`) | Le détail exact n'est pas affiché : il faut relire le contexte de l'outil |
| « Commande bloquée : pattern dangereux » | Un des quinze motifs interdits (`security.rs:5-21`) | Reformuler la commande. Aucun réglage ne le débloque |
| « Lecture / Écriture interdite hors des zones autorisées » | Chemin hors des dossiers configurés (`security.rs:133`, `:192`) | Réglages > Avancé > Accès fichiers |
| « Dossier non autorisé » à l'ouverture d'une conversation | Le dossier de travail n'est pas dans la liste (`fr.json:562-564`) | Ajouter le dossier dans les réglages |
| « Le contexte obligatoire dépasse la capacité du modèle » | Instructions + rapports + outils actifs dépassent la limite d'entrée avant tout message (`fr.json:1374-1375`) | Désactiver des connecteurs ou extensions, ou changer de modèle. **L'application ne le suggère pas** |
| « La réponse s'arrête sans explication après un long travail » | 200 tours atteints (`agent_loop_limits.rs:1`) — mais la phrase affichée est générique | Relancer dans un nouveau message. Voir « Anomalies relevées », point 1 |
| « L'agent a répété six fois la même chose puis s'est arrêté » | Coupe-circuit à **6 appels identiques** (`circuit_breaker.rs:1`) | Reformuler la demande : la répétition à l'identique signale que l'agent n'a pas d'information nouvelle |
| « L'agent répète la même action sans jamais s'arrêter » | Un argument change à chaque fois : le coupe-circuit ne se déclenche pas (`circuit_breaker.rs:27-30`) | Arrêter la réponse à la main |
| « Le résultat de l'outil manque après la fin du flux. » | Le flux s'est terminé avant que l'outil rende son résultat (`agent-chat-stream-finalize.ts:134-146`) | Vérifier l'état obtenu avant de relancer : l'action a peut-être eu lieu |
| Sous-agent « échoué » sans détail | Effondrement, préparation ratée ou rapport non remis — trois causes, deux messages (`subagent_panic_supervisor.rs:5` ; `subagent_completion.rs:7-8`) | Relire le résumé du sous-agent avant de rouvrir la conversation |
| « Je ne peux pas écrire dans la conversation d'un sous-agent » | Session en lecture seule (`fr.json:1349`) | Écrire dans la conversation parente |

---

## Renvois

- `04-agent/permissions.md` — les trois modes en détail, et ce que chacun autorise
- `04-agent/sous-agents.md` — comment les sous-agents sont créés et supervisés
- `05-outils/vue-densemble.md` — la liste des outils et ce que chacun fait
- `10-reglages/avance.md` — le réglage « Accès fichiers » et la liste des dossiers autorisés
- `13-depannage/mcp-extensions-channels.md` — connecteurs, extensions et canaux externes
- `13-depannage/ollama.md` — les pannes du moteur local
- `13-depannage/providers-et-cles.md` — clés API et fournisseurs
- `11-securite/vault-et-cles-api.md` — pourquoi certaines commandes touchant des fichiers sensibles déclenchent une demande

---

## Anomalies relevées

Relevées en lisant le code, **non corrigées** — à arbitrer avec l'équipe avant de publier la page.

1. **La phrase « L'agent a atteint sa limite de tours » ne s'affiche jamais au moment de l'arrêt.** Le backend envoie à l'interface le **message français brut** — « Limite de tours agent atteinte (200). » (`agent_chat_run_spawn.rs:119-136`, qui transmet `public_message` tel quel). L'interface cherche ce texte comme **clé** dans son catalogue de codes (`agent-chat-stream-callbacks.ts:156-159`), ne le trouve pas, et retombe sur « Le flux s'est interrompu. » (`errors.streamInterrupted`, `fr.json:1369`). La phrase correcte n'apparaît qu'au **rechargement de la conversation**, où c'est le code enregistré (`max_turns`) qui est traduit (`agent-chat-restored-failure.ts:15` ; `agent-session-failure.ts:26-28`). Conséquence : le même incident produit deux messages différents selon qu'on regarde tout de suite ou après avoir rouvert la conversation.

2. **La phrase du coupe-circuit ne s'affiche jamais, ni tout de suite ni après rechargement.** La fonction de classement teste `message.contains("circuit")` **en respectant la casse**, sur le message brut (`stream_diagnostics_failure.rs:150-152`). Or le message produit commence par « **C**ircuit breaker : … » (`circuit_breaker.rs:22-25`). Le test échoue, le classement retombe sur `unknown`, puis sur `stream_error` (`stream_diagnostics_failure.rs:90-97`). L'utilisateur lit donc « Le flux de réponse a été interrompu. Réessaye. » (`fr.json:1425`) au lieu de « L'agent s'est arrêté après plusieurs échecs. » (`fr.json:1423`). **Aucun test ne couvre ce chemin** : `circuit` n'apparaît dans aucun test de `stream_diagnostics_failure_tests.rs`. La clé `errors.circuitBreaker` est donc traduite en sept langues et jamais atteinte.

3. **Le titre de la fenêtre d'autorisation affiche un nom technique pour huit outils sur douze.** Seuls `write_file`, `edit_file`, `bash` et `web_fetch` ont un libellé (`fr.json:584-587`) ; pour les autres, l'interface retombe sur le nom brut de l'outil (`permission-dialog.tsx:47`, paramètre `defaultValue`). Un utilisateur lit donc « Autoriser à forecast_backtest ? » ou « Autoriser à apply_subagent_changes ? ».

4. **Les messages de refus destinés à l'agent sont en français en dur.** « Commande bloquée : pattern dangereux… », « Lecture interdite hors des zones autorisées », « Écriture bloquée : fichier non lu avant modification… », « L'utilisateur a refusé cette action. », « Le sous-agent n'a pas pu terminer correctement. » — aucun ne passe par le catalogue de traductions. Ils sont adressés au modèle, pas à l'utilisateur, ce qui explique le choix ; mais le résumé technique affiché sous une réponse interrompue (`stream_diagnostics_failure.rs:59-88`) l'est bien, lui, et reste en français quelle que soit la langue de l'application.

5. **`src/lib/app-error.ts` ne couvre aucune erreur d'agent.** Malgré son nom, la liste des trente types d'erreur qu'il traduit est presque entièrement consacrée à Git — branches, commits, fusions, dépôt distant (`app-error.ts:37-81`). Le seul type qui pourrait prêter à confusion, `permission_denied`, désigne un refus du **dépôt distant** lors d'un envoi, pas un refus d'autorisation d'outil (`app-error.ts:74` → `fr.json:399`). À ne pas citer sur une page consacrée aux permissions de l'agent.

6. **En mode « Accès complet », une commande touchant des fichiers sensibles ne déclenche aucune demande.** Le contrôle dédié sort immédiatement dans ce mode (`permission_policy.rs:47-50`). C'est cohérent avec le nom du mode, mais mérite d'être écrit : la protection des fichiers sensibles n'est pas un filet indépendant, elle vit dans la garde de permission.

---

## Points à confirmer

**Écart de vocabulaire — à trancher avant publication**

1. **Le libellé du mode `manual` ne correspond pas à celui du guide de ce dossier.** `00-comment-utiliser-ces-fichiers.md:87` impose d'écrire **« Demande d'approbation »** ; l'application affiche **« Demander l'autorisation »** (`permissionMode.manualLabel`, `fr.json:550`). Le code fait foi selon la hiérarchie des sources, mais la consigne du guide est explicite. Il faut choisir, et corriger l'un des deux — pas les deux pages indépendamment.

**Non vérifié — hors de portée d'une lecture du code**

2. **Aucun de ces états n'a été provoqué à l'écran.** Provoquer un dépassement de capacité de contexte, une limite de 200 tours ou un effondrement de sous-agent demande de fabriquer la panne. Les faits viennent du code et sont sûrs ; l'enchaînement visible ne l'est pas.
3. **La position de la fenêtre d'autorisation dans la conversation** : elle porte `role="dialog"` avec `aria-modal="false"` (`permission-dialog.tsx:51`), donc elle n'est pas modale. Reste à voir si elle est atteignable au clavier sans souris, et si elle sort correctement de son conteneur.
4. **Ce que voit l'utilisateur quand le journal des autorisations ne peut pas être écrit** — disque plein, dossier en lecture seule. Le code ignore silencieusement l'échec d'écriture (`permission_gate.rs:124-131`, tous les résultats sont jetés) : l'incident n'a donc **aucune trace visible**. À confirmer comme choix assumé.
5. **La formulation dans les six autres langues** n'a pas été relue. Chaque message cité ici l'a été en français seulement.

**Affichage non vérifié — liste de contrôle pour la passe d'interface finale**

6. Le rendu du bloc d'erreur d'un outil : où exactement s'affichent la phrase de catégorie et le conseil, et s'ils sont repliés par défaut.
7. Le rendu du résumé technique sous une réponse interrompue, et sa lisibilité dans les deux thèmes.
8. Le tableau de bord des sous-agents : comment se distinguent « échoué », « annulé » et « interrompu » — par la couleur seule, ou par autre chose.
9. Le message de dépassement de capacité de contexte est **long** — cinq nombres dans une seule phrase. À vérifier qu'il ne casse pas la mise en page sur une fenêtre étroite.
