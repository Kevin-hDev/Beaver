# Uncertainty

A forecast is never an absolute truth. Forecast therefore displays a central value and an uncertainty range to show the risk around the trajectory.

## Central value

The central value is the model's main estimate.

Example:

```text
2026-06-01 -> 142 orders predicted
```

It represents the most likely scenario given the data used.

## Uncertainty bounds

The bounds indicate a likely zone around the central value.

Example:

```text
Forecast: 142
Lower bound: 128
Upper bound: 157
```

Simple reading: the model estimates 142, but considers that a value around 128 to 157 remains plausible.

## Quantiles

Models can return quantiles.

| Field | Meaning |
| --- | --- |
| q10 | Likely low value |
| q50 | Central or median value |
| q90 | Likely high value |

The wider the gap between q10 and q90, the more uncertain the model is.

## Why uncertainty increases

Uncertainty can increase when:

- the history is short;
- the target varies strongly;
- the horizon is long;
- contextual variables are missing;
- a recent break appears in the data;
- several future scenarios are possible.

## Proper use

The central value is used to read the trend.

The uncertainty range is used to read the risk.

A serious decision must look at both.
