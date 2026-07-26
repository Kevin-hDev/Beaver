# Models

A model is the engine that computes the forecast. Forecast provides local and cloud families, then verifies their capabilities, readiness and current resource fit before execution.

## Available families

| Family | Publisher | Main use |
| --- | --- | --- |
| Chronos / Chronos-Bolt | Amazon | Fast local probabilistic forecasts |
| TimesFM | Google | General time-series forecasting |
| Toto 2.0 | Datadog | Metrics and monitoring series |
| MOIRAI 2.0 | Salesforce | Multi-series and contextual variables |
| FlowState | IBM | Local probabilistic forecasting |
| TabPFN-TS | PriorLabs | Experimental local forecasting |
| TiRex | NX-AI | Experimental local forecasting |
| Kairos | Foundation Model Research | Experimental local forecasting |
| Sundial | THUML | Local probabilistic forecasting |
| TimeGPT | Nixtla | Cloud forecasting with an API key |

Exact capabilities depend on the variant. The in-app catalog is the source of truth for frequencies, horizons, covariates, multi-series support and intervals.

## Manual mode

In Manual mode, you choose the model in the selector and Forecast enforces that choice.

The LLM still checks readiness and exact compatibility. If the model cannot handle the data or requested confidence, it asks for another model or level instead of silently replacing your choice.

## Auto mode

In Auto mode, the LLM must select one model from a short list already filtered by Forecast.

The backend excludes models that are not ready, incompatible with the data or exact confidence, too large for current resources, or cloud-based when cloud use is not allowed.

Hardware context is exposed to the LLM only during this Forecast selection. Before comparable backtests exist, Auto describes a model as compatible or capability-based recommended, never as the best.

## Installation and preparation

The Prepare action downloads the model, installs its required runtime and performs a real validation. This happens during preparation, not during the first forecast.

Several preparations can be queued. Variants from the same family may share a runtime.

| State | Meaning |
| --- | --- |
| Not installed | Model files are absent |
| Update required | Files exist, but runtime or validation must be refreshed |
| Invalid | Installation is incomplete or failed validation |
| Ready | Model and runtime passed validation |
| Provider required | A cloud provider API key is missing |

A local model is selectable only when ready. Uninstalling it removes its files and removes a shared runtime only when no other model still needs it.

## Cloud models

A cloud model sends the required data to the configured provider. Auto uses it only when cloud access is allowed, the provider is ready and data policy permits external transfer.

Forecast never silently falls back from local to cloud.
