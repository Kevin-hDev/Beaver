# Preflight documentaire — 2026-09-07

Fixture versionnée, expurgée et sans secret. Elle ne contient aucune clé, aucun token, aucune image, aucun contenu de vault et aucun body de requête authentifiée.

## Exécution

    date_utc=2026-09-07T12:13:16Z
    branch=codex/ajout-gemini-glm-astra
    head=d4ec159d983dadc7b96665b697e45036310c7e34
    base=0d58838a002b4868bbde063a8c8cbc329368853a

Depuis la base du plan : cinq commits, 113 fichiers et aucune différence sur les chemins du chantier (catalogues, LLM, compression, titres). Les changements locaux du site/maquettes et fast-mode-dark.png / fast-mode-light.png sont hors périmètre et préservés.

## Instantané OpenRouter public

Commande sans authentification :

    curl -fsSL --max-time 30 https://openrouter.ai/api/v1/models | jq '[.data[] | select(.id == "google/gemini-3.8-flash" or .id == "z-ai/glm-5.3-flash" or .id == "openai/gpt-6-astra") | {id, context_length, top_provider, reasoning, supported_parameters}]'

Capture : 2026-09-07T12:13:16Z.
SHA-256 de la réponse complète non filtrée : 851162143fa10ef95f772cf63d819d96af51f117e1b71f3f30b262109211cf0e.

Résultat JSON sélectionné :

    [
      {
        "id": "openai/gpt-6-astra",
        "context_length": 1050000,
        "top_provider": {"context_length": 1050000, "max_completion_tokens": 128000, "is_moderated": true},
        "reasoning": {"mandatory": true, "default_enabled": true, "supported_efforts": ["max", "xhigh", "high", "medium", "low"], "default_effort": "medium"},
        "supported_parameters": ["include_reasoning", "max_completion_tokens", "max_tokens", "reasoning", "reasoning_effort", "response_format", "seed", "structured_outputs", "tool_choice", "tools"]
      },
      {
        "id": "google/gemini-3.8-flash",
        "context_length": 1048576,
        "top_provider": {"context_length": 1048576, "max_completion_tokens": 65536, "is_moderated": false},
        "reasoning": {"mandatory": true, "default_enabled": true, "supported_efforts": ["high", "medium", "low"], "default_effort": "medium"},
        "supported_parameters": ["include_reasoning", "max_tokens", "reasoning", "reasoning_effort", "response_format", "seed", "stop", "structured_outputs", "temperature", "tool_choice", "tools", "top_p"]
      },
      {
        "id": "z-ai/glm-5.3-flash",
        "context_length": 1310720,
        "top_provider": {"context_length": 1048576, "max_completion_tokens": 131072, "is_moderated": false},
        "reasoning": {"mandatory": true, "default_enabled": true, "supported_efforts": ["max", "high", "low"], "default_effort": "max"},
        "supported_parameters": ["frequency_penalty", "include_reasoning", "logit_bias", "logprobs", "max_tokens", "min_p", "presence_penalty", "reasoning", "reasoning_effort", "repetition_penalty", "response_format", "seed", "stop", "structured_outputs", "temperature", "tool_choice", "tools", "top_k", "top_logprobs", "top_p"]
      }
    ]

Vérification dédiée exécutée : curl -fsSL --max-time 30 https://openrouter.ai/api/v1/models | jq '[.data[] | select(.id == "z-ai/glm-5.3-flash:free") | {id, context_length, top_provider, reasoning, supported_parameters}]'. Résultat exact : []. La variante n'est pas ajoutée comme alias stable.

## Alibaba — état initial du contrat exact

    model=ZHIPU/GLM-5.3-Flash
    region=documented China (Beijing); workspace/account=unknown
    context_length=1048576 (confirmed by exact model page)
    max_output_tokens=131072 (confirmed as capacity)
    reasoning=mandatory; thinking.type=enabled; enable_thinking=true only
    reasoning_effort=low|high|max; default=max
    clear_thinking=false preserves complete ordered reasoning_content
    tool_stream=true supported when stream=true
    preserve_thinking=not documented for this model; do not send
    output_limit_field=UNKNOWN/BLOCKED
    structured_outputs=UNKNOWN/BLOCKED (official pages conflict)
    image_plus_tools=UNKNOWN

Sources officielles :

- https://help.aliyun.com/en/model-studio/glm-5-3-flash-by-zhipu
- https://help.aliyun.com/en/model-studio/glm-zhipu
- https://help.aliyun.com/en/model-studio/qwen-api-via-openai-chat-completions
- https://help.aliyun.com/en/model-studio/qwen-api-via-dashscope

La fiche exacte annonce structured output supporté, tandis que le guide GLM générique l'annonce non supporté. La référence générale recommande max_completion_tokens pour les modèles de pensée mais exclut de sa liste les modèles directement fournis par des tiers et marque max_tokens déprécié.

## Diagnostics locaux différés

Preuve fournie par le parent, sans lecture de secrets :

    configured-providers.json: google,zai,openai,openrouter,qwen declared
    Beaver app: not running during preflight
    Ollama daemon: not running during preflight
    bundled ollama --version: 0.32.15
    src-tauri/ollama-version.txt: 0.32.1
    Qwen region/workspace: stored in vault, not read
    authenticated catalog: not loaded

Ces lignes sont des diagnostics différés, pas des preuves d'indisponibilité. Le parent doit relever /api/show, l'URL réelle du daemon et le catalogue authentifié au lancement final.

## Chemins internes vérifiés

    title on welcome: src/hooks/use-session-actions.ts:74-96 (local first 40 chars)
    title after first send: src/hooks/use-chat-actions.ts:61-64 (local first 40 chars)
    compression summary: src-tauri/src/services/compress/orchestrator_summary.rs:30-40
    clone summary: src-tauri/src/services/agent_local/clone_session.rs:136-156
    silent request config: src-tauri/src/services/llm/stream_silent.rs:141-163
    silent current state: tools=[], think=false, reasoning_mode=None

Les titres ne font aucun appel modèle. La décision low des requêtes internes reste à tester ; elle n'est pas présumée par cette fixture.

Risque runner pour la tâche 4 : run_turn ne transmet actuellement aucun plafond de sortie ; la boucle normale a ses propres retries et plusieurs appels. FixtureCandidate devra avoir une borne effective de sortie et d'appels avant tout live payant. Une limite de modèle (128000) ne vaut pas plafond de test.

Statut final du préflight : la sous-route Alibaba est la seule sous-route documentaire encore bloquée ; compte authentifié et Ollama sont différés ; OpenRouter public est confirmé ; aucune implémentation n'a commencé ici.
