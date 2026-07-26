# Agentes LLM

El LLM dirige Forecast desde la conversación activa. Puede preparar o buscar datos, auditar su calidad, seleccionar un modelo autorizado, ejecutar cálculos y explicar resultados.

## Flujo obligatorio

Para cada dataset nuevo, sigue este orden:

1. Comprende el objetivo, periodo, horizonte y confianza solicitada.
2. Lee o construye los datos y distingue sus fuentes.
3. Llama a `forecast_data_audit`.
4. Corrige los errores bloqueantes o explícalos.
5. Llama a `forecast_models` con el perfil validado.
6. En Manual, respeta el modelo impuesto y verifica la compatibilidad exacta.
7. En Auto, elige un único candidato devuelto.
8. Llama a `forecast` con el perfil, modelo autorizado y confianza sin cambios.
9. Usa `forecast_read` para las páginas y análisis necesarios.
10. Explica la previsión, incertidumbre y límites.

Repite la auditoría cuando cambien los datos, objetivo, frecuencia, horizonte o confianza.

## Modo Manual

No alteres nunca la selección persistida. Si el modelo falta, no está preparado o es incompatible, pide una acción clara en vez de elegir otro silenciosamente.

## Modo Auto

Elige un candidato devuelto y no evites las exclusiones del backend. Respeta una solicitud explícita solo si Forecast confirma que sigue siendo segura.

Transmite a `forecast` el identificador de selección y las razones cortas autorizadas. No llames mejor a una selección basada solo en capacidades y recursos.

## Evaluación y comparación

Cuando el usuario pida el mejor modelo:

1. Ejecuta `forecast_backtest` con modelos compatibles.
2. Comprueba el estado y los fallos individuales.
3. Lee la clasificación con `forecast_compare_models`.
4. Compara con Naive, Naive estacional, Drift y ETS.
5. Presenta error, cobertura, velocidad y memoria.

No presentes un backtest parcial como completo ni un modelo como mejor si no supera una referencia creíble.

## Procedencia y explicación

Indica siempre si un valor procede de un archivo, una fuente externa, un cálculo o una hipótesis. No inventes datos importantes en silencio.

Usa la conversación existente para explicar, comparar o relanzar. Presenta honestamente los análisis avanzados ausentes o poco fiables.
