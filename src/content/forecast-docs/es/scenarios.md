# Escenarios

Un escenario explora una hipótesis a partir de un análisis existente. No sustituye los datos observados ni la previsión original.

## Ajuste global

Un ajuste porcentual crea una curva derivada, por ejemplo demanda +10%, ingresos -5% o capacidad +15%. Es rápido, pero no vuelve a ejecutar el modelo.

## Escenario contextual

Un escenario contextual modifica covariables futuras y vuelve a ejecutar el modelo cuando es compatible. Puede cambiar presupuesto, precio, clima, capacidad o una serie concreta.

Los valores modificados siguen siendo hipótesis.

## Creación y edición

El espacio Forecast agrupa la creación, modificación y eliminación de escenarios. El panel conserva su lectura rápida. El LLM también puede gestionarlos con `forecast_analyze`.

## Comparar curvas

Compara la previsión original y los escenarios sobre el mismo periodo. Revisa el inicio de la divergencia, su amplitud, la incertidumbre, las series afectadas y las covariables modificadas.

## Conjunto de modelos

Un conjunto no es un escenario de negocio. Combina entre dos y cuatro modelos con backtest correcto, ponderados por el inverso del MASE. Se marca como no evaluado independientemente.

## Buen uso

Asigna a cada escenario un nombre claro, hipótesis medible, periodo, fuente de valores, explicación del cambio y comparación con la previsión original.
