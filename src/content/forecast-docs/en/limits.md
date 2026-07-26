# Limits

Forecast predicts a likely trajectory. It does not guarantee that the future will unfold exactly as predicted.

## Key takeaways

A forecast always depends on:

- the quality of the history;
- the stability of the observed phenomenon;
- the available contextual variables;
- the requested horizon;
- the chosen model.

If the data is weak, inconsistent, or incomplete, the forecast will be fragile.

## What Forecast cannot guarantee

Forecast does not guarantee:

- a certain future outcome;
- a real causal relationship between two variables;
- a good prediction after a brutal break;
- an automatic business interpretation;
- identical reliability across all domains.

Example: if a crisis, an outage, a price war, or an exceptional event occurs without being represented in the data, the model may underestimate the change.

## Local and cloud models

Local models keep the computation on the machine.

Cloud models require sending the useful data to the configured provider. You should therefore avoid sending sensitive data if that is not acceptable for the use case.

## Good practice

A forecast must be used as a decision aid.

The right reading is:

- likely trend;
- risk range;
- variables that can explain the movement;
- alternative scenarios;
- final human decision.
