# Datasets

La calidad de una previsión empieza por los datos. Forecast separa las filas históricas, la información futura ya conocida y las hipótesis creadas para escenarios.

## Estructura mínima

Un dataset utilizable contiene una columna de fecha, una columna objetivo, una frecuencia y un horizonte. Una columna de serie opcional separa productos, regiones o sensores, y las covariables añaden contexto.

## Zona histórica

Las filas históricas contienen una fecha y un objetivo observado. Deben estar ordenadas, ser suficientemente numerosas y corresponder a la frecuencia elegida.

Forecast comprueba fechas inválidas o desordenadas, duplicados, periodos ausentes, valores vacíos o no numéricos, valores atípicos, longitud del historial y coherencia entre series.

Un error estructural bloquea la ejecución. Un riesgo no bloqueante permanece visible como aviso.

## Zona futura

Las filas futuras pueden omitir el objetivo. Son útiles cuando incluyen información ya conocida, como calendario, precios, presupuestos, campañas, previsiones meteorológicas o capacidad prevista.

No presentes como hecho una información futura desconocida.

## Auditoría antes de prever

Cada dataset nuevo pasa por `forecast_data_audit`. La auditoría valida los datos, el horizonte, la frecuencia y el nivel de confianza solicitado.

Una auditoría válida crea un perfil reutilizable. El LLM lo utiliza para seleccionar el modelo y lanzar la previsión sin reenviar todos los datos.

Repite la auditoría si cambian los datos, el objetivo, el horizonte, la frecuencia o la confianza.

## Datos creados por el LLM

El LLM puede leer CSV, hojas de cálculo o JSON, buscar contexto y crear columnas. Debe distinguir datos leídos de un archivo, encontrados en la web, calculados o supuestos para una simulación.

## Vista previa

La sección Datos muestra filas, puntos históricos, filas futuras, series, periodos ausentes y valores atípicos, además del mapeo y una vista previa limitada.
