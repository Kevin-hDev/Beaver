# SPEC — GLM-5.3, Grok 4.6, Gemini 3.7 Flash et transport xAI OAuth

- **Statut :** normative pour la phase 1
- **Snapshot étudié :** dépôt Beaver du 22 août 2026
- **Vérification :** 22 août 2026

## Autorité documentaire

Cette SPEC est l’autorité unique de la phase 1 pour :

- l’ajout de `glm-5.3` chez Z.AI par clé API ;
- l’ajout de `grok-4.6` chez xAI par clé API et OAuth ;
- l’ajout de `gemini-3.7-flash` chez Google par clé API ;
- le maintien de `grok-4.5` tant que xAI le publie ;
- la mise à jour des descriptions provider partagées par l’onboarding et les
  réglages ;
- la séparation entre le transport xAI API et le transport d’abonnement OAuth ;
- la restitution durable d’un échec sans texte dans une session.

Les registres de modèles restent l’autorité des capacités statiques. Le catalogue
OAuth xAI reste l’autorité des modèles et transports disponibles pour le compte
connecté. Un champ dynamique validé peut enrichir le registre, jamais affaiblir
ses limites de sécurité.

## État observé dans Beaver

### Z.AI

`src-tauri/resources/provider-models/zai.json` commence à `glm-5.2` et ne contient
pas `glm-5.3`. Z.AI n’expose pas de catalogue dynamique actuellement utilisé par
Beaver : le modèle est donc invisible.

### xAI API

Le registre xAI commence à `grok-4.5`. Le catalogue dynamique `/models` peut
faire apparaître `grok-4.6`, mais les capacités locales ne connaissent pas ses
niveaux de raisonnement. `grok-4.5` doit rester présent.

### xAI OAuth

La route `xai-oauth` est actuellement résolue vers :

```text
https://api.x.ai/v1/chat/completions
```

Elle n’ajoute que le Bearer OAuth et le User-Agent Beaver. Elle n’utilise ni le
proxy d’abonnement, ni ses en-têtes de contrat, ni son catalogue dédié.

La session `e79ff35a-3d3b-4707-bc6b-1242dc68243c` contient cinq exécutions
diagnostiquées mais seulement trois messages utilisateur. Les appels Grok 4.5
et 4.6 ont reçu un HTTP 429 avec le code sûr `resource-exhausted` sur la route
directe. Cela prouve un échec provider réel et commun aux deux modèles ; cela ne
prouve pas un défaut propre à Grok 4.6.

L’interface garde l’erreur en mémoire pendant le flux, mais ne crée aucun
message assistant lorsqu’aucun segment n’a été reçu. Après rechargement, le
diagnostic Rust existe encore, mais l’interface ne le restitue pas.

### Google Gemini et onboarding

Le registre Google commence à `gemini-3.6-flash` et ne contient pas
`gemini-3.7-flash`. Les descriptions provider utilisées à la fois par
l’onboarding et les réglages citent encore GLM-5.2, Grok 4.5 et Gemini 3.6
Flash. Il n’existe pas de copie propre à l’onboarding : les traductions
`apiKeys.providers` restent l’autorité unique à mettre à jour.

## Contrats officiels retenus

### GLM-5.3

Source : <https://z.ai/blog/glm-5.3>

- identifiant API : `glm-5.3` ;
- thinking obligatoire ;
- efforts : `low`, `high`, `max` ;
- défaut : `max` ;
- `thinking.type: "disabled"` est refusé ;
- `max` est recommandé pour le code.

Les valeurs exactes de contexte et de sortie sont inscrites dans le registre
uniquement après concordance entre la documentation officielle courante et un
appel réel borné. Le benchmark publié ne sert pas seul de contrat de quota API.

### Grok 4.6 API

Source : <https://docs.x.ai/developers/grok-4-6>

- identifiant API : `grok-4.6` ;
- contexte publié : 500 000 tokens ;
- entrée texte et image, sortie texte ;
- tools ;
- efforts : `low`, `medium`, `high`, `xhigh` ;
- défaut : `high` ;
- Chat Completions et Responses sont supportés.

### Gemini 3.7 Flash

Sources :

- <https://ai.google.dev/gemini-api/docs/models/gemini-3.7-flash> ;
- <https://ai.google.dev/gemini-api/docs/latest-model> ;
- <https://ai.google.dev/gemini-api/docs/deprecations>.

- identifiant API stable : `gemini-3.7-flash` ;
- contexte : 1 048 576 tokens ;
- sortie maximale : 65 536 tokens ;
- entrées texte, image, vidéo, audio et PDF, sortie texte ;
- tools et thinking ;
- efforts : `low`, `medium`, `high`, défaut `medium` ;
- Gemini 3.5 et 3.6 restent publiés sans date d’arrêt : ils ne sont pas retirés.

### xAI OAuth

Sources officielles :

- <https://github.com/xai-org/grok-build/blob/main/crates/codegen/xai-grok-shell/README.md#using-authjson-for-api-access>
- <https://github.com/xai-org/grok-build/blob/main/crates/codegen/xai-grok-shell/src/remote/client.rs>

Le jeton de session d’abonnement cible le proxy
`https://cli-chat-proxy.grok.com/v1`, pas `https://api.x.ai/v1`.

Le contrat observé dans le client officiel comprend au minimum :

- `Authorization: Bearer <token>` ;
- `X-XAI-Token-Auth: xai-grok-cli` ;
- une identité utilisateur obtenue par l’endpoint officiel `/user` lorsque le
  chemin l’exige ;
- un identifiant et une version client véridiques pour Beaver ;
- le modèle choisi dans l’en-tête de routage xAI ;
- le catalogue d’abonnement `/models-v2`, borné et filtré ;
- un backend par modèle, notamment `chat_completions` ou `responses`.

Beaver ne copie pas une fausse version du client officiel. Si le proxy refuse
la version Beaver, l’intégration échoue proprement et demande une mise à jour ;
elle ne se fait pas passer pour Grok Build.

## Architecture cible

### Routes séparées

| Provider Beaver | Auth | Origine d’inférence | Catalogue |
|---|---|---|---|
| `xai` | clé API | `https://api.x.ai/v1` | `/models` + registre |
| `xai-oauth` | abonnement OAuth | `https://cli-chat-proxy.grok.com/v1` | `/models-v2` |

Le code ne doit plus canoniser ces deux routes au point de partager leur
origine, leurs en-têtes ou leur logique d’erreur. Les capacités de modèle
peuvent être partagées, le transport non.

### Catalogue OAuth xAI

Un module Rust dédié :

- lit au plus 500 modèles et un body borné ;
- valide identifiant, nom, contexte, sortie, efforts et backend ;
- refuse les doublons ;
- refuse les origines autres que les domaines xAI explicitement autorisés ;
- n’expose dans Agent Local que les modèles textuels utilisables par le chat ;
- conserve Grok 4.5 si le catalogue le publie ;
- ne fabrique pas Grok 4.6 si le compte ne le reçoit pas ;
- met en cache cinq minutes et garde un dernier catalogue valide en cas de
  panne brève ;
- échoue fermé si aucune route sûre n’est disponible.

Les modèles image/vidéo purs ne doivent pas apparaître dans le sélecteur de
chat. Ils nécessitent leurs fonctionnalités dédiées.

### Identité OAuth

Après login ou refresh xAI, Beaver interroge `/user` avec un body borné et
valide `userId` et les champs optionnels. Le `userId` reste dans le vault Rust,
n’est jamais envoyé au frontend et n’est jamais journalisé.

Le refresh conserve l’identité seulement si le principal reste identique ;
sinon le login est invalidé. Toute comparaison de credential sensible utilise
le mécanisme en temps constant existant.

### Inference OAuth

Le catalogue choisit le backend. Beaver implémente les deux formes annoncées :

- Chat Completions réutilise le constructeur et le parseur SSE communs après
  injection du contrat xAI OAuth ;
- Responses utilise un adaptateur xAI dédié, sans emprunter les en-têtes ni
  l’origine Codex OAuth.

La sélection de l’origine vient d’un type interne validé, jamais d’une URL
arbitraire reçue du frontend. Les redirections authentifiées sont refusées.

### Raisonnement

Les métadonnées du registre acceptent des `reasoning_modes` et un
`default_reasoning_mode`. Le défaut doit appartenir à la liste des modes.

- GLM-5.3 : `low`, `high`, `max`, défaut `max`, aucun `off` ;
- Grok 4.6 : `low`, `medium`, `high`, `xhigh`, défaut `high` ;
- Gemini 3.7 Flash : `low`, `medium`, `high`, défaut `medium` ;
- Grok 4.5 : inchangé ;
- les valeurs dynamiques plus restrictives gagnent sur le registre ;
- une valeur inconnue est rejetée, jamais transmise telle quelle.

Pour GLM-5.3, le payload contient toujours `thinking.type: "enabled"` et un
effort valide. Pour Grok 4.6, `reasoning_effort` suit exactement le choix.
Pour Gemini 3.7 Flash, l’adaptateur OpenAI-compatible est validé par un appel
réel avant d’envoyer un champ de niveau : aucun nom de paramètre provenant de
l’API Interactions n’est transposé par supposition.

### Descriptions provider et sélecteur

Les sept fichiers `src/i18n/*.json` mettent à jour les descriptions partagées :

- Z.AI cite GLM-5.3 ;
- xAI cite Grok 4.6 sans supprimer Grok 4.5 du catalogue ;
- Google cite Gemini 3.7 Flash.

Ces textes alimentent l’onboarding et la page des clés API via la même source.
Aucun texte spécifique à l’onboarding n’est ajouté.

Le sélecteur de modèles n’affiche aucun prix pour ces nouveaux modèles. Cette
phase ne crée ni badge tarifaire, ni montant codé en dur, ni nouvelle source de
prix. Le badge `[Free]` existant reste régi uniquement par une gratuité complète
et explicitement vérifiée ; aucun des nouveaux modèles n’est marqué gratuit par
défaut.

### Erreurs et session

Une erreur sans texte n’est pas un message assistant. L’autorité reste
`AgentSession.diagnostic_runs`.

Le frontend dérive le dernier échec terminal de cette collection et l’affiche
après rechargement lorsqu’aucune réponse plus récente ne le rend obsolète. Il
utilise un code stable traduit, pas `safe_summary` comme texte produit.

Pour xAI OAuth :

- un 401 déclenche un refresh unique ;
- un second 401 demande une reconnexion ;
- un 403 signale un abonnement ou un accès indisponible ;
- un 429 `resource-exhausted` sans indication de reprise est une limite de
  quota/capacité et n’est pas réessayé trois fois à l’identique ;
- un vrai rate limit avec `Retry-After` suit une reprise bornée et annulable ;
- aucun body brut n’est affiché ou journalisé.

## Limites et sécurité

- Aucun token, refresh token, user ID ou email dans l’IPC, les logs ou les
  fixtures.
- Toutes les réponses provider sont bornées avant parsing.
- Les listes externes ont une taille maximale et refusent les doublons.
- Les URL et en-têtes issus d’un catalogue sont validés avant usage.
- Aucun en-tête réservé fourni par un appelant ne peut remplacer celui du
  transport.
- Une erreur de catalogue, d’identité ou de validation bloque le chemin OAuth.
- Les tests réseau utilisent des fixtures anonymisées et un serveur local.
- Les appels réels utilisent un prompt minimal, un seul outil déterministe et
  un budget explicitement accepté.

## Critères d’acceptation

La phase 1 est terminée seulement si :

1. `glm-5.3` est visible chez Z.AI et un tour réel renvoie du texte ;
2. ses trois efforts sont visibles, `max` est choisi par défaut et `off` est
   impossible ;
3. `gemini-3.7-flash` est visible chez Google, garde 3.5 et 3.6, et réussit un
   tour texte puis un tool call ;
4. les descriptions de l’onboarding et des réglages citent GLM-5.3, Grok 4.6
   et Gemini 3.7 Flash dans les sept langues ;
5. aucun prix n’est ajouté au sélecteur de modèles ;
6. `grok-4.6` et `grok-4.5` sont visibles par clé API quand xAI les publie ;
7. Grok 4.6 envoie `xhigh` quand il est choisi ;
8. `xai-oauth` n’envoie aucun jeton vers `api.x.ai` pour le chat ou le catalogue ;
9. Grok 4.6 OAuth réussit un tour texte, un tool call et un second tour ;
10. la route réellement appelée et le backend réellement servi sont prouvés
   sans secret dans une fixture datée ;
11. un échec sans texte reste visible après fermeture et réouverture de la
   session ;
12. aucune régression n’est observée sur Grok 4.5, Z.AI existant, xAI API et Kimi
   OAuth ;
13. tous les tests ciblés, le socle complet et les deux thèmes sont verts.
