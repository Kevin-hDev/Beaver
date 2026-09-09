# Prompt système et extensions : ce qui est possible aujourd'hui

**Emplacement site** — Extensions › *Réécrire le prompt système* dans le sommaire du mockup. **Cet emplacement est contesté** : voir l'encadré « Décision à prendre » ci-dessous.
**Répond à** — « Une extension peut-elle changer les instructions que Beaver donne au modèle ? »
**Sources** — `EXTENSIONS.md` (racine du dépôt), `src-tauri/resources/extension-host/contract.json`, `contract.mjs`, `extension-api.mjs`, `src-tauri/build.rs` + `extension_contract_build.rs`, `src-tauri/src/services/extensions/core_bridge.rs` et `types.rs`, `src-tauri/src/services/agent_local/system_prompt_resolver.rs`, `chat_prompts.rs`, `chat_prompt_sections.rs`, `extension_discovery_prompt.rs`, `extension_skill_loader.rs`, `extension_tool_set_native_only.rs`, `skill_catalog.rs`, `src-tauri/src/commands/agent_chat_task/common.rs`
**Vérification** — Vérifié dans le code, y compris par recherches négatives explicitement listées (section « Comment le verdict a été établi »). Rien n'a été vérifié à l'écran.

---

## Encadré liminaire — Décision prise le 9 septembre 2026

> **DÉCISION (Kevin, 9 septembre 2026) : l'issue 2 est retenue, définitivement.** Aucune capacité de modification du prompt système ne sera construite côté extensions — « ce sera sûrement jamais utile vu que c'est faisable nativement ». La personnalisation du prompt système reste native partout, et cette page se publie sous la forme du présent fichier : dire que c'est natif, renvoyer vers *Agent › Prompts système*, décrire ce que les extensions font réellement autour du prompt. Le plan d'expansion de l'API (PART_2) va dans le même sens : il exclut explicitement le remplacement du prompt système dans sa section « Ce qui ne doit pas changer ». Le titre de la page dans le sommaire du site doit donc être celui de ce fichier, pas « Réécrire le prompt système ».
>
> L'analyse qui a fondé la décision est conservée ci-dessous.

> **La page prévue par le sommaire n'avait pas de fondement dans le code actuel.**
>
> Le sommaire du mockup annonce une page « Réécrire le prompt système » dans le groupe Extensions. Le titre suppose qu'une extension peut remplacer ou modifier le prompt système de Beaver.
>
> **Cette capacité n'existe pas.** Aucune interface de programmation offerte aux extensions ne touche au prompt système, ni pour le lire, ni pour l'écrire, ni pour s'y insérer. La preuve est détaillée plus bas, avec les recherches négatives qui l'établissent.
>
> Ce qui existe réellement — remplacer, personnaliser ou désactiver le prompt système — est une **fonctionnalité native des réglages de Beaver**, sans rapport avec les extensions. Elle est déjà documentée dans le brief *Agent › Prompts système*.
>
> **Trois issues possibles, au choix du propriétaire :**
>
> 1. **Retirer la page du plan** et laisser le sujet entièrement au brief *Agent › Prompts système*. C'est l'issue la moins coûteuse et celle que le code justifie aujourd'hui.
> 2. **Conserver une page courte** à cet emplacement, sur le modèle du présent fichier : dire que la personnalisation est native, renvoyer vers la bonne page, et décrire ce que les extensions font réellement autour du prompt. Utile si le sommaire du site doit rester proche du mockup.
> 3. **Construire la capacité**, puis écrire la page. C'est une décision produit, pas une décision de rédaction — et elle ouvre un risque de sécurité majeur, décrit dans l'encadré « Pourquoi cette capacité n'est pas anodine ».
>
> Tant que cette décision n'est pas prise, **ne publiez pas de page portant le titre « Réécrire le prompt système » dans le groupe Extensions** : elle décrirait une fonctionnalité inexistante.

Le brief `04-agent/prompts-systeme.md` avait déjà relevé le problème dans son propre en-tête, en proposant de déplacer le sujet vers le groupe Agent. Le présent fichier confirme ce constat par l'examen du code des extensions.

---

## Plan de page proposé

*(dans l'hypothèse où l'issue 2 est retenue)*

1. La personnalisation du prompt système est native
2. Ce qu'une extension peut faire, et ce qu'elle ne peut pas
3. Le seul texte que Beaver ajoute au prompt à cause des extensions
4. Comment un contenu d'extension atteint quand même le modèle
5. Les limites qui encadrent ce texte

---

## Contenu

### 1. La personnalisation du prompt système est native

Beaver permet de consulter, modifier, remplacer et désactiver le prompt système. Ce réglage vit dans l'application, pas dans les extensions.

- Le prompt effectivement envoyé est calculé par `system_prompt_resolver.rs`. Ses seules entrées sont : les réglages enregistrés, le nom du modèle, le mode, le niveau de détail, le prompt natif du modèle Ollama le cas échéant, et le prompt par défaut de Beaver (`system_prompt_resolver.rs:7-96`). **Aucune extension ne figure parmi ces entrées.**
- Quatre états sont possibles pour un prompt : par défaut, prompt Beaver, personnalisé, désactivé (`system_prompt_types.rs`, via `PromptSelection`).

**Toute la description de ce mécanisme appartient au brief `04-agent/prompts-systeme.md`.** Ne pas la dupliquer ici : renvoyer.

### 2. Ce qu'une extension peut faire, et ce qu'elle ne peut pas

Une extension communique avec Beaver par un canal fermé : une liste de méthodes déclarée dans un contrat, et rien d'autre. Cette liste est écrite deux fois, une fois côté extension et une fois côté Beaver, et les deux versions concordent.

**Les douze méthodes que peut appeler une extension** (`contract.json:11-24`, et le même ensemble côté Beaver dans `core_bridge.rs:80-128`, où tout autre nom renvoie une erreur, `:126`) :

`app.info`, `sessions.list`, `sessions.get`, `projects.list`, `mcp.connectors.list`, `mcp.tool.call`, `channels.config.get`, `secrets.provider.get`, `secrets.mcp.oauth.get`, `secrets.mcp.env.get`, `secrets.channel.get`, `host.load.stage`.

Aucune ne concerne les prompts, ni les réglages de l'agent.

**Ce qu'une extension peut déclarer** (`extension-api.mjs:120-181`) : des outils (`registerTool`), des skills (`registerSkill`), des ressources (`registerResource`), des éléments d'interface (`ui`), et l'écoute d'événements (`on`). En mode avancé, elle peut en plus recouvrir un outil natif (`unstable.registerReplacement`) — **qui ne concerne que les outils** : la fonction se contente d'appeler l'enregistrement d'outil ordinaire avec un drapeau (`extension-api.mjs:174-179`).

**Les événements écoutables** : un seul, `session.turn.started` (`contract.json:26`). Il signale qu'un tour de conversation a commencé ; il ne permet pas d'intervenir dans la construction du prompt.

### 3. Le seul texte que Beaver ajoute au prompt à cause des extensions

Il existe bien un paragraphe ajouté au prompt système à cause des extensions — mais **il est écrit par Beaver, pas par une extension**, et son texte est fixe.

- Il est ajouté à la fin du message système, en mode agentique uniquement (`chat_prompts.rs:124`, à l'intérieur de la branche `else` ouverte ligne `:109`).
- **En mode Chatbot, il n'est pas ajouté** : cette branche appelle `prepend_chat_system_prompt` et rien d'autre (`chat_prompts.rs:107-108`).
- Il n'apparaît que si **les deux** outils de découverte sont actifs, `list_extensions` et `inspect_extensions` ; si un seul manque, rien n'est ajouté (`extension_discovery_prompt.rs:3-12`, comportement couvert par le test `:33-56`).
- Son contenu explique au modèle comment consulter le catalogue d'extensions : inspecter directement 1 à 4 identifiants exacts, n'utiliser la liste complète que pour la vue d'ensemble, et ne jamais chercher une extension par mot-clé (`extension_discovery_prompt.rs:19-26`).

Ce texte est en anglais dans le code, comme le reste du prompt système.

### 4. Comment un contenu d'extension atteint quand même le modèle

C'est le point important pour la sécurité, et il ne passe pas par le prompt système.

**Les skills d'extension n'entrent pas dans le prompt système.** Le prompt contient une liste « Available skills » (`chat_prompt_sections.rs:9-23`, insérée par `chat_prompts.rs:177-184`). Cette liste est construite depuis le catalogue global (`agent_chat_task/common.rs:114-132`), et **ce catalogue exclut explicitement les skills d'extension** : toute entrée dont l'identifiant commence par `extension:` en est retirée (`skill_catalog.rs:35-41`). Le commentaire qui accompagne le filtre dit pourquoi : cet espace de noms est autorisé session par session et ne doit jamais entrer dans le catalogue global (`skill_catalog.rs:22`).

**En revanche, le contenu d'un skill d'extension entre dans la conversation quand le modèle le charge.** Le modèle demande un skill par son identifiant ; si celui-ci commence par `extension:`, Beaver charge le fichier Markdown fourni par l'extension (`extension_skill_loader.rs:3-18`). Deux garde-fous :

- le texte est préfixé par sa provenance, sous la forme `Skill source: <identifiant de l'extension>` (`extension_skill_loader.rs:27`) — le modèle voit donc d'où vient l'instruction ;
- le chargement est refusé si la session est en mode « natif seul », c'est-à-dire une session où les extensions sont écartées (`extension_skill_loader.rs:10-12`, état tenu par `extension_tool_set_native_only.rs:42-46`).

**Les descriptions déclarées par l'extension sont également du texte tiers vu par le modèle** : la description d'un outil est obligatoire, non vide, et bornée (`extension-api.mjs:57-66`). C'est par ces descriptions qu'un outil d'extension se présente au modèle.

### 5. Les limites qui encadrent ce texte

Voir le tableau ci-dessous. Toutes les valeurs viennent de `contract.json`, et ce fichier est réellement la source unique — ce n'est pas une convention, c'est vérifiable :

- côté extension, le contrat est lu au démarrage depuis `contract.json` (`contract.mjs:46-49`) ;
- côté Beaver, les constantes Rust ne sont pas recopiées à la main : elles sont **produites à la compilation** à partir du même fichier (`src-tauri/build.rs:9`, `extension_contract_build::generate()`), puis intégrées au code (`services/extensions/types.rs:7`).

C'est ce qui donne son poids au constat de cette page : la liste des méthodes autorisées n'existe qu'à un seul endroit, et elle ne contient rien qui touche au prompt système.

---

## Tableaux

### Ce qu'une extension peut et ne peut pas faire vis-à-vis du prompt système

| Capacité | Disponible ? | Source |
|---|---|---|
| Remplacer le prompt système | **Non** | Aucune méthode dans `contract.json:11-24` ; `core_bridge.rs:126` |
| Lire le prompt système | **Non** | Idem |
| Ajouter du texte au prompt système | **Non** | Idem ; `chat_prompts.rs:107-131` n'appelle aucun code d'extension |
| Intercepter la construction du prompt | **Non** | Un seul événement, `session.turn.started` (`contract.json:26`) |
| Lire ou modifier les réglages de prompt | **Non** | `channels.config.get` ne renvoie que la configuration des canaux (`core_bridge.rs:118-121`) |
| Recouvrir un **outil** natif | **Oui**, en mode avancé | `extension-api.mjs:174-179` ; `EXTENSIONS.md:687-722` |
| Fournir un skill que le modèle peut charger | **Oui** | `extension_skill_loader.rs:3-18` |
| Fournir des descriptions d'outils lues par le modèle | **Oui** | `extension-api.mjs:50-66` |

### Limites applicables au texte fourni par une extension

| Limite | Valeur | Source |
|---|---|---|
| Longueur d'une description d'outil, de skill ou de ressource | **2 000** caractères | `contract.json:37` (`maxExtensionTextChars`) |
| Outils par extension | **64** | `contract.json:34` |
| Skills par extension | **32** | `contract.json:38` |
| Ressources par extension | **64** | `contract.json:39` |
| Taille d'une ressource texte | **262 144** octets | `contract.json:43` |
| Événements écoutés par extension | **64** | `contract.json:35` |
| Durée maximale d'un gestionnaire d'événement | **5 000** ms | `contract.json:76` |

### Ordre d'assemblage du message système, en mode agentique

Cet ordre montre où le paragraphe lié aux extensions se place. Source : `chat_prompts.rs:107-131`, puis `agent_chat_task/common.rs:134-156`.

| Rang | Élément ajouté | Origine | Source |
|---|---|---|---|
| 1 | Prompt résolu selon vos réglages | Beaver ou vous | `chat_prompts.rs:110-123` |
| 2 | Paragraphe de découverte des extensions | Beaver, texte fixe | `chat_prompts.rs:124` |
| 3 | Liste des skills disponibles (**sans** ceux des extensions) | Beaver | `chat_prompts.rs:125-127` |
| 4 | `AGENTS.md` et personnalité | Vous | `chat_prompts.rs:128` |
| 5 | Langue de réponse | Réglage | `chat_prompts.rs:130` |
| 6 | Mémoire, contexte git, dossier de sorties, mode Plan | Beaver | `common.rs:150-155` |

---

## Encadrés

> **La porte « avancée » est fermée, pas entrouverte.**
>
> Une extension déclarée en mode avancé dispose d'un appel générique, `beaver.unstable.call(...)`. On pourrait croire qu'il donne accès à des fonctions cachées du cœur de Beaver. Ce n'est pas le cas.
>
> Cet appel n'accepte que les méthodes marquées « avancées » dans le contrat (`extension-api.mjs:167-172`, puis le filtre `extension-api.mjs:219-225`). Or **aucune méthode du contrat actuel ne porte ce marquage** : les douze méthodes déclarées sont toutes marquées « stable » (`contract.json:12-23`). L'ensemble des méthodes que `unstable.call` peut atteindre est donc vide aujourd'hui.
>
> La documentation du dépôt le formule elle-même : le niveau avancé « ne donne pas automatiquement accès à une méthode du cœur absente du contrat », et cet appel « existe pour les futures méthodes déclarées avancées, mais le contrat actuel n'en publie aucune : ce n'est pas une porte d'accès arbitraire au backend » (`EXTENSIONS.md:719-722`).

> **Pourquoi cette capacité n'est pas anodine — à lire si l'issue 3 est envisagée.**
>
> Le prompt système est le seul texte que le modèle reçoit avant tout contenu extérieur. C'est lui qui fixe les règles auxquelles le reste est censé obéir.
>
> Permettre à du code tiers de le réécrire reviendrait à laisser une extension annuler ces règles pour toute la session — y compris celles qui encadrent l'usage des outils. Une extension malveillante ou simplement compromise en amont n'aurait plus besoin de tromper le modèle : elle lui donnerait directement ses instructions.
>
> C'est la différence avec ce qui existe aujourd'hui. Le contenu d'une extension atteint le modèle **à l'intérieur de la conversation**, après le prompt système, et signé par sa provenance (`extension_skill_loader.rs:27`). Il reste un texte que le prompt système encadre, au lieu de devenir le cadre lui-même.
>
> Si la capacité devait être construite, la question à trancher d'abord n'est pas technique mais produit : quel bénéfice justifie de retirer ce garde-fou, et quelle limite le remplace.

> **Ne confondez pas deux mécanismes qui portent le même nom.**
>
> « Remplacement » désigne dans Beaver le fait pour une extension de recouvrir un **outil** natif — la recherche Web, par exemple. Cela ne concerne à aucun moment le prompt système. Voir le brief *Extensions › Remplacer un outil*.

---

## Pièges et erreurs fréquentes

| Piège | Cause | Résolution |
|---|---|---|
| Chercher dans les extensions le moyen de changer le prompt système | Le sommaire du mockup range le sujet dans Extensions | Le réglage est dans les réglages de Beaver. Renvoyer vers *Agent › Prompts système* |
| Croire que le mode avancé ouvre des fonctions supplémentaires du cœur | Le nom `unstable.call` le suggère | Aucune méthode avancée n'est publiée aujourd'hui (`contract.json:12-23`) |
| Attendre qu'un skill d'extension apparaisse dans la liste des skills du prompt | Les skills d'extension sont volontairement hors du catalogue global | Ils se découvrent par les outils d'extension et se chargent par leur identifiant `extension:` |
| S'étonner que le paragraphe sur les extensions n'apparaisse pas | Il ne s'ajoute qu'en mode agentique, et seulement si les deux outils de découverte sont actifs | Comportement voulu (`extension_discovery_prompt.rs:3-12`, `chat_prompts.rs:107-124`) |

---

## Renvois

- **`04-agent/prompts-systeme.md`** — la page qui traite réellement du sujet : modes, niveaux, portées, les quatre états, les prompts natifs Ollama, comment modifier et restaurer. **Renvoi principal de cette page.**
- **`07-integrations/extensions-centre.md`** — installer, activer et approuver une extension.
- **`07-integrations/extensions-remplacer-un-outil.md`** — le mécanisme de remplacement, qui ne concerne que les outils.
- **`07-integrations/extensions-ecrire.md`** — l'interface de programmation offerte aux auteurs, dont `registerSkill` et `registerResource`.
- *Agent › Instructions permanentes* — `AGENTS.md` et la personnalité, l'autre texte qui rejoint le message système.

---

## Comment le verdict a été établi

Cette section n'est pas destinée au site. Elle est là pour qu'un relecteur puisse refaire la vérification sans repartir de zéro, et pour qu'on sache quoi revérifier si la réponse change.

**Recherches négatives**, motifs `systemPrompt`, `system_prompt`, `systemprompt`, sans distinction de casse :

| Périmètre | Résultat |
|---|---|
| `EXTENSIONS.md` (1 066 lignes) | **Aucune occurrence** |
| `src-tauri/resources/extension-host/`, hors `node_modules/` et `vendor/` | **Aucune occurrence** |
| `src-tauri/src/services/extensions/` (plus de 200 fichiers) | **Aucune occurrence** |

**Vérifications positives :**

- la liste des méthodes est close des deux côtés (`contract.json:11-24` et `core_bridge.rs:80-128`, avec `_ => Err(())` ligne `:126`) ;
- cette liste n'existe qu'une fois : la version Rust est engendrée à la compilation depuis `contract.json` (`build.rs:9`, `types.rs:7`). Il n'y a donc pas de seconde liste, plus permissive, cachée ailleurs ;
- le résolveur de prompt ne reçoit aucune donnée d'extension (`system_prompt_resolver.rs:7-96`) ;
- le contrat ne déclare aucune méthode de niveau avancé — une seule occurrence du mot « advanced » dans `contract.json`, ligne `82`, et c'est un code de diagnostic ;
- l'assemblage du message système n'appelle aucun code d'extension, hormis le paragraphe fixe de la section 3 (`chat_prompts.rs:93-131`).

---

## Points à confirmer

1. **La décision de l'encadré liminaire** — retirer la page, la conserver recadrée, ou construire la capacité. C'est le point bloquant : tout le reste en dépend. À trancher par le propriétaire du produit.
2. **Le contenu exact renvoyé par les outils `list_extensions` et `inspect_extensions`.** Le paragraphe de découverte annonce que la liste complète fournit « les descriptions » (`extension_discovery_prompt.rs:23-24`), ce qui implique que du texte rédigé par l'auteur de l'extension entre dans la conversation par ce chemin. Le contenu précis de la réponse n'a pas été lu — il vit dans `src-tauri/src/services/extensions/discovery_listing.rs` et `discovery_inspection.rs`. À vérifier avant d'affirmer quoi que ce soit de précis sur ce chemin.
3. **Ce qui déclenche le mode « natif seul » d'une session.** Le mécanisme qui écarte les extensions d'une session a été lu (`extension_tool_set_native_only.rs`), mais pas ce qui le met en marche ni comment l'utilisateur le déclenche. À couvrir plutôt dans le brief *Extensions › Centre des extensions*.
4. **Rien n'a été vérifié à l'écran.** Aucune capture, aucune vérification des libellés d'interface. À reprendre dans la passe d'interface de fin de parcours, si la page est conservée.
5. **La date de validité de ce constat.** Il porte sur l'état du dépôt au **9 septembre 2026**, version **1.2.2**. Le contrat des extensions est un fichier qui évolue : si une méthode de niveau « avancé » y apparaît un jour, le verdict de cette page change. Le point de contrôle est court — la liste `methods.hostToCore` de `contract.json` et la liste `events` de la ligne `26`.
