# Forecast tools

The seven Forecast tools form a controlled workflow. Large outputs stay in Forecast storage while the LLM exchanges compact identifiers.

## Recommended order

For a new dataset, use:

```text
forecast_data_audit
  → forecast_models
  → forecast
  → forecast_read
  → forecast_backtest
  → forecast_compare_models
```

Use `forecast_analyze` afterwards for notes, scenarios or ensembles.

## `forecast_data_audit`

Call this tool before the first forecast for every new dataset. Provide data or file, target, date, frequency, horizon and exact confidence.

It validates dates, duplicates, missing periods, invalid values, history length, series, future rows and outliers. A valid response returns a reusable `data_profile_id`.

## `forecast_models`

Inspect the active policy and interval capabilities.

In Manual, verify the forced model and exact confidence compatibility. In Auto, pass `data_profile_id`, choose one returned candidate and retain its `selection_id`.

Hardware information is exposed only in this Forecast response. Never round confidence to fit a model.

## `forecast`

Run the validated forecast with the profile, target, date, horizon, frequency and unchanged confidence. Add series and covariates only when supported.

In Auto, also pass the model, `selection_id`, selection source and allowed reasons. The response returns an `analysis_id`.

Never use a model different from the Manual choice or Auto authorization.

## `forecast_read`

Omit `analysis_id` to list a bounded set of analyses, or provide it to read one analysis.

Predictions use `offset` and `limit`, with at most 200 points per page. Reading may also return decomposition, residual anomalies, chronological permutation importance and drift.

Report unavailable or low-reliability analytics honestly. Never invent a substitute.

## `forecast_backtest`

Run bounded rolling validation on a saved analysis. Request compatible models and a bounded number of windows.

The tool evaluates models and attempts Naive, Seasonal Naive, Drift and ETS on identical periods. Always inspect status and model failures.

## `forecast_compare_models`

Read the saved post-backtest ranking. It summarizes errors, coverage, duration, observed memory and baseline status.

Call a model best only when a complete comparable result supports that claim.

## `forecast_analyze`

Modify an existing analysis with:

| Action | Purpose |
| --- | --- |
| `annotate` | Add a dated note |
| `scenario` | Create a global or contextual scenario |
| `scenario_update` | Edit a scenario |
| `scenario_delete` | Delete a scenario |
| `ensemble` | Combine two to four successfully backtested models |

Create an ensemble only after a successful multi-model backtest. State that it uses inverse-MASE weighting and was not independently evaluated.

## Restarting the workflow

Call `forecast_data_audit` and `forecast_models` again when the dataset, mapping, target, frequency, horizon, confidence, covariate needs, series structure or resource conditions change.
