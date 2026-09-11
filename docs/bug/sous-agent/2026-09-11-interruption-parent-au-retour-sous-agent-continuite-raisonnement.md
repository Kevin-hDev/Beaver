# Interruption du parent au retour d'un sous-agent par continuité de raisonnement

- Date de l'investigation : 11 septembre 2026
- Statut : cause racine confirmée, correction non appliquée
- Session témoin : `2b92ba1e-1622-4860-bf06-838fe93b44f3`

## Résultat

L'interruption n'est pas propre à Codex. Beaver injecte actuellement le rapport d'un sous-agent comme un message `assistant` sans état natif de continuité. Les fournisseurs stricts interprètent alors ce rapport comme une ancienne réponse du modèle dont le raisonnement aurait été perdu et Beaver bloque la requête avant son envoi.

Le rapport est une donnée produite par Beaver et destinée au modèle parent. Il doit avoir le rôle `user`. Les véritables réponses produites par le fournisseur restent les seules à avoir le rôle `assistant` ou `model` et à porter leur raisonnement, leurs signatures ou leurs éléments opaques.

Cette correction doit être faite dans le constructeur partagé du rapport. Un traitement particulier pour Codex ou pour chaque fournisseur dupliquerait la même règle et laisserait les autres routes exposées.

## Symptôme observé

L'interface a affiché :

> La continuité du raisonnement n'est pas compatible. Change de modèle ou démarre une nouvelle conversation.
>
> Interruption après le dernier tool get_subagent (unknown).

La réponse du parent s'est arrêtée après le retour du premier sous-agent. Le second sous-agent a terminé plus tard, mais le parent était déjà interrompu.

## Preuves de la session

La session enregistrée contient les éléments suivants :

| Élément | Valeur observée |
| --- | --- |
| Fournisseur | `codex-oauth` |
| Modèle | `gpt-5.6-luna` |
| Raisonnement | `medium` |
| Préservation demandée par l'utilisateur | `off` |
| Messages persistés | 7 |
| Tokens accumulés | 914 |
| Nombre de compressions | 0 |
| Début du run | `2026-09-11T12:02:27.307856Z` |
| Échec | `2026-09-11T12:03:55.082236Z` |
| Code brut de l'échec | `reasoning_continuity_invalid` |
| Code persisté | `stream_error` |
| Résumé visible | `Interruption après le dernier tool get_subagent (unknown).` |

Chronologie utile :

- `12:02:45.918918Z` : `get_subagent` termine avec succès et indique que le premier enfant interrogé travaille encore.
- `12:03:54.984078Z` : le sous-agent `eb686903-6001-49f5-ba6b-291007a26ccb` termine.
- `12:03:55.006573Z` : son rapport caché est persisté avec `delivered: false`.
- `12:03:55.028810Z` : Beaver prépare le quatrième appel du parent.
- `12:03:55.046502Z` : le diagnostic du payload indique `items=0`.
- `12:03:55.082236Z` : la préparation échoue avec `reasoning_continuity_invalid`.
- `12:04:23.938215Z` : le second sous-agent `e7213304-7d55-4118-9ffd-78dfc1ebe485` termine après l'arrêt du parent.

Le payload vide et l'intervalle de 53 millisecondes entre le début du quatrième tour et l'échec montrent que la requête n'a pas atteint Codex. Le blocage vient de la validation locale de Beaver.

Les deux rapports restent `delivered: false`. Tant que leur rôle n'est pas corrigé, une nouvelle tentative peut reproduire le même blocage.

## Cause racine dans Beaver

Tous les retours de sous-agents passent par :

```text
subagent_report_context::append_context
  -> report_batch_to_message
  -> ChatMessage::assistant(..., continuation=None)
```

Le constructeur partagé se trouve dans `src-tauri/src/services/agent_local/subagent_report_context.rs`. La ligne fautive est :

```rust
ChatMessage::assistant(report_batch_content(reports), None, None, None, None)
```

Le test `report_context_is_assistant_and_xml_escaped` exige actuellement ce mauvais rôle. Il protège donc le défaut au lieu de le détecter.

La contradiction est visible après rechargement : `conversation_history_build::context_message` reconstruit déjà ce contexte durable avec `ProviderRole::User`. Une même conversation n'a donc pas le même sens selon que le rapport est injecté en direct ou relu depuis le disque.

`git blame` rattache le passage au rôle `assistant` au commit `1836a968` du 27 août 2026, intitulé `feat(reasoning): preserve native thinking across turns`. Cela situe l'introduction de la régression ; la cause fonctionnelle reste le rôle incorrect du rapport.

## Ce que les fournisseurs attendent

Les contrats diffèrent dans leur format, mais partagent la même séparation :

- une sortie réellement produite par le modèle garde son rôle `assistant` ou `model` ;
- l'état natif de raisonnement reste attaché à cette sortie et doit être renvoyé intact lorsque le fournisseur l'exige ;
- le résultat d'un outil utilise le rôle ou l'objet prévu par le protocole ;
- une information nouvelle produite par l'application est une entrée du modèle et ne doit pas être présentée comme une ancienne sortie du modèle.

Exemples officiels vérifiés le 11 septembre 2026 :

- [OpenAI Responses](https://platform.openai.com/docs/api-reference/responses-streaming?lang=python) demande de renvoyer les éléments de raisonnement des sorties précédentes lorsque l'application gère elle-même le contexte.
- [Anthropic Extended Thinking](https://platform.claude.com/docs/en/about-claude/models/extended-thinking-models) demande de conserver les blocs de réflexion complets et inchangés avec la réponse `assistant` qui a lancé l'outil.
- [Google Gemini Thought Signatures](https://ai.google.dev/gemini-api/docs/generate-content/thought-signatures) demande de renvoyer les signatures exactement ; les réponses du modèle ont le rôle `model` et les réponses de fonction le rôle `user`.
- [OpenRouter Reasoning Tokens](https://openrouter.ai/docs/guides/best-practices/reasoning-tokens) demande de renvoyer `reasoning_details` sans modification sur le message `assistant` correspondant.
- [DeepSeek Thinking Mode](https://api-docs.deepseek.com/guides/thinking_mode/) demande de conserver `reasoning_content` dans les chaînes d'outils.
- [Qwen Deep Thinking](https://www.alibabacloud.com/help/en/model-studio/deep-thinking) précise que le raisonnement historique préservé appartient aux anciens messages `assistant` et compte comme entrée au tour suivant.
- [Kimi K2.7 Code](https://huggingface.co/moonshotai/Kimi-K2.7-Code) impose `preserve_thinking` et conserve le raisonnement sur les anciens messages `assistant`.
- [Mistral Function Calling](https://docs.mistral.ai/studio/conversations/function-calling) ajoute la réponse du modèle à l'historique avant le résultat d'outil.
- [xAI Reasoning](https://docs.x.ai/developers/model-capabilities/text/reasoning) permet de renvoyer le raisonnement chiffré des sorties précédentes afin de poursuivre le contexte.
- [Cerebras Chat Completions](https://inference-docs.cerebras.ai/api-reference/chat-completions) définit le rôle `assistant` comme les messages envoyés par le modèle en réponse à l'utilisateur.
- [Z.AI Thinking Mode](https://docs.z.ai/guides/capabilities/thinking-mode) demande de renvoyer le `reasoning_content` historique complet, inchangé et dans son ordre original lorsque sa préservation est active.
- [Ollama Chat](https://docs.ollama.com/api/chat) utilise `assistant` pour la réponse du modèle et expose sa réflexion dans `message.thinking`.

Aucune de ces règles ne justifie de fabriquer une continuité pour un rapport de sous-agent. Beaver ne peut pas signer ou inventer un état natif à la place du fournisseur.

## Impact par fournisseur et modèle actif

La matrice suivante décrit le comportement actuellement enregistré comme `LiveValidated` dans Beaver.

| Route | Modèle et mode | Continuité | Conséquence actuelle du rapport `assistant` |
| --- | --- | --- | --- |
| Codex OAuth | `gpt-5.6-luna`, Medium | Obligatoire | Blocage local avant le réseau |
| OpenAI API | `gpt-5.6-luna`, Medium | Obligatoire | Même blocage |
| xAI API | `grok-4.6`, High | Obligatoire | Même blocage |
| Compte xAI | `grok-4.6`, High | Obligatoire | Même blocage |
| Anthropic | `claude-haiku-4-5-20251001`, Low/Medium/High | Obligatoire | Même blocage |
| Google | `gemini-3.5-flash`, Medium | Obligatoire | Même blocage |
| Cerebras | `gpt-oss-120b`, High | Obligatoire | Même blocage |
| Mistral | `mistral-small-2603`, High | Obligatoire | Même blocage |
| DeepSeek | `deepseek-v4-flash`, Low/High/Max | Obligatoire | Même blocage |
| Qwen | `qwen3.8-flash`, Low/Medium/Xhigh | Obligatoire | Même blocage |
| Moonshot API | `kimi-k2.7-code`, Auto | Obligatoire | Même blocage |
| OpenRouter | `moonshotai/kimi-k2.5`, Auto/Medium | Obligatoire | Même blocage |
| Ollama | `gemma4:e2b-it-q4_K_M`, Auto | Optionnelle | Pas ce blocage ; le rôle reste faux |
| Ollama | `qwen3.5:4b`, Auto | Optionnelle | Pas ce blocage ; le rôle reste faux |
| Z.AI API | `glm-4.5-flash`, Auto | Optionnelle | Pas ce blocage ; le rôle reste faux |

Les routes obligatoires de type Responses rejettent tout message `assistant` sans enveloppe. Les routes Chat et Anthropic font la même vérification après leur barrière de continuité. L'adaptateur Ollama ne parcourt que les messages qui possèdent réellement une enveloppe. La route Z.AI active tolère une enveloppe absente parce que sa politique actuelle est optionnelle.

## Cas Kimi et ZCode

Kimi possède plusieurs routes distinctes :

| Connexion | État actuel |
| --- | --- |
| API Moonshot, `kimi-k2.7-code` | Active et exposée au blocage |
| OpenRouter, `moonshotai/kimi-k2.5` | Active et exposée au blocage |
| Compte Kimi Code, `moonshot-oauth` / `kimi-for-coding` | Route présente, mais rejeu natif encore désactivé ; pas ce blocage local aujourd'hui |

`ZCode` n'est pas une route de modèle dans Beaver. C'est une source d'import des règles et skills depuis `~/.zcode`. Les appels de modèles apparentés passent par :

- Z.AI API pour les modèles GLM ;
- Ollama pour un modèle local ou un modèle Cloud exposé par Ollama.

Dans l'inventaire actuel, `glm-4.5-flash` via Z.AI est le seul couple GLM en rejeu `LiveValidated`, avec une continuité optionnelle. Les couples `glm-5.3`, `glm-5.3-flash` et `glm-5.3-flash:cloud` restent désactivés pour le rejeu natif. Ils ne déclenchent donc pas aujourd'hui cette validation précise, mais le rôle `assistant` du rapport reste sémantiquement faux et deviendrait bloquant dès l'activation d'une politique obligatoire.

## Compression et compteur de contexte

La compression n'a pas participé à l'incident :

- `compression_count` vaut `0` ;
- la session ne contient que 7 messages persistés ;
- `accumulated_tokens` vaut 914 ;
- l'échec survient pendant la construction du payload de continuité.

Le test `fitting_subagent_report_survives_saturated_context_intact` réussit et confirme qu'un rapport qui tient dans le budget reste intact lorsque le contexte est saturé.

Changer seulement le rôle conserve le préfixe `Subagent report context:`, le contenu XML, la politique de sécurité et la priorité appliquée par le budget de contexte. La compression ne doit donc pas recevoir de règle spéciale supplémentaire.

La non-régression attendue est : après sélection ou réduction du contexte, le rapport est toujours présent, intact et avec le rôle `user`.

## Défaut secondaire de diagnostic

`stream_diagnostics_failure::classify_error` ne reconnaît pas `reasoning_continuity_invalid`. Le code tombe alors sur `stream_error`, tandis que le résumé du run garde `unknown` et mentionne le dernier outil réussi.

Cela produit le message trompeur `Interruption après le dernier tool get_subagent (unknown)`. `get_subagent` n'a pas échoué : le diagnostic enregistré confirme son succès.

La correction doit ajouter `reasoning_continuity_invalid` aux codes sûrs reconnus afin que l'interface indique la vraie phase et la vraie cause.

## Correction recommandée

La correction minimale à la cause racine est :

```rust
ChatMessage::user(report_batch_content(reports))
```

Elle doit remplacer la construction `ChatMessage::assistant(...)` dans `report_batch_to_message`.

Cette modification aligne :

- l'injection en direct ;
- la reconstruction depuis le stockage, déjà en `ProviderRole::User` ;
- la signification attendue par tous les fournisseurs ;
- la politique système existante qui traite le rapport comme une preuve non fiable.

Il ne faut pas :

- fabriquer une enveloppe de raisonnement pour le rapport ;
- ignorer les messages commençant par un préfixe dans chaque validateur fournisseur ;
- relâcher globalement les contrôles de continuité ;
- créer un correctif propre à Codex.

Ces alternatives masqueraient le mauvais rôle et affaibliraient la protection qui empêche de renvoyer un historique natif incomplet ou falsifié.

## Tests nécessaires avant de déclarer le correctif terminé

- Le constructeur du rapport produit un message `user`, sans continuité, et conserve l'échappement XML.
- Un rapport injecté traverse chaque couple `LiveValidated` sans erreur de continuité. Le test doit parcourir le registre plutôt que recopier une liste de fournisseurs.
- Le scénario complet « deux enfants, le premier termine, le parent reprend, le second travaille encore » atteint bien le prochain appel modèle.
- Une annulation ou un échec avant succès garde les rapports en `delivered: false`.
- Une réponse modèle réellement privée de son enveloppe continue d'être bloquée sur les routes obligatoires.
- Ollama et Z.AI continuent de construire leurs payloads sans régression.
- Le rapport survit intact à la réduction du contexte avec le rôle `user`.
- `reasoning_continuity_invalid` est persisté et affiché comme tel, sans désigner `get_subagent` comme cause.

## Vérifications exécutées pendant l'investigation

Les commandes ciblées suivantes ont chacune exécuté un test réel et réussi :

```text
cargo test codex_payload_replays_items_before_current_user_without_legacy_tool_storage --lib -- --nocapture
cargo test report_policy_and_body_match_api_and_ollama_payloads --lib -- --nocapture
cargo test report_context_is_assistant_and_xml_escaped --lib -- --nocapture
cargo test responses_continuity_blocks_wrong_scope_and_required_missing_state --lib -- --nocapture
cargo test only_live_validated_ollama_models_replay_in_production --lib -- --nocapture
cargo test optional_replay_does_not_create_fallback_for_missing_envelope --lib -- --nocapture
cargo test only_exact_live_fixture_pairs_are_activated --lib -- --nocapture
cargo test fitting_subagent_report_survives_saturated_context_intact --lib -- --nocapture
```

Le test qui attend explicitement le rôle `assistant` est vert parce qu'il encode le comportement fautif. Son succès confirme l'état actuel du code, pas sa justesse.

Les premières tentatives utilisant un filtre `--exact` incomplet avaient exécuté zéro test. Elles ne sont pas retenues comme preuve.

## Autorités utilisées

- Session persistée : `~/.local/share/cl-go-dash/agent-sessions/2b92ba1e-1622-4860-bf06-838fe93b44f3.json`
- Construction du rapport : `src-tauri/src/services/agent_local/subagent_report_context.rs`
- Reconstruction durable : `src-tauri/src/services/agent_local/conversation_history_build.rs`
- Validation Responses : `src-tauri/src/services/llm/reasoning_wire/responses.rs`
- Validation Chat : `src-tauri/src/services/llm/reasoning_wire/chat_text.rs`
- Validation Anthropic : `src-tauri/src/services/llm/reasoning_wire/replay_apply_anthropic.rs`
- Adaptateur Ollama : `src-tauri/src/services/agent_local/ollama_wire.rs`
- Inventaire actif : `src-tauri/src/services/reasoning_continuity/registry_tests.rs`
- Classification des erreurs : `src-tauri/src/services/agent_local/stream_diagnostics_failure.rs`
