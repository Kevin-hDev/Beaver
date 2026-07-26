# Datasets

Forecast quality starts with the data. Forecast separates historical rows, future information that is already known and assumptions created for scenarios.

## Minimum structure

A usable dataset contains at least:

| Element | Purpose |
| --- | --- |
| Date column | Places each observation in time |
| Target column | Holds the value to predict |
| Frequency | Defines the time step: hour, day, week, month, quarter or year |
| Horizon | Defines how many future steps to predict |

An optional series column separates products, regions or sensors. Covariates add context.

## Historical area

Historical rows contain a date and an observed target. They must be ordered, long enough and consistent with the selected frequency.

Forecast checks invalid or unordered dates, duplicates, missing periods, missing or non-numeric values, outliers, history length relative to the horizon and consistency across series.

A structural error blocks the run. A non-blocking risk remains visible as a warning.

## Future area

Future rows may omit the target because it is the value to predict. They are useful when they contain information already known for future periods, such as calendars, planned prices, budgets, campaigns, weather forecasts or expected capacity.

Never present unknown future information as a fact.

## Audit before prediction

Every new dataset goes through `forecast_data_audit` before prediction. The audit validates the data, horizon, frequency and requested confidence level.

A valid audit creates a reusable profile. The LLM uses it to select a model and run the forecast without repeatedly sending all data through the conversation.

Run a new audit when data, target, horizon, frequency or confidence changes.

## Data created by the LLM

The LLM may read CSV, spreadsheet or JSON data, research context and create useful columns. It must clearly distinguish data read from a file, found online, calculated or assumed for a simulation.

This provenance makes real, derived and hypothetical values understandable.

## Workspace preview

The Data section displays rows, history points, future rows, series, missing periods and outliers. It also shows the target, date, frequency, covariates and a bounded preview.
