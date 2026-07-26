# Tools Forecast

Los siete tools Forecast forman un flujo controlado. Los resultados grandes permanecen en el almacenamiento Forecast y el LLM intercambia identificadores compactos.

## Orden recomendado

```text
forecast_data_audit
  → forecast_models
  → forecast
  → forecast_read
  → forecast_backtest
  → forecast_compare_models
```

Usa `forecast_analyze` después para notas, escenarios o conjuntos.

## `forecast_data_audit`

Llama a este tool antes de la primera previsión de cada dataset. Proporciona datos o archivo, objetivo, fecha, frecuencia, horizonte y confianza exacta.

Valida fechas, duplicados, periodos ausentes, valores inválidos, historial, series, futuro y valores atípicos. Una respuesta válida devuelve `data_profile_id`.

## `forecast_models`

Inspecciona la política activa y los intervalos. En Manual, verifica el modelo impuesto. En Auto, proporciona `data_profile_id`, elige un candidato y conserva `selection_id`.

La información de hardware solo aparece en esta respuesta. No redondees la confianza.

## `forecast`

Ejecuta la previsión con el perfil, objetivo, fecha, horizonte, frecuencia y confianza sin cambios. Añade serie y covariables solo si son compatibles.

En Auto, añade modelo, `selection_id`, origen y razones autorizadas. La respuesta devuelve `analysis_id`.

## `forecast_read`

Omite `analysis_id` para listar análisis o inclúyelo para leer uno. Usa `offset` y `limit`, con un máximo de 200 puntos por página.

La lectura puede incluir descomposición, anomalías residuales, importancia por permutación cronológica y deriva. No inventes sustitutos cuando falten.

## `forecast_backtest`

Ejecuta una validación temporal limitada sobre un análisis guardado. Evalúa modelos y referencias Naive, Naive estacional, Drift y ETS en periodos idénticos.

Comprueba siempre el estado y los fallos.

## `forecast_compare_models`

Lee la clasificación guardada: errores, cobertura, duración, memoria observada y estado de referencias. Solo llama mejor a un modelo si un resultado completo lo justifica.

## `forecast_analyze`

Usa `annotate`, `scenario`, `scenario_update`, `scenario_delete` o `ensemble`. Crea un conjunto solo tras un backtest multi-modelo correcto y explica que usa ponderación inversa al MASE y no fue evaluado independientemente.

## Reiniciar el flujo

Repite `forecast_data_audit` y `forecast_models` cuando cambien datos, mapeo, objetivo, frecuencia, horizonte, confianza, covariables, estructura de series o recursos.
