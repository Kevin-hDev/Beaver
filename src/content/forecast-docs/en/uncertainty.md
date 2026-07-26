# Uncertainty

A serious forecast is more than one curve. Forecast combines the central value with an interval representing model uncertainty at the requested confidence level.

## Central value

The central value is generally the median, often named `q50`. Roughly half of possible results are below and half are above. It does not guarantee the actual value will follow that path.

## Confidence level

Accepted confidence ranges from 50% to 99% in whole percentage-point steps for continuous models. When the user gives no preference, the LLM uses 80%.

Some models honestly provide only fixed levels, currently 60% or 80%. Forecast always preserves the exact request:

- Auto returns only compatible candidates;
- Manual reports incompatibility;
- no request is silently rounded.

## Bounds and quantiles

An 80% central interval generally uses `q10`, `q50` and `q90`. A 90% interval generally uses `q05`, `q50` and `q95`. Labels follow the levels actually computed.

## Uncertainty fan

The fan chart shows how intervals widen or narrow over the horizon. Wider bounds mean lower precision for that period. A narrow interval is useful only when correctly calibrated.

## Measured coverage

After backtesting, Forecast compares the announced level with the share of validation values actually covered. An 80% interval should contain roughly 80% of observations.

Short history can make this estimate unstable, and the interface reports that limitation.

## Good use

Use uncertainty to compare risk across periods, distinguish robust trends from fragile paths, prepare cautious thresholds, verify calibration and compare scenarios without confusing assumptions with certainty.
