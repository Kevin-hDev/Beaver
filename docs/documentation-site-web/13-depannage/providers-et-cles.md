# Dépannage — Fournisseurs et clés

**Emplacement site** — Référence › Dépannage › Fournisseurs et clés
**Répond à** — « Ma clé est refusée, mon quota est atteint, ma connexion a expiré, ou un fournisseur a disparu de la liste : qu'est-ce qui se passe et qu'est-ce que je fais ? »
**Sources** — `src-tauri/src/commands/api_keys.rs`, `commands/llm.rs`, `commands/search.rs` ; `src-tauri/src/services/api_keys_http.rs`, `api_keys_validate.rs`, `api_keys_state.rs` ; `src-tauri/src/services/llm/provider_error.rs`, `llm/types.rs`, `llm/retry.rs`, `llm/route_profile/policy_types.rs`, `llm/api_key_probe.rs` ; `src-tauri/src/services/llm_oauth/` (`refresh.rs`, `store.rs`, `types.rs`, `oauth_http.rs`) ; `src-tauri/src/services/codex_oauth/` (`token.rs`, `store.rs`) ; `src-tauri/src/services/mcp_oauth/storage.rs` ; `src-tauri/src/services/search/mod.rs` ; `src/hooks/use-api-keys.ts`, `src/hooks/cloud-models.ts`, `src/hooks/use-available-models.ts`, `src/hooks/oauth-models.ts` ; `src/components/api-keys/api-keys-config-dialog.tsx`, `src/components/api-keys/api-keys-tab.tsx`, `src/components/providers/oauth-provider-detail.tsx`, `src/components/agent-local/model-selector-list.tsx` ; `src/lib/agent-error-codes.ts` ; `src/i18n/fr.json`
**Vérification** — Vérifié dans le code, ligne par ligne, le 10 septembre 2026 sur la version **1.2.2**. Aucun de ces états n'a été provoqué à l'écran : les provoquer suppose une clé révoquée, un quota dépassé ou un fournisseur en panne. La liste de contrôle d'affichage est en fin de fichier.

---

## Avertissement au rédacteur

Cette page ne réexplique **ni la saisie d'une clé** (*Modèles › Fournisseurs par clé API*), **ni la connexion par compte web** (*Modèles › Fournisseurs par compte web*), **ni le fonctionnement du coffre** (*Sécurité › Le coffre et vos clés API*). Elle part des symptômes et y renvoie.

Trois règles pour toute la page :

1. **Chaque entrée commence par ce que l'utilisateur voit**, mot pour mot.
2. **Aucune promesse de résolution qui n'existe pas.** Plusieurs situations décrites ici ne se règlent pas dans Beaver, mais chez le fournisseur, sur son site. Le dire franchement, c'est l'information utile.
3. **Ne jamais écrire « Beaver va réessayer ».** Il ne réessaie pas ; la section 5 dit pourquoi, et c'est une décision assumée.

**⚠ Correction à porter au brief *Sécurité › Le coffre et vos clés API*.** Ce fichier annonce que l'utilisateur voit « Clé API invalide ou non autorisée », « Clé valide mais quota dépassé » et « test de la clé refusé ». **Il ne les voit pas** : le dialogue de saisie jette le message reçu et affiche toujours « L'opération a échoué. Réessaye. » (`api-keys-config-dialog.tsx:92-94`). Voir « Anomalies relevées », point 1.

---

## Plan de page proposé

1. La question à se poser en premier : clé ou compte ?
2. « L'opération a échoué. Réessaye. » à l'enregistrement d'une clé
3. « Authentification échouée, vérifie ta clé API »
4. Quota, limite de débit, crédits épuisés
5. Erreurs réseau, et pourquoi rien n'est réessayé
6. Une connexion par compte web a expiré
7. Un fournisseur a disparu de la liste
8. Le fournisseur est là, mais aucun modèle n'apparaît
9. Les fournisseurs de recherche web

---

## Contenu

### 1. La question à se poser en premier : clé ou compte ?

Beaver connaît **deux façons d'accéder à un fournisseur**, et les symptômes se ressemblent alors que les remèdes n'ont rien à voir.

| | Clé API | Compte web |
|---|---|---|
| Ce que vous fournissez | Une chaîne de caractères copiée chez le fournisseur | Une connexion dans un navigateur |
| Ce qui expire | Rien, sauf si vous révoquez la clé | **Le jeton, régulièrement** |
| Ce que Beaver renouvelle tout seul | Rien | **Le jeton, automatiquement** |
| Ce qui se passe quand ça casse | La clé est refusée à chaque requête | **La connexion est effacée** et le fournisseur disparaît |
| Fournisseurs concernés | Tous ceux de l'écran Clés API | **OpenAI (Codex), Grok (xAI), Kimi** |

**La question à poser en tête de page** : votre fournisseur est-il dans l'onglet **« Clés API »** ou dans l'onglet **« OAuth »** (`fr.json:15`) ? Les sections 2 à 5 traitent du premier cas, la section 6 du second.

### 2. « L'opération a échoué. Réessaye. » à l'enregistrement d'une clé

**Symptôme** — dans le dialogue d'ajout d'une clé, après **« Ajouter et tester »** (`fr.json:96`), l'indicateur passe de **« Test de la clé en cours… »** (`fr.json:93`) à un message d'erreur : **« L'opération a échoué. Réessaye. »** (`fr.json:1338`). Quand tout va bien, le message est **« Clé valide, provider connecté. »** (`fr.json:94`) et le dialogue se ferme après une demi-seconde (`api-keys-config-dialog.tsx:89-91`).

**Ce qu'il faut écrire en premier : ce message ne dit jamais ce qui a échoué.** Le code distingue pourtant sept causes, toutes remplacées par ce texte unique à l'affichage (`api-keys-config-dialog.tsx:92-94`).

**Les sept causes réelles, avec leur remède :**

| Ce qui s'est passé | Détecté par | Remède |
|---|---|---|
| La clé est **vide ou dépasse 256 caractères** | `api_keys_validate.rs:1`, `:26-28` | Vérifier qu'aucun espace ni retour à la ligne n'a été collé avec |
| La clé contient un **caractère de contrôle** | `api_keys_validate.rs:29-31` | Recopier la clé depuis le site du fournisseur, sans passer par un traitement de texte |
| Le fournisseur **n'est pas au catalogue** | `api_keys_validate.rs:13-17` | Passer par l'écran des clés API, pas par une valeur saisie à la main |
| Le fournisseur **a refusé la clé** (401 ou 403) | `api_keys_http.rs:139` | Section 3 |
| Le fournisseur **a signalé un quota dépassé** (429) | `api_keys_http.rs:140` | Section 4 |
| Le fournisseur **a refusé pour une autre raison** | `api_keys_http.rs:141` | Vérifier l'état du service chez le fournisseur |
| Le **test n'a pas pu partir** — réseau, proxy, service injoignable | `api_keys_http.rs:23`, `:33` | Section 5 |

**Le cas particulier de Qwen**, à mentionner parce qu'il produit un échec là où la clé est pourtant valable : une clé commençant par **`sk-sp-`** est refusée avant même d'être testée (`api_keys_http.rs:86-92`). Ce sont les clés d'espace de travail, que Beaver ne prend pas en charge. Il faut une clé de compte.

**Résolution générale, dans cet ordre :**

1. **Recopier la clé** depuis le site du fournisseur, en une seule fois, sans espace autour.
2. **Vérifier qu'elle est active** dans le tableau de bord du fournisseur.
3. **Vérifier que le compte a du crédit** — beaucoup de fournisseurs refusent une clé valable sur un compte sans solde, et cela se présente comme une clé invalide.
4. **Réessayer plus tard** si le service du fournisseur est en incident.

### 3. « Authentification échouée, vérifie ta clé API »

**Symptôme** — ce message apparaît dans une conversation, ou à la place de la liste des modèles d'un fournisseur (`fr.json:1399`).

**Cause** — le fournisseur a répondu **401** à une requête de Beaver (`services/llm/provider_error.rs:82`). Autrement dit : la clé enregistrée n'est pas ou plus acceptée.

**Les quatre causes possibles, à énoncer parce que la troisième surprend :**

1. La clé a été **révoquée ou supprimée** chez le fournisseur.
2. La clé a été **mal copiée** au moment de l'enregistrement — et personne ne peut plus la relire pour vérifier : aucune commande de Beaver ne permet d'afficher une clé enregistrée (*Sécurité › Le coffre et vos clés API*).
3. Le **compte a été suspendu** ou son moyen de paiement a expiré. Beaucoup de fournisseurs répondent alors 401, ce qui se présente comme une clé invalide.
4. La clé est **valable mais pas pour ce modèle** — certains fournisseurs restreignent l'accès par clé.

**Résolution** — la clé ne pouvant pas être relue, il n'y a qu'une voie fiable : **générer une nouvelle clé chez le fournisseur et la ressaisir dans Beaver**. Vérifier au passage l'état du compte et son solde.

⚠️ **Supprimer la clé dans Beaver ne la révoque pas** chez le fournisseur. Si vous pensez qu'elle a fuité, il faut la supprimer sur le site du fournisseur — c'est le seul geste qui la rend inutilisable.

**Un cas voisin, avec un message différent : « La connexion OAuth a expiré. Reconnecte le provider. »** (`fr.json:1417`). Celui-là concerne les comptes web, pas les clés : voir la section 6. Un même code 401 produit l'un ou l'autre message selon le contexte — sur un test de clé c'est « Authentification échouée » (`llm/provider_error.rs:193-195`), sur un catalogue de modèles c'est le message OAuth (`llm/provider_error.rs:184-187`).

### 4. Quota, limite de débit, crédits épuisés

Trois situations distinctes, trois messages différents, et il faut les séparer sur le site parce que les remèdes n'ont rien à voir.

**a) « Le service ou son fournisseur en amont limite actuellement les requêtes. Réessaie plus tard. Un solde disponible ne supprime pas ces limites, notamment sur les modèles gratuits. »** (`fr.json:1397`)

*Cause* — le fournisseur a répondu **429** (`llm/provider_error.rs:84`). C'est une limite de **débit** : trop de requêtes dans un intervalle court.

*Remède* — attendre. Le message le dit lui-même, et il a raison de le dire : **recharger son compte ne lève pas cette limite**. Elle est particulièrement basse sur les modèles annoncés gratuits.

*Détail vérifié* : quand le fournisseur indique une durée d'attente dans sa réponse, Beaver la lit, à condition qu'elle soit un nombre entier de secondes inférieur à **86 400** (`llm/provider_error.rs:11`, `:14-21`). Ce que l'interface en fait n'a pas été retrouvé — voir « Points à confirmer ».

**b) « xAI indique qu'un abonnement Grok ou des crédits sont requis. »** (`fr.json:1405`) et **« Le quota de l'abonnement xAI est épuisé ou momentanément indisponible. »** (`fr.json:1407`)

*Cause* — le fournisseur a répondu **402**, et sa réponse portait le code de dépassement de plafond propre à xAI (`llm/provider_error.rs:98-108`).

*Remède* — chez xAI : vérifier l'abonnement Grok ou ajouter des crédits. Rien à faire dans Beaver.

**c) « Moonshot ne parvient pas à confirmer un abonnement Kimi Code actif. »** (`fr.json:1404`)

*Cause* — même mécanisme, sur un message précis renvoyé par Moonshot en réponse à un 402 (`llm/provider_error.rs:89-97`).

*Remède* — vérifier l'abonnement Kimi Code chez Moonshot. Ce message apparaît **aussi juste après une connexion réussie**, sous une forme plus longue : « Connexion réussie, mais Moonshot ne parvient pas à confirmer un abonnement Kimi Code actif. Vérifie l'abonnement puis réessaie. » (`fr.json:23`).

**Deux autres refus, à ranger ici parce que l'utilisateur les confond avec un quota :**

- **« OpenRouter demande une confirmation de majorité pour ce modèle. Confirme tes 18 ans ou plus dans openrouter.ai/settings/preferences, puis réessaie. »** (`fr.json:1398`) — le seul message de tout le produit qui donne l'adresse exacte de la page à ouvrir. À conserver tel quel.
- **« Le provider refuse actuellement l'accès au modèle. »** (`fr.json:1406`) — refus d'accès générique sur un 402 ou un 403 non reconnu (`llm/provider_error.rs:109`).

### 5. Erreurs réseau, et pourquoi rien n'est réessayé

**Les messages, tous vérifiés :**

| Message | Cause | Source |
|---|---|---|
| **« Connexion au provider impossible. »** | La requête n'a pas pu partir ou atteindre le fournisseur | `fr.json:1408` |
| **« La connexion au provider a été perdue. Réessaye. »** | La réponse s'est interrompue en cours | `fr.json:1418` |
| **« Le provider a mis trop de temps à répondre. Réessaye. »** | Délai dépassé | `fr.json:1419` |
| **« Le provider est momentanément surchargé. Réessaye dans un instant. »** | Le fournisseur signale une surcharge | `fr.json:1420` |
| **« Le provider est temporairement indisponible. Réessaie dans un instant. »** | Service en incident | `fr.json:1409` |
| **« Le provider a refusé la requête. »** | Refus non classé — tout code HTTP hors 401, 402, 403 et 429 | `fr.json:1410` ; `llm/provider_error.rs:86` |
| **« Le provider n'a pas pu terminer la requête. Réessaye. »** | Échec générique | `fr.json:1421` |

**Le point central de cette section, et il est contre-intuitif : Beaver ne réessaie jamais tout seul une requête à un fournisseur distant.** Le nombre de reprises est **zéro** pour toutes les routes distantes (`llm/route_profile/policy_types.rs:74-79`).

**Et c'est délibéré.** Le fichier qui porte cette décision l'écrit en tête, en commentaire : « Les reprises automatiques restent fermées tant qu'un transport ne transmet pas une clé d'idempotence : une réponse perdue peut déjà avoir été facturée » (`services/llm/retry.rs:6-8`). Autrement dit : une requête qui semble perdue a peut-être abouti chez le fournisseur, et la réessayer automatiquement la facturerait deux fois. Beaver préfère vous rendre la main.

**Conséquence pratique** : quand un message se termine par « Réessaye », c'est une consigne, pas une politesse. **C'est à vous de relancer.**

⚠️ **Le moteur local, lui, réessaie jusqu'à dix fois** (voir *Dépannage › Ollama*, section 10). L'asymétrie est voulue : un moteur local ne facture rien.

**Résolution** — vérifier sa connexion, vérifier la page d'état du fournisseur, puis relancer la requête à la main. Sur un réseau d'entreprise, vérifier qu'un proxy ne bloque pas le domaine du fournisseur.

### 6. Une connexion par compte web a expiré

**Symptôme** — **« La connexion a expiré. Reconnecte ce provider. »** dans l'écran du fournisseur (`fr.json:23`), ou **« La connexion OAuth a expiré. Reconnecte le provider. »** dans une conversation (`fr.json:1417`).

**Comment ça marche normalement, et il faut le dire avant de parler de la panne.** Un jeton d'accès a une durée de vie courte. Beaver le renouvelle **tout seul, avant expiration**, sans jamais vous solliciter :

| Compte | Marge avant expiration | Source |
|---|---|---|
| **Grok (xAI)** et **Kimi** | **60 secondes** | `llm_oauth/types.rs:49-52` |
| **OpenAI (Codex)** | **5 minutes** | `codex_oauth/store.rs:6`, `:22-26` |
| Connecteurs MCP | **30 secondes** | `mcp_oauth/storage.rs:46-49` |

**Quand le renouvellement échoue, deux issues très différentes** (`llm_oauth/refresh.rs:72-99`) :

- **Le fournisseur répond « non autorisé »** — jeton de renouvellement révoqué, mot de passe changé, session fermée depuis le site du fournisseur. Beaver **efface la connexion** (`refresh.rs:94-97`) et la considère comme inexistante. Le fournisseur disparaît alors de la liste des comptes connectés : c'est la section 7.
- **Le renouvellement échoue pour une autre raison** — réseau, service en panne. La connexion est **conservée** ; il suffit de réessayer plus tard.

Cette distinction est importante et elle est invisible à l'écran : dans un cas il faut se reconnecter, dans l'autre il faut attendre.

**Un détail rassurant à écrire** : pour OpenAI, si le renouvellement échoue alors que le jeton actuel est encore valable quelques minutes, Beaver **continue avec le jeton en cours** au lieu d'échouer (`codex_oauth/token.rs:39-43`). Une coupure réseau passagère au mauvais moment ne casse donc pas la session.

**Résolution :**

1. Ouvrir l'onglet **OAuth**, sélectionner le fournisseur, cliquer sur **« Se connecter »** (`fr.json:20`).
2. Suivre le parcours dans le navigateur. Les étapes affichées sont **« En attente de connexion… »**, **« Validation requise »**, puis **« Connecté »** ; en cas d'échec, **« Connexion impossible »**, et si vous abandonnez, **« Connexion annulée »** (`fr.json:21`).
3. Si le message revient immédiatement après une reconnexion réussie, ce n'est pas un problème de connexion mais d'abonnement : voir la section 4.

**Se déconnecter** (**« Se déconnecter »**) efface l'entrée du coffre pour ce fournisseur seulement ; les autres connexions ne sont pas touchées.

### 7. Un fournisseur a disparu de la liste

**Symptôme** — un fournisseur qui était là hier n'apparaît plus dans le sélecteur de modèles, ou plus dans la liste des comptes connectés.

**Quatre causes, à traiter séparément :**

**a) Sa clé a été supprimée.** La liste des fournisseurs configurés est reconstruite à chaque modification, et l'interface en est prévenue immédiatement (`commands/api_keys.rs:70`, événement `providers-changed`). Un fournisseur sans clé n'apparaît plus.

**b) Sa connexion par compte web a été effacée après un refus de renouvellement.** C'est le cas décrit en section 6 : **Beaver efface lui-même la connexion** quand le fournisseur déclare le jeton de renouvellement invalide (`llm_oauth/refresh.rs:80-83`, `:94-97`). Rien ne prévient que c'est arrivé — le fournisseur est simplement absent au chargement suivant.

**c) Son catalogue de modèles a échoué — et là, il ne disparaît pas complètement.** Le fournisseur reste visible dans le sélecteur avec un compteur à **0**, et **la raison s'affiche à l'intérieur quand on le déplie** (`model-selector-list.tsx:107`, `:138-142`). C'est le comportement souhaitable, et il mérite d'être expliqué sur le site : un fournisseur à zéro modèle n'est pas cassé, il faut le déplier pour lire pourquoi.

**d) Le catalogue entier est injoignable.** Si Beaver n'arrive pas à obtenir la liste des fournisseurs disponibles, **tous** les fournisseurs configurés passent en échec avec la même raison, « Le catalogue de modèles est temporairement indisponible. Réessaie plus tard. » (`cloud-models.ts:29-37` ; `fr.json:1400`).

**Résolution :**

1. **Ouvrir l'écran Clés API** : si le fournisseur y figure, sa clé est toujours enregistrée et le problème est ailleurs (cas c ou d).
2. **Ouvrir l'onglet OAuth** : si le fournisseur y figure comme déconnecté, c'est le cas (b) — se reconnecter.
3. **Déplier le fournisseur dans le sélecteur de modèles** pour lire la raison exacte.
4. **Cliquer sur « Réessayer »** dans l'écran du fournisseur (`fr.json:22`), qui relance la récupération du catalogue (`oauth-provider-detail.tsx:45-56`).

### 8. Le fournisseur est là, mais aucun modèle n'apparaît

**Symptôme** — le fournisseur est visible, son compteur affiche **0**, et une ligne grisée à l'intérieur donne la raison.

**Les six raisons possibles**, avec le message affiché (`oauth-provider-detail.tsx:14-21` pour les comptes web, `lib/agent-error-codes.ts` pour les autres) :

| Message | Ce qui s'est passé | Remède |
|---|---|---|
| « Le catalogue de modèles est temporairement indisponible. Réessaie plus tard. » | Cause non identifiée, ou catalogue injoignable | Réessayer |
| « Le catalogue est temporairement limité. Réessaie dans un instant. » | Limite de débit sur la requête de catalogue | Attendre |
| « Le provider refuse actuellement l'accès au catalogue. » | Refus d'accès | Vérifier le compte chez le fournisseur |
| « La connexion a expiré. Reconnecte ce provider. » | Jeton expiré | Section 6 |
| « Connexion réussie, mais Moonshot ne parvient pas à confirmer un abonnement Kimi Code actif… » | Abonnement non confirmé | Section 4 |
| « xAI indique que l'abonnement Grok ou les crédits requis sont absents ou épuisés. » | Abonnement ou crédits | Section 4 |

**À écrire clairement** : ces six messages viennent du **catalogue**, pas d'une conversation. Un fournisseur peut très bien avoir un catalogue en échec et fonctionner par ailleurs, ou l'inverse.

**Un dernier message, rencontré en cours de conversation** : **« Ce modèle n'est plus disponible. Choisis un autre modèle puis réessaie. »** (`fr.json:1402`). Le fournisseur a retiré ce modèle. Il n'y a rien à réparer : il faut en choisir un autre.

### 9. Les fournisseurs de recherche web

Les trois fournisseurs de recherche — **Brave**, **Exa**, **Firecrawl** — utilisent le même écran de clés et les mêmes messages que les fournisseurs de modèles, avec une différence à signaler.

**Leur test de clé est un vrai appel facturable** : une recherche sur Brave (`api_keys_http.rs:112-116`), une recherche sur Exa (`:117-122`), une consultation de crédits sur Firecrawl (`:123-125`). Tester une clé consomme donc, sur deux d'entre eux, une unité du quota.

**Un fournisseur de recherche sans test** renvoie **« Test non implémenté pour <nom> »** (`services/search/mod.rs:198`). Ce message est en dur dans le code, non traduit, et n'atteint pas l'utilisateur dans le parcours normal — voir « Anomalies relevées ».

---

## Tableaux

### Tableau — Quel code HTTP produit quel message

L'essentiel de la classification tient en cinq lignes (`services/llm/provider_error.rs:76-110`). À publier : c'est ce qui permet de comprendre pourquoi deux pannes différentes affichent le même texte.

| Réponse du fournisseur | Ce que Beaver en conclut | Message affiché |
|---|---|---|
| **401** | Authentification refusée | « Authentification échouée, vérifie ta clé API » — ou le message OAuth selon le contexte |
| **402** | Paiement requis ; le corps de la réponse est examiné pour xAI et Moonshot | « xAI indique qu'un abonnement… », « Moonshot ne parvient pas… », ou « Le provider refuse actuellement l'accès au modèle. » |
| **403** | Refus d'accès ; le corps est examiné pour le cas OpenRouter | « OpenRouter demande une confirmation de majorité… » ou « Le provider refuse actuellement l'accès au modèle. » |
| **429** | Limite de débit | « Le service ou son fournisseur en amont limite actuellement les requêtes… » |
| **Tout le reste** | Refus non classé | « Le provider a refusé la requête. » |

### Tableau — Les limites du côté de Beaver

| Ce qui est borné | Valeur | Source |
|---|---|---|
| Longueur d'une clé API | **256 caractères** | `api_keys_validate.rs:1` |
| Délai d'un test de clé | **10 secondes** | `api_keys_http.rs:5` |
| Reprises automatiques vers un fournisseur distant | **0** | `llm/route_profile/policy_types.rs:74-79` |
| Durée d'attente lue dans une réponse 429 | au plus **86 400 secondes** | `llm/provider_error.rs:11` |
| Marge de renouvellement Grok et Kimi | **60 secondes** | `llm_oauth/types.rs:51` |
| Marge de renouvellement OpenAI | **5 minutes** | `codex_oauth/store.rs:6` |
| Marge de renouvellement d'un connecteur MCP | **30 secondes** | `mcp_oauth/storage.rs:46-49` |
| Longueur d'un jeton Grok ou Kimi | **4 096 caractères** | `llm_oauth/store.rs:8` |
| Corps d'erreur d'un fournisseur conservé pour les traces | **200 caractères**, filtré | `services/llm/mod.rs:146-153` |

### Tableau — Où se règle chaque problème

| Symptôme | Se règle dans Beaver | Se règle chez le fournisseur |
|---|---|---|
| Clé mal collée | **Oui** — la ressaisir | — |
| Clé révoquée | Ressaisir la nouvelle | **Oui** — en générer une |
| Compte sans crédit | — | **Oui** |
| Limite de débit atteinte | — | Attendre ; recharger ne sert à rien |
| Abonnement Grok ou Kimi non confirmé | — | **Oui** |
| Confirmation de majorité OpenRouter | — | **Oui**, sur openrouter.ai |
| Connexion par compte expirée | **Oui** — se reconnecter | — |
| Connexion effacée après révocation | **Oui** — se reconnecter | Vérifier que la session n'a pas été fermée depuis leur site |
| Réseau, proxy, fournisseur en panne | Relancer à la main | — |
| Catalogue de modèles en échec | **Oui** — « Réessayer » | — |

---

## Encadrés

> **⚠ Beaver ne réessaie jamais une requête distante.** À placer en tête de la section 5.
> C'est un choix, pas un oubli : une requête qui semble perdue a peut-être abouti chez le fournisseur, et la rejouer automatiquement la facturerait deux fois. Quand un message se termine par « Réessaye », c'est à vous de relancer. Le moteur local, qui ne facture rien, réessaie jusqu'à dix fois.

> **⚠ Supprimer une clé dans Beaver ne la révoque pas.** À placer dans la section 3.
> Elle reste parfaitement valable sur votre compte chez le fournisseur. Si vous pensez qu'elle a fuité, la seule action efficace est de la **supprimer sur le site du fournisseur**.

> **ℹ Une clé enregistrée ne peut plus être relue.** À placer dans la section 3.
> Y compris par Beaver lui-même : aucune commande ne permet d'afficher une clé. En cas de doute sur une clé enregistrée, il faut en générer une nouvelle chez le fournisseur — il n'y a pas de moyen de vérifier celle qui est en place.

> **⚠ Recharger votre compte ne lève pas une limite de débit.** À placer dans la section 4.
> Le message de Beaver le dit déjà : une limite de requêtes par minute est indépendante de votre solde, et elle est particulièrement basse sur les modèles annoncés gratuits.

> **ℹ Un fournisseur à zéro modèle n'est pas cassé.** À placer dans la section 7.
> Dépliez-le : la raison exacte s'affiche à l'intérieur. C'est la seule façon de savoir si c'est une limite passagère, une connexion expirée ou un problème d'abonnement.

> **⚠ Tester une clé de recherche consomme du quota.** À placer dans la section 9.
> Le test de Brave et d'Exa exécute une vraie recherche. Sur un quota mensuel serré, testez une fois, pas dix.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| « L'opération a échoué. Réessaye. » sans autre indication | Un message unique couvre les sept causes | Tableau de la section 2 |
| Ma clé Qwen est refusée alors qu'elle marche ailleurs | Les clés d'espace de travail `sk-sp-` ne sont pas prises en charge | Utiliser une clé de compte |
| Ma clé est refusée après un espace collé sans le vouloir | La validation refuse au-delà de 256 caractères et sur caractère de contrôle | Recopier proprement, sans passer par un traitement de texte |
| Mon fournisseur a disparu du jour au lendemain | Sa connexion par compte a été effacée après un refus de renouvellement | Se reconnecter dans l'onglet OAuth |
| Je me reconnecte et le message revient aussitôt | Ce n'est pas la connexion, c'est l'abonnement | Section 4 |
| J'attends que Beaver réessaie et rien ne se passe | Aucune reprise automatique n'existe | Relancer soi-même |
| « Clé valide » mais aucun modèle ne s'affiche | Le test et le catalogue sont deux appels différents | Déplier le fournisseur pour lire la raison |
| Mes clés ont disparu après une réinstallation du système | La clé maîtresse du coffre vivait dans le trousseau | *Sécurité › Le coffre et vos clés API* |
| Le message parle d'un fournisseur « en amont » | Un intermédiaire comme OpenRouter relaie un autre fournisseur | Le refus vient du fournisseur final, pas de l'intermédiaire |

---

## Renvois

- *Modèles › Fournisseurs par clé API* — la liste des fournisseurs, où récupérer une clé et comment la saisir
- *Modèles › Fournisseurs par compte web* — se connecter avec un compte OpenAI, Grok ou Kimi
- *Sécurité › Le coffre et vos clés API* — où va la clé, pourquoi elle ne peut plus être relue
- *Intégrations › Connecteurs MCP* — les jetons OAuth des connecteurs, qui suivent le même mécanisme
- *Outils › Recherche web* — les trois fournisseurs de recherche et leur usage
- *Dépannage › Ollama* — le moteur local, qui n'a besoin d'aucune clé
- *Dépannage › Installation* — les échecs qui précèdent tout usage de fournisseur

---

## Anomalies relevées

Constatées en écrivant cette page, **non corrigées**. À arbitrer par l'équipe.

1. **Le dialogue de saisie d'une clé jette le message précis du fournisseur.** Le code produit sept messages distincts — dont « Clé API invalide ou non autorisée » et « Clé valide mais quota dépassé » (`api_keys_http.rs:139-141`) — et le dialogue les remplace tous par « L'opération a échoué. Réessaye. » (`api-keys-config-dialog.tsx:92-94`). L'information la plus utile du parcours est produite puis effacée. **Ce point corrige aussi le brief *Sécurité › Le coffre et vos clés API***, qui présente ces trois messages comme visibles par l'utilisateur.

2. **Les messages du test de clé sont en français en dur dans le code Rust.** « Clé API invalide ou non autorisée », « Clé valide mais quota dépassé », « test de la clé refusé », « test de la clé impossible » (`api_keys_http.rs:23`, `:33`, `:139-141`), « provider inconnu », « clé API invalide (vide ou trop longue) », « clé API contient des caractères de contrôle » (`api_keys_validate.rs:15`, `:27`, `:30`), « fournisseur inconnu » (`api_keys_http.rs:127`). Aucun ne passe par les fichiers de traduction, alors que l'application est livrée en sept langues. Ils sont aujourd'hui masqués par l'anomalie 1 ; le jour où celle-ci sera corrigée, ils apparaîtront tels quels en français à un utilisateur japonais.

3. **Même constat côté connexions par compte web.** « Connexion requise », « Renouvellement impossible », « Connexion impossible », « Connexion modifiée » (`llm_oauth/refresh.rs:35`, `:76`, `:98`, `:103` ; `llm_oauth/store.rs:17`) sont en dur en français.

4. **Une connexion effacée par Beaver ne prévient personne.** Quand le fournisseur refuse le renouvellement, la connexion est supprimée du coffre (`llm_oauth/refresh.rs:80-83`, `:94-97`) et le fournisseur disparaît de la liste au chargement suivant. Aucun message n'est affiché au moment de la suppression, et l'utilisateur n'a aucun moyen de savoir que c'est Beaver qui a agi, ni pourquoi.

5. **`src/lib/app-error.ts` ne couvre aucune erreur de fournisseur.** Ce fichier, souvent cité comme la correspondance entre codes et messages traduits, ne contient que les erreurs Git (branches, fusion, dépôt). La correspondance réellement utilisée pour les fournisseurs est `src/lib/agent-error-codes.ts`. À corriger dans les briefs qui renverraient au premier.

6. **Un message inatteignable.** « Test non implémenté pour <nom> » (`services/search/mod.rs:198`) ne peut être produit que si un fournisseur de recherche est ajouté au catalogue sans être ajouté à cette liste. Il est en dur, non traduit, et interpole un identifiant technique.

---

## Points à confirmer

**Écarts à arbitrer avant publication**

1. **Que fait l'interface de la durée d'attente lue dans une réponse 429 ?** La valeur est extraite et bornée à 86 400 secondes (`llm/provider_error.rs:14-21`), mais son chemin jusqu'à l'écran n'a pas été suivi jusqu'au bout. Si elle n'est pas affichée, ne rien promettre sur le site ; si elle l'est, le dire, parce que c'est une information rare et précieuse.

2. **Le message affiché quand un fournisseur configuré n'a plus de clé valable au démarrage** n'a pas été identifié. La liste des fournisseurs configurés (`configured-providers.json`) et l'état réel des clés dans le coffre sont deux choses distinctes : à vérifier qu'un désaccord entre les deux se présente proprement.

3. **Le comportement quand le coffre est indisponible** (session Linux sans service de secrets) sur l'écran des clés API : le brief *Sécurité › Le coffre et vos clés API* laisse ce point ouvert, et il l'est toujours ici. Toute opération sur une clé renvoie alors « coffre indisponible », message en dur non traduit.

4. **La distinction entre « connexion expirée » et « connexion effacée » n'est pas visible.** Section 6, cas 1 et 2 : dans un cas il faut se reconnecter, dans l'autre attendre. Décider si le site explique cette différence — ce qui suppose de dire à l'utilisateur de reconnaître un état que l'application ne lui montre pas — ou s'il se contente de la consigne unique « réessayez, puis reconnectez-vous si ça persiste ».

5. **Le sort d'une conversation interrompue par une erreur de fournisseur.** Non exploré : la conversation est-elle reprennable en l'état, faut-il renvoyer le message, le brouillon est-il conservé ? C'est la première question de quelqu'un qui perd une réponse longue.

6. **Le cas d'un intermédiaire qui relaie l'erreur d'un autre fournisseur.** Beaver sait extraire le nom du fournisseur en amont d'une réponse OpenRouter (`llm/provider_error.rs:139-142`, `:147-166`). Où ce nom apparaît à l'écran, et sous quelle forme, n'a pas été retrouvé.

**Affichage non vérifié — liste de contrôle pour la passe d'interface finale**

7. **Le dialogue de saisie d'une clé** dans ses quatre états : vide, test en cours, succès (une demi-seconde avant fermeture), échec. Le message d'échec est-il lisible, persistant, et le champ conserve-t-il la clé saisie ?
8. **La représentation d'une clé déjà enregistrée** — points, mention « configurée », champ vide. Non observé ; le brief du coffre laisse déjà ce point ouvert.
9. **Le fournisseur à zéro modèle dans le sélecteur** : à quoi ressemble la ligne grisée qui porte la raison, et si elle est visible sans déplier.
10. **L'écran d'un compte web déconnecté** : ce qui distingue « jamais connecté » de « connexion effacée après révocation ».
11. **Les étapes de connexion par compte web** (« En attente de connexion… », « Validation requise », « Connexion impossible », « Connexion annulée ») et le parcours avec code à saisir (« Saisissez ce code sur la page ouverte dans votre navigateur »), non observés.
12. **Les messages d'erreur de fournisseur dans une conversation** : leur emplacement, leur persistance, et s'il existe un bouton pour relancer la requête — décisif, puisque aucune reprise automatique n'existe.
