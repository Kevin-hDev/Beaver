# Scenarios

A scenario explores an assumption from an existing analysis. It does not replace observed data or the original forecast.

## Global adjustment

A percentage adjustment creates a derived curve, such as demand up 10%, revenue down 5% or capacity up 15%.

It is quick to read but does not rerun the model and does not prove that a real cause would create the same effect.

## Contextual scenario

A contextual scenario changes one or more future covariates and reruns the model when supported.

Examples include changing a planned budget, price, weather, future capacity or one target series. Modified values remain assumptions.

## Creation and editing

The Forecast workspace keeps scenario creation, editing and deletion in its dedicated section. The panel retains quick reading of existing scenarios.

The LLM can also manage them through `forecast_analyze` when requested in chat.

## Comparing curves

Compare the original forecast and useful scenarios over the same period. Check when curves diverge, the size of the gap, uncertainty, affected series and covariates actually changed.

A small difference can be normal when the chosen variable has little influence.

## Model ensemble

An ensemble is not a business scenario. It combines two to four models that succeeded in a multi-model backtest, weighted by inverse MASE.

Forecast marks it as not independently evaluated until a dedicated backtest confirms its performance.

## Good use

Give every scenario a clear name, measurable assumption, time range, value source, explanation of changes and comparison with the original forecast.
