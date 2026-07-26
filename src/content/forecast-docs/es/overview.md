# Vista general

Forecast está vinculado directamente a la conversación activa. El LLM prepara o busca los datos, ejecuta los cálculos y explica los resultados. El chat sigue siendo el centro de control y dos superficies complementarias permiten leer y explorar cada análisis.

## Flujo principal

El flujo normal es:

1. el usuario describe en el chat qué quiere prever;
2. el LLM lee, crea o enriquece los datos necesarios;
3. Forecast audita la calidad de los datos;
4. el modo Manual impone el modelo elegido y Auto selecciona entre candidatos seguros;
5. Forecast calcula y guarda la previsión;
6. el panel muestra inmediatamente el resultado principal;
7. el usuario continúa la conversación o abre el espacio Forecast.

No existe un chat Forecast separado. Pide una explicación, comparación o nueva ejecución mediante un mensaje normal.

## Superficies complementarias

| Superficie | Función |
| --- | --- |
| Chat | Preparar datos, guiar al LLM y pedir explicaciones |
| Panel Forecast | Leer rápidamente el gráfico, indicadores y avisos |
| Espacio Forecast | Explorar datos, gráficos, evaluaciones, escenarios, notas e informe |

El panel se mantiene compacto. El espacio Forecast se abre en una ventana dedicada sin ocultar ni sustituir la conversación.

## Espacio Forecast

El espacio permanece vinculado a la sesión y al análisis activos. Al seleccionar otro análisis en el panel, la ventana abierta se actualiza automáticamente.

| Sección | Contenido |
| --- | --- |
| Datos | Resumen, mapeo, calidad y vista previa |
| Previsión | Gráfico principal, incertidumbre, estacionalidad, filtros y tabla |
| Evaluación | Backtest temporal, referencias y fiabilidad de intervalos |
| Comparación | Clasificación comparable y posible conjunto de modelos |
| Escenarios | Creación y edición de hipótesis |
| Notas | Contexto, riesgos, decisiones y anotaciones |
| Informe | Análisis detallado y exportaciones |

## Análisis guardado

Un análisis conserva las columnas y ajustes efectivos, el perfil de calidad, el modelo y el origen de la selección, la previsión y sus intervalos, escenarios, notas, backtests y la procedencia necesaria para reproducirlo.

## Idea esencial

Forecast produce una estimación estructurada, no una certeza. Interpreta cada curva junto con la calidad de los datos, la incertidumbre, las referencias y los límites del contexto disponible.
