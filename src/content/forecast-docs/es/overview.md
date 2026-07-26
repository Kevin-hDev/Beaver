# Visión general

Forecast sirve para prever la evolución futura de un dato medible. El módulo analiza el historial, las tendencias recientes y las variables de contexto para producir una predicción fechada, con un margen de incertidumbre y escenarios comparables.

## Definición sencilla del forecasting

El forecasting consiste en observar datos pasados y actuales para estimar lo que puede ocurrir a continuación.

Ejemplos:

- prever las ventas de los próximos 30 días;
- estimar la facturación del mes siguiente;
- anticipar la carga del servidor de las próximas horas;
- proyectar el precio o el volumen de un activo;
- simular el efecto de un contexto futuro conocido.

El modelo no lee el futuro. Calcula una trayectoria probable a partir de patrones visibles en los datos.

## Lo que Forecast añade a un chat LLM

Un LLM puede leer una tabla y escribir una explicación. Forecast añade un motor especializado que calcula realmente una serie futura.

La diferencia es importante:

| Chat LLM solo | Forecast |
| --- | --- |
| Explica un archivo | Calcula puntos futuros fechados |
| Puede razonar cualitativamente | Produce una curva numérica |
| Puede inventar si los datos son difusos | Usa un contrato de datos estricto |
| Resume una tendencia | Genera una previsión, unos límites y escenarios |

El LLM sigue siendo útil alrededor del motor: prepara los datos, elige las columnas, puede buscar información en la web, construye un dataset, lanza Forecast y luego explica el resultado.

## Objeto principal: el objetivo

El objetivo es la columna que Forecast debe predecir.

Ejemplos:

- `ventes`
- `ca_total_eur`
- `commandes_total`
- `temperature`
- `stock_price`
- `incidents_count`

Toda la previsión gira en torno a este objetivo: el modelo aprende su comportamiento pasado y luego estima sus valores futuros.

## Lo que contiene un resultado Forecast

Un resultado Forecast contiene:

- los valores históricos utilizados;
- la previsión futura punto por punto;
- un rango de incertidumbre;
- las variables de contexto utilizadas;
- los escenarios creados a partir de esta previsión;
- los metadatos necesarios para revisar, comparar y exportar el resultado.

No es un archivo pasivo. Es un objeto de trabajo completo para entender lo que es probable, lo que es arriesgado y lo que cambia si el contexto evoluciona.

## Lógica general

El flujo de trabajo estándar es:

1. proporcionar un dataset;
2. elegir la fecha, el objetivo y, eventualmente, las series;
3. seleccionar las variables de contexto útiles;
4. lanzar un modelo de previsión;
5. leer la curva futura y la incertidumbre;
6. crear escenarios para probar hipótesis;
7. pedir al LLM que explique los resultados o que prepare nuevos datos.
