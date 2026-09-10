# Travailler avec Git dans Beaver

**Emplacement site** — Automatisation › Git
**Répond à** — « Que puis-je faire avec Git sans quitter Beaver, et qu'est-ce qui reste au terminal ? »
**Sources** — `src-tauri/src/services/git/` (`repo.rs`, `branch.rs`, `branch_create.rs`, `branch_commit.rs`, `branch_index_backup.rs`, `branch_delete.rs`, `branch_merge.rs`, `branch_merge_error.rs`, `status.rs`, `commit_files.rs`, `history.rs`, `diff_preview.rs`, `diff_preview_model.rs`, `diff_preview_serialize.rs`, `blob_preview.rs`, `remote.rs`, `remote_status.rs`, `remote_target.rs`, `remote_credentials.rs`, `network_policy.rs`, `github_auth.rs`, `watcher.rs`, `worktree_list.rs`, `worktree_delete.rs`, `action_error.rs`) et leurs fichiers de tests ; `src-tauri/src/commands/git.rs`, `git_mutations.rs`, `git_history.rs` ; `src-tauri/src/invoke_handler_tail.rs` ; `src-tauri/src/services/agent_local/project_store.rs` ; `src/components/agent-local/` (sélecteur de branches, dialogues de commit, fusion, suppression, historique) ; `src/hooks/use-git-branch.ts`, `git-refresh.ts`, `use-git-mutations.ts`, `use-git-history.ts`, `use-git-watcher.ts` ; `src/lib/branch-name.ts`, `src/lib/app-error.ts` ; `src/i18n/fr.json`
**Vérification** — Vérifié dans le code, ligne par ligne, le 10 septembre 2026. Aucun affichage observé à l'écran.

---

## Plan de page proposé

1. Ce que Beaver fait avec Git, et ce qu'il ne fait pas
2. Où ça se passe dans l'application
3. La condition d'entrée : un projet enregistré
4. Changer de branche, en créer une
5. Les modifications non validées
6. Valider (commit)
7. Publier (push)
8. Fusionner (merge)
9. Supprimer une branche ou un worktree
10. Lire l'historique et les différences
11. Les worktrees
12. La détection automatique des changements

---

## Contenu

### 1. Ce que Beaver fait avec Git, et ce qu'il ne fait pas

**Le tableau le plus utile de la page**, et il doit venir en premier. Chaque « non » est une absence vérifiée dans le code, pas une omission de la documentation.

| Opération | Disponible | Source |
|---|---|---|
| Lister et changer de branche | **Oui** | `services/git/branch.rs:27-60`, `:100-123` |
| Créer une branche | **Oui** | `services/git/branch_create.rs:24-67` |
| Supprimer une branche | **Oui**, trois modes | `services/git/branch_delete.rs` |
| Valider (commit) | **Oui**, tout le dossier de travail | `services/git/branch_commit.rs:9-26` |
| Valider une sélection de fichiers | **Non** | Aucune commande n'accepte une liste de chemins : `commands/git_mutations.rs:27-31`, `commands/git.rs:78-83` |
| Publier (push) | **Oui** | `services/git/remote.rs:46-95` |
| Récupérer (fetch) | **Non** | Absent de `services/git/` |
| Tirer (pull) | **Non** | Absent de `services/git/` |
| Fusionner (merge) | **Oui**, entre branches **locales** | `services/git/branch_merge.rs:49-69` |
| Cloner un dépôt distant | **Non** | Le mot « clone » dans Beaver désigne le clone d'une conversation |
| Voir l'historique des commits | **Oui**, branche courante | `services/git/history.rs:44-77` |
| Voir les différences d'un fichier | **Oui**, commit ou dossier de travail | `services/git/diff_preview.rs:58-112` |
| Lister et supprimer des worktrees | **Oui** | `services/git/worktree_list.rs`, `worktree_delete.rs` |
| Créer un worktree | **Non** depuis l'interface Git | Aucune commande de création dans `invoke_handler_tail.rs:75-99` |
| Résoudre un conflit de fusion | **Non** | La fusion est annulée en cas de conflit (`branch_merge.rs:130-135`) |

**La conséquence la plus importante à écrire :** Beaver ne va jamais chercher les nouveautés du dépôt distant. Le retard affiché — « Des changements distants sont disponibles » (`src/i18n/fr.json:391`) — est calculé sur les références **déjà présentes localement** (`services/git/remote_status.rs:48-57`, `:68-74`). Elles ne sont mises à jour que par un `git fetch` lancé ailleurs, ou par le succès d'un push de Beaver lui-même (`remote.rs:97-115`).

### 2. Où ça se passe dans l'application

Git apparaît à **deux endroits**, avec des rôles différents.

**Le sélecteur de branches**, dans la barre de contrôle d'une conversation (`src/components/agent-local/chat-project-controls.tsx:63-70`). Il porte le nom de la branche courante et ouvre un menu contenant :

- un champ de recherche, **« Rechercher dans les branches »** (`src/i18n/fr.json:193`) ;
- la liste des branches, sous le titre **« Branches »** (`:194`) ;
- la liste des worktrees, sous **« Worktrees »** (`:195`) ;
- le compteur **« Non validés : {{count}} fichier(s) »** (`:196`) ;
- l'entrée **« Créer et extraire une nouvelle branche… »** (`:197`).

**La section Git du résumé de conversation** (`src/components/agent-local/session-summary-git-section.tsx`), qui porte les actions et l'historique : **« Commit »**, **« Push »**, **« Merge dans {{branch}} »** (`src/i18n/fr.json:370-372`), et la liste des commits sous le titre **« Commits »** (`:359`).

**Quand le dossier n'est pas un dépôt Git**, le sélecteur de branches disparaît entièrement (`src/components/agent-local/branch-selector.tsx:124`), et le résumé affiche **« Aucun dépôt Git »** (`src/i18n/fr.json:357`). C'est le bon comportement : une fonctionnalité indisponible est invisible, pas affichée en panne.

### 3. La condition d'entrée : un projet enregistré

**Toutes** les opérations Git passent par un même contrôle : le dossier doit exister **et** se trouver à l'intérieur d'un projet enregistré dans Beaver (`commands/git.rs:7-13` → `services/agent_local/project_store.rs:125-135`). Un dépôt ouvert autrement est refusé avec le message technique « Projet non autorisé » (`project_store.rs:134`).

**Un sous-dossier d'un dépôt fonctionne** : la détection remonte l'arborescence jusqu'au dépôt parent (`services/git/repo.rs:4-6`, test `:47-59`).

**Un dépôt sans dossier de travail (« bare ») est refusé** (`repo.rs:8-12`).

### 4. Changer de branche, en créer une

**Seules les branches locales sont listées** (`services/git/branch.rs:38`, `:55`) — les branches distantes n'apparaissent pas. La branche courante est en tête, les autres par ordre alphabétique (`branch.rs:60`). La liste s'arrête à **500 branches** (`branch.rs:6`, `:41-43`).

**Créer une branche part de la position courante et s'y place aussitôt** (`branch_create.rs:66-67`). **Elle ne valide rien** : les modifications en cours restent en place et suivent sur la nouvelle branche (test `services/git/tests.rs:122`).

**Le nom de branche est validé des deux côtés**, avec les mêmes règles (`branch_create.rs:73-103` côté moteur, `src/lib/branch-name.ts:9-45` côté saisie ; le commentaire `branch_create.rs:72` impose de les garder synchronisés) :

| Règle | Message affiché en cas d'échec |
|---|---|
| **100 caractères** au maximum | **« Nom trop long (max 100 caractères) »** (`src/i18n/fr.json:211`) |
| Pas de nom vide, pas de tiret en tête, pas de `..`, pas de `\ : ~ ^ ? * [ @{ //`, pas de `/` en début ou en fin, pas de suffixe `.lock`, pas d'espace ni de caractère de contrôle | **« Nom invalide : caractères interdits ou format incorrect »** (`:210`) |
| Chaque segment séparé par `/` doit être non vide et ne pas commencer ni finir par un point | idem |

**Trois refus supplémentaires**, avec leur message (`branch_create.rs:9-49`) :

- la branche existe déjà — **« Cette branche existe déjà »** (`src/i18n/fr.json:212`) ;
- le dépôt n'a aucun commit — **« Le dépôt n'a aucun commit, crée un commit d'abord »** (`:213`) ;
- le dépôt est lié à GitHub et le connecteur GitHub n'est pas connecté — **« Connecte GitHub avant de créer une branche »** (`:214`).

Ce dernier point mérite un paragraphe sur le site. **Beaver exige une connexion GitHub pour créer une branche dans un dépôt dont un distant pointe vers GitHub** (`services/git/github_auth.rs:5-25`), même si la branche est purement locale. Un dialogue dédié le propose : **« Connectez GitHub pour créer une branche »**, avec le bouton **« Connecter GitHub »** (`src/i18n/fr.json:228`, `:230`). Les trois formes d'adresse reconnues comme GitHub sont `https://github.com/`, `ssh://git@github.com/` et `git@github.com:` (`services/git/repo.rs:35-39`).

**Le changement de branche est refusé s'il reste des fichiers non validés** (`branch.rs:109-112`). Beaver refuse **avant** de toucher au disque : il n'essaie pas d'extraire puis d'annuler. Le dialogue affiché s'intitule **« Effectuez un commit pour changer de branche »** et explique : **« Les modifications apportées aux fichiers suivants seraient remplacées par l'extraction : »** (`src/i18n/fr.json:200-201`). Il propose **« Effectuer le commit et changer de branche… »** (`:202`), qui crée un commit portant le sujet `WIP: save changes before switching to <branche>` puis extrait (`branch_commit.rs:104-110`). Si rien n'est modifié, il extrait directement sans créer de commit (`branch_commit.rs:38-41`).

### 5. Les modifications non validées

**Six catégories de fichiers** sont distinguées à l'affichage (`services/git/status.rs:113-136`, libellés `src/i18n/fr.json:1311-1316`) : **« Créé »**, **« Modifié »**, **« Supprimé »**, **« Renommé »**, **« Copié »**. Chaque fichier porte son nombre de lignes ajoutées et supprimées (`status.rs:11-18`).

**La liste s'arrête à 200 fichiers**, mais le compte réel est conservé et la coupure est signalée (`status.rs:8`, `:20-24`, `:81-86`). Le message correspondant est **« {{count}} fichiers modifiés. Les chiffres affichés sont partiels. »** (`src/i18n/fr.json:355`).

**Pour un fichier neuf, les lignes ne sont comptées que sous 1 Mo** ; au-delà, Beaver annonce zéro plutôt que de lire un gros fichier (`status.rs:186-198`).

**Attention à un écart réel**, qu'il faut décrire pour éviter une incompréhension : le compteur du sélecteur de branches et celui du refus d'extraction **ne parcourent pas les dossiers non suivis** (`branch.rs:65-70`, test `services/git/tests.rs:139`), alors que la liste affichée les parcourt (`status.rs:36-40`). Un dossier neuf contenant trente fichiers compte donc pour **un** dans le refus d'extraction, et pour **trente** dans la liste. Le comportement est verrouillé par un test, donc voulu — mais il n'est écrit nulle part dans l'interface.

### 6. Valider (commit)

Le dialogue s'intitule **« Commit des modifications »**, avec le champ **« Message du commit (facultatif) »** et le bouton **« Commit »** (`src/i18n/fr.json:392-393`, `:395`).

**Le commit prend tout le dossier de travail**, sans sélection possible (`branch_commit.rs:77-79`).

**Le message est facultatif.** Laissé vide, le commit porte `Save changes from Beaver` (`branch_commit.rs:20`, test `branch_commit_tests.rs:30`). Il n'y a **aucune longueur minimale**, un maximum de **2 000 caractères**, les fins de ligne sont normalisées, et les caractères de contrôle autres que le saut de ligne et la tabulation sont refusés (`branch_commit.rs:7`, `:112-131`). Le message d'erreur est **« La description du commit est invalide ou trop longue. »** (`src/i18n/fr.json:222`).

**S'il n'y a rien à valider, l'opération réussit sans rien faire** (`branch_commit.rs:12-15`). Aucun commit vide n'est créé.

**L'auteur du commit est votre identité Git**, dans cet ordre : la configuration du dépôt (`user.name` / `user.email`), puis l'identité que le binaire `git` déclare (`branch_commit.rs:133-156`). **Si aucune identité n'est configurée, le commit échoue** avec **« Configure ton nom et ton adresse e-mail Git avant de créer un commit. »** (`src/i18n/fr.json:221`). Beaver ne fabrique pas d'auteur de substitution.

**Une protection discrète mérite d'être mentionnée** : avant chaque commit, le fichier d'index du dépôt est copié ; si le commit échoue, l'index d'origine est remis en place (`services/git/branch_index_backup.rs:13-36`, appelé `branch_commit.rs:19`, `:45`). Un commit raté ne laisse donc pas le dépôt dans un état intermédiaire.

### 7. Publier (push)

Le bouton s'appelle **« Push »** (`src/i18n/fr.json:371`), et l'indicateur au-dessus dit **« {{count}} commit(s) à Push vers {{destination}} · {{branch}} »** (`:369`), la destination étant **« GitHub »** ou **« le dépôt distant »** (`:389-390`).

**Beaver pousse la branche courante vers son distant, sans forçage** (`services/git/remote.rs:82-85`). Le distant est choisi dans cet ordre : celui configuré pour la branche, puis `origin`, puis le premier de la liste, dans une limite de 32 (`services/git/remote_target.rs:24-44`).

**Après un push réussi**, Beaver met à jour la référence distante locale et pose la branche amont (`remote.rs:97-115`). C'est le seul moment où l'état du distant est rafraîchi.

**L'authentification est essayée dans un ordre précis, chaque étape une seule fois** (`services/git/remote_credentials.rs:27-59`) :

1. l'assistant d'identifiants Git du système ;
2. le jeton GitHub du connecteur, **uniquement si l'adresse est `https://github.com/`** (`remote.rs:32-44`) ;
3. la clé SSH de l'agent ;
4. le nom d'utilisateur ;
5. la valeur par défaut de la bibliothèque.

**S'il n'y a aucun identifiant, l'appel échoue** — il ne « laisse pas passer » (`remote_credentials.rs:58`, test `:86`). Le jeton est gardé dans un conteneur qui efface son contenu quand il est libéré (`remote_credentials.rs:8`, `remote.rs:6`, `:49`).

**Un dépôt GitHub en SSH ne demande aucun jeton OAuth** (test `remote_tests.rs:66-80`) — le jeton ne sert qu'aux adresses HTTPS.

**Les cinq erreurs de push, avec leur message :**

| Situation | Message affiché | Source |
|---|---|---|
| Aucun identifiant utilisable | **« Connexion Git requise pour effectuer le Push. »** | `src/i18n/fr.json:398` |
| Le distant refuse | **« Le dépôt distant a refusé le Push. Vérifie tes droits ou la protection de la branche. »** | `:399` |
| La branche distante a avancé | **« La branche distante contient des changements plus récents. Synchronise le dépôt avant de refaire le Push. »** | `:402` |
| Réseau injoignable | **« Connexion au dépôt distant impossible. »** | `:400` |
| La branche a changé depuis l'affichage | **« Le projet ou la branche a changé. Relance le Push. »** | `:401` |

Le troisième cas est celui qui reviendra le plus : **Beaver ne sait pas rattraper un retard**, puisqu'il ne récupère rien. La résolution passe par un `git pull` ailleurs.

**Deux délais réseau sont fixés à 30 secondes** — connexion et inactivité (`services/git/network_policy.rs:3-4`, `:15-22`), appliqués une fois au démarrage (`src-tauri/src/startup.rs:105-112`). **Aucune liste d'adresses autorisées n'existe** : n'importe quel distant configuré dans le dépôt est utilisable.

### 8. Fusionner (merge)

Le dialogue s'intitule **« Merge dans {{branch}} »** et explique **« Choisis la branche à intégrer dans {{branch}}. »**, avec le champ **« Branche source »** (`src/i18n/fr.json:373-375`).

**La fusion est locale** : une branche locale dans la branche courante (`services/git/branch_merge.rs:49-69`). Elle ne va jamais chercher une branche distante.

**L'aperçu annonce l'ampleur avant d'agir** : **« {{count}} commit(s) seront ajoutés à {{branch}}. »** (`:376`), et le cas échéant **« {{count}} modification(s) feront l'objet d'un Commit avant le Merge. »** (`:377`). Quand il n'y a rien à faire : **« Cette branche est déjà mergée dans {{branch}}. »** (`:378`).

**Quatre refus** (`branch_merge.rs:57-66`, `:88-90`, `:94-104`, `:116-122`) : source identique à la cible ou rien à fusionner, branche courante différente de celle attendue, dépôt déjà au milieu d'une autre opération Git, et modifications non validées — sauf si l'utilisateur a choisi de les valider d'abord, via **« Commit puis Merge dans {{branch}} »** (`:380`).

**En cas de conflit, la fusion est annulée et l'état d'avant est vérifié.** Beaver lance l'annulation, puis **contrôle** que la position est revenue à sa valeur d'origine, que le dépôt est propre et qu'aucun fichier n'est modifié ; ce n'est qu'en cas de succès de ce contrôle qu'il annonce un conflit (`branch_merge.rs:130-135`, `:154-184`, test `branch_merge_tests.rs:124`). Le message est **« Le Merge crée un conflit. Il a été annulé sans modifier la branche. »** (`src/i18n/fr.json:382`).

**Beaver ne résout pas les conflits.** Il n'y a pas d'éditeur de conflit dans l'application : la résolution se fait au terminal ou dans un éditeur.

**Les crochets du dépôt sont neutralisés pendant la fusion** (`branch_merge.rs:125`, `:137-152`, test `:146`) : le chemin des crochets pointe vers un dossier temporaire vide, la signature cryptographique est désactivée, et aucune invite du terminal ne peut bloquer l'opération. C'est un choix de sécurité — un crochet de dépôt est du code exécutable — et il mérite d'être écrit sur le site.

### 9. Supprimer une branche ou un worktree

**Un aperçu précède toujours la suppression** (`services/git/branch_delete.rs:10-45`). Il annonce : si c'est la branche courante, la branche de repli choisie, les fichiers non validés, et le nombre de commits non fusionnés — **« {{count}} commit(s) ne sont pas fusionné(s) »** (`src/i18n/fr.json:244`).

**La branche de repli** est la branche courante si elle diffère de la cible, sinon `main`, `master`, `develop`, `dev` dans cet ordre, sinon la première autre branche (`branch_delete.rs:128-148`). Le message est **« Cette branche est utilisée. Beaver passera sur {{branch}} avant de la supprimer. »** (`:242`). Sans aucune branche de repli : **« Crée d'abord une autre branche pour pouvoir supprimer celle-ci. »** (`:243`).

**Trois modes**, avec leurs libellés (`services/git/branch_delete.rs:47-97` ; `commands/git_mutations.rs:11-17`) :

| Mode | Ce qu'il fait | Libellé affiché |
|---|---|---|
| Propre | Refuse s'il reste des modifications ou des commits non fusionnés | **« Confirmer »** (`:237`) |
| Abandonner | Jette les modifications puis supprime | **« Supprimer et abandonner les modifications »** (`:249`), **« Supprimer sans fusionner »** (`:250`) |
| Préserver | Valide, se replie, fusionne, puis supprime | **« Fusionner puis supprimer »** (`:246`), **« Enregistrer, fusionner puis supprimer »** (`:247`) |

**Le comptage des commits non fusionnés s'arrête à 10 000** (`branch_delete.rs:8`, `:170`).

**Un worktree suit la même logique** (`services/git/worktree_delete.rs`), avec les mêmes trois modes et un **délai de dix secondes** sur l'opération de suppression (`worktree_delete.rs:9`, `:109-112`). Le worktree courant ne peut pas être supprimé (`:85-91`).

### 10. Lire l'historique et les différences

**L'historique porte sur la branche courante uniquement** (`services/git/history.rs:105-118`). Il est chargé par pages de **24 commits** (`history.rs:6`, `src/hooks/use-git-history.ts:19`), avec un plafond de **50 par page** côté moteur (`history.rs:7`) et **1 000 commits** chargés au total côté interface (`src/components/agent-local/session-summary-commits.tsx:10`). La page suivante se charge à l'approche du bas de la liste (`session-summary-commits.tsx:11`, `:88-92`).

**Chaque commit affiche sa première ligne de message seulement**, bornée à **160 caractères** (`history.rs:9-10`, `:152-164`). Sans message : **« Commit sans message »** (`src/i18n/fr.json:362`).

**Le parcours s'arrête à 10 000 commits** ; au-delà, la requête échoue (`history.rs:8`, `:52-55`) avec **« Historique des commits indisponible. »** (`:364`).

**Ouvrir un commit** montre les fichiers qu'il touche, **200 au maximum** (`services/git/commit_files.rs:6`, `:36` ; affichage `session-summary-commits.tsx:78`), puis, fichier par fichier, ses différences.

**Les différences se lisent dans deux modes** (`services/git/diff_preview.rs:58-112`) : un commit contre son parent, ou le dossier de travail contre le dernier commit. Trois lignes de contexte entourent chaque modification (`diff_preview.rs:120-124`).

**Les bornes d'affichage d'une différence** (`services/git/diff_preview_serialize.rs:4-7`) : **100 blocs**, **2 000 lignes**, **16 Ko par ligne**, **2 Mo au total**. Chaque dépassement **tronque** au lieu d'échouer, et le signale : **« Aperçu limité : le fichier contient davantage de modifications. »** (`src/i18n/fr.json:1309`).

**Un fichier binaire n'affiche aucune différence**, seulement l'indication qu'il est binaire (`diff_preview.rs:143-152`). Un fichier illisible affiche **« Les différences de ce fichier ne peuvent pas être affichées. »** (`:1308`).

**Les aperçus de fichiers versionnés** ont leurs propres bornes : **2 Mo** pour du texte, refusé si le contenu n'est pas de l'UTF-8 valide (`commands/git_history.rs:5`, `:101-104`) ; **10 Mo** pour un `pdf` ou un `docx` (`services/git/blob_preview.rs:5`) ; **50 Mo** en plafond absolu, y compris pour les tableurs (`blob_preview.rs:4`, `commands/git_history.rs:146-156`). Toute erreur d'aperçu remonte le même message générique côté moteur, « Aperçu Git indisponible » (`commands/git_history.rs:159-161`) — aucun chemin, aucun détail interne.

### 11. Les worktrees

Un worktree est un second dossier de travail relié au même dépôt, permettant d'avoir deux branches ouvertes en même temps sur le disque.

**Beaver les liste mais n'en crée pas** : la création se fait au terminal (`git worktree add`). Aucune commande de création n'existe (`invoke_handler_tail.rs:75-99`).

**La liste s'arrête à 100 worktrees** (`services/git/worktree_list.rs:4`, `:84`), et exclut deux catégories : les dépôts sans dossier de travail, et **tous les worktrees internes créés par les sous-agents de Beaver**, rangés sous `~/.local/share/cl-go-dash/subagent-worktrees` (`worktree_list.rs:15`, `:89-91`, `:113-116`, test `:163`). C'est voulu : ce sont des dossiers de travail temporaires, pas les vôtres.

**Changer de worktree change de dossier de travail**, ce qui ne se fait pas dans la même conversation. Le dialogue **« Changement de dossier »** l'explique : **« Ce worktree utilise un autre dossier de travail. Crée une nouvelle session pour garder la conversation actuelle intacte. »** (`src/i18n/fr.json:539-540`), et propose **« Nouvelle session »** (`:542`).

### 12. La détection automatique des changements

Beaver surveille le dépôt et rafraîchit son affichage tout seul (`services/git/watcher.rs`).

**Ce qui est surveillé**, à l'intérieur du dossier `.git` : le fichier de position courante, le dossier des branches (de façon récursive) et le fichier des références compactées (`watcher.rs:107-109`, `:135-145`). Les changements sont regroupés sur **200 millisecondes** (`watcher.rs:11`).

**Le dossier de travail, lui, n'est pas surveillé par le système de fichiers** : il est relu **toutes les trois secondes**, et l'interface n'est prévenue que si quelque chose a réellement changé — nom de branche, position, nombre de fichiers modifiés, ou une empreinte des 200 premières entrées du dossier (`watcher.rs:12`, `:60-84`, `:151`, `:160-164` ; empreinte `status.rs:89-111`).

Concrètement : **une modification faite au terminal apparaît dans Beaver en trois secondes au plus**, y compris quand le nombre de fichiers modifiés ne change pas — c'est explicitement testé (`services/git/watcher_tests.rs:26`, `:43`, `:72`, `:84`).

**Un seul dépôt est surveillé à la fois pour toute l'application** (`watcher.rs:14-19`, `:92-105`). Ouvrir un autre projet arrête la surveillance du précédent.

**Si aucun dossier `.git` n'est trouvé, la surveillance ne démarre pas et ne signale rien** (`watcher.rs:87-90`).

---

## Tableaux

### Les bornes, en un seul endroit

| Élément | Borne | Source |
|---|---|---|
| Branches listées | **500** | `services/git/branch.rs:6` |
| Worktrees listés | **100** | `services/git/worktree_list.rs:4` |
| Distants examinés | **32** | `services/git/repo.rs:21` ; `remote_target.rs:3` |
| Fichiers non validés listés | **200** | `services/git/status.rs:8` |
| Fichiers listés dans un commit | **200** | `services/git/commit_files.rs:6` |
| Longueur d'un nom de branche | **100 caractères** | `services/git/branch_create.rs:74` |
| Longueur d'un message de commit | **2 000 caractères** | `services/git/branch_commit.rs:7` |
| Commits par page d'historique | **24** par défaut, **50** au maximum | `services/git/history.rs:6-7` |
| Commits parcourus | **10 000** | `services/git/history.rs:8` |
| Commits non fusionnés comptés | **10 000** | `services/git/branch_delete.rs:8` |
| Aperçu de différence | **100 blocs**, **2 000 lignes**, **16 Ko/ligne**, **2 Mo** | `services/git/diff_preview_serialize.rs:4-7` |
| Aperçu texte d'un fichier versionné | **2 Mo** | `commands/git_history.rs:5` |
| Aperçu `pdf` / `docx` | **10 Mo** | `services/git/blob_preview.rs:5` |
| Plafond absolu d'un fichier versionné | **50 Mo** | `services/git/blob_preview.rs:4` |
| Longueur d'un chemin | **4 096 octets** | `services/git/status.rs:9` |
| Délai réseau (connexion, inactivité) | **30 s** chacun | `services/git/network_policy.rs:3-4` |
| Délai de suppression d'un worktree | **10 s** | `services/git/worktree_delete.rs:9` |
| Regroupement du surveillant | **200 ms** | `services/git/watcher.rs:11` |
| Relecture du dossier de travail | **3 s** | `services/git/watcher.rs:12` |

### Ce que Beaver ne remonte jamais comme détail interne

Le moteur renvoie des **codes**, jamais des messages techniques, et l'interface les traduit (`services/git/action_error.rs:4-39` ; correspondance `src/lib/app-error.ts:51-81`). Un test vérifie qu'aucun champ de message n'accompagne un code (`services/git/action_error_tests.rs:23-30`, l. 29) et un autre qu'aucun chemin de fichier n'apparaît dans le texte affiché (`action_error_tests.rs:33`).

---

## Encadrés

> **⚠ À placer en tête de page — Beaver ne récupère rien du dépôt distant.**
> Ni `fetch`, ni `pull`. Le retard affiché par rapport au distant est calculé sur ce que votre dépôt local sait déjà, et n'est actualisé que par une récupération lancée ailleurs, ou par un push réussi depuis Beaver. Pour rester à jour, gardez un terminal à côté.

> **ℹ À placer dans la section Commit — Le commit prend tout.**
> Il n'existe pas de sélection de fichiers : le bouton Commit valide l'ensemble du dossier de travail. Pour découper un travail en plusieurs commits, passez par le terminal.

> **⚠ À placer dans la section Créer une branche — GitHub est exigé même pour une branche locale.**
> Si un distant de votre dépôt pointe vers GitHub, Beaver demande que le connecteur GitHub soit connecté avant de créer une branche — y compris quand cette branche ne sera jamais publiée.

> **ℹ À placer dans la section Fusion — Un conflit ne casse rien.**
> Beaver annule la fusion, puis vérifie que le dépôt est bien revenu à son état d'avant avant de vous annoncer le conflit. Il ne propose pas de résoudre le conflit : c'est au terminal ou dans un éditeur que cela se fait.

> **ℹ À placer dans la section Détection — Ce que vous faites au terminal apparaît tout seul.**
> Un commit, une extraction ou une simple édition faite hors de Beaver se voit dans l'application en trois secondes au plus, sans rien rafraîchir à la main.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| « Le sélecteur de branches n'apparaît pas » | Le dossier n'est pas un dépôt Git, ou n'est pas un projet enregistré dans Beaver | Enregistrer le dossier comme projet ; vérifier qu'il contient un dépôt |
| « Je ne peux pas changer de branche » | Des fichiers ne sont pas validés | Utiliser « Effectuer le commit et changer de branche… », ou valider soi-même |
| « Le refus dit 1 fichier, la liste en montre 30 » | Deux façons de compter : le refus ne parcourt pas les dossiers neufs, la liste si | Comportement attendu ; se fier à la liste |
| « Créer une branche m'oblige à connecter GitHub » | Un distant du dépôt pointe vers GitHub | Connecter le connecteur GitHub, ou créer la branche au terminal |
| « Le Push est refusé : la branche distante a changé » | Beaver ne récupère pas les nouveautés | `git pull` au terminal, puis relancer le Push |
| « Push impossible, connexion Git requise » | Aucun identifiant utilisable — assistant d'identifiants, jeton GitHub HTTPS, ou clé SSH | Configurer l'un des trois ; un dépôt GitHub en SSH n'utilise pas le jeton |
| « Le commit échoue : nom et e-mail Git » | Aucune identité Git n'est configurée | `git config user.name` et `git config user.email` |
| « Je ne vois pas mes branches distantes » | Seules les branches locales sont listées | Créer une branche locale de suivi au terminal |
| « L'aperçu du fichier est coupé » | La différence dépasse 2 000 lignes ou 2 Mo | Comportement attendu, signalé par « Aperçu limité » |
| « Mon worktree n'apparaît pas » | Plus de 100 worktrees, ou c'est un worktree interne de sous-agent | Comportement attendu |
| « Impossible de supprimer cette branche » | C'est la branche courante, ou il n'existe aucune branche de repli | Changer de branche d'abord, ou créer une autre branche |

---

## Renvois

- `07-integrations/mcp-connecteurs.md` — connecter GitHub
- `03-interface/terminal-integre.md` — ce qui reste au terminal : `fetch`, `pull`, `worktree add`, résolution de conflits
- `04-agent/sous-agents.md` — les worktrees internes que Beaver crée pour ses sous-agents et qui n'apparaissent pas dans la liste
- `03-interface/cloner-une-conversation.md` — le clone de conversation, à ne pas confondre avec le clone d'un dépôt
- `11-securite/durcissement.md` — la neutralisation des crochets de dépôt pendant une fusion

---

## Anomalies relevées

Constatées dans le code, non corrigées.

1. **L'état « position détachée » traverse la frontière et n'est jamais lu.** Le moteur le calcule (`services/git/branch.rs:85`), le type de l'interface le déclare (`src/hooks/git-refresh.ts:10`), et aucun composant ne le consulte. Le libellé **« HEAD détaché »** (`src/i18n/fr.json:198`) est déclenché par une autre condition : un nom de branche vide (`src/components/agent-local/branch-selector.tsx:127`, `:136`). Or le moteur remplit ce nom avec la chaîne littérale `HEAD` quand il ne peut pas faire mieux (`branch.rs:90`), et non par une chaîne vide. Le libellé pourrait donc ne jamais s'afficher dans la situation qu'il décrit.

2. **Un champ du statut distant est calculé et jamais utilisé.** Le moteur renvoie six champs, dont l'indication qu'une branche amont est configurée (`services/git/remote_status.rs:6-14`) ; le type recopié à la main côté interface n'en déclare que cinq (`src/hooks/git-refresh.ts:15-21`), et ce champ n'apparaît nulle part. La distinction entre « branche amont configurée » et « référence distante en cache » est donc perdue à la frontière.

3. **Un code d'erreur traduit ne peut jamais être produit par ce domaine.** Le message **« La branche principale est protégée et ne peut pas être nettoyée. »** (`src/i18n/fr.json:219`) est associé au code `protected_branch` (`src/lib/app-error.ts:61`), mais aucun fichier de `services/git/` ne renvoie ce code : il ne vient que du nettoyage de clone de conversation (`services/agent_local/clone_git_cleanup.rs:126`). Ce message ne peut donc pas apparaître lors d'une suppression depuis le sélecteur de branches.

4. **Deux clés de traduction existent dans les sept langues et ne sont utilisées nulle part** : **« La branche active a changé. Relance le Merge. »** et **« De nouvelles modifications sont apparues. Vérifie-les avant le Merge. »** (`src/i18n/fr.json:383-384`). Les erreurs correspondantes existent bien côté moteur (`services/git/branch_merge_error.rs:9-12`), mais elles sont traduites par d'autres clés.

5. **Les listes tronquées ne le disent pas.** Les bornes de 500 branches et 100 worktrees coupent en silence : aucun champ ne signale la coupure, aucun libellé n'existe. Le compteur de fichiers non validés, lui, remonte bien sa troncature (`services/git/history.rs:26-32`, affichée via `src/i18n/fr.json:355`). Deux traitements différents pour le même problème.

6. **La suppression d'un worktree masque ses échecs.** Si le binaire `git` renvoie une erreur au moment de lister les worktrees, la fonction renvoie une liste **vide** au lieu d'une erreur (`services/git/worktree_list.rs:26-28`), et l'interface avale aussi l'exception (`src/hooks/use-git-branch.ts:100`). Un dépôt dont la commande échoue est donc indiscernable d'un dépôt sans worktree.

7. **Deux dialogues de la même famille ne passent pas par la même primitive de couche flottante.** Le dialogue de commit et celui de fusion utilisent le portail commun (`src/components/agent-local/git-commit-dialog.tsx:24`, `git-merge-dialog.tsx:67`) ; le dialogue de suppression et celui de conflit d'extraction se rendent directement dans l'arbre parent (`git-delete-dialog.tsx:46`, `branch-conflict-dialog.tsx:54`). Le dialogue de suppression est monté depuis le menu déroulant du sélecteur de branches (`src/hooks/use-git-deletion-flow.tsx:68-77` → `branch-selector.tsx:170`), donc à l'intérieur d'un conteneur — configuration exactement propice à un découpage.

8. **Deux composants court-circuitent les hooks Git.** Le dialogue de conflit appelle directement la commande de liste des fichiers modifiés, redéfinit localement un type qui existe déjà, et journalise l'erreur brute en console (`src/components/agent-local/branch-conflict-dialog.tsx:10-15`, `:40`, `:42`). Le contrôle de projet fait de même pour le commit avec changement de branche (`chat-project-controls.tsx:93`).

9. **Le vocabulaire mêle français et anglais dans la même phrase.** « {{count}} fichier(s) à Commit », « {{count}} commit(s) à Push vers … », « Merge dans {{branch}} », « Commit puis Merge dans {{branch}} » (`src/i18n/fr.json:368-369`, `:372`, `:380`). C'est un choix défendable — ce sont les mots que les utilisateurs de Git emploient — mais il n'est écrit nulle part, et le reste de l'application traduit tout.

10. **Le tutoiement domine dans ce domaine.** « Choisis la branche… », « Vérifie tes droits… », « Crée une nouvelle session… », « Connecte GitHub… » (`src/i18n/fr.json:374`, `:399`, `:540`, `:214`), alors que d'autres écrans vouvoient.

11. **La date d'un commit est affichée sans heure** (`src/components/agent-local/session-summary-commits.tsx:196`). Plusieurs commits du même jour sont donc indistinguables par leur date dans la liste.

---

## Points à confirmer

**Écarts à arbitrer avant publication**

1. **Le vocabulaire Commit / Push / Merge en anglais** (anomalie 9) : la documentation reprend-elle ces mots tels quels, comme l'interface, ou emploie-t-elle « valider », « publier », « fusionner » ? Les deux se défendent. **Il faut trancher une fois**, parce que le site et l'application ne peuvent pas se contredire sur les noms des boutons. Ma recommandation : reprendre les mots de l'interface entre guillemets quand on cite un bouton, et employer le français dans le texte courant — c'est ce que fait ce brief.

2. **Faut-il documenter l'absence de `fetch` et de `pull` en tête de page ?** C'est la limite la plus structurante du domaine, et celle qui produira le plus de messages d'incompréhension. Ma recommandation : oui, dès le premier tableau.

3. **L'exigence de connexion GitHub pour créer une branche locale** est-elle un choix produit assumé ou un effet de bord ? Le code est explicite (`services/git/github_auth.rs:5-25`), mais la raison n'est écrite nulle part. À faire confirmer avant d'écrire une justification sur le site.

4. **Le double comptage des fichiers modifiés** (section 5) est verrouillé par un test, donc voulu. Faut-il l'expliquer aux utilisateurs, ou l'équipe préfère-t-elle unifier le comptage ?

**Non vérifié — hors de portée d'une lecture du code**

5. **Le parcours de connexion GitHub** depuis le dialogue de création de branche n'a pas été suivi : les libellés « Ouverture de la connexion GitHub… » et « Vérification de la connexion GitHub… » (`src/i18n/fr.json:231-232`) supposent des états transitoires non observés.

6. **Le comportement réel d'un push refusé** demande un dépôt distant et des droits insuffisants ; il n'a pas été provoqué. Les cinq messages viennent de la lecture du code.

7. **Le conflit de fusion** n'a pas été provoqué non plus. Le chemin d'annulation et sa vérification sont couverts par un test du projet (`branch_merge_tests.rs:124`), ce qui rend le fait sûr, mais l'enchaînement visible à l'écran ne l'est pas.

8. **La formulation dans les six autres langues.** Tous les libellés cités ont été relus en français seulement.

**Affichage non vérifié — liste de contrôle pour la passe d'interface**

9. **Le menu du sélecteur de branches** : c'est une couche flottante ouverte depuis une barre de contrôle, avec deux listes, un champ de recherche et un dialogue monté à l'intérieur (anomalie 7). **C'est le point à vérifier en premier** : à contrôler qu'aucune partie n'est coupée, en haut comme en bas de fenêtre, dans les deux thèmes.

10. **Le dialogue de conflit d'extraction** : la liste des fichiers concernés, son défilement quand ils sont nombreux, et l'absence d'état de chargement pendant qu'elle se charge (`branch-conflict-dialog.tsx:40`).

11. **L'affichage d'une différence** : les couleurs d'ajout et de suppression dans les deux thèmes, le défilement horizontal des lignes longues, et le rendu du message de troncature.

12. **La liste des commits** : le chargement à l'approche du bas, l'état vide (**« Aucun commit sur cette branche »**, `src/i18n/fr.json:361`) et l'état d'erreur (**« Historique des commits indisponible. »**, `:364`).

13. **Les six catégories de statut de fichier** : « Créé », « Modifié », « Supprimé », « Renommé », « Copié » — et le fait que deux d'entre elles portent le **même libellé « Modifié »** (`src/i18n/fr.json:1312` et `:1316`, pour les statuts `modified` et `changed`). À vérifier à l'écran : deux catégories distinctes affichées sous un libellé identique se distinguent-elles autrement que par la couleur ?
