# Datasets

Un dataset es la tabla que Forecast utiliza para aprender el pasado y predecir el futuro. Debe estar estructurado para que el modelo entienda qué predecir, en qué fechas y con qué contexto.

## Estructura mínima

Un dataset de Forecast contiene como mínimo:

| Columna | Rol |
| --- | --- |
| Fecha | Indica cuándo ocurrió cada observación |
| Objetivo | Valor a predecir |

Ejemplo sencillo:

```text
date        commandes
2026-05-01  120
2026-05-02  135
2026-05-03  128
```

Con esta tabla, Forecast puede aprender la dinámica de los pedidos y predecir las próximas fechas.

## Zona historial

La zona historial contiene los valores reales ya conocidos.

Sirve al modelo para detectar:

- tendencia;
- estacionalidad;
- ritmo;
- picos;
- caídas;
- variaciones normales.

El objetivo debe estar relleno en esta zona.

## Zona futuro

La zona futuro contiene las fechas a predecir.

En esta zona, el objetivo está vacío, porque es justamente lo que Forecast debe calcular.

Ejemplo:

```text
date        commandes
2026-05-01  120
2026-05-02  135
2026-05-03
2026-05-04
```

Aquí, Forecast debe predecir `commandes` para el 3 y el 4 de mayo.

## Futuro conocido

El futuro conocido añade variables de contexto en las líneas futuras.

Ejemplo:

```text
date        commandes   pluie_mm   promo
2026-05-01  120        0          0
2026-05-02  135        4          1
2026-05-03             12         0
2026-05-04             0          1
```

El objetivo futuro está vacío, pero la lluvia y la promo ya son conocidas o supuestas. El modelo puede usar esta información para producir una previsión más realista.

## Columna serie

La columna serie se usa cuando un mismo archivo contiene varios objetos a predecir.

Ejemplos:

- varias tiendas;
- varios productos;
- varias ciudades;
- varios activos financieros;
- varios servidores.

Ejemplo:

```text
date        magasin   ventes   promo
2026-05-01  paris    120      0
2026-05-01  lyon     92       1
2026-05-02  paris    132      1
2026-05-02  lyon     95       0
```

Forecast puede entonces predecir cada serie teniendo en cuenta el grupo al que pertenece.

## Dataset creado por un agente

Un agente LLM puede crear o enriquecer un dataset.

Puede por ejemplo:

- convertir un Excel en JSON;
- añadir una columna `weekend`;
- recuperar eventos en la web;
- transformar una información textual en puntuación;
- rellenar las líneas futuras con hipótesis;
- limpiar fechas o columnas.

El agente debe indicar claramente qué datos provienen del archivo, de la web, de un cálculo o de una hipótesis.
