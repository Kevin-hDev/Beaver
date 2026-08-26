# Task 14 report

## Implémentation livrée

- `reasoning_wire` conserve les fragments natifs Chat, Gemini, Mistral, OpenRouter, Ollama et Responses sans les aplatir, avec les fixtures anonymisées associées.
- La borne est progressive : le premier dépassement libère l'état natif, marque la capture `partial` avec le seul code fermé `capture_limit_exceeded`, et interdit ensuite toute enveloppe complète.
- Ollama observe chaque objet JSON unique avant l'affichage et ne finalise que sur `done: true`.
- Chat Completions reçoit le contexte canonique admis, observe la valeur JSON unique avant le parser visible et ne finalise qu'après un `finish_reason` valide.
- `StreamResult` porte désormais l'enveloppe privée séparée, transmise aux messages assistants tout en gardant le texte de réflexion visible distinct.

## Limite de cette passe

Le flux Responses/Codex possède les fixtures et l'adaptateur `response.completed`, mais son accumulateur ne reçoit pas encore le contexte de provenance canonique. Il reste donc fail-closed : aucune enveloppe n'est attachée sans cette provenance, et aucun rejeu multi-tour n'est ouvert.

## Validation exécutée

- `cargo test reasoning_wire --lib` : 3/3 vert.
- `cargo test stream_chunk --lib` : 15/15 vert.
- `cargo test codex_client --lib` : 86/86 vert.
- `cargo fmt --all` et `git diff --check` : verts.
- `graphify update .` lancé après les modifications.

Les commandes Cargo ont utilisé l'override CEF local fourni par le parent ; aucun appel provider ni réseau n'a été effectué.
