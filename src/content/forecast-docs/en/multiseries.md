# Multi-series

Multi-series lets you predict several series within a single analysis. Instead of running a separate forecast for each object, Forecast receives a single dataset with a column that identifies each series.

## Example

A file can contain the sales of several stores:

```text
date        magasin   ventes   promo
2026-05-01  paris    120      0
2026-05-01  lyon     92       1
2026-05-02  paris    132      1
2026-05-02  lyon     95       0
```

Here, `magasin` is the series column.

Forecast understands it must predict sales for `paris` and for `lyon`.

## What it is used for

Multi-series is useful when several series share a common logic.

Examples:

- sales by store;
- orders by restaurant;
- traffic by server;
- price by asset;
- incidents by region.

The model can exploit more information than an isolated forecast, especially if the series look alike or share contextual variables.

## Horizon per series

Each series must provide a consistent temporal structure.

If the horizon is `31`, each series must have 31 future points to predict.

Example:

```text
paris -> 31 future rows
lyon  -> 31 future rows
```

An inconsistent horizon makes comparison difficult and can block the model.

## Reading in Forecast

In the interface, the user can select the displayed series.

Scenarios can then be read:

- on a specific series;
- on several series;
- in comparison with the baseline forecast.

Multi-series does not change the principle of Forecast. It simply adds a dimension: "which series is currently being predicted?"
