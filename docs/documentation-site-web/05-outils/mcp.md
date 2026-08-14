# Connecteurs externes — `search_mcp_tools`

**Emplacement site** — Outils › Connecteurs externes
**Répond à** — « Comment l'agent utilise les services que j'ai connectés — Notion, Linear, Slack… ? »
**Sources** — `tool_mcp.rs`, `tool_mcp_call.rs`, `tool_definitions_mcp.rs`, `permission_gate.rs`, `mcp_bridge/registry.rs`, `mcp_bridge/arguments.rs`, `tool_result_contract.rs`
**Vérification** — Vérifié dans le code

> Cette page décrit **l'outil** — comment l'agent s'en sert. La configuration des connecteurs (ajouter un service, l'authentifier, l'activer par conversation) est dans `07-integrations/mcp-connecteurs.md`.

---

## Plan de page proposé

1. À quoi ça sert
2. Un outil unique pour tous les services
3. Chercher un outil externe
4. Appeler un outil externe
5. L'approbation
6. Ce qui est vérifié avant l'appel
7. Ce que Beaver fait de la réponse
8. Quand ça échoue

---

## Contenu

### À quoi ça sert

MCP est un protocole standard qui permet à un assistant d'utiliser des services extérieurs : gestionnaire de tâches, base de connaissances, messagerie, outil de conception, plateforme de déploiement.

Concrètement, une fois un connecteur configuré, l'agent peut créer un ticket, lire une page, poster un message — sans que Beaver ait écrit une seule ligne de code spécifique à ce service.

L'outil `search_mcp_tools` fait partie du groupe **Connecteurs externes**, qui est **verrouillé**. Mais il ne sert à rien tant qu'aucun connecteur n'est configuré et activé : dans ce cas, il répond simplement « Aucun connecteur MCP activé ».

### Un outil unique pour tous les services

C'est le point de conception à expliquer, parce qu'il n'est pas évident.

Beaver **n'expose pas** les outils des connecteurs comme autant d'outils distincts au modèle. Un connecteur peut offrir cinquante fonctions ; les injecter toutes dans le contexte à chaque requête consommerait énormément de place, pour des fonctions qui ne servent presque jamais.

À la place, l'agent dispose d'**un seul outil à deux temps** :

1. **Chercher** — « quels outils externes existent pour parler d'issues ? »
2. **Appeler** — « exécute cet outil précis avec ces arguments »

Le coût dans le contexte est donc celui d'un seul outil, quel que soit le nombre de connecteurs configurés.

### Chercher un outil externe

- L'agent envoie un ou plusieurs mots-clés. Beaver interroge **tous les connecteurs activés** et renvoie les outils dont le nom, la description ou le nom du connecteur contiennent l'un des mots.
- Une **recherche vide liste tout** ce qui est disponible.
- Chaque outil est identifié sous la forme `service.nom_de_l_outil`, avec sa description.
- Au plus **15 outils par connecteur** sont remontés dans une recherche.
- Quand un connecteur ne répond pas, le résultat le dit — il liste les connecteurs ignorés au lieu de faire comme s'ils n'existaient pas.
- Si **aucun** connecteur ne répond, c'est une erreur franche, pas une liste vide.

### Appeler un outil externe

- L'agent donne l'identifiant complet de l'outil et ses arguments.
- Beaver vérifie que le connecteur est bien activé, puis transmet.
- L'appel est abandonné au bout de **60 secondes**.

### L'approbation

**Chercher ne demande rien. Appeler demande une approbation** en mode Demande d'approbation.

La distinction est logique : parcourir un catalogue n'a aucun effet, alors qu'un appel peut créer, modifier ou supprimer quelque chose dans un service extérieur — hors de la machine, et hors de portée d'une annulation.

### Ce qui est vérifié avant l'appel

- L'identifiant doit avoir la forme `service.outil`.
- Le nom du connecteur et celui de l'outil sont validés séparément. Un nom d'outil accepte au plus **64 caractères**, uniquement des lettres, des chiffres, des tirets et des soulignés.
- Le connecteur doit être **activé**. Un connecteur configuré mais éteint est inaccessible.
- **Les arguments sont validés contre le schéma déclaré par le service lui-même** avant d'être envoyés. Un appel mal formé est refusé localement, sans partir sur le réseau.

### Ce que Beaver fait de la réponse

La réponse d'un service externe est du contenu que Beaver ne contrôle pas. Elle est donc **nettoyée avant d'entrer dans la conversation** :

- Les **caractères de contrôle invisibles** sont retirés — notamment ceux qui inversent le sens de lecture du texte ou qui masquent du contenu. Un service compromis ne peut pas afficher une chose à l'utilisateur et en transmettre une autre au modèle.
- La réponse est plafonnée à **4 096 caractères**. Au-delà, elle est tronquée et le résultat le signale.
- **Une erreur renvoyée par le service reste une erreur.** Elle n'est jamais présentée à l'agent comme un succès sous prétexte que la communication, elle, a fonctionné.

### Quand ça échoue

Les échecs sont classés, et cette classification décide si l'agent peut réessayer.

| Type d'échec | Réessayable | Pourquoi |
|---|---|---|
| Service injoignable | **Oui** | Aucun appel n'est parti : réessayer est sans risque |
| Erreur renvoyée par le service | Non | Le service a reçu l'appel et l'a refusé |
| Réponse incompréhensible | Non | Le service a reçu l'appel ; son état est inconnu |
| Échec de transmission | Non | Idem |
| Délai de 60 secondes dépassé | Non | **L'action a peut-être abouti** |

Le dernier cas est le plus important à expliquer sur le site : un délai dépassé ne signifie pas que rien ne s'est passé. Le message d'erreur demande explicitement de vérifier l'état du service avant de relancer. C'est ce qui évite de créer deux fois le même ticket.

---

## Tableaux

### Les deux modes

| | Chercher | Appeler |
|---|---|---|
| Ce que fait l'agent | Explore le catalogue | Exécute une action |
| Approbation | Non | **Oui** |
| Effet extérieur | Aucun | Possible et durable |
| Délai maximal | Aucun explicite | **60 secondes** |
| Résultat | Liste d'outils avec leur description | Réponse du service |

### Les limites

| Limite | Valeur |
|---|---|
| Outils remontés par connecteur dans une recherche | **15** |
| Longueur d'un nom d'outil | **64 caractères** |
| Réponse d'un service transmise au modèle | **4 096 caractères** |
| Délai d'un appel | **60 secondes** |

---

## Encadrés

> **Chercher est gratuit, appeler engage.**
> Un appel MCP agit sur un service extérieur. Il n'est pas annulable depuis Beaver, et son effet survit à la conversation. C'est pour cela qu'il demande une approbation.

> **Un délai dépassé ne veut pas dire « rien ne s'est passé ».**
> Le message d'erreur le dit explicitement : vérifier l'état du service avant de relancer. Sans cette précaution, une action lente peut être exécutée deux fois.

> **Les réponses des services externes sont nettoyées.**
> Les caractères invisibles capables de tromper l'affichage sont retirés avant que la réponse n'entre dans la conversation. Un service compromis ne peut pas faire dire une chose au texte affiché et une autre au texte transmis au modèle.

> **Un connecteur désactivé est inaccessible, pas seulement caché.**
> La vérification a lieu au moment de l'appel, pas seulement au moment de la recherche.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| « Aucun connecteur MCP activé » | Aucun connecteur configuré, ou tous éteints | Voir `07-integrations/mcp-connecteurs.md` |
| « Aucun outil MCP ne correspond à la recherche » | Les mots-clés ne correspondent à rien | L'agent réessaie avec une recherche vide pour tout lister |
| « Catalogue MCP indisponible » | Aucun connecteur n'a répondu | Vérifier que les services démarrent, et l'authentification |
| « Outil MCP indisponible » | Connecteur désactivé, ou outil inexistant | Vérifier l'activation du connecteur |
| « Arguments MCP invalides » | L'agent a mal formé son appel | Il corrige seul en général |
| « Appel MCP expiré » | Service lent ou bloqué | **Vérifier l'état du service avant de relancer** |
| L'agent n'utilise pas un connecteur pourtant configuré | Il n'a pas pensé à chercher, ou ses mots-clés n'ont rien donné | Lui nommer le service explicitement |

---

## Renvois

- `07-integrations/mcp-connecteurs.md` — ajouter et configurer un connecteur
- `07-integrations/mcp-oauth.md` — les services qui demandent une authentification
- `04-agent/permissions.md` — pourquoi un appel demande une approbation
- `04-agent/contexte.md` — pourquoi les outils MCP ne sont pas exposés un par un
- `13-depannage/mcp-extensions-channels.md`

---

## Points à confirmer

- **La découverte repose sur une correspondance de mots-clés simple** — le mot doit apparaître tel quel dans le nom, la description ou l'identifiant du connecteur. Il n'y a ni synonymes, ni recherche approximative, ni classement par pertinence. Conséquence : un agent qui cherche « ticket » ne trouvera pas un outil qui parle d'« issue ». **Une refonte de ce mécanisme est un sujet connu de l'équipe** ; la page doit décrire l'état actuel sans le présenter comme définitif, et donner le conseil pratique : nommer le service dans sa demande.
- La limite de **15 outils par connecteur** dans une recherche est silencieuse : rien n'indique qu'un connecteur en offrait davantage. À signaler à l'équipe ; sur le site, mentionner qu'une recherche large peut ne pas tout montrer.
- Je n'ai **pas vérifié à l'écran** comment se présentent une recherche MCP et une demande d'approbation d'appel dans la conversation.
- Le comportement en cas de **connecteur qui répond très lentement à la recherche** (et non à l'appel) n'a pas été vérifié : la recherche n'a pas de délai maximal propre dans le code lu.
