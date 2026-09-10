# Documentation du site web — points à traiter

Liste des fichiers Markdown à produire dans `docs/documentation-site-web/`.
Chaque fichier est un **brief de contenu** destiné au développeur qui construira
la page de documentation du site : faits vérifiés, valeurs exactes, procédures,
tableaux et pièges. Ce n'est pas la prose finale.

Le gabarit et les conventions sont dans `00-comment-utiliser-ces-fichiers.md`.
Mockup de référence : `docs/beaver-site/mockup/docs.html`.

Référence projet : Beaver **v1.2.2** (`package.json`, `src-tauri/Cargo.toml`, relu le 9 septembre 2026).

On coche au fur et à mesure.

> **Passe d'audit du 9 septembre 2026.** Les sections 06, 07 et les fichiers
> transverses ont été relus contre le code. Les points réglés depuis leur
> relevé sont marqués **✅ RÉGLÉ** avec leur date et leur référence dans le code
> — ils ne sont pas supprimés : un point réglé garde sa trace. Les points dont
> la référence était périmée mais dont le fond tient sont marqués ⚠️ et
> corrigés sur place. Les constats nouveaux sont marqués 🆕.

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
- [x] `04-agent/compression.md` — dégelé et écrit le 10 septembre 2026 (refonte de la compression déclarée terminée par Kevin) : fenêtre de contexte, déclenchement automatique au seuil / manuel par `/compress`, conservé vs résumé, profils, gardes anti-boucle
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

- [x] `06-modeles/ollama-runtime.md` — runtime managé, réutilisation d'un daemon existant, port attribué par le système (11434 n'est que le port de détection), arrêt propre, absence de journal du moteur — **brief réécrit de zéro le 9 septembre 2026**, le module `ollama_manager/` ayant remplacé l'ancien `ollama_lifecycle.rs`
- [x] `06-modeles/ollama-modeles.md` — parcourir, installer, supprimer, modèles partagés avec Ollama.app, téléchargements
- [x] `06-modeles/ollama-personnalisation.md` — modelfiles, paramètres, prompts système par modèle, modèles custom
- [x] `06-modeles/providers-api.md` — table des **11** providers LLM (Groq retiré, Anthropic et Qwen ajoutés — `route_profile/catalog_api.rs:42-191`), où récupérer la clé, comment la saisir, test de connexion — **sans aucun tarif**, voir la décision ci-dessous
- [x] `06-modeles/providers-comptes-web.md` — OpenAI/Codex, Grok, Kimi : authentification web, jetons, limites
- [x] `06-modeles/catalogue-et-favoris.md` — explorateur LLM, familles, détails d'un modèle, favoris
- [x] `06-modeles/raisonnement.md` — effort de raisonnement, différences par provider, affichage du thinking
- [x] `06-modeles/usage-et-couts.md` — limites, crédits, tokens, requêtes, estimation de coût, historique
- [x] `06-modeles/materiel-et-vram.md` — détection GPU, table VRAM, choisir une taille de modèle

## 7 — Intégrations

- [x] `07-integrations/recherche-web.md` — Brave, Exa, Firecrawl, SearXNG local : différences, configuration, routage
- [x] `07-integrations/mcp-connecteurs.md` — ajouter un connecteur local ou distant, activation par conversation, runtime, durcissement
- [x] `07-integrations/mcp-oauth.md` — connecteurs cloud, callback OAuth, stockage des jetons, révocation
- [x] `07-integrations/extensions-centre.md` — écrit le 9 septembre 2026 : centre d'extensions, découverte, catalogue, installation (double consentement), mise à jour, désactivation, suppression
- [x] `07-integrations/extensions-remplacer-un-outil.md` — écrit le 9 septembre 2026 : substitution d'un outil natif (API advanced, instable, sans liste blanche), masquage, priorité, diagnostics
- [x] `07-integrations/extensions-prompt-systeme.md` — écrit le 9 septembre 2026, **recadré** : l'enquête code a établi qu'aucune API d'extension ne touche au prompt système (contrat de 12 méthodes, aucune sur les prompts) ; la page documente ce qui est réellement possible ; décision tranchée par Kevin le 9 septembre 2026 : la capacité ne sera jamais construite côté extensions (inutile, c'est natif), la page recadrée est la forme définitive
- [x] `07-integrations/extensions-ecrire.md` — écrit le 9 septembre 2026 : écrire sa propre extension, structure, manifeste, hôte, cycle de vie, outils, vues, événements, limites et sécurité

> ✅ **Les quatre pages Extensions sont dégelées depuis le 9 septembre 2026.**
> Le gel portait sur une implémentation interrompue ; ce n'est plus le cas :
> `EXTENSIONS.md` fait autorité sur le sujet, le module `services/extensions/`
> est implémenté et couvert de tests, les deux outils de découverte sont
> intégrés au mode Plan, et la politique de permission par effet d'extension est
> en place et testée (`agent_local/tool_plan_guard.rs:46-48`).
> **Les quatre briefs restent à écrire**, avec `EXTENSIONS.md` comme source
> principale.
>
> C'est la section qui demandera le plus de soin : installer du code tiers qui
> remplace des outils et réécrit le prompt système engage la sécurité de
> l'utilisateur. Il devra être guidé pas à pas, avec les risques énoncés
> explicitement.
- [x] `07-integrations/channels-gateway.md` — Telegram, Slack, Discord : mise en place, mapping vers les sessions, audit, sécurité

## 8 — Forecast

- [x] `08-forecast/vue-densemble.md` — à quoi sert l'espace Forecast, parcours type de bout en bout
- [x] `08-forecast/donnees-et-audit.md` — import, profil de données, qualité, fréquence, valeurs manquantes, anomalies
- [x] `08-forecast/modeles-locaux.md` — Chronos, TimesFM, Toto 2.0, MOIRAI 2.0, FlowState, TabPFN-TS, TiRex, Kairos, Sundial : capacités, installation, matériel requis
- [x] `08-forecast/modele-cloud-timegpt.md` — Nixtla TimeGPT-2 / 2.1, clé API, limites
- [x] `08-forecast/selection-du-modele.md` — sélection manuelle vs automatique, critères utilisés
- [x] `08-forecast/evaluation-et-comparaison.md` — backtests glissants, MASE, sMAPE, MAE, couverture, baselines, ensembles pondérés
- [x] `08-forecast/analyse-avancee.md` — décomposition, dérive, importance des variables, anomalies
- [x] `08-forecast/scenarios-notes-rapports.md` — scénarios, notes, vue Rapport
- [x] `08-forecast/exports.md` — CSV, Excel, JSON, PNG, SVG, PDF, presse-papier

## 9 — Automatisation

- [x] `09-automatisation/reveils.md` — écrit le 10 septembre 2026 : trois rythmes, scheduler interne, conversation dédiée en accès complet (choix assumé, tranché par Kevin), pause, occurrence ratée jamais rattrapée, différence macOS / Windows-Linux à la fermeture
- [x] `09-automatisation/historique-des-reveils.md` — écrit le 10 septembre 2026 : journal `wakeups.jsonl`, 4 statuts, 7 codes d'erreur, rétention 500 lignes ramenées à 250
- [x] `09-automatisation/git-workflow.md` — écrit le 10 septembre 2026 : ouvre sur ce qui n'existe pas (fetch, pull, clone, commit partiel, résolution de conflit), puis branches, worktrees, commit, push, merge, historique, diffs, surveillant

## 10 — Réglages (référence)

- [x] `10-reglages/reference-complete.md` — plan des 5 sections et 16 onglets, ce que chacun contient
- [x] `10-reglages/general-et-preferences.md` — langue, démarrage, mascotte, raccourcis, apparence
- [x] `10-reglages/agent.md` — mémoire, prompt système, outils, avancé
- [x] `10-reglages/modeles.md` — Ollama, Forecast, LLM
- [x] `10-reglages/integrations.md` — providers, connecteurs, channels, extensions
- [x] `10-reglages/application.md` — conversations archivées, accès fichiers, à propos

## 11 — Sécurité et confidentialité

- [x] `11-securite/modele-de-securite.md` — écrit le 9 septembre 2026 : ce qui sort de la machine, ce qui n'en sort jamais, les 5 couches, le bac à sable noyau et sa condition d'activation, les limites assumées (mode par défaut `auto`)
- [x] `11-securite/vault-et-cles-api.md` — écrit le 9 septembre 2026 : XChaCha20-Poly1305, clé maîtresse dans le keyring OS, cycle de vie réel (déchiffré une fois au démarrage, copies zéroïsées), les 7 commandes exposées — aucune ne renvoie de secret
- [ ] `11-securite/acces-fichiers.md` — portée du répertoire, protection contre la traversée de chemin, permissions par plateforme
- [ ] `11-securite/durcissement.md` — collections bornées, HTTP sécurisé, MCP (allowlist, pas de shell), navigateur isolé, logs filtrés
- [ ] `11-securite/mises-a-jour-verifiees.md` — métadonnées strictes, téléchargements bornés, health checks, échec fermé
- [x] `11-securite/confidentialite-des-donnees.md` — écrit le 9 septembre 2026 : qui voit les conversations, les 6 connexions automatiques recensées, télémétrie absente (vérifiée par recherche exhaustive, méthode publiée), effacement complet
- [ ] `11-securite/signaler-une-vulnerabilite.md` — procédure, périmètre, délais

## 12 — Référence

- [ ] `12-reference/stockage-local.md` — arborescence `~/.local/share/cl-go-dash/`, rôle de chaque fichier, sauvegarde et restauration
- [ ] `12-reference/formats-supportes.md` — fichiers lisibles, éditables, prévisualisables, formats Office et tableurs
- [ ] `12-reference/limites-et-quotas.md` — 32 flux actifs, 16 PTY, 2000 messages, 10 onglets navigateur, 32 outils optionnels, tailles maximales
- [ ] `12-reference/journaux.md` — où sont les logs, ce qu'ils contiennent, ce qui en est exclu, rotation
- [ ] `12-reference/glossaire.md` — définitions courtes de tous les termes du produit

## 13 — Dépannage

- [x] `13-depannage/installation.md` — écrit le 9 septembre 2026 : 13 sections symptôme → cause → résolution (Gatekeeper avec signature ad hoc, SmartScreen, Linux .deb/apt uniquement, téléchargement d'Ollama, permissions disque, machine sans réseau, GPU, désinstallation)
- [x] `13-depannage/ollama.md` — écrit le 10 septembre 2026 : trois états du moteur, port 11434 jamais occupé par Beaver, limites du moteur externe (ni téléchargement ni suppression ni création), annulation qui efface les morceaux, GPU, modèle trop lourd, les 20 messages du moteur cités
- [x] `13-depannage/providers-et-cles.md` — écrit le 10 septembre 2026 : clé refusée, quota, réseau, expiration/effacement d'une connexion par compte, catalogue vide ; fait central : aucune reprise sur les requêtes cloud (choix assumé, une réponse perdue peut être déjà facturée)
- [x] `13-depannage/agent-et-outils.md` — écrit le 10 septembre 2026 : trois endroits où lire un échec, fenêtre d'autorisation, 15 motifs shell bloqués d'office, fichier hors zone, contexte saturé, 200 tours, coupe-circuit à 6 appels identiques, sous-agents, diagnostics
- [x] `13-depannage/mcp-extensions-channels.md` — écrit le 10 septembre 2026 : connecteur qui ne démarre pas, 8 codes d'échec d'outil MCP, OAuth (7 causes), hôte d'extensions (3 redémarrages/300 s, mode sûr), canaux (4 refus muets, limites 12/120/300 par minute), journal d'audit
- [x] `13-depannage/forecast.md` — écrit le 10 septembre 2026 : les 30 causes derrière « Le calcul a échoué », installation (Python 3.12 exact, pas de reprise), mémoire, données refusées, TimeGPT, délais réels ; le seul chemin qui montre la vraie cause est le résultat d'outil via l'agent
- [x] `13-depannage/faq.md` — écrit le 10 septembre 2026 : 10 questions vérifiées (données, sauvegarde, trousseau, hors-ligne, fermeture selon l'OS, AGPL v3, désinstallation) + tableau de renvois

## 14 — Projet

- [ ] `14-projet/architecture-technique.md` — stack, arborescence backend/frontend, choix structurants (pour la page « sous le capot »)
- [ ] `14-projet/build-depuis-les-sources.md` — prérequis dev, commandes, tests, lint, build par OS
- [ ] `14-projet/contribuer.md` — CLA, conventions, checks obligatoires, ouverture d'une PR
- [ ] `14-projet/licence.md` — AGPL v3, obligations, licence commerciale, composants tiers, historique Apache 2.0
- [ ] `14-projet/versions-et-changelog.md` — politique de version, où lire les notes, historique

---

## Suivi

Compteurs recomptés le 9 septembre 2026. **Les précédents étaient périmés** : ils annonçaient 94 fichiers prévus et 6 gelés.

- Fichiers prévus : **111** (toutes les entrées cochables de ce fichier, `00-comment-utiliser-ces-fichiers.md` compris)
- Fichiers rédigés : **71** (sections 0 à 7 terminées, plus les 4 briefs bloquants pour le lancement écrits le 9 septembre 2026 : modèle de sécurité, coffre, confidentialité, dépannage installation)
- Fichiers gelés : **2** — Mode Plan et Compression du contexte, voir `_geles/README.md`
- Fichiers restant à écrire : **40** (sections 8 à 14), dont **38 rédigeables immédiatement**

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
| `manual` | Demander l'autorisation |
| `chat` | Chatbot |

`subagent` est interne, jamais proposé à l'utilisateur.

Corrigé le 10 septembre 2026 : l'ancien libellé « Demande d'approbation » de ce tableau ne correspondait pas à l'application, qui affiche « Demander l'autorisation » (`fr.json:550`, vérifié). Décision de Kevin : le site suit l'application — 29 occurrences corrigées sur 15 pages.

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
- ✅ **RÉGLÉ le 9 septembre 2026 — `CLAUDE.md` affirme que la release CI est
  créée en non-draft.** C'était faux, ça ne l'est plus :
  `.github/workflows/release.yml:439-443` crée la release avec
  `--verify-tag --latest` et **sans** `--draft`, et le chemin de reprise force
  explicitement `--draft=false` (`:436`). `CLAUDE.md` dit désormais vrai.
  *Conservé ici parce que ce constat sert d'exemple dans
  `00-comment-utiliser-ces-fichiers.md` : la hiérarchie des sources y est
  illustrée par cette erreur. L'exemple reste pédagogique, le fait ne l'est plus.*
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
- ✅ **RÉGLÉ le 9 septembre 2026 — `search_extension_tools` est verrouillé mais
  n'appartient à aucun groupe.** L'outil **n'existe plus** : aucune occurrence
  dans `src-tauri/src/`. Il a été remplacé par deux outils de découverte
  distincts, `list_extensions` et `inspect_extensions`
  (`services/extensions/mod.rs:192-193`), tous deux autorisés en mode Plan
  (`agent_local/tool_plan_guard.rs:10-11`). Le chantier Extensions est par
  ailleurs dégelé.
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

- ✅ **RÉGLÉ le 9 septembre 2026 — Les paliers gratuits de Google et Mistral
  affichés dans l'application ne sont plus publiables.** Réglé côté produit :
  les textes ont été réécrits **sans aucun chiffre** — `src/i18n/fr.json:41`
  (« Niveau gratuit pour certains modèles, selon le compte et les limites ») et
  `:42` (« Mode gratuit et plan payant ; disponibilité selon le compte »). Les
  commentaires datés du 30 juillet 2026 ont disparu avec la réécriture du
  module : `services/llm/catalog.rs` n'est plus qu'une vue publique dérivée de
  l'autorité `route_profile/catalog_api.rs`.
- **Aucune borne sur le nombre de modèles favoris** (`services/favorite_models.rs`).
  Le fichier grandit sans limite, alors que le projet borne ses collections
  partout ailleurs. Défaut mineur, mais c'est une exception à une règle tenue.
- **La vérification des mises à jour de modèles s'arrête à 100 familles**
  (`commands/ollama_updates.rs`) sans le signaler. Au-delà, certains modèles ne
  sont jamais examinés et rien ne l'indique.
- **Rien ne signale à l'utilisateur que Beaver réutilise un moteur Ollama
  existant.** Le constat reste ouvert ; **sa référence, elle, était périmée** :
  `ollama_lifecycle.rs` n'existe plus, le module a été réécrit. La détection
  d'un moteur externe est désormais dans
  `services/ollama_manager/manager_process.rs` (adoption après une sonde sur
  `127.0.0.1:11434`, `port.rs:66-84`). Dans ce cas, les réglages de moteur de
  Beaver n'ont aucun effet, la création de modèle personnalisé est refusée, et
  l'utilisateur n'a aucun moyen de le comprendre. Recommandation inchangée :
  l'afficher dans l'écran des modèles.
- 🆕 **Relevé le 9 septembre 2026 — le moteur Ollama n'écrit plus aucun fichier
  de traces.** Sa sortie est redirigée vers `/dev/null` sous macOS et Linux (`ollama_manager/spawn_gate_unix.rs:63-67`) et
  n'est redirigée nulle part sous Windows (`spawn_gate_windows.rs`, `STARTUPINFOW`
  sans handles). Le fichier `logs/ollama-sidecar.log` que `CLAUDE.md` documente
  encore **n'est plus écrit par le code**. Un modèle qui refuse de se charger ne
  laisse donc aucune trace exploitable. **À trancher côté produit**, et
  `CLAUDE.md` est à corriger.
- **Sur Mac Intel, la mémoire n'est pas mesurée** (`gpu_vram/macos.rs` sort
  immédiatement hors architecture Apple), et le contexte tombe au palier minimal
  de 8 192 jetons. À confirmer comme volontaire.
- **Le nom « Codex » désigne le mécanisme de connexion OpenAI** dans tout le
  code. À ne pas reprendre sur le site sans arbitrage : ce nom désigne un produit
  précis chez OpenAI et induirait en erreur sur ce à quoi l'utilisateur se
  connecte.

### Relevé en écrivant la section 7

- **La recherche web sans clé exige un interpréteur Python installé sur la
  machine**, et ce prérequis n'est documenté nulle part — ni README, ni prérequis
  d'installation, ni interface. **Toujours d'actualité, et aggravé.** ⚠️ *La
  description technique de ce point était fausse : `services/searxng/runtime.rs:106`
  ne contient plus ce code, et la règle a changé.* Beaver n'accepte plus
  « n'importe quel Python 3 récent » : un manifeste livré avec le moteur
  (`resources/searxng-sidecar/wheels/.runtime.json`) déclare **une version
  exacte**, et chaque commande candidate est interrogée puis rejetée si elle ne
  la rapporte pas au numéro près (`services/searxng/python_runtime.rs:75-95` et
  `:137-152`). Au 9 septembre 2026, le manifeste demande **CPython 3.14** — une
  version que presque aucune machine ne porte par défaut, sur aucun des trois
  systèmes. Un utilisateur ayant déjà Python peut donc se voir refuser la
  recherche sans clé, sans message explicite. **Le point le plus urgent de cette
  section** : embarquer un interpréteur, ou l'annoncer dans les prérequis avec un
  message qui dit quelle version installer.
- **Le moteur de recherche local ne s'arrête jamais en cours de session.**
  ✅ *Vérifié le 9 septembre 2026 : c'est bien le comportement du code.* Le seul
  appel d'arrêt vient du nettoyage de fermeture de l'application
  (`src/app_exit/cleanup.rs:124-125`). Aucune mise en veille après inactivité
  n'existe — contrairement aux processus MCP, arrêtés après 10 minutes. Un
  processus Python résident consomme donc de la mémoire pendant toute la session.
  **Le constat produit reste ouvert.**
- ✅ **RÉGLÉ le 9 septembre 2026 — le mode de permission appliqué aux messages
  reçus par le gateway n'est pas déterminé.** Il l'est désormais, et la réponse
  tient en deux parties :
  1. Le gateway **demande** le mode « Accès complet » pour la conversation qu'il
     lance (`services/gateway/agent_bridge_run.rs:110-112`), **mais cette demande
     est plafonnée par le réglage de l'utilisateur** : si le mode demandé est plus
     permissif que celui enregistré, c'est le mode enregistré qui l'emporte
     (`commands/agent_chat_task/common.rs:49-56`). Un utilisateur en « Demande
     d'approbation » n'est donc **pas** basculé en « Accès complet » par un
     message Telegram. Le mode Plan est explicitement désactivé pour ces
     conversations (`agent_bridge_run.rs:116`).
  2. **Aucune demande d'approbation ne part vers la messagerie**
     (`agent_bridge_run.rs:113`, aucun émetteur dédié) : elle s'affiche dans la
     fenêtre de Beaver, sur la machine. **Personne ne peut y répondre à
     distance.**

  Ce qui reste ouvert, c'est le point voisin déjà listé plus haut : **le mode par
  défaut de l'application est « Accès complet »** (`storage_migration.rs:83`).
  Sur une installation où l'utilisateur n'a rien changé, un message reçu par
  messagerie déclenche donc un agent en accès complet. Le réglage par défaut
  devient d'autant plus important avec ce point tranché.
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

### Relevé en écrivant les sections 11 et 13 (9 septembre 2026)

Constats sur le code et le produit, rangés du plus au moins urgent. Le détail
sourcé est dans les « Points à confirmer » des quatre briefs concernés.

- ✅ **TRANCHÉ le 9 septembre 2026 — Les réveils programmés tournent en Accès
  complet sans plafonnement, et c'est voulu.** Le scheduler demande `FullAccess`
  (`scheduler/agentic.rs:112`), non plafonné par le réglage utilisateur
  (`commands/agent_chat_task/common.rs:48`), contrairement au gateway. Décision
  de Kevin : le but d'un réveil est précisément de s'exécuter sans demande
  d'approbation — l'utilisateur n'est pas devant l'écran dans la quasi-totalité
  des cas, une demande sans personne pour y répondre bloquerait l'exécution.
  Le consentement se donne à la création du réveil. À documenter comme un
  fonctionnement voulu (fait dans `modele-de-securite.md` ; à reprendre dans
  `09-automatisation/reveils.md` quand il sera écrit).
- **`SECURITY.md` est doublement décalé** : il écrit « all inbound messages are
  hashed and logged » alors que c'est l'**identifiant de l'expéditeur** qui est
  haché (le contenu n'est jamais journalisé — la réalité est meilleure que le
  document), et il ne mentionne **nulle part** le bac à sable du shell, la
  protection la plus forte du produit. À corriger au prochain passage.
- **Le message d'erreur du trousseau accuse la mauvaise cause** : il s'affiche
  pour toute panne du coffre et cite `gnome-keyring`/`kwallet`, y compris sous
  Windows et macOS. À corriger dans les 7 langues.
- **L'aperçu des liens contacte n'importe quel site sans clic**, dès qu'une
  adresse s'affiche dans une conversation, et il est activé par défaut
  (`chat-markdown.tsx:78`, `models/config.rs:54`). Désactivable, mais le défaut
  est bavard. À trancher côté produit.
- **Trois vérifications de version partent toutes les heures** (app via
  `api.github.com`, moteur Ollama via `github.com`, modèles via `ollama.com` —
  qui reçoit le nom des familles installées), sans aucun réglage pour les
  couper, alors que l'aperçu des liens, lui, se désactive. Incohérence à
  trancher.
- **Le compte web Kimi transmet le nom de la machine** et un identifiant stable
  (`llm_oauth/headers.rs:86-88`, `:103-109`). À énoncer sur la page comptes web.
- **Le caviardage `sanitize_chat_messages` n'est appelé que sur le chemin
  Ollama** ; aucun équivalent trouvé côté providers cloud. À confirmer, puis à
  trancher.
- **Aucune procédure ne supprime l'entrée de trousseau** `cl-go-dash`/`master-key` :
  l'effacement complet documenté reste inapplicable sur ce dernier pas.
- **`install.ps1` n'a qu'un seul message d'erreur** pour tous ses contrôles
  (`install.ps1:182-186`) : l'utilisateur n'apprend jamais lequel a échoué.
- **Aucun contrôle d'espace disque** avant le téléchargement d'Ollama (~3 Gio).
- Erreurs corrigées dans des briefs existants le jour même : `premier-lancement.md`
  décrivait deux contrôles de téléchargement inexistants ; `installation-linux.md`
  renvoyait vers `logs/ollama-sidecar.log`, disparu ; `modele-de-securite.md`
  aligné sur la liste complète des connexions automatiques de
  `confidentialite-des-donnees.md`.

### Relevé en construisant les pages du site (10 septembre 2026)

- **Le message du garde-fou anti-boucle est en dur et non traduit** (texte
  franco-anglais lu tel quel par un utilisateur en espagnol ou japonais). Détail
  sourcé dans `04-agent/fonctionnement.md`. La page du site décrit le
  comportement sans citer le message, donc publiable ; le défaut produit reste
  à corriger ou à assumer.

Relevé en écrivant les briefs Forecast et Réglages (10 septembre 2026, trois agents, chaque point sourcé) :

**Espace Forecast — parcours et exports**

- **Textes des exports non traduits ou fautifs** : légende du graphique avec « Prevision » sans accent (`export/chart.rs:87-91`), noms de feuilles du classeur mi-français mi-anglais, titres du PDF désaccentués (`export/pdf.rs:33-47`).
- **Faux repère monétaire** : le symbole € s'ajoute dès que le nom de la colonne cible contient `eur` — donc aussi « valeur », « heures », « couleur » (`export/chart.rs:167-178`).
- **Alertes de qualité affichées sans leur nature** : « Erreur bloquante · 3 » sans dire de quoi il s'agit ; les 22 codes internes n'ont aucune traduction (`workbench/forecast-workbench-data.tsx:85-95`).
- **Messages génériques qui masquent des causes connues** : « Le calcul a échoué » couvre une douzaine de refus nommés, « Données Forecast invalides » quatre, « Export impossible » cinq.
- **Deux mots pour la même chose** : « Covariables » à la configuration, « Variables externes » dans la section Données (`fr.json`).
- **Deux clés de navigation mortes** : `forecast.nav.scenarios` et `forecast.nav.notes`, traduites en sept langues, jamais affichées (`forecast-nav.tsx:9-13`).
- **Rendu des accents dans le PDF incertain** : police Courier sans encodage déclaré, échappement limité à trois caractères (`export/pdf.rs:14`, `:144-149`) — à contrôler en produisant un vrai PDF.

**Espace Forecast — modèles et analyses**

- **Erreur distante unique pour quatre causes** : clé refusée, quota, panne, réponse illisible (`client_nixtla.rs:31`, `client_nixtla_retry.rs:26-32`).
- **Multivarié TimeGPT-2.1 désactivé en silence** quand les séries ne sont pas alignées (`nixtla_multiseries.rs:65-67`).
- **Le bouton d'ensemble pondéré ne peut jamais s'afficher** : il exige deux modèles évalués, l'interface n'en évalue qu'un (`forecast-evaluation-view.tsx:61` vs `use-forecast-evaluation.ts:63`).
- **Fiabilité de l'importance des variables calculée, jamais affichée** (`variables.rs:53-61` vs `forecast-advanced-analysis-utils.ts:35-49`).
- **Échec d'analyse avancée présenté comme un manque de données** (`sanitize.rs:12-17`, `:64-73`).
- **Anomalies du graphe étiquetées `llm`** alors qu'elles sont calculées en Rust (`forecast-view-data.ts:104`).
- **Borne −95/+500 appliquée au mode « Valeur »** d'un scénario de contexte, où elle n'a pas de sens (`scenario_context.rs:99-103`).
- **Étiquette de machine déduite de la taille disque, pas de la mémoire** : TabPFN-TS-3 « Machine moyenne » pour 8 192 Mo requis (`forecast-model-meta.ts:161-164`).
- **Commentaire faux sur le stockage des fenêtres de backtest** (`forecast-reliability-data.ts:11-15` vs `types.rs:106-112`).
- **La section Rapport annonce provenance et notes, absentes du composant** (`forecast-workbench-report.tsx:12-21`).
- **L'état `invalid` d'un modèle n'a aucun libellé utilisateur** (`model_manager/mod.rs:19-92` vs `fr.json`).
- **Identifiant TimeGPT-2 Standard incertain depuis le 30 juillet 2026**, de l'aveu d'un commentaire du code, faute de clé pour vérifier (`client_nixtla.rs:88-97`) — le site ne doit pas promettre les quatre variantes tant que ce n'est pas tranché.
- **Deux boutons inactifs à l'écran d'import** : « Coller des données » et « Depuis une URL » — à câbler ou à retirer (règle : visible et fonctionnel, ou invisible).

**Réglages**

- **« Réinitialiser » de l'accès fichiers fait l'inverse de son libellé** : il rétablit l'accès au disque entier (`path-list-editor.tsx:34-36`, défaut `models/config.rs:116-119`). La plus coûteuse du lot.
- **« Tout supprimer » des chats archivés ignore la recherche et le filtre actifs** (`archived-chats-settings.tsx:52-60`).
- **Borne de 2 000 conversations archivées appliquée en silence** (`archived-chats-settings.tsx:17`, `:27`).
- **Libellés non traduits** : `forecast.models.sidebarTitle` = « Models », `settings.tabs.providers` = « Providers » (`fr.json`).
- **Deux noms pour le même onglet** : « System prompt » dans la liste, « Instructions système » en titre (`system-prompt-settings.tsx:10`).
- **Clé orpheline `settings.tabs.apiKeys`** depuis que les clés sont un sous-onglet de Providers.
- **Forecast seul onglet à tirer son libellé hors de `settings.tabs.*`** (`core-occupants.tsx:46-47`).
- **Bornes divergentes du budget mémoire** : moteur ≥ 256 tokens, interface ≥ 512 (`memory_types.rs:47` vs `memory-settings.tsx:16`).
- **Mises à jour de modèles Ollama détectées mais jamais affichées** (`use-update-checker.ts:78` vs `updates-settings.tsx`).

### Relevé en écrivant les briefs Compression et Automatisation (10 septembre 2026, deux agents, chaque point sourcé dans le brief correspondant)

Le détail complet et les sources fichier:ligne sont dans la section « Anomalies relevées » de chacun des quatre briefs.

**Compression du contexte** (`04-agent/compression.md`)

- **« Contexte compressée »** — faute d'accord dans la chaîne la plus visible de la fonctionnalité (`fr.json:339`), à répercuter sur les sept langues.
- **Le résumé généré n'est consultable nulle part** : marqueur non cliquable, contenu jamais rendu — l'utilisateur ne peut pas vérifier ce que Beaver a retenu. L'écart le plus important du domaine.
- **Aucune nouvelle tentative en cas d'échec réseau du résumé** : le module de reprise existe, le seul appel de production passe `0` tentative (`orchestrator_summary.rs:122`). Décision du 10 septembre 2026 : non documenté sur le site, à corriger dans l'app.
- **Description de `/compress` en anglais, en dur, sans clé i18n** dans la palette de commandes.
- Deux clés de traduction orphelines (`fr.json:1093-1094`) ; code résiduel dans `checkpoint_files.rs` (énumération à une variante).
- `compression-profiles.json` absent de l'inventaire des données — **corrigé le 10 septembre 2026 dans CLAUDE.md et AGENTS.md** ; reste à l'ajouter au brief `12-reference/stockage-local.md` quand il sera écrit.

**Réveils** (`09-automatisation/reveils.md`)

- **Description bornée à 200 caractères à la saisie, 300 côté moteur** : deux autorités sur la même limite.
- **Les motifs d'erreur précis du moteur sont remplacés à l'écran par un message unique** (« l'opération a échoué ») — et ces motifs sont en dur, en français, dans le code Rust, hors i18n.
- **Le drapeau « issu d'un réveil » traverse la frontière et n'est jamais lu** : une conversation de réveil ne se distingue que par son préfixe `Heartbeat •`.
- **Le panneau latéral n'affiche que les réveils actifs** : un ponctuel exécuté disparaît au moment où on le cherche.
- **Deux vocabulaires pour la même chose** : onglet « Heartbeat » (jamais traduit), écran « Réveils », préfixe `Heartbeat •`. **Tranché le 10 septembre 2026 : « Heartbeat » reste en anglais dans toutes les langues — c'est le terme universel des planificateurs, une traduction ne signifierait plus la même chose. Ce n'est plus une anomalie.**
- Tutoiement et vouvoiement mêlés dans le même écran ; « master switch » en anglais et ne désigne rien de nommé ainsi.
- **La liste identifie un réveil par son modèle, pas par son nom** : deux réveils sur le même modèle sans description sont indistinguables.
- Repli du sélecteur de fournisseur sur une option `Ollama` en dur.

**Historique des réveils** (`09-automatisation/historique-des-reveils.md`)

- **L'heure prévue est enregistrée et jamais montrée** ; pour une occurrence ratée, l'heure affichée est celle du démarrage suivant — le chiffre est juste, la phrase est fausse.
- **L'identifiant de la conversation produite est journalisé et jamais exploité** : aucun lien entre une exécution et sa conversation, alors que l'information existe des deux côtés.
- Jetons produits enregistrés, jamais affichés ; champ `error` hérité en lecture tolérante ; pas de numéro de version dans `wakeups.jsonl`.

**Git** (`09-automatisation/git-workflow.md`)

- **L'état « position détachée » est calculé et jamais lu** ; le libellé « HEAD détaché » est déclenché par une condition (nom vide) que le moteur ne produit pas — il pourrait ne jamais s'afficher.
- **Un champ du statut distant est perdu à la frontière** : type recopié à la main côté interface avec cinq champs sur six.
- Code d'erreur `protected_branch` traduit mais jamais produit par ce domaine ; deux clés de traduction mortes (sept langues).
- **Bornes de 500 branches et 100 worktrees appliquées en silence**, alors que le compteur de fichiers non validés, lui, signale sa troncature.
- **Échec du listage des worktrees masqué en liste vide**, côté moteur et côté interface.
- Deux dialogues hors du portail flottant commun (suppression, conflit d'extraction) — configuration propice à un menu coupé en deux ; deux composants court-circuitent les hooks Git.
- Vocabulaire franco-anglais (« fichier(s) à Commit », « Merge dans… ») : défendable mais écrit nulle part ; tutoiement dominant ; date de commit sans heure (deux commits du même jour indistinguables).

### Relevé en écrivant les briefs Dépannage (10 septembre 2026, trois agents, chaque point sourcé dans le brief correspondant)

Le détail complet et les sources fichier:ligne sont dans la section « Anomalies relevées » de chacun des six briefs de `13-depannage/`.

**Ollama** — un redémarrage sans effet annoncé en notification de succès (« Ollama externe réutilisé ») ; « Restart Ollama » non traduit en français seul des sept langues ; rien ne vérifie que ce qui répond sur 11434 est bien Ollama ; le moteur local disparaît de la liste des modèles sans message là où un fournisseur distant en échec y reste avec sa raison ; « Le dossier des modèles chevauche… » décrit une cause indevinable, sans procédure de résolution.

**Clés et fournisseurs** — le dialogue de saisie jette les sept messages précis et affiche toujours « L'opération a échoué. Réessaye. » ; messages du test de clé et des comptes web en français en dur dans le Rust, hors i18n ; une connexion effacée par Beaver après refus de renouvellement ne prévient personne (cause du symptôme « mon fournisseur a disparu ») ; `app-error.ts` ne couvre que Git, la vraie correspondance est `agent-error-codes.ts` ; « Test non implémenté pour <nom> » inatteignable, en dur, avec identifiant technique.

**Agent et outils** — la fenêtre d'autorisation nomme l'outil par son identifiant technique non traduit ; les échecs de flux arrivent en français brut là où l'interface attend un code, et retombent sur le générique ; le journal de diagnostics de permissions ne se lit qu'en ouvrant le fichier à la main ; « Écriture bloquée : fichier non lu » en dur en français dans le Rust ; le mode Chatbot n'annonce nulle part qu'il n'a que deux outils.

**MCP, extensions, canaux** — l'échec d'un connecteur accuse le jeton dans tous les cas alors que le moteur distingue six causes ; le motif d'un échec OAuth traverse la frontière et l'interface l'ignore ; `auditUnavailable` sans traduction (une panne de disque s'affiche en générique) ; dix-huit diagnostics d'interface d'extension traduits en sept langues et jamais affichés ; « Validation du connecteur en cours… » déclaré, affiché, jamais atteint ; la page de retour OAuth du navigateur en français en dur, hors thème et hors langue ; le motif d'un refus écrasé en `"blocked"` dans l'audit — impossible d'y distinguer un utilisateur non autorisé d'un compte mal configuré.

**Forecast** — le panneau n'a que trois messages pour une trentaine de pannes : tout est écrasé par « Le calcul a échoué. Vérifie les colonnes, le modèle et la clé API. », seule la voie agent montre la vraie cause ; l'agent contrôle `is_installed` là où le panneau contrôle `is_ready` ; un seul code pour quinze causes d'installation ; anomalies de données affichées sans leur nom ; aucun contrôle d'espace disque avant plusieurs gigaoctets ; ressources vérifiées au calcul mais pas à l'installation ; TimeGPT indistinct entre clé, quota et panne ; messages du moteur en dur en français ; « stockage plein » conseille une action inapplicable là où il s'affiche ; « sidecar » exposé à l'utilisateur.

**FAQ / transverse** — six messages demandent de restaurer « depuis une sauvegarde » que l'application n'aide jamais à faire ; « La recherche des mises à jour a échoué » ne se déclenche que sur l'échec de la vérification Ollama, pas celle de Beaver ; **l'application n'affiche nulle part sa licence** (À propos : version, cadre, système — aucune mention AGPL v3 ni droit d'auteur, alors que la licence impose de transmettre ces informations) — le plus sérieux du lot ; langue et thème stockés dans le stockage local du composant d'affichage, hors du dossier de données : perdus à tout déplacement de machine, invisibles pour une sauvegarde ; langue par défaut anglaise sans tenir compte du système là où le thème, lui, suit le système.

### Constat de conception à mettre en avant sur le site

- **Le bac à sable du shell ne s'active que si l'accès disque est restreint**
  (`shell_sandbox/launch.rs:42`). Avec le réglage par défaut — la racine du
  disque — les commandes tournent sans isolation. Dès que la portée est réduite,
  chaque commande est enfermée par le système lui-même : Seatbelt sur macOS,
  Landlock sur Linux, profil restreint sur Windows. C'est le seul réglage de
  Beaver dont l'effet est garanti par le noyau et non par le code de
  l'application, et **c'est un argument fort qui n'apparaît nulle part dans la
  documentation existante**.

## Relevé en traduisant le site en anglais (10 septembre 2026)

La passe de traduction (7 agents, 98 pages) a mis au jour des textes que l'utilisateur anglophone voit **en français** dans l'application, parce qu'ils sont codés en dur hors du système i18n. La documentation anglaise cite leur traduction : elle annonce donc des messages que l'écran n'affiche pas, et l'écart se refermera seulement quand ces chaînes passeront par i18n.

| Fichier:ligne | Chaîne française codée en dur | Traduction citée par la doc EN |
|---|---|---|
| `install.sh:170` | Système non pris en charge. | "Unsupported system" |
| `install.sh:135,145` | Une application est déjà installée. Utilise sa mise à jour intégrée. | "An application is already installed. Use its built-in update." |
| `install.sh:125,127,128,130,133,138,153,154,173` | Installation impossible. | "Installation failed" |
| `install.sh:175` | Impossible de récupérer la version. | "Could not retrieve the version" |
| `install.sh:152` | Paquet d'installation invalide. | "Invalid installation package" |
| `install.ps1:185` | ERREUR Installation impossible. | (non citée) |
| `services/agent_local/tool_skill_loader.rs:22` | Identifiant de skill invalide | "Invalid skill identifier" |
| `services/agent_local/tool_skill_loader.rs:23` | Skill introuvable | "Skill not found" |
| `services/agent_local/tool_skill_loader.rs:24` | Skill indisponible | (non citée) |
| `services/agent_local/subagent_explorer_bash.rs:52,66,76` | Commande d'exploration refusée. | "Exploration command refused." |
| `services/agent_local/subagent_explorer_bash_options.rs:1` | Option d'exploration refusée. | "Exploration option refused." |

**Le constat qui compte plus que la liste — mécanisme exact, lu dans le code de rendu** (`src/lib/tool-error-message.ts` + `components/agent-local/tool-detail-row.tsx:149-163`) : une erreur d'outil s'affiche en deux parties. Le **titre** passe par `toolErrorMessage()` et ses `CATEGORY_KEYS` (dix catégories, complétude garantie par TypeScript) — toujours traduit. Le **détail** en dessous vaut la chaîne brute du backend, SAUF si le code d'erreur figure dans `ERROR_CODE_KEYS` — qui ne contient **qu'un seul code** (`web_search_runtime_unavailable`). Conséquence : **~252 littéraux français sur 69 fichiers** de `services/agent_local/` (chemin `ToolResult`) s'affichent tels quels en détail d'erreur sur une interface anglaise — titre « Not found » en anglais, « Skill introuvable » en français juste dessous. Échantillon : « requête vide », « requête trop longue », « aucun provider configuré », « limite de requêtes », « délai dépassé », « réponse invalide ».

**Trois catégories à distinguer — le correctif diffère** (vérifié jusqu'à la ligne JSX qui rend le texte, `tool-detail-row.tsx:156-162`) :
1. **Détail sous titre traduit** (erreurs avec code, ex. `tool_skill_loader.rs:22-24` — codes stables `invalid_skill_id`/`skill_not_found`/`skill_unavailable` déjà attribués par `tool_dispatcher_error.rs:28-40`) → peupler `ERROR_CODE_KEYS` + clés i18n, sans toucher une chaîne Rust.
2. **Titre entièrement brut** — déclencheur vérifié : PAS les `Err(String)` nus (sur le chemin vivant, les deux constructeurs d'erreur attachent toujours un code), mais une asymétrie de repli : `tool-detail-row.tsx:113` lit `t.result_meta?.error` SANS repli, là où `tool-result-model.ts:25` a le repli `legacyError()`. Pour une conversation enregistrée avant l'existence de `result_meta`, le modèle reçoit un code propre et l'écran la chaîne brute (démontré sur données réelles : 42 appels, 1 erreur sans code). Correctif : une ligne dans `tool-detail-row.tsx`, calquée sur `legacyError()` qui existe juste à côté. Mécanisme précisé le 10 septembre 2026 (contre-vérifié par deux agents indépendants) : dans `types_tool_result.rs:19-23`, `is_error: bool` est toujours sérialisé mais `error: Option<ToolErrorInfo>` porte `skip_serializing_if = "Option::is_none"` — une conversation d'avant le contrat se relit donc avec `is_error: true` et `error` absent ; l'erreur sans code vient du disque, jamais de l'exécution (les trois seuls poseurs de statut d'erreur — `error()` :51-62, `cancelled()` :80-90, `with_error_info()` :114-124 — attachent tous un `ToolErrorInfo`, et les neuf constructeurs de `types_tool_result_errors.rs` délèguent à `Self::error`).
3. **Texte destiné au modèle** (`with_error_hint`) : jamais rendu, le français n'y est pas un défaut.

Leçon de méthode à retenir (trois allers-retours pour l'apprendre) : aucun critère indirect n'a tenu — ni « c'est un `Err` », ni « ça passe par `ToolResult` ». La seule vérification fiable est de remonter jusqu'à la ligne JSX qui rend le texte. Trois faux positifs écartés en contre-vérification : `directory_access.rs:10` (jamais affiché, écran réel via `directoryAccess.*`), `file_preview_office.rs:66` (jeté par les quatre `.catch` de prévisualisation, écran réel `filePreview.fileNotFound`), `with_error_hint(...)` (destiné au modèle, zéro occurrence dans `src/`). Les pages Répertoire de travail et Fichiers citaient les deux premiers comme visibles — corrigées le 10 septembre 2026. Deuxième problème, distinct : `install.sh`/`install.ps1` écrivent leurs messages français directement à l'écran, sans frontend pour les rattraper — l'internationalisation se joue dans les scripts eux-mêmes.

Trouvé aussi pendant la même passe : la page française du terminal citait « Trop de terminaux ouverts », message qui n'existe pas — le vrai est `terminal.liveLimitReached` (« La limite de terminaux actifs est atteinte. » / "The active terminal limit has been reached.") ; corrigé sur les deux langues le 10 septembre 2026.

### Guillemets trompeurs côté français (relevé pendant la traduction, 10 septembre 2026)

Des pages présentent **entre guillemets** des messages qui n'existent pas dans l'application — des paraphrases d'auteur qui se lisent comme des citations d'écran. La traduction anglaise les a reproduites fidèlement (même registre descriptif, sans inventer de libellé), mais **la correction se fait d'abord côté français**, sinon les deux langues divergeront :

- Pages Modèles/Intégrations, onze paraphrases citées comme messages : « Le moteur n'est pas installé », « Limite de requêtes atteinte », « environnement requis introuvable », « Endpoint MCP non autorisé », « Commande MCP non autorisée », « Limite de connecteurs atteinte », « Endpoint OAuth non autorisé », « Token expiré et pas de refresh », « Configuration Gateway invalide », « Le schéma d'un outil doit décrire un objet. », « Une extension Node.js possède un accès complet. »
- `modeles-raisonnement` : l'échelle écrite Faible / Élevé / Maximum là où l'app affiche Limitée / Forte / Max (`agentLocal.reasoning*`), et un palier nommé « Automatique » qui s'appelle « Activée » (fr) / « On » (en) (`agentLocal.reasoningAuto`).
- `modeles-personnalisation` : « Automatique » comme valeur par défaut de la longueur de contexte — à vérifier contre le libellé réel.
- Thèmes des paramètres Ollama (Contexte / Longueur / Créativité / Répétition / Échantillonnage) : noms simplifiés par rapport aux groupes réels de l'application.
- `reference-coffre-et-cles` (corrigé le 10 septembre 2026, deux langues + brief `11-securite/vault-et-cles-api.md`) : trois « symptômes » cités comme messages d'écran (« clé API invalide (vide ou trop longue) », « limite du coffre atteinte », « Clé valide mais quota dépassé ») ne s'affichent jamais — le dialogue remplace toute erreur par « L'opération a échoué. Réessaye. » (`api-keys-config-dialog.tsx:92-94`, déjà relevé en anomalie 1 de `13-depannage/providers-et-cles.md`). Reformulés sans guillemets ; le vrai message générique cité une fois. Les deux phrases que le traducteur avait ajoutées côté EN (« shown in French whatever the language ») affirmaient l'inverse du comportement réel et ont été retirées.
- `reference-themes` (corrigé le 10 septembre 2026) : la page française citait les thèmes colorés sous des noms anglais, dont un inexistant (« Cobalt Frost », qui n'est que le nom du fichier CSS). L'app affiche « Émeraude nocturne / Cobalt givré / Brume astrale / Éclipse écarlate » en français (`fr.json:823-826`) et « Emerald Night / Frosted Cobalt / Astral Mist / Crimson Eclipse » en anglais (`en.json:823-826`). Page fr + brief `03-interface/themes-et-apparence.md` corrigés ; `reglages-preferences` disait déjà juste dans les deux langues.

Règle à ajouter aux briefs : **des guillemets = une citation d'écran vérifiée dans fr.json ; une paraphrase s'écrit sans guillemets.**

### Chaînes en dur supplémentaires signalées par le lot Forecast/Réglages (10 septembre 2026)

Visibilité **non tracée jusqu'à la ligne de rendu** (leçon de méthode ci-dessus : à confirmer avant tout correctif) — signalées par la traduction parce que les pages les citent :

| Chaîne française en dur | Source |
|---|---|
| « Ce groupe d'outils est verrouillé. » | `services/agent_local/tool_group_catalog.rs:107` |
| « Ce tool est verrouillé. » | `services/agent_local/tool_catalog.rs:147` |
| « Dossier de sortie invalide. » | `commands/config.rs:71` |
| Toute la table d'erreurs d'audit Forecast | `services/forecast/data_quality/types.rs:55-108` |
| Les deux refus de ressources | `services/forecast/hardware_profile.rs:108-110` |

Noms littéraux voisins mais **assumés** (le lecteur doit les retrouver à l'identique) : thème de code « Défaut » (`use-settings.ts:35`, seul nom traduit parmi des noms propres), feuilles Excel « Historique »/« Prévisions » (`export/xlsx.rs:13,20` — la huitième s'appelle déjà « Input data » : incohérence de langue dans un même classeur), légende du graphique exporté « Historique / Prevision / Confiance » (`export/chart.rs:88-90`, « Prevision » **sans accent**). Ces trois-là mériteraient une passe produit (anglais partout dans les exports, ou français cohérent).

Incohérence corrigée le 10 septembre 2026 : le verdict `constrained` s'affichait « Juste » sur la page Modèles locaux et « Contraint » sur Sélection du modèle — aligné sur « Contraint » (fr) / « Constrained » (en) partout.

### Vocabulaire quantification — relevé le 10 septembre 2026 (retour de Kevin sur la table VRAM du site)

La doc du site disait « compression » pour parler de la quantification des modèles (Q4/Q5/Q8/f16), alors que l'app dit déjà « quantification » (`vramTableDesc`, fr.json:1084). Corrigé le 10 septembre 2026 sur les 8 pages concernées (les deux langues) et dans 4 briefs ; unités « Go »/« GB » ajoutées aux cellules des tables VRAM. Les sens « compression du contexte » et « compression d'archives Office » n'ont pas bougé. Reste deux anomalies **côté app**, dans fr.json :

- `models.quantization` (fr.json:679) : le libellé français est le mot anglais « Quantization » — l'écran de détail d'un modèle l'affiche tel quel. Attendu : « Quantification ».
- `vramFormula` (fr.json:1086) : la formule française affiche « 0.5 GB » (point décimal + unité anglaise). Attendu : « 0,5 Go ». La page du site cite l'écran tel qu'il est aujourd'hui ; si l'app est corrigée, mettre à jour la citation dans `modeles-materiel.html` (fr) et le brief `materiel-et-vram.md`.
- Cas laissé ouvert : `06-modeles/ollama-runtime.md` l.164 parle de « compression du cache » (le réglage `OLLAMA_KV_CACHE_TYPE` d'Ollama, quantification du cache d'attention — un troisième objet). Phrase exacte mais vocabulaire à trancher si on veut « quantification du cache ».
