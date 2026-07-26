# Troubleshooting

This section helps you understand why a forecast does not give the expected result.

## Invalid JSON data

This error means Forecast did not receive a usable table.

It can come from:

- malformed JSON;
- a file not converted correctly;
- an empty or truncated `data` field;
- a wrong row format;
- missing columns.

If the user provides a file, the agent must check that the file is read correctly before converting the data.

## Model unavailable

This error can come from:

- a local model not installed;
- a stopped sidecar;
- a missing API key;
- a model incompatible with the parameters;
- data too short for the requested model.

The right reflex is to check the model, then test with a minimal dataset.

## Contextual variables ignored

A variable can be ignored or useless if:

- it does not exist in the history;
- it is empty in the future;
- it is constant;
- it is mistyped;
- it does not match the horizon;
- it contains text not converted into a usable number or category.

In this case, inspect the dataset before blaming the model.

## Flat result

A flat forecast can be normal if the target is stable.

It can also indicate:

- history too short;
- poorly chosen frequency;
- missing context;
- low-variability target;
- model too simple;
- uninformative known future.

## Scenario with no visible effect

A contextual scenario may show little difference if:

- the modified variable has little influence;
- the modification is too small;
- the model does not use this variable as a strong signal;
- the future variable was not actually transmitted;
- the curve is hidden by the filters.

You must check the graph, the filters, the tooltip, and the scenario data.
