# Concepts clés

**Emplacement site** — Démarrage › Concepts clés (et source du glossaire de référence)
**Répond à** — « Qu'est-ce que ça veut dire ? » pour chaque terme employé ailleurs dans la documentation
**Sources** — `README.md`, `CLAUDE.md`, `src-tauri/src/services/agent_local/`, `src/components/`, `src-tauri/src/services/agent_local/tool_catalog.rs`
**Vérification** — Vérifié dans le code pour les termes techniques marqués ✓ ; issu du README pour les autres

---

## Plan de page proposé

Une page de définitions courtes, groupées par thème, chaque terme en `<dt>`/`<dd>` ou en titre de niveau 3. Sept groupes :

1. L'agent et son travail
2. Le contrôle de ce que fait l'agent
3. Déléguer et se souvenir
4. Les instructions permanentes
5. Les modèles
6. Les intégrations
7. L'automatisation, les données, la sécurité

**Recommandation de conception** : rendre chaque terme adressable par ancre (`#sous-agent`), pour que les autres pages puissent y renvoyer directement au lieu de redéfinir.

---

## Contenu

### Groupe 1 — L'agent et son travail

**Agent** — le modèle de langage augmenté d'outils. Il reçoit une demande, décide des actions, les exécute, recommence jusqu'à avoir terminé. Différence avec un chat : le chat répond, l'agent agit.

**Conversation (ou session)** — un fil de discussion. Chacune a son historique, son modèle, son répertoire de travail, ses outils actifs et ses permissions. On navigue de l'une à l'autre par la barre latérale ; elles ne s'ouvrent pas en onglets. ✓

**Clone** — une copie d'une conversation arrêtée à un message précis, ouverte dans un onglet à côté de l'originale. Deux modes : abandonner ce qui suivait, ou en garder un résumé. ✓ Les onglets n'existent que pour les clones, **trois au maximum** par groupe.

**Outil** — une capacité concrète : lire un fichier, lancer une commande, chercher sur le web. L'agent choisit lui-même lequel appeler et avec quels arguments.

**Groupe d'outils** — les outils sont rangés par famille. On active ou désactive un groupe entier. ✓ Groupes réels : `terminal`, `files`, `file_search`, `web`, `mcp`, `skills`, `automations`, `user_choice`, `subagents`, `plan_mode`, `todo_list`, `git_branches`, `forecast`, `spreadsheet`, `document`, `images`.

**Outil verrouillé** — toujours disponible, non désactivable. ✓ Les cinq groupes verrouillés : `terminal`, `files`, `file_search`, `web`, `mcp`.

**Outil optionnel** — s'active dans les réglages. ✓ Maximum **32** outils optionnels actifs simultanément (`MAX_OPTIONAL_TOOLS`).

### Groupe 2 — Le contrôle de ce que fait l'agent

**Mode de permission** — détermine ce que l'agent peut faire sans demander. ✓ Trois modes proposés à l'utilisateur : **Accès complet** (`auto` dans le code), **Demande d'approbation** (`manual`), **Chatbot** (`chat`). Un quatrième, `subagent`, existe uniquement en interne pour les sessions enfants et contourne la garde ; il n'est jamais proposé.

**Garde de permission** — le mécanisme qui intercepte une action avant exécution et demande approbation. ✓ Ne s'applique qu'à une liste précise d'outils, et sous conditions pour certains. Détail complet dans la page *Permissions*.

**Mode Plan** — l'agent explore sans rien modifier, rédige un plan Markdown, le soumet. Aucune écriture avant approbation. Les plans sont conservés dans `plans/<session_id>/*.md`. ✓

**Liste de tâches (todos)** — la liste que l'agent tient lui-même pour découper un travail long. Visible en direct. ✓ Cinq outils associés : `todo_write`, `todo_history`, `todo_pause`, `todo_resume`, `todo_delete`.

**Répertoire de travail** — le dossier auquel une conversation est rattachée. Détermine ce que l'agent peut lire et écrire. ✓

**Projet** — un répertoire de travail enregistré, retrouvé d'une conversation à l'autre. Stocké dans `projects.json`. ✓

### Groupe 3 — Déléguer et se souvenir

**Sous-agent** — une conversation enfant lancée par l'agent principal pour traiter une partie du travail de façon isolée, avec son propre contexte. ✓ Neuf outils de contrôle.

**Worktree** — une copie de travail Git séparée, dans laquelle un sous-agent modifie des fichiers sans toucher au dossier courant. Les changements s'inspectent avant d'être appliqués ou jetés. ✓ Stockées dans `subagent-worktrees/`.

**Mémoire** — des notes conservées d'une conversation à l'autre. Deux portées : **globale** et **par projet**. Modes manuel ou automatique. ✓ Stockage dans `memory/global/` et `memory/projects/`.

**Contexte** — l'information que le modèle garde sous les yeux : demande, historique, résultats d'outils, mémoire, instructions système. Limité, et surveillé par l'application. ✓

**Compression** — le résumé automatique de l'historique ancien quand le contexte se remplit, pour que la conversation continue sans être coupée. ✓

### Groupe 4 — Les instructions permanentes

**Prompt système** — les instructions données au modèle avant le premier message. ✓ Deux jeux : **Chatbot** et **Agentique**, chacun en variante **Compact** et **Détaillée**. Consultables, modifiables, remplaçables, désactivables.

**AGENTS.md** — fichier d'instructions permanentes écrit par l'utilisateur, lu à chaque conversation. Conventions à respecter, commandes du projet, interdits. ✓

**Personnalité** — fichiers Markdown décrivant le ton et le comportement attendus, injectés dans les instructions. ✓ Stockés dans `memory/core/`.

**Skill** — un mode d'emploi rangé dans un dossier, chargé à la demande quand la tâche correspond. ✓ Dossier `skills/`, outil `load_skill`.

### Groupe 5 — Les modèles

**Modèle local** — installé sur le disque, servi par Ollama. Aucune donnée ne quitte la machine, aucun coût par requête, performance dépendante du matériel.

**Modèle distant** — exécuté chez un fournisseur, atteint par le réseau. Généralement plus rapide et plus capable, facturé à l'usage.

**Provider (fournisseur)** — le service qui héberge un modèle distant. **Dix** fournisseurs LLM sont gérés.

**Clé API** — l'identifiant secret d'authentification auprès d'un fournisseur. Stockée chiffrée, jamais exposée à l'interface. ✓

**Compte web** — alternative à la clé API pour **trois** fournisseurs : OpenAI/Codex, Grok (xAI), Kimi (Moonshot, expérimental).

**Ollama** — le logiciel qui exécute les modèles locaux. Téléchargé et piloté par Beaver. ✓

**Raisonnement (thinking)** — la phase où certains modèles réfléchissent avant de répondre. Affiché par Beaver, avec réglage d'intensité quand le modèle le supporte. ✓

### Groupe 6 — Les intégrations

**MCP (Model Context Protocol)** — standard permettant à des programmes externes d'exposer des outils au modèle. Ajoute des capacités sans modifier Beaver.

**Connecteur** — une instance de serveur MCP configurée dans Beaver, locale ou distante. ✓ Configuration dans `mcp-connectors.json`.

**Extension** — un module installable qui étend l'application elle-même, distribué et mis à jour depuis une source Git. ✓

**Channel (canal)** — passerelle vers Telegram, Slack ou Discord. ✓

**Gateway** — le service d'arrière-plan qui fait tourner les canaux, associe chaque conversation externe à une session, et journalise les échanges. ✓ Audit dans `logs/gateway-audit.jsonl`.

### Groupe 7 — Automatisation, données, sécurité

**Réveil (wakeup)** — une instruction programmée. ✓ Trois types : `once`, `daily`, `weekly`. Résultat dans une conversation dédiée.

**Ordonnanceur (scheduler)** — le mécanisme interne qui déclenche les réveils. ✓ Interne à l'application ; n'utilise pas les tâches planifiées du système.

**Forecast** — l'espace de prévision de séries temporelles : audit, choix de modèle, prévision, évaluation, comparaison, export.

**Backtest** — évaluation qui rejoue le passé pour mesurer la qualité d'un modèle sur des données dont on connaît la suite.

**Coffre (vault)** — le fichier chiffré contenant clés API et jetons. ✓ Chiffrement **XChaCha20-Poly1305**, fichier `secrets.enc`.

**Trousseau (keyring)** — le magasin de secrets du système : Trousseau macOS, DPAPI Windows, Secret Service Linux. Contient la clé maîtresse du coffre, et rien d'autre. ✓

**Zéroïsation** — l'effacement volontaire d'un secret en mémoire après usage, pour qu'il ne subsiste pas dans un fichier d'échange ou une image mémoire. ✓

---

## Encadrés

Aucun. Une page de glossaire supporte mal les encadrés : ils cassent la lecture en balayage, qui est le seul usage réel de ce type de page.

---

## Pièges et erreurs fréquentes

**Confusion « mode de permission » / « mode du panneau ».** Deux sélecteurs distincts coexistent dans l'interface (`permission-mode-selector.tsx` et `mode-selector.tsx`). Le second contrôle l'affichage du panneau latéral, pas les permissions. Employer des noms nettement différents sur le site.

**Confusion « sous-agent » / « extension » / « connecteur ».** Les trois ajoutent des capacités, par des mécanismes sans rapport : le sous-agent est une session enfant, le connecteur expose des outils via MCP, l'extension modifie l'application. Le glossaire doit les distinguer explicitement.

**Confusion « mémoire » / « contexte ».** La mémoire persiste entre conversations ; le contexte est ce que le modèle voit dans la conversation courante. C'est la confusion la plus fréquente chez les utilisateurs d'agents.

---

## Renvois

Chaque terme renvoie vers sa page détaillée. Les renvois structurants :

- Outils, groupes, verrouillés/optionnels → *Outils › Vue d'ensemble*
- Modes de permission, garde → *Agent › Permissions*
- Sous-agents, worktrees → *Agent › Sous-agents*
- Mémoire, contexte, compression → *Agent › Mémoire* et *Agent › Contexte*
- Coffre, trousseau, zéroïsation → *Sécurité*

---

## Points à confirmer

- **Le nom français de « worktree ».** Le terme anglais est répandu chez les utilisateurs de Git, mais opaque pour les autres. Décider entre « worktree », « copie de travail » ou les deux (terme anglais entre parenthèses).
- **Le terme « provider ».** L'interface l'emploie en anglais. Choisir entre « fournisseur » et « provider » et s'y tenir dans les sept langues.
- **La liste des dix fournisseurs LLM.** Comptée d'après le tableau du README ; recouper avec le catalogue réel du code avant publication.
