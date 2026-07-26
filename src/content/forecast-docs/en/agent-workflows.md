# LLM agents

The LLM drives Forecast from the active conversation. It may prepare or research data, audit quality, select an authorized model, run calculations and explain results.

## Required workflow

For every new dataset, follow this order:

1. Understand the target, period, horizon and requested confidence.
2. Read or build the data and distinguish its sources.
3. Call `forecast_data_audit`.
4. Fix blocking errors or explain them to the user.
5. Call `forecast_models` with the validated profile.
6. In Manual, respect the forced model and verify exact compatibility.
7. In Auto, choose exactly one returned candidate.
8. Call `forecast` with the profile, authorized model and unchanged confidence.
9. Call `forecast_read` for the required pages and analytics.
10. Explain the prediction, uncertainty and limitations.

Run the audit again when data, target, frequency, horizon or confidence changes.

## Manual mode

Never alter the user's persisted selection. If the forced model is absent, unprepared or incompatible, ask for a clear action instead of silently choosing another model.

## Auto mode

Choose one returned candidate and never bypass backend exclusions. Respect an explicit model request only when Forecast confirms it is still safe.

Pass the selection identifier and allowed short reasons to `forecast`. Do not call a capability-and-resource choice the best model. Prefer comparable backtest rankings when available.

## Evaluation and comparison

When the user requests the best model or a reliable comparison:

1. Run `forecast_backtest` on compatible models.
2. Check global status and individual failures.
3. Read the ranking with `forecast_compare_models`.
4. Compare models with Naive, Seasonal Naive, Drift and ETS.
5. Present trade-offs between error, coverage, speed and memory.

Never present a partial backtest as complete. Never call a model best when it does not beat a credible baseline.

## Data provenance

You may add calendars, indicators, events or web data when useful. Always state whether a value was read from a file, found externally, calculated or assumed for a scenario.

Never silently invent important data.

## Explanation in chat

Use the existing conversation. Do not wait for a special button to explain, compare, rerun or interpret a forecast.

Connect explanations to data, intervals, backtests and visible assumptions. Report unavailable or low-reliability advanced analytics honestly.
