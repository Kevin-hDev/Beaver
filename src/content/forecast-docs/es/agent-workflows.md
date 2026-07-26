# Agentes LLM

Los agentes LLM pueden usar Forecast como un motor especializado. Su rol no se limita a leer un archivo y clicar en una tool: pueden preparar los datos, buscar contexto en la web, construir un dataset, lanzar la predicción y explicar el resultado.

## Lo que el agente puede hacer

Un agente puede intervenir en varios momentos:

| Etapa | Rol del agente |
| --- | --- |
| Preparación | Leer un Excel, CSV o JSON |
| Búsqueda | Ir a buscar información externa en la web |
| Dataset | Crear o completar columnas útiles |
| Lanzamiento | Llamar a `forecast` con los parámetros correctos |
| Lectura | Usar `forecast_read` para recuperar el resultado |
| Escenario | Crear hipótesis y relanzar el modelo |
| Explicación | Resumir tendencia, incertidumbre, variables y límites |

Ejemplo: para una previsión financiera, el agente puede leer el archivo local, buscar el contexto de mercado reciente, producir columnas como `news_score` o `event_flag` y luego lanzar Forecast.

## Flujo de trabajo recomendado

El agente debe seguir este orden:

1. entender la petición del usuario;
2. inspeccionar los datos disponibles;
3. identificar el objetivo a predecir;
4. identificar las fechas, la frecuencia y el horizonte;
5. buscar o crear las variables de contexto útiles si es necesario;
6. verificar que las líneas futuras son coherentes;
7. elegir un modelo compatible;
8. lanzar `forecast`;
9. revisar el resultado con `forecast_read`;
10. explicar la previsión y proponer escenarios útiles.

## Creación de datos por el agente

El agente puede crear datos si el usuario se lo pide o si la predicción lo requiere.

Ejemplos:

- añadir una columna `weekend` a partir de la fecha;
- crear `month_end_flag`;
- transformar un evento web en puntuación numérica;
- rellenar una zona futura con hipótesis meteorológicas;
- construir un dataset de prueba para validar un flujo de trabajo;
- convertir un archivo Excel en JSON explotable.

El agente debe siempre explicar qué columnas ha creado y por qué.

## Reglas de seguridad y de calidad

El agente no debe inventar silenciosamente un dato importante.

Si crea una variable, debe distinguir:

- dato leído en un archivo;
- dato encontrado en la web;
- dato calculado;
- hipótesis de simulación.

Esta separación es esencial para que el usuario sepa qué es real, calculado o supuesto.

## Comandos slash

Los comandos slash sirven de guías rápidas para los agentes y los usuarios.

Ejemplos:

- `/forecast`: entender el módulo Forecast;
- `/forecast-predict`: preparar y lanzar una predicción;
- `/forecast-dataset`: construir un dataset limpio;
- `/forecast-scenarios`: crear hipótesis útiles;
- `/forecast-cmd`: entender las tools disponibles.

Estos comandos deben dar un procedimiento corto, claro y directamente accionable.
