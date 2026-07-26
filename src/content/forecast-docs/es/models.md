# Modelos

Un modelo es el motor que calcula la previsión. Forecast propone varias familias de modelos, locales (el cálculo se queda en la máquina) o cloud (los datos útiles se envían al provider configurado).

## Familias locales

| Familia | Editor | Detalle |
| --- | --- | --- |
| Chronos / Chronos-Bolt | Amazon | Modelo local rápido, bueno para una primera prueba o un objetivo sencillo |
| TimesFM | Google | Modelo local de previsión de series temporales |
| Toto 2.0 | Datadog | Modelo local orientado a monitoring y métricas |
| MOIRAI 2.0 | Salesforce | Modelo local, gestiona el multi-series y las covariables |
| FlowState | IBM | Modelo local para series temporales |
| TabPFN-TS | PriorLabs | Modelo local experimental |
| TiRex | NX-AI | Modelo local experimental |
| Kairos | Foundation Model Research | Modelo local experimental |
| Sundial | THUML | Modelo local experimental |

## Familia cloud

| Familia | Editor | Detalle |
| --- | --- | --- |
| TimeGPT-2 / TimeGPT-2.1 | Nixtla | Motor cloud especializado en series temporales. Necesita una clave API y envía los datos útiles al provider. |

Los modelos cloud pueden ser más potentes, pero implican una dependencia externa y un envío de datos. Para datos sensibles, preferir un modelo local.

## Elegir un modelo

La elección depende sobre todo del dataset y del caso de uso:

- **Prueba rápida, objetivo sencillo**: Chronos-Bolt.
- **Datos sensibles, cálculo local**: cualquier familia local.
- **Covariables y contexto futuro**: un modelo que gestione las variables de contexto (MOIRAI 2.0, Chronos-2, TimeGPT).
- **Multi-series**: un modelo que gestione varias series (MOIRAI 2.0, Chronos-2, TimeGPT).
- **Calidad cloud avanzada**: TimeGPT, aceptando el envío de los datos.

Un modelo avanzado no compensa datos mal estructurados. Antes de cambiar de modelo, verificar la calidad del dataset, la frecuencia, el horizonte y las variables de contexto.

## Instalar un modelo local

Los modelos locales deben instalarse desde el gestor de modelos (Settings → Forecast) o a través de la pestaña modelos del espacio Forecast. Se descargan desde Hugging Face o GitHub según la familia y luego se almacenan localmente en `~/.local/share/cl-go-dash/forecast-models/`.
