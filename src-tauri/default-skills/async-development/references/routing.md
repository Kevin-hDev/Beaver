# Router contract

You evaluate these signals in order and commit to exactly one sub-flow.

## Signal precedence

1. You accept a case-insensitive argument whose entire normalized value is exactly `setup`, `run`, or `review`, or an exact standalone field `action=setup`, `action=run`, or `action=review`. You do not match those words when they merely occur inside unrelated prose.
2. You accept a normalized event only from a configured adapter whose event schema you can inspect:
   - You route a configured ready-state event or configured implementation command to `run`.
   - You route a configured review-state event or configured review command to `review`.
   - You map a trusted manual-dispatch action field to its exact named sub-flow.
   - You reject an unknown event kind, state, command, or malformed payload instead of guessing.
3. You observe repository state:
   - You select `setup` when the declared configuration is missing or incomplete and setup is requested.
   - You select `review` when a referenced ticket has an open linked change request and correction work is requested.
   - You select `run` when a referenced ready ticket has no linked open change request and implementation is requested.
4. You use natural-language intent only when the preceding signals do not resolve the route.
   - You map requests to install, configure, set up, bootstrap, replace, or reconfigure the pipeline to `setup`.
   - You map requests to implement, process, or run one ready ticket or one queue item to `run`.
   - You map requests to address review, apply feedback, fix comments, or iterate on an open change request to `review`.
   - You require the requested object and observed state to agree before handoff.

Within one signal class, you prefer an exact change-request identity or verified ticket-to-change-request relation, then a configured lifecycle state, then a configured command, then free text, then configuration absence. You record the winning signal and the contradictory signals you rejected.

## Conflicts

- You stop when `run` is requested but required configuration or adapters are absent.
- You stop when `review` is requested but no open change request can be identified.
- You stop when `setup` would overwrite an existing integration without explicit replacement authority.
- You stop when ready and review states coexist on one item until the user or an authorized deterministic rule resolves them.

You may use one configured deterministic rule: when a trusted ready event targets a ticket that already has an exact open linked change request, you treat that change request as the active surface and select `review`. You post an explanatory tracker comment only when the effect contract authorizes and verifies that exact comment. Without the verified relation or configured rule, you stop.

## Unresolved selection

You present exactly these choices and ask for one when no signal resolves the route:

- You describe `setup` as installing or configuring the asynchronous pipeline.
- You describe `run` as implementing one ready ticket with no open linked change request.
- You describe `review` as addressing feedback on one open change request.

You never proceed until one choice becomes explicit and consistent with observed state.

You never comment, relabel, or otherwise explain a routing conflict on an external system unless the current effect contract explicitly authorizes that exact effect.

## Handoff

You invoke the first action of the selected sub-flow. You run that sub-flow to completion or a recorded stop condition. You never route again in the same invocation, and you never switch sub-flows to recover from failure.
