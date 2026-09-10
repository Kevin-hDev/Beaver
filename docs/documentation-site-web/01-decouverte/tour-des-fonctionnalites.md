# Tour des fonctionnalités

**Emplacement site** — Démarrage › Tour des fonctionnalités (page d'aiguillage, à placer juste après la présentation)
**Répond à** — « Qu'est-ce que ça sait faire, et où est-ce que je lis le détail ? »
**Sources** — `README.md`, `src/components/` (23 domaines), `src-tauri/src/services/`, `src/features/extension-ui/core-occupants.tsx`, `src/components/settings/settings-sections.ts`, `src-tauri/src/services/llm/route_profile/catalog_api.rs:42-191`, `src-tauri/src/services/forecast/catalog_specs/mod.rs:16-41`, `src-tauri/src/services/agent_import/source_specs.rs:49-101`, `src-tauri/src/services/browser/cef_runtime_policy.rs:72-75`
**Vérification** — Vérifié dans le code pour les nombres (fournisseurs, familles Forecast, sources d'import, langues, thèmes, navigateur sur Linux) ; issu du README pour le reste, recoupé avec l'arborescence réelle des composants et des services

---

## Rôle de cette page

C'est une **page d'aiguillage**, pas une page de contenu. Son seul travail : faire comprendre l'étendue en une lecture, et envoyer vers la bonne section.

Contrainte de rédaction : **un paragraphe court par domaine, jamais plus de quatre phrases**. Une page de panorama qui détaille cesse d'être un panorama et devient une page qu'on ne lit pas.

---

## Plan de page proposé

Dix-sept domaines, dans cet ordre. L'ordre suit l'usage réel — ce qu'on découvre en premier vient en premier — et non l'architecture technique.

1. Agent et outils
2. Planification et permissions
3. Conversations et projets
4. Sous-agents
5. Mémoire persistante
6. Navigateur intégré
7. Git
8. Forecast
9. Fournisseurs et usage
10. Connecteurs MCP et canaux
11. Extensions
12. Réveils
13. Runtime Ollama géré
14. Espace de travail
15. Démarrage guidé et migration
16. Stockage local sécurisé
17. Langues et apparence

---

## Contenu

### 1. Agent et outils
Modèles locaux ou distants. Catalogue d'outils : fichiers, commandes shell, recherche par nom ou contenu, recherche web, Git, documents bureautiques, tableurs, images, prévision, connecteurs externes. Outils regroupés par famille ; certains toujours disponibles, d'autres activables, **32 optionnels maximum** simultanément.
→ *Agent local*, *Outils de l'agent*

### 2. Planification et permissions
Mode Plan : exploration en lecture seule, plan Markdown, approbation avant toute écriture. Plans conservés sur le disque. Indépendamment, chaque action modifiante peut être soumise à confirmation, en réglage global ou par conversation.
→ *Agent › Permissions*, *Agent › Mode Plan*

### 3. Conversations et projets
Navigation par la barre latérale. Pièces jointes. Mise en file d'attente de messages pendant que l'agent travaille. Archivage. Une conversation **naît dans un projet ou en dehors**, et cela ne change plus ensuite : il n'existe aucun moyen de rattacher après coup une conversation existante à un projet — ne jamais l'écrire. **Clonage** : repartir d'un message précis, en abandonnant la suite ou en en gardant un résumé des erreurs et des décisions.
→ *Interface › Conversations*, *Interface › Cloner une conversation*

### 4. Sous-agents
Délégation à des sessions isolées avec leur propre contexte. Suivi de l'état en direct, correction en cours de route, inspection des fichiers modifiés, application ou rejet des changements. Copies de travail Git nettoyées ensuite.
→ *Agent › Sous-agents*

### 5. Mémoire persistante
Notes conservées entre conversations, en portée globale ou par projet, en mode manuel ou automatique. Rangées par sujet, bornées en taille, consultables depuis les réglages. Accès en lecture seule pour les sous-agents.
→ *Agent › Mémoire persistante*

### 6. Navigateur intégré
Jusqu'à **dix onglets par conversation**. Sessions connectées conservées d'une fois sur l'autre. Détection des serveurs de développement locaux. Partage du panneau latéral avec les prévisualisations et Forecast.
**Disponible sur macOS et Windows uniquement** — mention obligatoire, c'est la seule fonctionnalité majeure non disponible partout. Il s'agit d'une limitation **par conception, pas temporaire** : hors macOS et Windows, la capacité vaut `BrowserCapability::Hidden` (`src-tauri/src/services/browser/cef_runtime_policy.rs:72-75`) et l'entrée disparaît du sélecteur au lieu de s'afficher en panne (`src/components/agent-local/mode-selector.tsx:97`).
→ *Interface › Navigateur intégré*

### 7. Git
Créer, changer, fusionner, supprimer branches et copies de travail. Committer et pousser. Parcourir les modifications non commitées. Inspecter les différences récentes ou anciennes dans l'historique.
→ *Automatisation › Workflow Git*

### 8. Forecast
Prévision de séries temporelles : audit de qualité des données, choix de modèle manuel ou automatique, exécution locale ou cloud, backtests glissants, comparaison, ensembles pondérés, analyse avancée, scénarios, notes, rapport, exports. **Dix familles** de modèles locaux — `chronos-bolt`, `chronos-2`, `timesfm-2-5`, `toto-2`, `moirai-2`, `flowstate`, `tabpfn-ts`, `tirex`, `kairos`, `sundial` — plus **TimeGPT** (`timegpt-2`) en cloud, soit onze familles au total.
→ *Forecast*

### 9. Fournisseurs et usage
**Onze** fournisseurs LLM par clé API, dont **trois** acceptant aussi une connexion par compte web. Affichage des limites, crédits, jetons consommés, requêtes et estimation de coût quand le fournisseur expose ces informations.
→ *Modèles et providers*

### 10. Connecteurs MCP et canaux
Connecteurs MCP locaux ou distants, activables conversation par conversation, ajoutant leurs outils à l'agent. Passerelle d'arrière-plan vers Telegram, Slack et Discord, avec journal d'audit.
→ *Intégrations*

### 11. Extensions
Modules installables qui **modifient l'interface elle-même**, distribués et mis à jour depuis une source Git. Une extension contribue son propre onglet de réglages, des entrées de navigation, un thème et des skills. Trois outils verrouillés du groupe `extensions` les rendent visibles à l'agent. À ne pas confondre avec un connecteur MCP, qui n'ajoute que des outils.
→ *Intégrations › Extensions*

### 12. Réveils
Instructions programmées une fois, chaque jour ou chaque semaine. Ordonnanceur interne à l'application. Chaque exécution atterrit dans une conversation dédiée, l'historique est conservé.
→ *Automatisation › Réveils*

### 13. Runtime Ollama géré
Téléchargement **proposé** au premier lancement dans le dossier de données — étape passable, rattrapable depuis **Réglages › Ollama**, jamais automatique. Réutilisation d'un démon déjà présent. Parcours et installation de modèles depuis l'application, édition des modelfiles, réglage des paramètres et des prompts système. Modèles partagés avec une installation Ollama existante.
→ *Modèles › Ollama*

### 14. Espace de travail
Terminal multi-onglets multiplateforme. Arbre de fichiers avec surveillance des modifications. Prévisualisations riches : texte, images, tableurs, documents bureautiques, aperçus de liens. Détail de l'occupation du contexte. Castor interactif.
→ *Interface*

### 15. Démarrage guidé et migration
Parcours d'accueil au premier lancement. Assistant d'import des instructions, skills et règles depuis **neuf** applications : Claude Code, Codex, Agents, Hermes, Qwen Code, ZCode, OpenClaw, OpenCode, Kimi Code.
→ *Installation › Onboarding*, *Installation › Import*

### 16. Stockage local sécurisé
Identifiants dans un coffre chiffré **XChaCha20-Poly1305**, clé maîtresse dans le trousseau du système. Aucun secret brut ne parvient à l'interface graphique.
→ *Sécurité*

### 17. Langues et apparence
Interface en **sept langues** : français, anglais, espagnol, allemand, italien, chinois, japonais. **Six thèmes** visuels. Réglage de la taille de police et du thème de coloration du code.
→ *Interface › Thèmes*, *Interface › Langues*

---

## Tableaux

Aucun tableau sur cette page. Un panorama en dix-sept entrées se lit en liste ; le mettre en tableau le rend illisible sur mobile.

**Suggestion de conception** : une grille de cartes cliquables, une par domaine, avec titre et une phrase. Le mockup dispose déjà des styles nécessaires.

---

## Encadrés

**Encadré « Disponibilité par plateforme »** — à placer au domaine 6.
> Le navigateur intégré est disponible sur macOS et Windows. Sur Linux, il n'apparaît pas dans l'interface : la fonctionnalité est masquée plutôt qu'affichée en panne. Toutes les autres fonctionnalités sont disponibles sur les trois systèmes.

---

## Pièges et erreurs fréquentes

**Transformer le panorama en documentation.** C'est le risque principal de cette page. Chaque domaine mérite quatre phrases au maximum ; le détail vit ailleurs.

**Oublier la restriction du navigateur.** Un utilisateur Linux qui découvre après installation que le navigateur intégré n'existe pas chez lui a été mal informé. La restriction doit apparaître dès le panorama.

**Annoncer des fonctionnalités au futur.** Cette page ne décrit que ce qui existe dans la version publiée — **1.2.2** au moment de l'audit. Tout ce qui n'est pas encore sorti, quelle qu'en soit la forme dans le CHANGELOG, n'y a pas sa place.

---

## Renvois

Cette page renvoie vers toutes les sections. Elle est le point de départ naturel du sommaire après la présentation.

---

## Points à confirmer

- ~~Le nombre exact de langues et de thèmes.~~ **Tranché** : les **sept** langues sont toutes proposées dans le sélecteur (`src/components/settings/general-settings-options.ts:14-22`) et les **six** thèmes le sont aussi, plus l'option Système (`src/lib/app-themes.ts:1-30`).
- ~~La disponibilité du navigateur intégré sur Linux.~~ **Tranché** : limitation **par conception**, état `Hidden` (`services/browser/cef_runtime_policy.rs:72-75`).
- ~~La liste des neuf applications sources d'import.~~ **Tranché** : `claude`, `codex`, `agents`, `hermes`, `qwen`, `zcode`, `openclaw`, `opencode`, `kimi`, plafonnées à neuf (`services/agent_import/source_specs.rs:49-101`, `agent_import/limits.rs:1`). Le registre est `source_specs.rs`, pas `registry.rs`.
- ~~Le nombre de familles de modèles Forecast.~~ **Tranché** : **dix** familles locales plus TimeGPT en cloud (`services/forecast/catalog_specs/mod.rs:16-41`).
- **La formulation des noms de fournisseurs et de familles de modèles sur le site.** Le code donne les identifiants techniques ; les noms commerciaux exacts à afficher restent à arrêter avec l'équipe du site.
