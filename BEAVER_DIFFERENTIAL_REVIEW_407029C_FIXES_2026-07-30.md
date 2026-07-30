# Revue différentielle des corrections après `407029c`

Date : 30 juillet 2026

Branche : `fix/onboarding-splash-layout`

Base examinée : `407029c`

État examiné : copie de travail non commitée

## Résumé exécutif

Le correctif initial atteignait son objectif sur `o3` et `gpt-4o`, mais la revue
externe avait identifié cinq régressions : deux bloquantes, deux moyennes et une
mineure. Les cinq sont corrigées et couvertes par des tests ciblés.

| Niveau | Avant | Après |
| --- | ---: | ---: |
| Bloquant | 2 | 0 |
| Moyen | 2 | 0 |
| Mineur | 1 | 0 |

Verdict : **APPROVE**, sous réserve du commit de cette copie de travail. Aucun
problème nouveau n'a été trouvé dans les appelants directs ou dans les suites
complètes.

## Méthode et périmètre

La revue est de type **large et chirurgicale** :

- analyse des cinq constats du rapport fourni ;
- inspection de tous les fichiers modifiés depuis `407029c` ;
- inspection des appelants directs du registre, de la résolution des plafonds,
  de la construction des requêtes HTTP et des capacités agentiques ;
- lecture de l'historique pour distinguer les intentions anciennes des
  régressions récentes ;
- tests unitaires, tests d'intégration HTTP, lint, typage et compilation stricte ;
- vérification réelle du rendu dans Chromium avec catalogue dense, état d'erreur
  et fenêtres plus petites que le minimum Tauri de 900 × 600.

La cartographie Graphify a été reconstruite après les modifications :
22 561 nœuds, 47 507 relations et 1 381 communautés.

## Architecture retenue

Le registre LiteLLM embarqué reste la base locale. Il n'est plus consulté par une
fonction universelle car le prix, les capacités et les plafonds n'ont pas la même
sémantique :

1. Les **prix** utilisent uniquement l'entrée du fournisseur réellement facturé.
2. Les **capacités** fusionnent l'entrée OpenRouter et celle du modèle d'origine.
3. Le **plafond de sortie OpenRouter** privilégie le modèle d'origine, car les
   copies `openrouter/...` du fichier embarqué peuvent être anciennes.
4. Le catalogue `/models` du fournisseur, lorsqu'il publie un plafond, alimente
   un registre mémoire borné et prend la priorité sur la copie LiteLLM.
5. Groq et Cerebras gardent une politique explicite qui interdit l'ajout
   automatique d'un plafond.

Cette séparation évite de réutiliser, par exemple, le prix OpenAI pour un modèle
acheté via OpenRouter tout en conservant ses outils, sa vision et son raisonnement.

Sources d'architecture vérifiées :

- [OpenRouter — schéma de l'API Models](https://openrouter.ai/docs/guides/overview/models)
  publie `top_provider.max_completion_tokens`.
- [Groq — limites d'utilisation](https://console.groq.com/docs/rate-limits)
  documente les limites par modèle et l'impact des paramètres de requête.
- [Cerebras — limites d'utilisation](https://inference-docs.cerebras.ai/support/rate-limits)
  documente les quotas par modèle.
- [Cerebras — modèles publics](https://inference-docs.cerebras.ai/api-reference/models/public-models)
  confirme l'endpoint de catalogue.

## Corrections des constats

### 1. Groq et Cerebras recevaient un plafond automatique

Statut : **corrigé**.

`ProviderSpec.auto_max_tokens` sépare maintenant clairement « aucun repli connu »
de « ne pas envoyer de plafond automatiquement » :

- Groq : `false` ;
- Cerebras : `false` ;
- les autres fournisseurs : `true`.

Le paramètre traverse la route jusqu'à
`stream_max_tokens::resolve`. Une valeur explicitement demandée par l'appelant
reste autorisée et est toujours limitée au maximum du modèle.

Références :

- `src-tauri/src/services/llm/catalog.rs:21`
- `src-tauri/src/services/llm/catalog.rs:34`
- `src-tauri/src/services/llm/route.rs:21`
- `src-tauri/src/services/llm/stream_max_tokens.rs:3`
- `src-tauri/src/services/llm/stream_http.rs:29`

Couverture :

- test pur de décision automatique/explicite ;
- test du corps HTTP final confirmant l'absence de `max_tokens` et de
  `max_completion_tokens` pour les deux fournisseurs.

### 2. OpenRouter pouvait couper silencieusement les réponses

Statut : **corrigé**.

Pour un identifiant `auteur/modèle`, la résolution recherche d'abord l'entrée du
modèle d'origine, puis la copie OpenRouter. Les limites embarquées vérifiées sont :

| Modèle OpenRouter | Limite résolue |
| --- | ---: |
| `google/gemini-2.5-pro` | 65 535 |
| `openai/gpt-4o` | 16 384 |
| `openai/o3-mini` | 100 000 |

Lorsque `/models` publie une valeur plus fraîche, elle est enregistrée dans le
registre mémoire et prend la priorité.

Références :

- `src-tauri/src/services/llm/model_registry_lookup.rs:29`
- `src-tauri/src/services/llm/model_registry_lookup.rs:66`
- `src-tauri/src/services/llm/model_metadata.rs:1`
- `src-tauri/src/services/llm/openai_compat_parsing.rs:32`
- `src-tauri/src/services/llm/runtime_models.rs:18`
- `src-tauri/src/commands/llm.rs:14`

Couverture :

- registre LiteLLM réellement embarqué ;
- corps HTTP final OpenRouter ;
- valeurs nulles, nulles positives et dépassements de `u32` refusés.

### 3. La recherche stricte faisait perdre les capacités OpenRouter

Statut : **corrigé**.

La recherche stricte est conservée pour le prix. Les capacités suivent une autre
politique et fusionnent les drapeaux du revendeur et du modèle d'origine avec un
OU logique. Un `false` incomplet ne peut donc plus masquer un `true` publié par
une autre source.

Références :

- `src-tauri/src/services/llm/model_pricing.rs:11`
- `src-tauri/src/services/llm/model_registry_lookup.rs:41`
- `src-tauri/src/commands/llm.rs:73`
- `src-tauri/src/commands/agent_chat_task/api.rs:148`

Cas couvert : `openrouter/openai/o3` conserve outils, vision et raisonnement sans
réutiliser le tarif direct OpenAI.

### 4. Le test CSS ne testait que du texte fragile

Statut : **corrigé et complété**.

Le test de contrat passe maintenant par le moteur CSSOM du navigateur de test :

- les sélecteurs groupés et les espaces sont normalisés ;
- les règles imbriquées sont parcourues ;
- une règle absente fait échouer le test ;
- chaque propriété attendue doit être définie exactement une fois.

Il est volontairement nommé « contrat CSS » : la géométrie réelle est vérifiée
séparément dans Chromium.

Référence : `src/components/onboarding/__tests__/onboarding-layout.test.ts:11`.

Vérification Chromium :

| État | Fenêtre | Résultat |
| --- | --- | --- |
| API, 32 fournisseurs | 800 × 500 | boutons visibles, grille seule défilable |
| API, 32 fournisseurs | 640 × 480 | boutons visibles et cliquables |
| API, 32 fournisseurs | 560 × 420 | document contenu, grille défilable |
| API, 32 fournisseurs | 360 × 420 | boutons visibles et cliquables |
| API, erreur en ligne | 360 × 420 | erreur lisible, boutons inchangés |
| Import, 12 sources | 360 × 420 | pied visible, liste seule défilable |

Pour chaque cas, les rectangles des boutons sont dans la fenêtre,
`elementFromPoint` retourne bien les boutons, et le document n'a pas de
débordement horizontal ou vertical.

### 5. L'ordre des fournisseurs configurés pouvait masquer « connecté »

Statut : **corrigé**.

La borne reste de 32 éléments affichés, mais le filtrage porte maintenant sur les
identifiants réellement visibles. Un fournisseur visible est donc marqué
« connecté » même s'il arrive après quarante anciens identifiants sans rapport.

Références :

- `src/components/onboarding/onboarding-api.tsx:27`
- `src/components/onboarding/__tests__/onboarding-api.test.tsx:92`

## Analyse du rayon d'impact

### Lecture du registre

Les consommateurs directs ont été séparés :

- `model_pricing` : entrée stricte du fournisseur ;
- `commands/llm` : filtre chat et enrichissement des capacités ;
- `agent_chat_task/api` : outils, vision et raisonnement avant la boucle agent ;
- `stream_max_tokens` : plafond de sortie ;
- tests associés.

Cette séparation réduit le rayon d'impact d'une future modification : changer la
politique de prix ne peut plus changer les outils, et inversement.

### Registre mémoire

Le registre issu de `/models` est borné à :

- 16 fournisseurs ;
- 500 modèles par fournisseur ;
- identifiant fournisseur de 32 caractères maximum ;
- identifiant modèle de 128 caractères maximum.

Le fournisseur le plus ancien est évincé à la capacité maximale. Les entrées sont
isolées par fournisseur, ce qui interdit les collisions de modèles homonymes.

### Chemin HTTP

Le plafond final est calculé avant la construction du JSON. L'absence de valeur
produit l'absence du champ ; aucun `null` n'est envoyé. Les champs spécifiques
`max_completion_tokens` restent limités aux familles qui l'exigent.

## Analyse détaillée des fonctions critiques

### `model_registry_lookup::capabilities`

**But.** Cette fonction traduit les entrées LiteLLM en trois capacités utiles à
l'application. Pour OpenRouter, elle doit conserver les informations propres à la
route et celles du modèle vendu, sans mélanger les prix.

**Entrées et hypothèses.**

1. `provider_id` est déjà canonique.
2. `model_id` peut contenir un auteur, par exemple `openai/o3`.
3. Le registre est borné par la ressource embarquée.
4. Une entrée sans `mode` reste compatible chat, conformément aux données existantes.
5. Une capacité positive provenant d'une source fiable ne doit pas être annulée
   par une valeur négative ou absente ailleurs.

**Sorties et effets.**

1. Retourne `None` si aucune entrée chat pertinente n'existe.
2. Retourne trois booléens fusionnés sinon.
3. Ne modifie aucun état et ne touche pas au registre mémoire runtime.

**Lecture par blocs.**

- Lignes 41–44 : verrou en lecture, donc plusieurs consultations sont possibles
  sans mutation.
- Lignes 51–54 : sélection de deux sources au maximum. Pourquoi deux ? Une route
  OpenRouter peut avoir ses propres capacités et un modèle d'origine.
- Lignes 55–63 : accumulation par OU. Pourquoi ? Une capacité est une propriété
  intrinsèque ou exposée par la route ; perdre un `true` désactive une fonction
  valable, tandis que conserver un `true` est vérifié ensuite par le fournisseur.

**Invariants.**

1. Aucun prix n'est lu.
2. Une entrée non-chat n'enrichit pas les capacités.
3. Le résultat ne peut pas perdre un `true` présent dans l'une des deux sources.

**Risques contrôlés.**

1. Auteur OpenRouter inconnu : repli sur la route, puis les capacités runtime.
2. Données LiteLLM anciennes : fusion avec `/models` dans les appelants.
3. Collision de nom : `find_provider_entry` vérifie le fournisseur.

### `model_registry_lookup::max_output_tokens`

**But.** Cette fonction fournit une limite de sortie locale sûre pour un modèle.
Pour OpenRouter, elle évite les anciennes copies plus basses qui causaient des
réponses coupées.

**Entrées et hypothèses.**

1. Le fournisseur est canonique.
2. Les modèles OpenRouter suivent généralement `auteur/modèle`.
3. `max_output_tokens` est préférable à l'ancien `max_tokens`.
4. Une limite nulle est invalide.
5. Une valeur supérieure à `u32::MAX` ne peut pas entrer dans le protocole local.

**Sorties et effets.**

1. Retourne une valeur positive représentable.
2. Retourne `None` si le modèle ou sa limite est inconnu.
3. Ne modifie pas le registre.

**Lecture par blocs.**

- Lignes 66–69 : lecture atomique du registre partagé.
- Lignes 71–80 : priorité à l'entrée d'origine OpenRouter. Pourquoi ? Les copies
  route observées sont moins fraîches et ont causé les troncatures signalées.
- Lignes 99–105 : conversion contrôlée et refus de zéro. Comment ? `try_from`
  empêche le débordement, puis le filtre impose une valeur positive.

**Invariants.**

1. Jamais de troncature numérique silencieuse.
2. Jamais de limite issue d'un modèle non-chat.
3. OpenRouter préfère le modèle d'origine avant sa copie de route.

**Risques contrôlés.**

1. Format sans auteur : repli sur l'entrée OpenRouter.
2. Donnée absente : le niveau supérieur utilise `/models` ou le repli fournisseur.
3. Registre incohérent : valeurs invalides ignorées.

### `stream_max_tokens::resolve`

**But.** Cette fonction décide si un plafond doit être envoyé et quelle valeur
utiliser. Elle réunit la demande explicite, le catalogue frais, LiteLLM et la
politique du fournisseur.

**Entrées et hypothèses.**

1. `requested` vient d'un appelant interne déjà typé en `u32`.
2. Le registre runtime peut être plus récent que LiteLLM.
3. `auto_max_tokens=false` signifie absence totale de plafond automatique.
4. Une demande explicite reste valable même si l'automatisme est désactivé.
5. Un plafond explicite ne doit jamais dépasser le maximum connu du modèle.

**Sorties et effets.**

1. Retourne `None` pour omettre le champ JSON.
2. Retourne une valeur explicite éventuellement réduite.
3. Effectue seulement des lectures de registres.

**Lecture par blocs.**

- Lignes 10–15 : priorité au catalogue runtime. Pourquoi ? Il décrit les modèles
  encore proposés par le fournisseur.
- Ligne 16 : décision pure centralisée.
- Lignes 25–30 : matrice explicite. Comment ? Les branches explicites sont
  traitées avant la politique automatique ; la branche `false` retourne ensuite
  toujours `None`.

**Invariants.**

1. Une demande explicite n'est jamais augmentée.
2. `auto_max_tokens=false` et `requested=None` produisent toujours `None`.
3. Le repli fournisseur n'est utilisé que si l'automatisme est actif.

**Risques contrôlés.**

1. Catalogue runtime absent : repli LiteLLM.
2. Modèle inconnu : repli fournisseur uniquement si autorisé.
3. Valeur explicite trop haute : réduction au plafond connu.

### `runtime_models::replace_provider`

**But.** Cette fonction remplace atomiquement le catalogue mémoire d'un
fournisseur par la dernière réponse `/models`. Elle fournit une configuration par
modèle encore proposé sans faire du registre la source de la liste affichée.

**Entrées et hypothèses.**

1. Le fournisseur vient d'une route interne canonique.
2. La réponse distante peut contenir des identifiants mal formés.
3. La réponse peut dépasser 500 modèles.
4. Le verrou peut être empoisonné après une panique.
5. Une réponse nouvelle remplace intégralement l'ancienne pour ce fournisseur.

**Sorties et effets.**

1. Met à jour au plus un catalogue fournisseur.
2. Évince le fournisseur le plus ancien au-delà de 16.
3. Ignore l'opération si l'identifiant ou le verrou est invalide.

**Lecture par blocs.**

- Lignes 18–24 : validation puis verrou d'écriture. Pourquoi échouer fermé ? Une
  entrée invalide ne doit jamais contaminer les décisions de requête.
- Lignes 25–30 : copie bornée et validation de chaque modèle.
- Lignes 39–49 : remplacement et éviction FIFO par récence de remplacement.
  Comment ? L'identifiant est retiré puis ajouté en queue ; un nouveau fournisseur
  à capacité pleine retire la tête.

**Invariants.**

1. Maximum 16 × 500 modèles.
2. Aucun identifiant avec `..`, slash initial ou caractère non autorisé.
3. Deux fournisseurs ne partagent jamais la même table interne.

**Risques contrôlés.**

1. Croissance mémoire externe : bornes et éviction.
2. Collision de modèles : tables imbriquées par fournisseur.
3. Catalogue partiel : remplacement atomique sous verrou.

### `stream_http::post_chat_request_with_timeout`

**But.** Cette fonction transforme une configuration interne en requête HTTP
authentifiée et classe la réponse. Elle constitue le dernier point où une mauvaise
résolution de plafond pourrait modifier le comportement visible.

**Entrées et hypothèses.**

1. Le modèle est une chaîne fournie par l'interface.
2. Le fournisseur doit correspondre à une route connue.
3. Les messages et outils ont déjà été bornés par leurs couches respectives.
4. Les secrets restent dans le backend Rust.
5. Le délai doit respecter les limites du client sécurisé.

**Sorties et effets.**

1. Envoie une requête réseau authentifiée.
2. Capture les en-têtes d'usage sans corps brut.
3. Retourne une réponse réussie ou une erreur générique classée.

**Lecture par blocs.**

- Lignes 33–37 : validation du modèle et résolution fermée de la route.
- Lignes 38–47 : résolution du plafond avant le JSON. Pourquoi ici ? C'est le
  point commun à toutes les routes de chat.
- Lignes 48–63 : sérialisation mesurée puis envoi par le client sécurisé.
- Lignes 65–94 : capture bornée et classification des erreurs. Comment ? Le corps
  fournisseur est lu par `read_bounded`, puis seulement des codes sûrs sont logués.

**Invariants.**

1. Fournisseur inconnu : aucune requête.
2. Modèle de plus de 128 caractères : aucune requête.
3. `None` n'ajoute aucun champ de plafond au JSON.

**Risques contrôlés.**

1. Fuite de secret : clé chargée uniquement dans le backend et corps non logué.
2. Redirection : le client authentifié les refuse, couvert par test.
3. Erreur fournisseur énorme : lecture bornée, couverte par test.

## Historique utile

L'absence volontaire de plafond pour Groq et Cerebras précédait `ab98dde`.
`ab98dde` avait augmenté les replis des autres fournisseurs tout en gardant
`None` pour ces deux routes. `407029c` a changé implicitement le sens de `None`
en consultant automatiquement LiteLLM ; la nouvelle propriété explicite rétablit
l'intention sans renoncer aux limites par modèle.

La recherche stricte provenait du calcul des prix, où elle est correcte. La
réutiliser telle quelle pour les capacités a révélé que ces domaines avaient des
règles différentes ; le nouveau découpage matérialise cette différence.

## Tests exécutés

| Contrôle | Résultat |
| --- | --- |
| `npm test` — Vitest | 1 661 réussis, 363 fichiers |
| `npm test` — CEF | 2 réussis |
| `npm test` — hôte d'extensions | 46 réussis |
| `cargo test` | 2 517 réussis, 0 échec |
| `cargo clippy --all-targets -- -D warnings` | aucun avertissement |
| `cargo check` | réussi |
| `cargo fmt --check` | réussi |
| `npm run lint` | réussi |
| `npx tsc --noEmit` | réussi |
| `git diff --check` | réussi |
| Chromium — géométrie réelle | 6 scénarios réussis |

## Limites de la revue

- Aucun appel payant n'a été envoyé aux API réelles : aucune clé fournisseur
  n'était nécessaire ni exposée.
- Le rendu a été exécuté dans Chromium avec les vrais composants et feuilles de
  style, mais avec les commandes Tauri simulées pour fournir des catalogues
  déterministes.
- Les champs de catalogue sont couverts par les schémas officiels et par des
  tests locaux ; un fournisseur peut toujours changer son schéma ultérieurement.
- Le rapport décrit une copie de travail non commitée ; son verdict doit être
  réévalué si d'autres modifications sont ajoutées avant le commit.
