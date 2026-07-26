# Forecasts

A forecast extends one or more series from their history, contextual variables and selected model. It contains a central estimate and uncertainty bounds when supported.

## Saved result

Every valid run creates an `analysis_id`. It links the result to the panel, workspace, scenarios, notes, evaluations and exports.

Before saving, Forecast validates point and series counts, future dates and ordering, finite values, quantile alignment and the effective horizon. A partial or inconsistent output is not saved as a valid analysis.

## Main chart

The main chart separates history from the forecast area. Filters can show or hide series, uncertainty, scenarios, events, comparisons, anomalies and quality signals.

You can drag to pan, use a wheel or trackpad to zoom, use jump bars to change detail, collapse cards and open the points table when exact values are needed.

Zoom does not trap page scrolling when no zoom change is possible.

## Companion charts

The workspace may display an uncertainty fan, a seasonal comparison and, after backtesting, a reliability chart.

For multi-series analyses, the active series stays synchronized across charts.

## Prediction table

The table is collapsed by default. When opened, it shows dates, central values and available bounds inside a height-limited scroll area.

For long analyses, `forecast_read` returns bounded pages instead of placing the entire series in the LLM context.

## Real-time updates

The panel and workspace read the same saved analysis. New forecasts, edits and active-analysis changes update the relevant views without closing and reopening the window.

## Correct interpretation

Read the curve together with data quality, uncertainty, horizon, breaks or anomalies, backtest results, baselines and assumptions. A smooth curve is not evidence of accuracy.
