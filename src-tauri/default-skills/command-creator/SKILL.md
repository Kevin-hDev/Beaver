---
name: command-creator
description: Creates or refactors compact single-objective skill bundles invoked from the slash menu with inline request text. Use for reusable one-shot operations. Not for multi-action workflows, rules, CLI/API integrations, or built-in app commands.
---

# Command Creator

You turn one repeatable, explicitly invoked operation into a compact standard skill bundle that appears in the slash menu and consumes the remaining user request as its input.

## Actions

You run these actions in order and read each action file before executing it.

| Action | Use it when | Output |
| --- | --- | --- |
| [`01-capture`](actions/01-capture.md) | You receive a request for a reusable one-shot operation | Confirmed command contract and destination |
| [`02-write`](actions/02-write.md) | The single objective and write boundary are confirmed | Compact slash-invokable skill bundle |
| [`03-validate`](actions/03-validate.md) | A command bundle was written or supplied | Structural and behavioral verdict |

## Rules

- You create a standard skill bundle because slash-invokable user operations use the skill catalog; you do not invent a separate command-file format.
- You keep one objective and no more than eight direct execution steps. You use a workflow skill when distinct jobs, branches, or supporting actions are necessary.
- You treat the user text remaining after the slash token as the operation input and never emit `$ARGUMENTS`, positional placeholders, or another CLI's command syntax.
- You require the user to confirm the slug, objective, input contract, output contract, exclusions, destination, and overwrite boundary before writing.
- You preserve unrelated content during refactors and keep an existing command bundle in place unless the user explicitly requests a move.
- You validate paths, links, metadata, trigger discrimination, and observable behavior before reporting success.
- You report `passed`, `failed`, `blocked`, or `skipped` and never claim a built-in command or backend integration was created.

## Resources

- Read [destination-resolution.md](references/destination-resolution.md) before confirming a target.
- Read [command-authoring.md](references/command-authoring.md) before drafting or refactoring a command bundle.
- Copy [command-skill-template.md](assets/command-skill-template.md) only after the contract is confirmed.
