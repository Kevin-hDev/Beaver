# Task 23 — raccordement Responses

## Livré

- La `ContinuationTarget` canonique traverse désormais l'admission API, la boucle, le retry et les constructeurs OpenAI Responses, xAI OAuth Responses et Codex OAuth.
- La cible conserve sa nature `Replay` ou `FixtureCandidate`; cette dernière reste compilée uniquement en debug.
- Le contrat `continuation_use` est recalculé à chaque requête depuis le dernier message : `ToolContinuation` après un résultat d'outil, sinon `UserContinuation`.
- Les items Responses sont réinjectés au point exact du message assistant, avant les `function_call_output`, sans stockage durable dans `extra_content`.
- Le garde fail-closed bloque avant réseau une route `Required` qui possède un assistant antérieur sans enveloppe valide; scope, modèle, mode, route, contrat, état partial et type de continuation sont validés par la politique existante.
- Aucun transport public `xai` Responses n'a été ajouté et aucune activation du registre n'a été modifiée.

## Vérifications exécutées

- `cargo test openai_responses --lib` : 6 verts.
- `cargo test xai_oauth_transport --lib` : 6 verts.
- `cargo test responses_reasoning --lib` : 1 vert.
- `cargo test codex_client --lib` : 90 verts.
- `cargo check -q` : vert, avec avertissements préexistants de code non utilisé et un avertissement linker macOS compact-unwind.
- `graphify update .` exécuté.
- `git diff --check` : vert.
- Recherche `codex.output_items|extra_content.*codex` dans `codex_client` : aucune occurrence.

## Risques restants

- Les activations live du registre restent volontairement désactivées : ce raccordement est donc couvert par fixtures debug, pas par une preuve réseau/live.
- La vérification complète des bornes Codex 128/129 et 8 MiB reste portée par les limites et captures existantes; cette tâche ajoute surtout le rejet de scope et l'ordre du payload réel.
