# Revue différentielle — registre local des modèles fournisseurs

## Executive Summary

| Sévérité non résolue | Nombre |
|---|---:|
| Critique | 0 |
| Haute | 0 |
| Moyenne | 0 |
| Basse | 0 |

**Risque global après correction :** faible
**Recommandation :** APPROUVER

**Indicateurs :**

- Référence : `3245dc9..worktree`
- Fichiers modifiés ou ajoutés dans le périmètre : 63
- Diff total, nouveaux fichiers inclus : environ +2 479 / -1 281 lignes
- Parcours à fort impact relus : catalogue UI, sélection de modèle, capacités, contexte, plafond de sortie, requêtes streaming et non-streaming
- Régression de sécurité restante détectée : 0

## What Changed

Le registre LiteLLM embarqué reste le catalogue de consultation de
`Settings/LLM` et le dernier repli. Les modèles utilisés par les fournisseurs
Beaver ont maintenant un registre local séparé, avec un fichier JSON par
fournisseur.

| Zone | Changement | Risque initial | État |
|---|---|---:|---|
| `resources/provider-models/*.json` | Inventaires, limites, capacités, sources et alias officiels | Haut | Corrigé et testé |
| `provider_model_registry*` | Chargement borné, validation stricte et résolution des alias | Haut | Corrigé et testé |
| `provider_model_lookup*` | Priorité Beaver → fournisseur → LiteLLM | Haut | Corrigé et testé |
| `openai_compat_*` | Liste dynamique, validation des données et paramètres de sortie | Haut | Corrigé et testé |
| `stream_*` | Limites et paramètres propres à chaque famille de modèles | Haut | Corrigé et testé |
| `litellm_catalog*` | Conservation du catalogue Settings et du repli | Moyen | Préservé et testé |

La revue porte sur l’état de travail non encore commité au-dessus de
`3245dc9`, le 30 juillet 2026.

## Critical Findings

Aucun problème critique ou haut ne reste ouvert.

Les problèmes suivants ont été trouvés pendant la revue puis corrigés :

1. Des noms inventés, retirés ou mal formés existaient dans plusieurs
   inventaires, notamment Mistral, Cerebras, OpenAI, Moonshot et Z.ai.
2. Une réponse `/models` pouvait encore écraser la configuration locale, alors
   que le registre Beaver doit être prioritaire.
3. `o3`, `o4` et les familles GPT-5 pouvaient recevoir `max_tokens` au lieu de
   `max_completion_tokens`.
4. Kimi K3 recevait une structure `thinking` incorrecte au lieu de
   `reasoning_effort`, et son plafond utilisait le mauvais champ.
5. Les données distantes de catalogue acceptaient encore des identifiants et
   métadonnées insuffisamment validés.
6. xAI restait figé sur une liste statique alors que son endpoint officiel
   `/v1/models` est disponible.

Chaque correction possède maintenant un test de non-régression ciblé.

## Test Coverage Analysis

Les changements à fort risque sont couverts par des tests sur :

- l’inventaire exact et l’absence des identifiants retirés ou inventés ;
- les limites et capacités corrigées ;
- les alias sans duplication dans la liste visible ;
- les collections bornées, les sources HTTPS et les dates valides ;
- la priorité du registre local sur les métadonnées distantes et LiteLLM ;
- les champs `max_tokens` et `max_completion_tokens` en streaming et hors
  streaming ;
- les formes de raisonnement Moonshot, OpenAI, Mistral et xAI ;
- le filtrage des identifiants et métadonnées reçus des fournisseurs ;
- la conservation du catalogue LiteLLM dans Settings.

Validations exécutées :

| Validation | Résultat |
|---|---:|
| `npm run lint` | Réussi |
| `npx tsc --noEmit` | Réussi |
| `npm test -- --run` | 1 661 + 2 + 46 tests réussis |
| `cargo fmt --check` | Réussi |
| `cargo check` | Réussi |
| `cargo clippy --all-targets -- -D warnings` | Réussi |
| `cargo test` | 2 546 tests réussis |

## Blast Radius Analysis

| Fonctionnement touché | Consommateurs | Risque après tests |
|---|---|---:|
| Chargement du registre local | Sélecteurs, capacités, contexte, génération | Faible |
| Résolution des limites | Chat manuel, agent local, compression | Faible |
| Résolution des capacités | Outils, vision, raisonnement | Faible |
| Construction des requêtes | Streaming et appels simples | Faible |
| Catalogue LiteLLM | Settings, prix, repli des modèles inconnus | Faible |

Le point le plus sensible était l’ordre de priorité des données. Il est
maintenant identique sur les listes, les capacités, le contexte et les plafonds :
registre Beaver d’abord, réponse du fournisseur ensuite, LiteLLM en dernier.

## Historical Context

L’ancien module `model_registry` mélangeait deux responsabilités :

- le catalogue LiteLLM visible dans Settings ;
- les réglages opérationnels des fournisseurs Beaver.

La séparation conserve la première responsabilité sous le nom
`litellm_catalog` et introduit `provider_model_registry` pour la seconde. Aucun
retrait de validation de sécurité historique n’a été détecté.

Le champ de statut des modèles n’a pas été ajouté : il ne sert ni au sélecteur
ni à l’exécution. Seuls les modèles encore proposés sont décrits localement, et
la disponibilité visible reste déterminée par l’endpoint officiel du
fournisseur quand il existe.

## Recommendations

### Immediate

Aucune action bloquante restante.

### Before Production

- Effectuer un test réel de `GET /models` avec une clé xAI et une connexion
  Kimi Code, si des identifiants de test sont disponibles.
- Garder `verified_at` et les URL officielles à jour lors de chaque modification
  d’un fichier fournisseur.

### Technical Debt

- Aucun refactoring supplémentaire n’est nécessaire dans ce périmètre.

## Analysis Methodology

**Stratégie :** revue différentielle approfondie et adversariale.

**Périmètre :**

- 63 fichiers modifiés ou ajoutés examinés par diff ;
- 100 % des chemins à risque élevé relus directement ;
- consommateurs amont et aval contrôlés jusqu’à la construction HTTP ;
- tests complets frontend et backend exécutés.

**Techniques :**

- comparaison avec `3245dc9` ;
- traçage des consommateurs du registre ;
- vérification des bornes et des erreurs fermées ;
- scénarios de métadonnées distantes contradictoires ou malveillantes ;
- contrôle des paramètres réellement envoyés aux API ;
- recoupement avec les documentations officielles.

**Limites :**

- aucun appel authentifié réel n’a été effectué vers les comptes fournisseurs ;
- la revue valide les contrats documentés, le parseur et les tests locaux.

**Confiance :** élevée sur le code et les régressions locales, élevée sur les
données vérifiées publiquement, moyenne à élevée sur les endpoints nécessitant
une clé réelle.

## Appendices

Sources principales :

- [Groq Models](https://console.groq.com/docs/models)
- [Google Gemini Models](https://ai.google.dev/gemini-api/docs/models)
- [Mistral Models](https://docs.mistral.ai/models/overview)
- [Cerebras Public Models](https://inference-docs.cerebras.ai/api-reference/models/public-models)
- [OpenAI Models](https://developers.openai.com/api/docs/models)
- [DeepSeek Pricing and Limits](https://api-docs.deepseek.com/quick_start/pricing)
- [xAI Models](https://docs.x.ai/developers/models)
- [Kimi Models](https://platform.kimi.com/docs/models.md)
- [Z.ai Models](https://docs.z.ai/guides/overview/overview)
- [OpenRouter Models](https://openrouter.ai/docs/api-reference/list-available-models)

La carte Graphify a été mise à jour après les changements de code :
22 648 nœuds et 47 678 relations.
