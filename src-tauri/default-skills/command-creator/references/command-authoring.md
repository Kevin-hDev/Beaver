# Command Authoring

You apply this contract to each generated slash-invokable command bundle.

## Identity and triggering

- You use a lowercase kebab-case name no longer than 64 characters and make the folder, frontmatter `name`, slash slug, and evaluation identifier identical.
- You write one single-line description no longer than 250 characters that states the output, concrete invocations, and nearby exclusions.
- You compare positive and negative cases with neighboring descriptions and keep distinct operations separate when their method or output differs.

## Body

- You state one objective and express it in no more than eight direct second-person imperative steps.
- You read the remaining user request after the slash token as input and never reference `$ARGUMENTS`, `$0`, `$1`, template interpolation, or another CLI's syntax.
- You validate required input before acting and ask one minimal question when a missing value changes the result.
- You define the observable output, authorized side effects, failure behavior, and checks without adding a role section or a multi-action router.
- You add a reference, asset, or script only when the operation cannot remain correct and concise without it; otherwise you keep the bundle flat.

## Evaluations and safety

- You include at least three trigger cases, two close non-trigger cases, missing or invalid input, tool failure, no-change behavior, and an observable-result judge.
- You validate paths, bound external collections, protect secrets, use argument arrays for system execution, and fail closed.
- You preserve unrelated content, avoid unrequested external effects, and report only checks and changes that actually occurred.
- You treat the result as a skill exposed in the slash menu, not as a built-in application command or a real external tool connection.
