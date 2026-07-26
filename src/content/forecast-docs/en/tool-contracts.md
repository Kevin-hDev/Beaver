# Forecast tools

Forecast tools let LLM agents use the prediction engine from chat. They must be called with precise parameters, because a wrong column or an inconsistent horizon produces a useless forecast.

## `forecast`

`forecast` runs a new prediction.

Main inputs:

| Parameter | Role |
| --- | --- |
| `file_path` | Excel, CSV, or JSON file to read |
| `data` | Already-prepared data in JSON |
| `date_column` | Column that contains the dates |
| `target_column` | Column to predict |
| `series_column` | Column that identifies the series |
| `covariate_columns` | Contextual variables to use |
| `frequency` | Temporal rhythm |
| `horizon` | Number of future points |
| `model` | Engine to use |

Main output:

- `analysis_id`, the identifier of the Forecast result.

## `forecast_read`

`forecast_read` reviews a Forecast result.

It is used to retrieve:

- the forecast;
- the history;
- the uncertainty;
- the scenarios;
- the available variables;
- the model metadata.

If no `analysis_id` is provided, the agent can use it to list the available results.

## `forecast_analyze`

`forecast_analyze` adds or modifies elements around a forecast.

It is notably used to:

- create an annotation;
- create a scenario;
- rerun a contextual scenario;
- modify a scenario;
- delete a scenario.

## What the agent must check

Before calling a tool, the agent must check:

- the target exists;
- the date is readable;
- the horizon matches the future rows;
- the covariates actually exist;
- the data created or found on the web is identified;
- the chosen model supports the need.

The agent must explain its choices instead of sending an opaque call.
