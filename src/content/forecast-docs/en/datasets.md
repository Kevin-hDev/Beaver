# Datasets

A dataset is the table Forecast uses to learn the past and predict the future. It must be structured so the model understands what to predict, on which dates, and with what context.

## Minimum structure

A Forecast dataset contains at minimum:

| Column | Role |
| --- | --- |
| Date | Indicates when each observation occurred |
| Target | Value to predict |

Simple example:

```text
date        commandes
2026-05-01  120
2026-05-02  135
2026-05-03  128
```

With this table, Forecast can learn the dynamics of commandes and predict the next dates.

## History zone

The history zone contains the real values already known.

It lets the model detect:

- trend;
- seasonality;
- rhythm;
- spikes;
- drops;
- normal variations.

The target must be filled in this zone.

## Future zone

The future zone contains the dates to predict.

In this zone, the target is empty, because that is precisely what Forecast must compute.

Example:

```text
date        commandes
2026-05-01  120
2026-05-02  135
2026-05-03
2026-05-04
```

Here, Forecast must predict `commandes` for May 3 and 4.

## Known future

The known future adds contextual variables on the future rows.

Example:

```text
date        commandes   pluie_mm   promo
2026-05-01  120        0          0
2026-05-02  135        4          1
2026-05-03             12         0
2026-05-04             0          1
```

The future target is empty, but the rain and the promo are already known or assumed. The model can use this information to produce a more realistic forecast.

## Series column

The series column is used when a single file contains several objects to predict.

Examples:

- several stores;
- several products;
- several cities;
- several financial assets;
- several servers.

Example:

```text
date        magasin   ventes   promo
2026-05-01  paris    120      0
2026-05-01  lyon     92       1
2026-05-02  paris    132      1
2026-05-02  lyon     95       0
```

Forecast can then predict each series while accounting for the group it belongs to.

## Dataset created by an agent

An LLM agent can create or enrich a dataset.

For example, it can:

- convert an Excel file to JSON;
- add a `weekend` column;
- fetch events from the web;
- turn textual information into a score;
- fill future rows with hypotheses;
- clean up dates or columns.

The agent must clearly indicate which data comes from the file, the web, a computation, or a hypothesis.
