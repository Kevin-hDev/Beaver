# Modes de permission

**Emplacement site** — Agent › Modes de permission (page centrale de la section Agent)
**Répond à** — « Qu'est-ce que l'agent peut faire sans me demander, et comment je change ça ? »
**Sources** — `src-tauri/src/services/agent_local/permission_gate.rs` (lignes 13-46, 48-49, 91-132), `permission_policy.rs`, `permission_bash.rs`, `permission_allow_cache.rs`, `sensitive_data.rs` (lignes 1-25, 98-111), `agent_settings.rs`, `src/components/agent-local/permission-mode-selector.tsx`, `src/hooks/use-permission-mode.ts`
**Vérification** — Vérifié dans le code, ligne par ligne. C'est la page la plus sensible de la documentation : chaque affirmation ici engage la sécurité de l'utilisateur.

---

## Avertissement au rédacteur

**Le tableau du mockup est faux.** Il présente « commandes shell » comme toujours soumises à approbation en mode Demande d'approbation. C'est inexact : une commande reconnue comme sûre passe sans confirmation.

Ne rien reprendre du mockup sur cette page. Tout est ci-dessous.

---

## Plan de page proposé

1. Les trois modes
2. Ce qui déclenche une demande
3. Les commandes shell : la règle particulière
4. Les données sensibles
5. Les trois réponses possibles
6. « Autoriser pour la session » et ses exceptions
7. Ce qui se passe en cas de doute
8. Changer de mode

---

## Contenu

### 1. Les trois modes

| Libellé affiché | Identifiant | Comportement |
|---|---|---|
| **Accès complet** | `auto` | L'agent exécute ses outils sans jamais demander |
| **Demande d'approbation** | `manual` | Les actions sensibles déclenchent une confirmation |
| **Chatbot** | `chat` | Aucun outil : réponses en texte uniquement |

Un quatrième mode, `subagent`, existe **uniquement en interne** pour les sessions déléguées. Il contourne la garde de permission comme le mode Accès complet. Il n'est jamais proposé à l'utilisateur — mais il faut savoir qu'il existe, parce qu'il signifie que **les sous-agents ne demandent pas de confirmation**. Ce point mérite une mention dans la page *Sous-agents*.

**Le mode par défaut à l'installation est Accès complet.** À dire explicitement.

### 2. Ce qui déclenche une demande

En mode Demande d'approbation, **douze outils** déclenchent systématiquement une confirmation :

`write_file`, `edit_file`, `web_fetch`, `write_spreadsheet`, `write_document`, `create_branch`, `checkout_branch`, `apply_subagent_changes`, `manage_automation`, `forecast_data_audit`, `forecast_run`, `forecast_backtest`

**Quatre outils** déclenchent une confirmation **sous condition** :

| Outil | Condition |
|---|---|
| `bash` | La commande n'est pas reconnue comme sûre — voir section 3 |
| `bash_control` | Des caractères sont envoyés au processus |
| `transform_image` | Au moins une opération est demandée |
| `search_mcp_tools` | Le mode est `call` — la simple recherche d'outils ne demande rien |

**Tous les autres outils passent sans confirmation**, y compris en mode Demande d'approbation : lire un fichier, lister un dossier, chercher par nom ou par contenu, chercher sur le web.

C'est un point de conception à expliquer plutôt qu'à subir : demander confirmation pour chaque lecture de fichier rendrait le mode inutilisable, et l'utilisateur finirait par tout approuver sans lire.

### 3. Les commandes shell : la règle particulière

Une commande `bash` passe **sans confirmation** si elle satisfait **les trois conditions à la fois** :

1. elle ne touche à aucune donnée sensible (section 4) ;
2. elle ne contient **aucun opérateur de contrôle** ;
3. elle correspond à l'un des motifs reconnus comme sûrs.

**Les opérateurs qui disqualifient une commande** — leur seule présence suffit :

`;` `&&` `||` `|` `` ` `` `$(` `<(` `>(` `<<` `>` `<` `&` `$'` retour à la ligne

Autrement dit : `ls` passe, `ls | grep foo` ne passe pas. La raison est solide — un opérateur permet d'enchaîner n'importe quelle commande derrière une commande anodine.

**Les motifs reconnus comme sûrs** — vingt et un au total :

| Catégorie | Commandes |
|---|---|
| Lecture de fichiers | `ls`, `cat`, `head`, `tail`, `wc`, `file`, `stat` |
| Recherche | `grep`, `find`, `tree` |
| Système | `pwd`, `echo`, `which`, `du`, `df` |
| Git en lecture | `git status`, `git log`, `git diff`, `git show`, `git remote`, `git tag`, `git branch` seul |
| Vérification de projet | `cargo check`, `cargo test`, `cargo clippy`, `cargo build`, `npx tsc`, `npm run`, `npm test` |

Toute autre commande déclenche une confirmation.

### 4. Les données sensibles

Une commande qui mentionne un chemin sensible est **toujours** soumise à confirmation, même si elle correspond à un motif sûr. Un `cat ~/.ssh/id_rsa` ne passe pas.

**Les marqueurs surveillés** — seize :

`.env`, `.ssh/`, `/.ssh`, `id_rsa`, `id_ed25519`, `id_ecdsa`, `id_dsa`, `.npmrc`, `.pypirc`, `.netrc`, `.aws/credentials`, `.config/gcloud`, `.kube/config`, `credentials`, `login.keychain`, `keychain-db`

**Quatre fichiers de l'application sont également protégés** : `config.json`, `secrets.enc`, `agent-settings.json`, `configured-providers.json`.

Raffinement à mentionner, il montre le soin apporté : le contenu des blocs de texte multiligne est retiré avant l'analyse. Écrire un fichier `.gitignore` qui **mentionne** `.env` ne déclenche pas d'alerte ; seul un vrai chemin `.env` passé en argument compte.

### 5. Les trois réponses possibles

Quand une confirmation est demandée :

| Réponse | Effet |
|---|---|
| **Autoriser** | L'action s'exécute, une fois |
| **Autoriser pour la session** | L'action s'exécute, et l'outil ne redemandera plus — sous conditions, voir section 6 |
| **Refuser** | L'action n'est pas exécutée |

### 6. « Autoriser pour la session » et ses exceptions

C'est le point le plus important de la page après la règle des commandes shell.

**Trois outils ne sont jamais mémorisés**, quoi que vous répondiez : **`bash`**, **`bash_control`** et **`search_mcp_tools`**. Chaque appel redemande.

La raison est bonne : ces trois outils peuvent faire n'importe quoi selon leurs arguments. Autoriser `bash` une fois pour la session reviendrait à passer en Accès complet sans le savoir.

**Pour les autres outils**, l'autorisation est mémorisée avec des bornes :

| Borne | Valeur |
|---|---|
| Durée de validité | **1 heure** |
| Sessions mémorisées | **64** au maximum |
| Outils mémorisés par session | **16** au maximum |

Les autorisations expirées sont purgées automatiquement.

### 7. Ce qui se passe en cas de doute

Le principe est **le refus par défaut**. Trois situations :

- **Annulation de la conversation** pendant qu'une demande est en attente → refus.
- **Plus de 64 demandes en attente** simultanément → refus automatique.
- **Réponse à une demande inconnue ou périmée** → ignorée et journalisée.

Chaque décision est écrite dans `logs/permission-diagnostics.jsonl`, avec rotation à **2 Mo**.

### 8. Changer de mode

- Le mode se change **depuis la conversation**, à tout moment, via un sélecteur dédié.
- Les touches **1**, **2** et **3** sélectionnent directement un mode quand le sélecteur est ouvert.
- Un mode par défaut se définit dans les réglages.
- Le mode est **propre à chaque conversation**.

---

## Encadrés

**Encadré « Le mode par défaut est Accès complet »** — avertissement, en tête de page.
> À l'installation, Beaver est en Accès complet : l'agent exécute ses outils sans demander. Passez en Demande d'approbation si vous préférez valider chaque action.

**Encadré « Les commandes shell ne sont pas toutes soumises »**
> En mode Demande d'approbation, une commande de lecture reconnue comme sûre — `ls`, `cat`, `git status` — s'exécute sans confirmation. Dès qu'une commande contient un enchaînement, une redirection ou un chemin sensible, la confirmation est demandée.

**Encadré « bash n'est jamais autorisé pour la session »** — avertissement.
> L'option « autoriser pour la session » ne s'applique pas aux commandes shell ni aux appels d'outils externes. Ces actions redemandent à chaque fois, parce qu'une même commande peut faire n'importe quoi selon ses arguments.

**Encadré « Les sous-agents ne demandent pas »**
> Les sessions déléguées à des sous-agents s'exécutent sans confirmation. Le contrôle se fait en amont, au moment de la délégation, et en aval, en inspectant leurs changements avant de les appliquer.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| Une commande shell s'exécute sans confirmation | Elle correspond à un motif sûr, sans opérateur ni chemin sensible | Comportement voulu ; passer en Chatbot pour tout bloquer |
| `ls` passe mais `ls \| grep x` demande confirmation | Le caractère `\|` disqualifie la commande | Normal |
| « Autoriser pour la session » sans effet sur `bash` | Cet outil n'est jamais mémorisé | Volontaire |
| L'autorisation redemande après un moment | Validité d'une heure | Normal |
| Une action est refusée sans que j'aie répondu | Conversation annulée, ou trop de demandes en attente | Relancer l'action |
| L'agent n'utilise aucun outil | Mode Chatbot | Changer de mode |
| Un sous-agent modifie des fichiers sans rien demander | Les sous-agents contournent la garde | Inspecter ses changements avant de les appliquer |

---

## Renvois

- *Agent › Mode Plan* — le contrôle qui va plus loin
- *Agent › Sous-agents* — pourquoi ils ne demandent pas
- *Outils › Terminal et shell* — l'outil `bash`
- *Sécurité › Durcissement*
- *Premier lancement* — le mode par défaut

---

## Points à confirmer

- **Les libellés exacts des trois réponses** dans l'interface — « Autoriser », « Autoriser pour la session », « Refuser » sont déduits des identifiants du code.
- **L'aspect de la demande de confirmation** : ce que voit l'utilisateur, ce qui lui est montré de la commande ou du chemin concerné. Non relevé, et important — une confirmation qui n'affiche pas ce qu'elle autorise ne sert à rien.
- **Le mode par défaut se règle-t-il vraiment dans les réglages ?** `agent-settings.json` contient `permissionMode`, mais l'écran correspondant n'a pas été vérifié.
- **Le comportement en mode Chatbot** : les outils sont-ils absents du catalogue envoyé au modèle, ou refusés à l'exécution ? Change ce qu'on peut affirmer.
- **La liste des 21 motifs sûrs mérite d'être revérifiée à chaque version.** Elle est au cœur de la sécurité du mode Demande d'approbation et peut évoluer.
- **`git branch` seul est sûr, mais `git branch -d` ?** Le motif exige la fin de chaîne, donc toute option déclenche une confirmation. À confirmer par un test.
