# Previsiones

Una previsión es el resultado calculado por Forecast para estimar los próximos valores de un objetivo. Responde a una pregunta concreta: "si los datos pasados y el contexto conocido se mantienen coherentes, ¿qué valores se pueden esperar a continuación?"

## Lo que representa una previsión

Una previsión contiene una serie de puntos futuros.

Cada punto corresponde a una fecha o un periodo:

```text
2026-06-01 -> 142 pedidos previstos
2026-06-02 -> 151 pedidos previstos
2026-06-03 -> 149 pedidos previstos
```

Estos valores no son una certeza. Representan la estimación del modelo.

## Entradas necesarias

Para lanzar una previsión, Forecast necesita:

| Elemento | Rol |
| --- | --- |
| Fecha | Sitúa cada línea en el tiempo |
| Objetivo | Valor a predecir |
| Frecuencia | Ritmo de los datos: día, hora, mes, etc. |
| Horizonte | Número de puntos futuros a predecir |
| Modelo | Motor utilizado para calcular la trayectoria |

Las variables de contexto y el multi-series no son obligatorios, pero se vuelven importantes en cuanto se quiere explicar o simular el futuro.

## Horizonte

El horizonte indica la profundidad de la previsión.

Ejemplos:

- horizonte `24` con frecuencia horaria: prever las próximas 24 horas;
- horizonte `31` con frecuencia diaria: prever los próximos 31 días;
- horizonte `12` con frecuencia mensual: prever los próximos 12 meses.

Cuanto más largo es el horizonte, más aumenta generalmente la incertidumbre.

## Resultado e identificador

Cada lanzamiento produce un identificador `analysis_id`.

Este identificador no significa "archivo guardado". Sirve para encontrar el resultado calculado: curva futura, incertidumbre, parámetros, variables, escenarios y anotaciones.

La aplicación lo utiliza para:

- reabrir una previsión;
- mostrar el gráfico;
- comparar varios resultados;
- crear o relanzar escenarios;
- permitir a un agente LLM revisar el resultado.

## Interpretación correcta

Una previsión debe leerse con tres preguntas:

- ¿la tendencia sube, baja o se mantiene estable?
- ¿la incertidumbre es pequeña o grande?
- ¿qué variables de contexto pueden explicar el movimiento?

Una curva sola no basta. Forecast se vuelve útil cuando la previsión se vincula a su contexto.
