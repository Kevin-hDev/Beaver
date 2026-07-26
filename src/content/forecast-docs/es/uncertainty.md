# Incertidumbre

Una previsión nunca es una verdad absoluta. Forecast muestra por ello un valor central y un rango de incertidumbre para visualizar el riesgo alrededor de la trayectoria.

## Valor central

El valor central es la estimación principal del modelo.

Ejemplo:

```text
2026-06-01 -> 142 pedidos previstos
```

Representa el escenario más probable según los datos utilizados.

## Límites de incertidumbre

Los límites indican una zona probable alrededor del valor central.

Ejemplo:

```text
Previsión : 142
Límite bajo : 128
Límite alto : 157
```

Lectura sencilla: el modelo estima 142, pero considera que un valor alrededor de 128 a 157 sigue siendo plausible.

## Cuantiles

Los modelos pueden devolver cuantiles.

| Campo | Significado |
| --- | --- |
| q10 | Valor bajo probable |
| q50 | Valor central o mediana |
| q90 | Valor alto probable |

Cuanto mayor es la distancia entre q10 y q90, más incierto es el modelo.

## Por qué aumenta la incertidumbre

La incertidumbre puede aumentar cuando:

- el historial es corto;
- el objetivo varía fuertemente;
- el horizonte es largo;
- faltan variables de contexto;
- aparece una ruptura reciente en los datos;
- son posibles varios escenarios futuros.

## Buen uso

El valor central sirve para leer la tendencia.

El rango de incertidumbre sirve para leer el riesgo.

Una decisión seria debe considerar ambos.
