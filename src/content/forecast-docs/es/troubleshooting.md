# Diagnóstico

Esta sección separa el comportamiento normal de los problemas que requieren una acción.

## Preparación del modelo

Un modelo puede estar No instalado, Actualización requerida, Inválido, Listo o Provider requerido. Usa Preparar para modelos ausentes o antiguos, reinstala los inválidos y configura el proveedor cloud cuando sea necesario.

Varias preparaciones entran en una cola y los archivos válidos se reutilizan.

## Ciclo del sidecar

El motor local arranca para una previsión o backtest y puede detenerse justo después. Es normal y libera recursos.

Solo hay problema si no llega a estar listo, falla la petición o Forecast devuelve un error.

## Auditoría rechazada

Puede deberse a columnas ausentes, fechas inválidas o duplicadas, frecuencia incoherente, historial insuficiente, futuro incorrecto o límites superados.

Corrige el problema y repite la auditoría. No uses un perfil antiguo si cambió el dataset.

## Confianza incompatible

Los modelos continuos aceptan niveles enteros entre 50% y 99%. Algunos modelos fijos solo aceptan 60% u 80%.

En Manual, cambia el nivel o el modelo. En Auto, repite la selección con el nivel exacto. No lo redondees.

## Selección Auto caducada

La selección está ligada al dataset, sesión y recursos. Si caduca, llama de nuevo a `forecast_models`, obtiene otro identificador y repite `forecast`.

## Resultado ausente

Comprueba que Forecast devolvió `analysis_id`, selecciona el análisis en el historial, verifica la sesión y vuelve a leerlo. Una salida rechazada por validación no se muestra como válida.

## Backtest parcial

Revisa el estado general y los fallos individuales. No consideres completa la clasificación hasta obtener resultados homogéneos para los modelos comparados.

## Covariables ignoradas

Una covariable puede faltar, estar vacía en el futuro, ser constante, tener tipo incorrecto, estar desalineada o no ser compatible. Revisa Datos, el modelo y los valores futuros.

## Resultado plano o escenario débil

Una curva plana puede reflejar un objetivo estable, historial corto, frecuencia incorrecta o contexto ausente. Un escenario puede tener poco efecto si el cambio es pequeño o la capa está oculta.
