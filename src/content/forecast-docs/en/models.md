# Models

A model is the engine that computes the forecast. Forecast offers several model families, local (the computation stays on the machine) or cloud (the useful data is sent to the configured provider).

## Local families

| Family | Publisher | Detail |
| --- | --- | --- |
| Chronos / Chronos-Bolt | Amazon | Fast local model, good for a first test or a simple target |
| TimesFM | Google | Local time-series forecasting model |
| Toto 2.0 | Datadog | Local model oriented toward monitoring and metrics |
| MOIRAI 2.0 | Salesforce | Local model, handles multi-series and covariates |
| FlowState | IBM | Local model for time series |
| TabPFN-TS | PriorLabs | Experimental local model |
| TiRex | NX-AI | Experimental local model |
| Kairos | Foundation Model Research | Experimental local model |
| Sundial | THUML | Experimental local model |

## Cloud family

| Family | Publisher | Detail |
| --- | --- | --- |
| TimeGPT-2 / TimeGPT-2.1 | Nixtla | Cloud engine specialized in time series. Requires an API key and sends the useful data to the provider. |

Cloud models may be more powerful, but involve an external dependency and data being sent out. For sensitive data, prefer a local model.

## Choosing a model

The choice mostly depends on the dataset and the use case:

- **Quick test, simple target**: Chronos-Bolt.
- **Sensitive data, local computation**: any local family.
- **Covariates and future context**: a model that handles contextual variables (MOIRAI 2.0, Chronos-2, TimeGPT).
- **Multi-series**: a model that handles several series (MOIRAI 2.0, Chronos-2, TimeGPT).
- **Advanced cloud quality**: TimeGPT, accepting that data is sent out.

An advanced model does not compensate for poorly structured data. Before changing models, check the dataset quality, the frequency, the horizon, and the contextual variables.

## Installing a local model

Local models must be installed from the model manager (Settings → Forecast) or via the models tab of the Forecast workspace. They are downloaded from Hugging Face or GitHub depending on the family, then stored locally in `~/.local/share/cl-go-dash/forecast-models/`.
