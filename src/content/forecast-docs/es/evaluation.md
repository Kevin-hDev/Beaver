# Evaluación y comparación

La evaluación mide un modelo sobre periodos históricos que no vio durante el cálculo. Compara resultados en las mismas ventanas temporales en vez de confiar en el nombre o tamaño del modelo.

## Backtest temporal deslizante

Forecast divide el historial en varias ventanas. En cada una, el modelo usa solo el pasado y prevé el periodo siguiente. Así se evita utilizar información futura.

Si el historial es corto, Forecast puede reducir las ventanas o el horizonte y muestra un aviso.

## Referencias

| Referencia | Principio |
| --- | --- |
| Naive | Repite el último valor conocido |
| Naive estacional | Repite el valor estacional comparable anterior |
| Drift | Prolonga la tendencia media |
| ETS | Modela nivel, tendencia y estacionalidad cuando es posible |

## Métricas

| Métrica | Lectura |
| --- | --- |
| MASE | Error frente a una previsión naive; menor es mejor |
| sMAPE | Error relativo simétrico; menor es mejor |
| MAE | Error absoluto medio en la unidad del objetivo |
| Cobertura | Valores reales incluidos en el intervalo |
| Duración | Tiempo observado |
| Memoria | Pico observado cuando está disponible |

Compara la cobertura medida con el nivel solicitado. Un intervalo del 80% que cubre solo el 40% está mal calibrado.

## Evaluación y Comparación

Evaluación lanza el backtest y muestra resultados detallados. Comparación utiliza resultados homogéneos y muestra compromisos entre precisión, cobertura, velocidad y recursos.

No presentes una ejecución parcial como una validación completa.

## Mejor modelo

Llama mejor a un modelo solo si usa las mismas ventanas, tiene un resultado completo, mejora las métricas pertinentes, supera una referencia creíble y respeta las restricciones del usuario.

Sin backtests comparables, habla únicamente de compatibilidad o recomendación por capacidades.

## Conjunto de modelos

Tras un backtest multi-modelo correcto, Comparación puede combinar de dos a cuatro modelos ponderados por el inverso del MASE. El conjunto se marca como no evaluado de forma independiente.
