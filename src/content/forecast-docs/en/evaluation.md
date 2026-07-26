# Evaluation and comparison

Evaluation measures a model on historical periods it did not see during fitting. It compares results on identical temporal windows instead of trusting a model's name or size.

## Rolling temporal backtest

Forecast splits history into several windows. For each window, the model uses only past data and predicts the following period.

This prevents future leakage. The number of windows or evaluation horizon may be reduced when history is short, and the interface displays a warning.

## Baselines

| Baseline | Principle |
| --- | --- |
| Naive | Repeats the last known value |
| Seasonal Naive | Repeats the comparable previous seasonal value |
| Drift | Extends the observed average trend |
| ETS | Models level, trend and seasonality when possible |

An advanced model is useful only when it provides a real gain over credible baselines.

## Displayed metrics

| Metric | Meaning |
| --- | --- |
| MASE | Error relative to a naive forecast; lower is better |
| sMAPE | Symmetric relative error; lower is better |
| MAE | Average absolute error in target units |
| Coverage | Share of actual values inside the announced interval |
| Duration | Observed evaluation time |
| Memory | Peak observed resource use when available |

Compare measured coverage with the requested theoretical level. An 80% interval covering only 40% of observations is poorly calibrated.

## Evaluation and Comparison sections

Evaluation runs the backtest and shows detailed results. Comparison uses only homogeneous results and highlights trade-offs between accuracy, coverage, speed and resources.

A result can be complete, partial or unavailable. Never treat a partial run as complete validation.

## Naming a best model

Call a model best only when models used the same windows, the result is complete, relevant metrics are better, it beats a credible baseline and user constraints remain satisfied.

Without comparable backtests, describe only compatibility or a capability-based recommendation.

## Model ensemble

After a successful multi-model backtest, Comparison can build an ensemble from two to four valid models. Forecast weights them by inverse MASE.

The ensemble is marked as not independently evaluated. Backtest it separately before claiming it is superior.
