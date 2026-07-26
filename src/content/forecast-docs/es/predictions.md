# Previsiones

Una previsión prolonga una o varias series a partir de su historial, variables contextuales y modelo seleccionado. Incluye una estimación central y límites de incertidumbre cuando están disponibles.

## Resultado guardado

Cada ejecución válida crea un `analysis_id` que vincula el panel, el espacio Forecast, escenarios, notas, evaluaciones y exportaciones.

Antes de guardar, Forecast valida cantidades, fechas, orden, valores finitos, cuantiles y horizonte. Una salida parcial o incoherente no se guarda como análisis válido.

## Gráfico principal

El gráfico separa historial y zona prevista. Los filtros muestran u ocultan series, incertidumbre, escenarios, eventos, comparaciones, anomalías y señales de calidad.

Puedes arrastrar para desplazarte, usar rueda o trackpad para ampliar, usar las barras de salto, plegar tarjetas y abrir la tabla de puntos. El zoom no bloquea el desplazamiento de la página cuando ya no puede cambiar.

## Gráficos complementarios

El espacio puede mostrar un abanico de incertidumbre, una comparación estacional y un gráfico de fiabilidad tras un backtest. En multi-serie, la serie activa permanece sincronizada.

## Tabla de previsiones

La tabla está plegada por defecto. Al abrirla muestra fechas, valor central y límites en una zona desplazable y limitada.

Para análisis largos, `forecast_read` devuelve páginas limitadas en lugar de cargar toda la serie en el contexto del LLM.

## Actualización en tiempo real

El panel y el espacio Forecast leen el mismo análisis. Nuevas previsiones, cambios y selección de análisis actualizan las vistas sin cerrar y volver a abrir la ventana.

## Interpretación correcta

Interpreta la curva junto con la calidad de datos, incertidumbre, horizonte, anomalías, backtests, referencias e hipótesis. Una curva suave no demuestra precisión.
