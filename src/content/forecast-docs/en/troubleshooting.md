# Diagnostics

This section separates normal behavior from problems requiring action.

## Model preparation

A model can be Not installed, Update required, Invalid, Ready or Provider required. Use Prepare for missing or outdated local models, reinstall invalid models and configure the cloud provider when required.

Multiple preparations enter a queue. Already valid files are reused when possible.

## Sidecar lifecycle

The local runtime starts for a forecast or backtest and may stop immediately after the operation. This is normal and releases resources.

There is a problem only when the runtime never becomes ready, the request fails or Forecast returns an error.

## Rejected data audit

An audit may block because of missing columns, invalid or duplicate dates, inconsistent frequency, insufficient history, incorrect future rows or exceeded limits.

Fix the reported issue and rerun the audit. Never use an old profile after the dataset changes.

## Incompatible confidence

Continuous models accept whole levels from 50% to 99%. Some fixed-grid models accept only 60% or 80%.

In Manual, choose a supported level or another model. In Auto, rerun selection with the exact requested level. Never round it silently.

## Expired Auto selection

An Auto selection is bound to the dataset, session and available resources. If it expires or conditions change, call `forecast_models` again, obtain a new identifier and rerun `forecast`.

## Result missing from the panel

A valid analysis normally opens the panel and synchronizes the workspace. Check that Forecast returned an `analysis_id`, select the analysis in history, verify the active session and read the analysis again.

An output rejected during validation is not displayed as valid.

## Partial backtest

A backtest may succeed for baselines and fail for one or more models. Inspect global status and individual failures.

Do not treat the ranking as complete until the models being compared have homogeneous results.

## Ignored covariates

A covariate may be missing, empty in the future, constant, incorrectly typed, misaligned with the horizon or unsupported by the model. Check the Data section, selected model and future values.

## Flat result or weak scenario effect

A flat curve may reflect a stable target, short history, incorrect frequency or missing context. A scenario may have little effect when its change is small, its variable has little influence or the layer is hidden.

Compare data and assumptions before treating it as an error.
