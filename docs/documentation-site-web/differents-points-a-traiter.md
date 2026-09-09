# Documentation du site web — points à traiter

Liste des fichiers Markdown à produire dans `docs/documentation-site-web/`.
Chaque fichier est un **brief de contenu** destiné au développeur qui construira
la page de documentation du site : faits vérifiés, valeurs exactes, procédures,
tableaux et pièges. Ce n'est pas la prose finale.

Le gabarit et les conventions sont dans `00-comment-utiliser-ces-fichiers.md`.
Mockup de référence : `docs/beaver-site/mockup/docs.html`.

Référence projet : Beaver v1.1.2 (+ v1.1.3 et Unreleased au CHANGELOG).

On coche au fur et à mesure.

---

## 0 — Méthode

- [x] `00-comment-utiliser-ces-fichiers.md` — gabarit, conventions, niveaux de vérification, correspondance avec le sommaire du mockup

---

## 1 — Découverte

- [x] `01-decouverte/presentation.md` — ce qu'est Beaver, à qui ça s'adresse, ce qui le distingue d'un chat classique
- [x] `01-decouverte/concepts-cles.md` — vocabulaire : agent, outil, session, sous-agent, permission, mémoire, MCP, réveil, provider
- [x] `01-decouverte/tour-des-fonctionnalites.md` — panorama de haut niveau, une ligne par grand domaine, avec les liens internes
- [x] `01-decouverte/local-vs-cloud.md` — modèles locaux (Ollama) vs modèles API vs comptes web, ce que ça change en coût, confidentialité et vitesse

## 2 — Installation et premier lancement

- [x] `02-installation/prerequis.md` — OS supportés, RAM, GPU, espace disque, connexion réseau
- [x] `02-installation/installation-macos.md` — `.dmg`, `install.sh`, Gatekeeper et absence de signature, désinstallation
- [x] `02-installation/installation-windows.md` — installeur NSIS, `install.ps1`, SmartScreen, désinstallation
- [x] `02-installation/installation-linux.md` — `.deb`, dépendances, détection GPU AMD/Nvidia, désinstallation
- [x] `02-installation/premier-lancement.md` — écran de setup, téléchargement d'Ollama, ce qui se crée sur le disque
- [x] `02-installation/onboarding.md` — parcours guidé : préférences, providers, import, écran de bienvenue
- [x] `02-installation/import-depuis-un-autre-assistant.md` — Claude Code, Codex, Agents, Hermes, Qwen Code, ZCode, OpenClaw, OpenCode, Kimi Code : ce qui est importé, sauvegardes
- [x] `02-installation/mise-a-jour.md` — vérification, manifestes SHA-256, notes de version, installation, retour arrière

## 3 — Interface

- [x] `03-interface/vue-densemble.md` — navigation principale (Sessions, Réveils, Personnalité, Réglages), barre d'outils, panneaux
- [x] `03-interface/conversations-et-onglets.md` — navigation par la sidebar, archivage, recherche, limites
- [x] `03-interface/cloner-une-conversation.md` — clone depuis un message précis, modes couper/résumer, onglets de clones, lien Git
- [x] `03-interface/panneau-lateral.md` — arbre de fichiers, prévisualisations, navigateur, Forecast : comment ils partagent le panneau
- [x] `03-interface/terminal-integre.md` — PTY multi-onglets, limites, comportement par OS
- [x] `03-interface/arbre-de-fichiers-et-previews.md` — navigation, watcher, previews texte/image/binaire/tableur/document/lien
- [x] `03-interface/navigateur-integre.md` — jusqu'à 10 onglets par conversation, sessions connectées, détection des serveurs de dev, disponibilité macOS/Windows
- [x] `03-interface/themes-et-apparence.md` — les 6 thèmes, taille de police, thème de code, accélération matérielle
- [x] `03-interface/mascotte.md` — le castor interactif, réglages, fenêtre dédiée
- [x] `03-interface/raccourcis-clavier.md` — table complète par OS (⌘ / Ctrl)
- [x] `03-interface/langues.md` — les 7 langues, changement, ce qui reste en anglais

## 4 — Agent local

- [x] `04-agent/fonctionnement.md` — boucle agentique, streaming, thinking, arrêt, file d'attente de messages, reprise
- [x] `04-agent/repertoire-de-travail.md` — ancrage du répertoire, portée d'accès disque, projets, garde-fous
- [x] `04-agent/permissions.md` — modes automatique / manuel / par conversation, permission gate, cache d'autorisations, commandes shell
- [ ] ⏸️ `04-agent/plan-mode.md` — **gelé** : le mode Plan doit être modifié, il n'accorde pas assez d'autorisations au modèle pour qu'une exploration se déroule correctement. Brief rédigé puis mis de côté dans `_geles/plan-mode.md.gele`
- [x] `04-agent/todos.md` — création, historique, pause/reprise, suppression, relance en cas d'oubli
- [x] `04-agent/sous-agents.md` — délégation, sessions isolées, worktrees, suivi, messages, application ou rejet des changements
- [x] `04-agent/memoire-persistante.md` — mémoire globale et par projet, modes manuel/auto, fichiers de sujets, bornes, accès des sous-agents
- [x] `04-agent/personnalite-et-agents-md.md` — `AGENTS.md`, fichiers `memory/core/`, injection de personnalité
- [x] `04-agent/prompts-systeme.md` — prompts Chatbot et Agentique, variantes Compact/Détaillé, remplacement, prompts natifs Ollama par modèle
- [x] `04-agent/skills-locaux.md` — format d'un skill, chargement, dossier `skills/`
- [x] `04-agent/contexte.md` — budget de contexte, élagage, écran d'usage du contexte, capacité dépassée
- [ ] ⏸️ `04-agent/compression.md` — **gelé** : la compression va être revue
- [x] `04-agent/pieces-jointes.md` — types acceptés, limites, traitement
- [x] `04-agent/diagnostics-et-erreurs.md` — diagnostics, redaction, circuit breaker, messages d'erreur courants

## 5 — Outils de l'agent

- [x] `05-outils/vue-densemble.md` — outils verrouillés vs optionnels, groupes, limite de 32 outils optionnels, activation dans les Réglages
- [x] `05-outils/terminal-et-shell.md` — `bash`, `bash_control` : exécution, arrière-plan, sandbox shell, validation
- [x] `05-outils/fichiers.md` — `read_file`, `write_file`, `edit_file`, `list_dir` : garde-fous d'écriture, limites, erreurs
- [x] `05-outils/recherche-fichiers.md` — `grep`, `glob` : syntaxe, timeouts, budget de résultats
- [x] `05-outils/web.md` — `web_search`, `web_fetch` : providers, fallback SearXNG, protections réseau
- [x] `05-outils/mcp.md` — `search_mcp_tools`, appel d'outils MCP depuis l'agent
- [x] `05-outils/skills-et-automatisations.md` — `load_skill`, `manage_automation`
- [x] `05-outils/choix-interactif.md` — `ask_user_choice` : quand l'agent pose une question à choix
- [x] `05-outils/sous-agents-outils.md` — les 9 outils de délégation et de revue des changements
- [x] `05-outils/git.md` — `create_branch`, `checkout_branch` et le workflow Git complet côté interface
- [x] `05-outils/forecast-outils.md` — les 7 outils Forecast utilisables depuis une conversation
- [x] `05-outils/tableurs.md` — `read_spreadsheet`, `write_spreadsheet` : formats, plages, mise en forme, limites
- [x] `05-outils/documents.md` — `read_document`, `write_document` : formats Office, styles, listes, numérotation
- [x] `05-outils/images.md` — `transform_image` : opérations, limites de taille

## 6 — Modèles et providers

- [x] `06-modeles/ollama-runtime.md` — runtime managé, réutilisation d'un daemon existant, port 11434, arrêt propre, logs
- [x] `06-modeles/ollama-modeles.md` — parcourir, installer, supprimer, modèles partagés avec Ollama.app, téléchargements
- [x] `06-modeles/ollama-personnalisation.md` — modelfiles, paramètres, prompts système par modèle, modèles custom
- [x] `06-modeles/providers-api.md` — table des 10 providers LLM, où récupérer la clé, comment la saisir, test de connexion — **sans aucun tarif**, voir la décision ci-dessous
- [x] `06-modeles/providers-comptes-web.md` — OpenAI/Codex, Grok, Kimi : authentification web, jetons, limites
- [x] `06-modeles/catalogue-et-favoris.md` — explorateur LLM, familles, détails d'un modèle, favoris
- [x] `06-modeles/raisonnement.md` — effort de raisonnement, différences par provider, affichage du thinking
- [x] `06-modeles/usage-et-couts.md` — limites, crédits, tokens, requêtes, estimation de coût, historique
- [x] `06-modeles/materiel-et-vram.md` — détection GPU, table VRAM, choisir une taille de modèle

## 7 — Intégrations

- [x] `07-integrations/recherche-web.md` — Brave, Exa, Firecrawl, SearXNG local : différences, configuration, routage
- [x] `07-integrations/mcp-connecteurs.md` — ajouter un connecteur local ou distant, activation par conversation, runtime, durcissement
- [x] `07-integrations/mcp-oauth.md` — connecteurs cloud, callback OAuth, stockage des jetons, révocation
- [ ] ⏸️ `07-integrations/extensions-centre.md` — **en attente** : centre d'extensions, découverte, catalogue, installation, mise à jour, désactivation, suppression
- [ ] ⏸️ `07-integrations/extensions-remplacer-un-outil.md` — **en attente** : substitution d'un outil natif, masquage, priorité, diagnostics
- [ ] ⏸️ `07-integrations/extensions-prompt-systeme.md` — **en attente** : réécriture du prompt système par une extension, portée, précédence
- [ ] ⏸️ `07-integrations/extensions-ecrire.md` — **en attente** : écrire sa propre extension, structure, hôte, canal de communication, source Git, limites et sécurité

> ⏸️ **Les quatre pages Extensions sont gelées.** L'implémentation a été
> interrompue en cours de route et doit être finalisée avant qu'on documente
> quoi que ce soit — documenter un comportement qui va changer produit une doc
> fausse le jour de la sortie.
>
> C'est la section qui demandera le plus de soin quand elle sera débloquée :
> installer du code tiers qui remplace des outils et réécrit le prompt système
> engage la sécurité de l'utilisateur. Il devra être guidé pas à pas, avec les
> risques énoncés explicitement.
- [x] `07-integrations/channels-gateway.md` — Telegram, Slack, Discord : mise en place, mapping vers les sessions, audit, sécurité

## 8 — Forecast

- [ ] `08-forecast/vue-densemble.md` — à quoi sert l'espace Forecast, parcours type de bout en bout
- [ ] `08-forecast/donnees-et-audit.md` — import, profil de données, qualité, fréquence, valeurs manquantes, anomalies
- [ ] `08-forecast/modeles-locaux.md` — Chronos, TimesFM, Toto 2.0, MOIRAI 2.0, FlowState, TabPFN-TS, TiRex, Kairos, Sundial : capacités, installation, matériel requis
- [ ] `08-forecast/modele-cloud-timegpt.md` — Nixtla TimeGPT-2 / 2.1, clé API, limites
- [ ] `08-forecast/selection-du-modele.md` — sélection manuelle vs automatique, critères utilisés
- [ ] `08-forecast/evaluation-et-comparaison.md` — backtests glissants, MASE, sMAPE, MAE, couverture, baselines, ensembles pondérés
- [ ] `08-forecast/analyse-avancee.md` — décomposition, dérive, importance des variables, anomalies
- [ ] `08-forecast/scenarios-notes-rapports.md` — scénarios, notes, vue Rapport
- [ ] `08-forecast/exports.md` — CSV, Excel, JSON, PNG, SVG, PDF, presse-papier

## 9 — Automatisation

- [ ] `09-automatisation/reveils.md` — réveils ponctuels, quotidiens, hebdomadaires ; scheduler interne ; conversations dédiées ; pause globale
- [ ] `09-automatisation/historique-des-reveils.md` — journal `wakeups.jsonl`, rétention, lecture des résultats
- [ ] `09-automatisation/git-workflow.md` — branches, worktrees, commits, push, merge, diffs, historique, changements non commités

## 10 — Réglages (référence)

- [ ] `10-reglages/reference-complete.md` — plan des 5 sections et 16 onglets, ce que chacun contient
- [ ] `10-reglages/general-et-preferences.md` — langue, démarrage, mascotte, raccourcis, apparence
- [ ] `10-reglages/agent.md` — mémoire, prompt système, outils, avancé
- [ ] `10-reglages/modeles.md` — Ollama, Forecast, LLM
- [ ] `10-reglages/integrations.md` — providers, connecteurs, channels, extensions
- [ ] `10-reglages/application.md` — conversations archivées, accès fichiers, à propos

## 11 — Sécurité et confidentialité

- [ ] `11-securite/modele-de-securite.md` — vue d'ensemble : ce qui sort de la machine, ce qui n'en sort jamais
- [ ] `11-securite/vault-et-cles-api.md` — XChaCha20-Poly1305, clé maîtresse dans le keyring OS, zéroïsation, JS n'accède jamais à une clé
- [ ] `11-securite/acces-fichiers.md` — portée du répertoire, protection contre la traversée de chemin, permissions par plateforme
- [ ] `11-securite/durcissement.md` — collections bornées, HTTP sécurisé, MCP (allowlist, pas de shell), navigateur isolé, logs filtrés
- [ ] `11-securite/mises-a-jour-verifiees.md` — métadonnées strictes, téléchargements bornés, health checks, échec fermé
- [ ] `11-securite/confidentialite-des-donnees.md` — ce que voient les providers, télémétrie, données locales, effacement
- [ ] `11-securite/signaler-une-vulnerabilite.md` — procédure, périmètre, délais

## 12 — Référence

- [ ] `12-reference/stockage-local.md` — arborescence `~/.local/share/cl-go-dash/`, rôle de chaque fichier, sauvegarde et restauration
- [ ] `12-reference/formats-supportes.md` — fichiers lisibles, éditables, prévisualisables, formats Office et tableurs
- [ ] `12-reference/limites-et-quotas.md` — 32 flux actifs, 16 PTY, 2000 messages, 10 onglets navigateur, 32 outils optionnels, tailles maximales
- [ ] `12-reference/journaux.md` — où sont les logs, ce qu'ils contiennent, ce qui en est exclu, rotation
- [ ] `12-reference/glossaire.md` — définitions courtes de tous les termes du produit

## 13 — Dépannage

- [ ] `13-depannage/installation.md` — Gatekeeper, SmartScreen, dépendances Linux, permissions disque
- [ ] `13-depannage/ollama.md` — daemon indisponible, port occupé, téléchargement interrompu, GPU non détecté, modèle trop lourd
- [ ] `13-depannage/providers-et-cles.md` — clé refusée, quota atteint, expiration OAuth, erreurs réseau
- [ ] `13-depannage/agent-et-outils.md` — outil bloqué, permission refusée, contexte saturé, boucle interrompue
- [ ] `13-depannage/mcp-extensions-channels.md` — connecteur qui ne démarre pas, OAuth échoué, extension incompatible, gateway silencieux
- [ ] `13-depannage/forecast.md` — modèle non installé, données rejetées, mémoire insuffisante
- [ ] `13-depannage/faq.md` — questions récurrentes qui ne relèvent d'aucune page ci-dessus

## 14 — Projet

- [ ] `14-projet/architecture-technique.md` — stack, arborescence backend/frontend, choix structurants (pour la page « sous le capot »)
- [ ] `14-projet/build-depuis-les-sources.md` — prérequis dev, commandes, tests, lint, build par OS
- [ ] `14-projet/contribuer.md` — CLA, conventions, checks obligatoires, ouverture d'une PR
- [ ] `14-projet/licence.md` — AGPL v3, obligations, licence commerciale, composants tiers, historique Apache 2.0
- [ ] `14-projet/versions-et-changelog.md` — politique de version, où lire les notes, historique

---

## Suivi

- Fichiers prévus : 94
- Fichiers rédigés : 63 (sections 0 à 7 terminées — section 7 complète hors les 4 pages Extensions gelées)
- Fichiers gelés : 6 (4 Extensions + Mode Plan + Compression) — voir `_geles/README.md`
- Fichiers rédigeables maintenant : 88

### Règle de sourcing

Le code fait foi. `README.md`, `CHANGELOG.md` et `SECURITY.md` sont maintenus
avec les releases et restent fiables. **Tout le contenu de `docs/` est daté** et
ne sert que de piste à vérifier — il a déjà produit trois erreurs relevées
ci-dessous. Détail dans `00-comment-utiliser-ces-fichiers.md`.

### Aucun tarif de provider sur le site — tranché

Le site **ne publie aucun prix** : ni tarif par million de jetons, ni prix
d'abonnement, ni palier de crédits. Les pages renvoient vers la page de
tarification officielle de chaque fournisseur.

**Pourquoi.** Beaver ne code aucun tarif en dur : il télécharge le catalogue de
LiteLLM (`services/llm/litellm_catalog_refresh.rs:9`), le met en cache dans
`litellm-models.json` et le rafraîchit par requête conditionnelle. Les prix
affichés dans l'application viennent donc d'une source vivante, mise à jour sans
intervention. Publier des prix sur le site créerait une **seconde autorité** qui
divergerait de celle que l'application utilise, et un utilisateur voyant deux
chiffres différents ne saurait lequel croire.

S'ajoute la raison de maintenance : les tarifs changent plusieurs fois par an, et
un prix faux sur une page de documentation fait douter de tout le reste de la
page. Le public visé — des utilisateurs d'applications agentiques — connaît déjà
ces ordres de grandeur.

**Ce qui reste à documenter**, et qui est du ressort du site : comment lire
l'écran d'usage de Beaver, d'où viennent ses chiffres, et pourquoi ce sont des
estimations et non une facture.

### Libellés des modes de permission — tranché

| Code | Libellé affiché |
|---|---|
| `auto` | Accès complet |
| `manual` | Demande d'approbation |
| `chat` | Chatbot |

`subagent` est interne, jamais proposé à l'utilisateur.

### Écarts relevés entre le mockup et le code

À trancher pendant la rédaction, notés ici pour ne pas les perdre :

- Le tableau des modes de permission du mockup simplifie le comportement réel :
  plusieurs outils ne déclenchent une approbation que sous condition
  (`permission_gate.rs:33-46`). À corriger dans `04-agent/permissions.md`.
- Le mockup ignore le mode interne `subagent`, qui contourne la garde.
- Le mockup ne mentionne pas le journal `logs/permission-diagnostics.jsonl`
  (rotation à 2 Mo), absent aussi du README et de `CLAUDE.md`.
- Le sommaire du mockup n'a ni section Interface ni section Réglages, alors que
  l'application compte 16 onglets de réglages qui demandent une page de référence.
- **Le mockup décrit un multi-onglet de conversations qui n'existe plus.** Il a
  été retiré du produit : la navigation passe uniquement par la barre latérale.
  Seuls les clones créent des onglets, 3 au maximum par groupe
  (`session_tabs_state.rs:6`).
- Le mockup nomme l'opération « brancher » ; le code dit « clone ». Terme à fixer.

### Erreurs de documentation interne à corriger dans le dépôt

Relevées en écrivant, indépendantes du site mais à traiter :

- **`CROSS-PLATFORM.md` annonce un support Fedora/RHEL via `dnf`.** `install.sh`
  n'appelle que `apt-get` (ligne 153) et ne construit que le suffixe `_amd64.deb`.
  L'affirmation est fausse.
- **`CLAUDE.md` affirme que la release CI est créée en non-draft.**
  `.github/workflows/release.yml` utilise `--draft` (lignes 353-385).
- **Le mode de permission par défaut est `auto`.** `storage_migration.rs` crée
  `agent-settings.json` avec `{"permissionMode":"auto"}`, alors que le mockup
  présente « Demande d'approbation » comme le mode recommandé au quotidien.
  L'application démarre donc dans le mode le moins prudent, sans le signaler.
- **`CROSS-PLATFORM.md` date d'avril 2026** (versions v0.6.x–v0.7.x) pour une
  version courante 1.1.2. Les constats GPU et les problèmes connus demandent
  une passe de vérification.
- **Deux dossiers créés au premier lancement ne sont documentés nulle part** :
  `translations/` et `inbox/` (avec `inbox/pending.json`).

### Relevé en écrivant la section 5 — à trancher côté produit

Ces points ne sont pas des erreurs de documentation mais des constats sur le
code. Rangés du plus au moins urgent.

- **`read_document` annonce un filtrage par pages qu'il n'applique pas.** La
  définition transmise au modèle décrit un paramètre de plage de pages pour les
  PDF ; `tool_document_read.rs:12` reçoit ce paramètre sous le nom `_pages` et
  ne s'en sert jamais. Le document entier est toujours extrait. **Seul écart
  franc entre promesse et implémentation de toute la section.** Soit
  implémenter, soit retirer le paramètre.
- **Aucune borne sur le texte extrait d'un PDF.** L'extraction Word est
  plafonnée à un million de caractères ; le chemin PDF (`read_pdf`) ne pose
  aucune limite équivalente avant de construire son résultat.
- **La limite d'outils optionnels vaut exactement le nombre d'outils
  optionnels.** `MAX_OPTIONAL_TOOLS = 32` et le catalogue en compte 32. Tout
  activer atteint donc la limite au caractère près, et la troncature se fait en
  silence (`.take(32)`). Le jour où un outil optionnel est ajouté, activer tous
  les groupes en désactivera un sans le dire.
- **`search_extension_tools` est verrouillé mais n'appartient à aucun groupe.**
  Il est donc toujours actif et **absent de l'écran Réglages › Agent › Outils**.
  Relève du chantier Extensions gelé : le documenter ou le retirer du catalogue
  tant que les extensions ne sont pas livrées.
- **L'écran des outils dit « Tools essentiels » et « Tools optionnels ».** Le
  mot anglais apparaît dans une interface française, alors que le reste de la
  page parle d'outils (`fr.json`, clés `settings.tools.lockedTitle` et
  `optionalTitle`). À vérifier dans les six autres langues.
- **« Authentification GitHub requise » peut remonter à la création d'une
  branche locale** (`tool_git_error.rs`, variante `GithubAuthRequired`). Créer
  une branche est une opération locale : si ce message atteint réellement
  l'utilisateur dans ce cas, il est trompeur.
- **Les sous-agents portent des noms visibles fixes** — « Claudiator » pour le
  codeur, « Geminitor » pour l'explorateur (`tool_definitions_subagent.rs:38`).
  Ces noms évoquent d'autres produits. À confirmer : sont-ils réellement
  affichés ? Sont-ils voulus ?
- **Deux paramètres de `delegate_task` sont marqués « legacy »** dans leur
  propre description (`name`, `display_name`). Code mort à nettoyer.
- **La limite de 15 outils MCP par connecteur dans une recherche est
  silencieuse** : rien n'indique que le connecteur en offrait davantage.
- **Une fonctionnalité non documentée existe** : les définitions d'agents
  spécialisés réutilisables, en Markdown dans le projet (paramètre `agent_path`,
  chemin type `.beaver/agents/<nom>.md`). Ni le README, ni `CLAUDE.md`, ni le
  mockup n'en parlent. À explorer avant publication.

### Relevé en écrivant la section 6

- **Les paliers gratuits de Google et Mistral affichés dans l'application ne sont
  plus publiables.** Deux commentaires de `services/llm/catalog.rs` (vérifiés le
  30 juillet 2026) le disent : ces fournisseurs ont retiré ces chiffres de leurs
  pages publiques, et ce qu'affiche Beaver repose sur des sources tierces. À
  revérifier ou à retirer des textes de l'application.
- **Aucune borne sur le nombre de modèles favoris** (`services/favorite_models.rs`).
  Le fichier grandit sans limite, alors que le projet borne ses collections
  partout ailleurs. Défaut mineur, mais c'est une exception à une règle tenue.
- **La vérification des mises à jour de modèles s'arrête à 100 familles**
  (`commands/ollama_updates.rs`) sans le signaler. Au-delà, certains modèles ne
  sont jamais examinés et rien ne l'indique.
- **Rien ne signale à l'utilisateur que Beaver réutilise un moteur Ollama
  existant.** Dans ce cas, les réglages de moteur de Beaver n'ont aucun effet
  (`ollama_lifecycle.rs:69-79`) — et l'utilisateur n'a aucun moyen de le
  comprendre. Recommandation : l'afficher dans l'écran des modèles.
- **Sur Mac Intel, la mémoire n'est pas mesurée** (`gpu_vram/macos.rs` sort
  immédiatement hors architecture Apple), et le contexte tombe au palier minimal
  de 8 192 jetons. À confirmer comme volontaire.
- **Le nom « Codex » désigne le mécanisme de connexion OpenAI** dans tout le
  code. À ne pas reprendre sur le site sans arbitrage : ce nom désigne un produit
  précis chez OpenAI et induirait en erreur sur ce à quoi l'utilisateur se
  connecte.

### Relevé en écrivant la section 7

- **La recherche web sans clé exige un interpréteur Python 3 installé sur la
  machine**, et ce prérequis n'est documenté nulle part — ni README, ni prérequis
  d'installation, ni interface. `services/searxng/runtime.rs:106` cherche
  `python3.13` à `python`, et échoue avec « runtime Python introuvable » sinon.
  Python est présent par défaut sur Linux, généralement sur macOS, **jamais sur
  Windows**. Un utilisateur Windows découvrira donc que la recherche ne fonctionne
  pas, sans message explicite. **Le point le plus urgent de cette section** :
  embarquer un interpréteur, ou l'annoncer dans les prérequis avec un message
  clair dans l'application.
- **Le moteur de recherche local semble ne jamais s'arrêter en cours de
  session.** `lifecycle.rs` l'arrête à la fermeture de l'application, mais aucune
  mise en veille après inactivité n'a été trouvée — contrairement aux processus
  MCP, arrêtés après 10 minutes. À vérifier : un processus Python résident
  consomme de la mémoire en continu.
- **Le mode de permission appliqué aux messages reçus par le gateway n'est pas
  déterminé.** Un message arrivé par Telegram déclenche un agent qui accède aux
  fichiers et lance des commandes ; on ne sait pas s'il passe par les demandes
  d'approbation, ni comment y répondre depuis une messagerie. **Question de
  sécurité à clarifier avant de publier la page.**
- **Les identifiants d'application OAuth pour Google et GitHub** sont rangés dans
  le coffre (`mcp_oauth/static_credentials.rs`). Leur origine n'est pas claire :
  si l'utilisateur doit enregistrer sa propre application chez le service, c'est
  une étape majeure absente du parcours documenté.
- **Le connecteur iMessage donne accès aux messages personnels** sur macOS. Il
  mérite un traitement à part sur le site, avec ses implications énoncées
  clairement plutôt qu'une ligne dans un tableau.
- **Les connecteurs locaux ne se mettent pas à jour.** Leurs versions sont figées
  dans le code (`stdio_catalog.rs`) : une mise à jour de connecteur demande une
  mise à jour de Beaver. C'est une protection — pas de version compromise
  installée silencieusement — mais aussi une contrainte à documenter.

### Constat de conception à mettre en avant sur le site

- **Le bac à sable du shell ne s'active que si l'accès disque est restreint**
  (`shell_sandbox/launch.rs:42`). Avec le réglage par défaut — la racine du
  disque — les commandes tournent sans isolation. Dès que la portée est réduite,
  chaque commande est enfermée par le système lui-même : Seatbelt sur macOS,
  Landlock sur Linux, profil restreint sur Windows. C'est le seul réglage de
  Beaver dont l'effet est garanti par le noyau et non par le code de
  l'application, et **c'est un argument fort qui n'apparaît nulle part dans la
  documentation existante**.
