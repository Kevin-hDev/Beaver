# Multi-series

El multi-series permite predecir varias series en un mismo análisis. En lugar de lanzar una previsión separada para cada objeto, Forecast recibe un único dataset con una columna que identifica cada serie.

## Ejemplo

Un archivo puede contener las ventas de varias tiendas:

```text
date        magasin   ventes   promo
2026-05-01  paris    120      0
2026-05-01  lyon     92       1
2026-05-02  paris    132      1
2026-05-02  lyon     95       0
```

Aquí, `magasin` es la columna serie.

Forecast entiende que debe predecir las ventas para `paris` y para `lyon`.

## Para qué sirve

El multi-series es útil cuando varias series comparten una lógica común.

Ejemplos:

- ventas por tienda;
- pedidos por restaurante;
- tráfico por servidor;
- precio por activo;
- incidentes por región.

El modelo puede aprovechar más información que una previsión aislada, sobre todo si las series se parecen o comparten variables de contexto.

## Horizonte por serie

Cada serie debe proporcionar una estructura temporal coherente.

Si el horizonte es `31`, cada serie debe tener 31 puntos futuros a predecir.

Ejemplo:

```text
paris -> 31 líneas futuras
lyon  -> 31 líneas futuras
```

Un horizonte incoherente hace difícil la comparación y puede bloquear el modelo.

## Lectura en Forecast

En la interfaz, el usuario puede seleccionar la serie mostrada.

Los escenarios pueden luego leerse:

- en una serie concreta;
- en varias series;
- en comparación con la previsión base.

El multi-series no cambia el principio de Forecast. Simplemente añade una dimensión: "¿qué serie se está prediciendo ahora?"
