# Modelos

Un modelo es el motor que calcula la previsión. Forecast ofrece familias locales y cloud y verifica sus capacidades, estado y ajuste a los recursos antes de ejecutarlas.

## Familias disponibles

| Familia | Editor | Uso principal |
| --- | --- | --- |
| Chronos / Chronos-Bolt | Amazon | Previsiones locales rápidas y probabilísticas |
| TimesFM | Google | Previsión general de series temporales |
| Toto 2.0 | Datadog | Métricas y monitorización |
| MOIRAI 2.0 | Salesforce | Multi-serie y variables contextuales |
| FlowState | IBM | Previsión local probabilística |
| TabPFN-TS, TiRex, Kairos, Sundial | Varios | Modelos locales especializados o experimentales |
| TimeGPT | Nixtla | Previsión cloud con clave API |

El catálogo de la aplicación es la referencia para frecuencias, horizonte, covariables, multi-serie e intervalos compatibles.

## Modo Manual

En Manual, eliges el modelo y Forecast impone esa elección. Si no está preparado o no acepta los datos o la confianza exacta, el LLM pide otra elección en vez de sustituirla en silencio.

## Modo Auto

En Auto, el LLM elige un único modelo de una lista corta ya filtrada. Forecast excluye modelos no preparados, incompatibles, demasiado pesados o cloud cuando el cloud no está autorizado.

La información de hardware se expone al LLM solo durante esta selección Forecast. Sin backtests comparables, Auto habla de compatibilidad o recomendación por capacidades, nunca del mejor modelo.

## Instalación y preparación

Preparar descarga el modelo, instala su motor y realiza una validación real antes de la primera previsión. Varias preparaciones pueden entrar en una cola y las variantes de una familia pueden compartir el motor.

| Estado | Significado |
| --- | --- |
| No instalado | Faltan los archivos |
| Actualización requerida | El motor o la validación deben actualizarse |
| Inválido | La instalación está incompleta o no supera la validación |
| Listo | Modelo y motor verificados |
| Provider requerido | Falta la clave del servicio cloud |

Un modelo local solo puede seleccionarse cuando está listo. Al desinstalarlo, el motor compartido se elimina únicamente si ningún otro modelo lo necesita.

## Modelos cloud

Un modelo cloud envía los datos necesarios al proveedor configurado. Auto solo lo usa con autorización, proveedor listo y política de datos compatible. Forecast nunca cambia silenciosamente de local a cloud.
