# Destination Resolution

You resolve rule destinations from project evidence before writing.

## Evidence order

1. You follow explicit user instructions for this request unless they conflict with safety or a higher-authority project policy.
2. You follow project documentation that declares the canonical rule or instruction location and its mirroring requirements.
3. You follow active existing rule files, templates, generators, validation configuration, and repository scripts.
4. You treat directory names or isolated files without supporting evidence as candidates, not proof.

## Convention record

You record these fields for every discovered surface:

- You name its path pattern and project evidence.
- You classify it as canonical, mirror, independent scope, legacy, generated, or ambiguous.
- You record its filename, extension, grouping, frontmatter, scope, precedence, links, and validation convention.
- You state whether the current rule belongs there and why.
- You state whether changes should be direct or produced by an existing project generator.

## Destination gate

- You ask the user to choose when no convention exists or evidence conflicts.
- You offer concrete project-local candidates and their tradeoffs without presenting a guess as a default.
- You require explicit confirmation before creating a new directory, new format, canonical owner, or mirror relationship.
- You preserve an existing generator relationship and modify its source rather than generated outputs when project evidence says outputs are generated.
- You include every confirmed required mirror in the intended target set.
- You name an unsupported or unverifiable target and report what evidence is missing instead of silently skipping it.

## Path safety

- You accept only paths resolved inside the project.
- You reject traversal components, unresolved external links, special files, and destinations whose canonical parent escapes the project.
- You create missing parent directories only when the user explicitly confirmed that exact destination.
