# Les outils de l'agent — vue d'ensemble

**Emplacement site** — Outils › Vue d'ensemble (page d'entrée de la section)
**Répond à** — « Qu'est-ce que l'agent sait faire, et comment je décide de ce qu'il a le droit de faire ? »
**Sources** — `src-tauri/src/services/agent_local/tool_group_catalog.rs`, `tool_catalog.rs`, `tool_catalog_filter.rs`, `tool_availability.rs`, `tool_definitions.rs`, `tool_prompt_filter.rs`, `tool_result_truncate.rs`, `tool_result_budget.rs`, `tool_validate.rs`, `src/components/settings/tools-settings.tsx`, `src/i18n/fr.json` (clé `settings.tools`)
**Vérification** — Vérifié dans le code, sauf les points listés en fin de fiche

---

## Plan de page proposé

1. Ce qu'est un outil
2. Outils essentiels et outils optionnels
3. La liste complète des groupes
4. Ce qui est actif à l'installation
5. Activer ou désactiver un groupe
6. Ce que voit le modèle
7. Les résultats trop gros
8. Ce qui se passe quand un appel est mal formé

---

## Contenu

### Ce qu'est un outil

- Un outil est une capacité que l'agent peut invoquer lui-même pendant qu'il répond : lire un fichier, lancer une commande, chercher sur le web, déléguer une tâche.
- L'agent ne fait rien d'autre que produire du texte et appeler des outils. **Tout ce que Beaver sait faire sur la machine passe par un outil**, sans exception.
- Décider quels outils sont actifs, c'est donc décider ce que l'agent peut atteindre. C'est le réglage le plus structurant de l'application, plus que le choix du modèle.
- Un outil désactivé n'est **jamais proposé au modèle** : le modèle ne sait pas qu'il existe. Et si jamais il l'appelle quand même — parce que le réglage a changé au milieu d'une conversation, par exemple — l'appel est refusé avec un message clair. Les deux barrières existent, la seconde ne sert qu'en rattrapage.

### Outils essentiels et outils optionnels

Deux catégories, visibles telles quelles dans **Réglages › Agent › Outils** :

- **Outils essentiels** (`locked` dans le code) — toujours actifs, aucun interrupteur. L'interface affiche l'étiquette **« Toujours actif »** à leur droite. Ils forment le socle sans lequel l'agent ne peut rien faire d'utile.
- **Outils optionnels** — un interrupteur par groupe. Certains sont actifs à l'installation, d'autres non.

Le réglage porte sur **un groupe entier**, jamais sur un outil isolé. Activer « Sous-agents » active les neuf outils du groupe d'un coup.

### La liste complète des groupes

Voir les deux tableaux plus bas. Résumé chiffré, à donner en une ligne sur le site :

- **5 groupes essentiels**, 11 outils.
- **11 groupes optionnels**, 32 outils.
- **5 groupes optionnels sont actifs par défaut**, 6 sont éteints.

### Ce qui est actif à l'installation

- Actifs d'office : les 5 groupes essentiels, plus **Skills**, **Automatisations**, **Choix utilisateur**, **Sous-agents** et **Plan mode**.
- Éteints d'office : **Todo list**, **Branches Git**, **Forecast**, **Spreadsheet / Excel**, **Document / Word**, **Images**.
- Le choix de départ suit une logique simple à énoncer : ce qui sert à presque toutes les conversations est allumé, ce qui relève d'un usage précis est éteint pour ne pas encombrer le modèle.

> **Conséquence à écrire noir sur blanc sur le site** : un utilisateur qui demande « lis-moi ce fichier Excel » sans avoir activé le groupe Spreadsheet obtiendra une réponse évasive ou un contournement par le terminal, **pas** un message d'erreur clair. Le modèle ignore que l'outil existe.

### Activer ou désactiver un groupe

- Chemin : **Réglages › Agent › Outils**.
- Un interrupteur par groupe optionnel. La bascule est immédiate et vaut pour toutes les conversations — ce n'est pas un réglage par conversation.
- L'écran affiche pour chaque groupe un titre et une phrase qui explique à quoi il sert et quand le couper.
- En cas d'échec d'enregistrement, l'interrupteur revient à sa position précédente et un message d'erreur s'affiche.
- **Dépendance à connaître** : le groupe Sous-agents est piloté par l'outil `delegate_task`. Si `delegate_task` est absent de la sélection, les huit autres outils de sous-agents sont retirés automatiquement, même s'ils avaient été demandés. L'inverse est vrai aussi : activer `delegate_task` réactive tout le groupe.

### Ce que voit le modèle

C'est le point que la page doit expliquer parce qu'il n'est visible nulle part dans l'interface :

- Les définitions d'outils envoyées au modèle sont **filtrées** avant chaque requête : un outil désactivé n'apparaît pas dans la liste.
- Le **prompt système est filtré lui aussi**. Toute ligne qui nomme un outil désactivé est retirée, et deux sections entières disparaissent quand leur groupe est éteint :
  - la section des choix interactifs quand `ask_user_choice` est coupé ;
  - la section de travail avec les sous-agents quand `delegate_task` est coupé.
- Ce filtrage de section existe parce qu'un retrait ligne à ligne laisserait des puces orphelines : plusieurs lignes de ces sections ne nomment aucun outil.
- **Aucune liste des outils désactivés n'est transmise au modèle.** C'est un choix délibéré : mentionner un outil absent conduisait le modèle à annoncer des capacités qu'il n'avait pas.

### Les résultats trop gros

Deux mécanismes se superposent, à présenter dans cet ordre.

**1. Troncature par appel.** Certains outils ont un plafond de caractères pour un seul résultat. Au-delà :

- le modèle reçoit un en-tête indiquant la taille totale en Ko, puis les **2 000 premiers caractères**, puis le nombre de caractères omis ;
- le résultat complet est écrit sur le disque dans `~/.local/share/cl-go-dash/tool-results/<id de conversation>/` ;
- le chemin de ce fichier est donné au modèle, qui peut le relire avec `read_file` s'il a besoin de la suite ;
- si l'écriture sur disque échoue, le résultat porte un avertissement explicite — la sortie complète est alors perdue, et le site doit le dire.

La découpe se fait **par caractère et non par octet** : un résultat plein d'accents ou d'émojis n'est jamais coupé au milieu d'un caractère.

**2. Budget global de la conversation.** L'ensemble des résultats d'outils encore présents dans la conversation est plafonné à **100 000 caractères**. Au-delà, les résultats les plus anciens sont vidés et remplacés par une mention indiquant que la sortie a été retirée du contexte, avec le chemin du fichier complet quand il existe. **Les deux résultats les plus récents ne sont jamais vidés.**

Les fichiers de `tool-results/` sont supprimés automatiquement au-delà de **24 heures**.

### Ce qui se passe quand un appel est mal formé

Tout appel d'outil est validé avant exécution :

- un paramètre obligatoire manquant, un type incorrect ou un paramètre inconnu produisent un message d'erreur qui **liste les paramètres acceptés** ;
- l'agent reçoit ce message et corrige de lui-même, en général au tour suivant ;
- l'utilisateur voit l'aller-retour dans la conversation mais n'a rien à faire.

---

## Tableaux

### Groupes essentiels — toujours actifs

| Groupe (libellé affiché) | Identifiant | Outils | Ce qu'il permet |
|---|---|---|---|
| Terminal | `terminal` | `bash`, `bash_control` | Lancer des commandes sur la machine |
| Fichiers | `files` | `read_file`, `write_file`, `edit_file`, `list_dir` | Lire, créer, modifier des fichiers |
| Recherche de fichiers | `file_search` | `grep`, `glob` | Chercher un fichier ou un motif dans le projet |
| Web | `web` | `web_search`, `web_fetch` | Chercher en ligne et ouvrir une page |
| Connecteurs externes | `mcp` | `search_mcp_tools` | Utiliser les connecteurs MCP configurés |

### Groupes optionnels

| Groupe (libellé affiché) | Identifiant | Par défaut | Outils |
|---|---|---|---|
| Skills | `skills` | **Actif** | `load_skill` |
| Automatisations | `automations` | **Actif** | `manage_automation` |
| Choix utilisateur | `user_choice` | **Actif** | `ask_user_choice` |
| Sous-agents | `subagents` | **Actif** | `delegate_task`, `list_subagents`, `get_subagent`, `cancel_subagent`, `message_subagent`, `archive_subagent`, `inspect_subagent_changes`, `apply_subagent_changes`, `discard_subagent_changes` |
| Plan mode | `plan_mode` | **Actif** | `plan_mode` |
| Todo list | `todo_list` | Éteint | `todo_write`, `todo_history`, `todo_pause`, `todo_resume`, `todo_delete` |
| Branches Git | `git_branches` | Éteint | `create_branch`, `checkout_branch` |
| Forecast | `forecast` | Éteint | `forecast_data_audit`, `forecast_run`, `forecast_models`, `forecast_analyze`, `forecast_read`, `forecast_backtest`, `forecast_compare_models` |
| Spreadsheet / Excel | `spreadsheet` | Éteint | `read_spreadsheet`, `write_spreadsheet` |
| Document / Word | `document` | Éteint | `read_document`, `write_document` |
| Images | `images` | Éteint | `transform_image` |

### Plafond de résultat par outil

| Outil | Plafond avant troncature |
|---|---|
| `web_fetch` | **50 000 caractères** |
| `bash`, `bash_control` | **30 000 caractères** |
| `grep`, `web_search`, `list_dir` | **10 000 caractères** |
| `glob` | **5 000 caractères** |
| Tout résultat en erreur, quel que soit l'outil | **30 000 caractères** |
| Les autres outils | Pas de plafond à ce niveau |

Aperçu conservé en cas de troncature : **2 000 caractères**.

Budget cumulé de tous les résultats d'outils dans une conversation : **100 000 caractères**.

> `read_file` n'a **pas** de plafond ici : sa limite est ailleurs (20 Mo par fichier, 2 000 lignes par appel). Voir `05-outils/fichiers.md`.

### Ce que chaque interrupteur retire au modèle

| Interrupteur coupé | Effet direct | Effet sur le prompt système |
|---|---|---|
| Choix utilisateur | L'agent ne peut plus poser de question à choix, il tranche seul | La section des choix interactifs disparaît entièrement |
| Sous-agents | Aucune délégation possible | La section sur les sous-agents disparaît entièrement, et toute ligne mentionnant un sous-agent est retirée |
| Spreadsheet / Excel | Les tableurs ne sont plus ni lus ni écrits | Les consignes sur les formules de tableur sont retirées |
| Tous les autres | L'outil disparaît de la liste proposée | Les lignes qui le nomment sont retirées |

---

## Encadrés

> **Un outil désactivé est d'abord invisible, pas refusé.**
> Le modèle ne reçoit ni sa définition, ni la moindre mention de son existence. Il ne dira donc pas « je n'ai pas le droit » : il contournera, ou il répondra à côté. Quand une capacité manque, la première chose à vérifier est l'écran Réglages › Agent › Outils.
>
> Un refus explicite existe malgré tout au moment de l'appel, en filet de sécurité : il ne se déclenche que si le réglage a changé pendant la conversation.

> **Le réglage est global, pas par conversation.**
> Contrairement aux connecteurs externes, qui s'activent conversation par conversation, la sélection d'outils vaut pour toute l'application.

> **Activer tout n'est pas gratuit.**
> Chaque outil actif occupe de la place dans le contexte, à chaque requête, y compris quand l'agent ne s'en sert pas. Sur un modèle local à petite fenêtre de contexte, activer les onze groupes optionnels laisse mécaniquement moins de place à la conversation. Renvoyer vers `04-agent/contexte.md`.

> **Les groupes ne se panachent pas.**
> Il n'y a pas d'interrupteur par outil dans l'interface : on active « Forecast » ou on ne l'active pas, on ne peut pas garder `forecast_read` en coupant `forecast_run`.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause probable | Résolution |
|---|---|---|
| « L'agent refuse de lire mon fichier Excel » | Groupe Spreadsheet éteint (défaut) | Réglages › Agent › Outils › activer Spreadsheet / Excel |
| « L'agent ne me demande jamais mon avis » | Groupe Choix utilisateur coupé | Le réactiver, ou accepter qu'il tranche seul |
| « L'agent ne tient pas de liste de tâches » | Groupe Todo list éteint par défaut | L'activer |
| « L'agent ne délègue plus rien » | Groupe Sous-agents coupé | L'activer — les neuf outils reviennent ensemble |
| « Le résultat de ma commande est coupé » | Plafond de 30 000 caractères sur `bash` | Comportement normal : le résultat complet est sur le disque et l'agent peut le relire |
| « L'agent dit qu'il a perdu la sortie d'une commande d'il y a dix messages » | Budget global de 100 000 caractères atteint, la sortie a été retirée du contexte | Comportement normal ; lui redonner le chemin du fichier de résultat |

---

## Renvois

- `04-agent/fonctionnement.md` — comment l'agent enchaîne les appels d'outils
- `04-agent/permissions.md` — quels outils déclenchent une demande d'approbation
- `04-agent/contexte.md` — pourquoi le nombre d'outils actifs pèse sur le contexte
- `10-reglages/agent.md` — l'onglet Outils dans le plan complet des réglages
- `12-reference/limites-et-quotas.md` — la table de toutes les limites de l'application
- Chaque page de la section 05 pour le détail d'un groupe

---

## Points à confirmer

- **Le nombre d'outils optionnels est exactement égal à la limite interne.** Le catalogue compte 32 outils optionnels et la borne `MAX_OPTIONAL_TOOLS` vaut 32. Tout activer atteint donc pile la limite. Le point est sans conséquence aujourd'hui, mais la troncature se fait **en silence** (`.take(32)`, sans message) : le jour où un outil optionnel est ajouté, activer tous les groupes en désactivera un sans le dire. À signaler à l'équipe produit ; ne pas écrire sur le site.
- **Un douzième outil essentiel existe dans le catalogue mais n'appartient à aucun groupe** : `search_extension_tools`. Il est donc toujours actif et **n'apparaît pas dans l'écran Réglages**. Il relève des extensions, chantier gelé. À trancher quand la section Extensions sera dégelée : le documenter, ou le retirer du catalogue tant que les extensions ne sont pas livrées.
- **Le groupe Plan mode est actif par défaut** alors que le mode Plan doit être revu (chantier gelé). Vérifier avant publication si le réglage par défaut change en même temps que la fonctionnalité.
- Je n'ai **pas ouvert l'écran Réglages › Agent › Outils**. La disposition décrite vient de la lecture du composant : titre, phrase d'introduction, une carte « Tools essentiels » puis une carte « Tools optionnels ». À vérifier visuellement, notamment l'ordre d'affichage des groupes et le libellé exact des deux titres — le code affiche « Tools essentiels » et « Tools optionnels », avec le mot anglais *tools* alors que le reste de la page dit « outils ». **Incohérence de vocabulaire à remonter à l'équipe produit.**
- Le comportement quand une extension remplace un outil natif désactivé est implémenté (le remplacement reste indisponible) mais relève du chantier gelé Extensions. Ne pas documenter maintenant.
