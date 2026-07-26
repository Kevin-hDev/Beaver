# Forecasts

A forecast is the result computed by Forecast to estimate the next values of a target. It answers a concrete question: "if the past data and the known context stay consistent, what values can we expect next?"

## What a forecast represents

A forecast contains a sequence of future points.

Each point corresponds to a date or a period:

```text
2026-06-01 -> 142 orders predicted
2026-06-02 -> 151 orders predicted
2026-06-03 -> 149 orders predicted
```

These values are not a certainty. They represent the model's estimate.

## Required inputs

To run a forecast, Forecast needs:

| Element | Role |
| --- | --- |
| Date | Places each row in time |
| Target | Value to predict |
| Frequency | Rhythm of the data: day, hour, month, etc. |
| Horizon | Number of future points to predict |
| Model | Engine used to compute the trajectory |

Contextual variables and multi-series are not mandatory, but they become important as soon as you want to explain or simulate the future.

## Horizon

The horizon indicates the depth of the forecast.

Examples:

- horizon `24` with hourly frequency: predict the next 24 hours;
- horizon `31` with daily frequency: predict the next 31 days;
- horizon `12` with monthly frequency: predict the next 12 months.

The longer the horizon, the more uncertainty generally increases.

## Result and identifier

Each run produces an `analysis_id` identifier.

This identifier does not mean "saved file". It is used to retrieve the computed result: future curve, uncertainty, parameters, variables, scenarios, and annotations.

The application uses it to:

- reopen a forecast;
- display the graph;
- compare several results;
- create or rerun scenarios;
- let an LLM agent review the result.

## Correct interpretation

A forecast must be read with three questions:

- is the trend rising, falling, or stable?
- is the uncertainty small or large?
- which contextual variables can explain the movement?

A curve alone is not enough. Forecast becomes useful when the forecast is connected to its context.
