# UI Sketches

Use text sketches to settle structure, not appearance.

## Drawing rules

- You draw one bordered box per screen or materially different state.
- You place major regions and key controls where they belong.
- You number every region and explain it in one short line below the sketch.
- You show empty, loading, error, and narrow layouts only when they change structure or the confirmed journey.
- You preserve the project's existing navigation and component patterns unless the source explicitly changes them.

## Example

```text
┌─────────────────────────────────────┐
│ (1) Header                          │
├──────────────┬──────────────────────┤
│ (2) Filters  │ (3) Results          │
│              │ ┌──────────────────┐ │
│              │ │ (4) Result item  │ │
│              │ └──────────────────┘ │
└──────────────┴──────────────────────┘
```

1. Header: keep the existing page identity and primary action.
2. Filters: expose only confirmed ways to narrow results.
3. Results: hold the main content and its empty state.
4. Result item: show the information required by the confirmed journey.

Do not use a sketch to choose colors, typography, animation, final wording, or component libraries.
