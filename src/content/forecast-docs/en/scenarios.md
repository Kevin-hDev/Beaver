# Scenarios

A scenario is used to test a hypothesis about the future. It compares the baseline forecast with a modified trajectory.

## Why create a scenario

The baseline forecast answers: "what does the model predict with the current data?"

A scenario answers: "what happens if the context changes?"

Examples:

- what happens to sales if a promotion is launched?
- what happens to revenue if traffic drops?
- what happens to an asset if the VIX rises?
- what happens to demand if a local event is added?

The scenario turns Forecast into a simulation tool, not just a forecast graph.

## Global adjustment

The global adjustment applies a simple variation to the curve.

Example:

```text
Baseline forecast: 100, 110, 120
Scenario +10%   : 110, 121, 132
```

This mode is fast and readable. It does not rerun the model, so it does not understand the relationships between variables.

## Contextual scenario

The contextual scenario modifies the future variables, then reruns the model.

Example:

```text
Hypothesis: vix_close +20%
Expected effect: the model recomputes the target with this more stressed market context.
```

This mode is more important for Chronos-2 and TimeGPT, because it uses covariates as real prediction signals.

## Modifiable variables

The available variables depend on the dataset.

They can represent:

- environment: weather, traffic, events;
- finance: volatility, rates, indices, news score;
- calendar: weekend, public holiday, month-end;
- business: promo, stock, budget, campaign;
- risk: alert, incident, competitive pressure.

Each modification must make business sense. Modifying a variable at random produces a scenario that is hard to interpret.

## Reading in the graph

When a scenario is selected, the graph must allow comparison of:

- actual history;
- baseline forecast;
- scenario forecast;
- displayed contextual variables;
- difference between baseline and scenario.

Typical diagram of a comparison:

```text
value
  ^
  |              ╭───── scenario (VIX +20%)
  |           ╭──╯
  |       · ·─·      ← baseline forecast
  |     ·
  |   ·
  | ·
  ──────────────────────────────> time
       history       │   future
                     │
                horizon
```

The main question is not "is the curve different?", but "which hypothesis moved the trajectory, on which date, and by how much?"

## Proper use

A good scenario must be clearly named.

Examples:

- `VIX +20% for 30 days`
- `Active weekend promo`
- `Heavy rain week 2`
- `Traffic -15% after incident`

A vague name like `test` or `crash` makes comparison pointless when several scenarios exist.
