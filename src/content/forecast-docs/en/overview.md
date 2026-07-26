# Overview

Forecast is used to predict the future evolution of a measurable quantity. The module analyzes history, recent trends, and contextual variables to produce a dated prediction, with a margin of uncertainty and comparable scenarios.

## Simple definition of forecasting

Forecasting means observing past and current data to estimate what may happen next.

Examples:

- predict sales for the next 30 days;
- estimate next month's revenue;
- anticipate server load over the coming hours;
- project the price or volume of an asset;
- simulate the effect of a known future context.

The model does not read the future. It computes a likely trajectory from patterns visible in the data.

## What Forecast adds to an LLM chat

An LLM can read a table and write an explanation. Forecast adds a specialized engine that actually computes a future series.

The difference matters:

| LLM chat alone | Forecast |
| --- | --- |
| Explains a file | Computes dated future points |
| Can reason qualitatively | Produces a numerical curve |
| May invent when data is vague | Uses a strict data contract |
| Summarizes a trend | Generates a forecast, bounds, and scenarios |

The LLM remains useful around the engine: it prepares the data, selects the columns, can search for information on the web, builds a dataset, runs Forecast, then explains the result.

## Main object: the target

The target is the column Forecast must predict.

Examples:

- `ventes`
- `ca_total_eur`
- `commandes_total`
- `temperature`
- `stock_price`
- `incidents_count`

The entire forecast revolves around this target: the model learns its past behavior, then estimates its future values.

## What a Forecast result contains

A Forecast result contains:

- the historical values used;
- the future forecast point by point;
- an uncertainty range;
- the contextual variables used;
- the scenarios created from this forecast;
- the metadata needed to review, compare, and export the result.

This is not a passive file. It is a complete working object for understanding what is likely, what is risky, and what changes if the context evolves.

## General logic

The standard workflow is:

1. provide a dataset;
2. choose the date, the target, and optionally the series;
3. select the useful contextual variables;
4. run a forecast model;
5. read the future curve and the uncertainty;
6. create scenarios to test hypotheses;
7. ask the LLM to explain the results or prepare new data.
