# Covariables

Las covariables son las variables de contexto que pueden influir en el objetivo. Describen el entorno alrededor del dato a predecir.

## Definición

Una covariable responde a la pregunta: "¿qué puede ayudar al modelo a entender por qué el objetivo sube, baja o cambia de ritmo?"

Ejemplos:

| Dominio | Objetivo | Covariables posibles |
| --- | --- | --- |
| Restaurante | Pedidos | clima, festivo, evento local, promo |
| Finanzas | Precio o volumen | VIX, tasas, news score, Bitcoin, índice dólar |
| SaaS | Carga servidor | usuarios activos, campaña, incidente, release |
| Retail | Ventas | stock, descuento, temporada, tráfico tienda |

## Por qué son importantes

Sin covariable, el modelo mira sobre todo el pasado del objetivo.

Con covariables, el modelo también puede usar el contexto:

- "las ventas suelen subir cuando hay una promo activa";
- "los pedidos bajan cuando llueve fuerte";
- "el precio reacciona cuando el VIX sube";
- "la carga aumenta tras una campaña de marketing".

Permiten por tanto producir una previsión más contextualizada que una simple prolongación del pasado.

## Tipos de variables de contexto

Una covariable puede ser:

| Tipo | Ejemplo | Lectura |
| --- | --- | --- |
| Numérica | `temperature = 28` | Valor medido |
| Porcentaje | `discount_pct = 15` | Intensidad de un efecto |
| Binaria | `weekend = 1` | Sí / no |
| Eventual | `concert_local = 1` | Evento presente |
| Puntuación | `news_score = 0.72` | Indicador calculado |
| Calendario | `jour_ferie = 1` | Contexto temporal |

## Historial y futuro conocido

Una covariable es más útil si existe en dos zonas:

- historial: el modelo aprende cómo influyó en el objetivo;
- futuro conocido: el modelo usa sus valores futuros para predecir el objetivo.

Ejemplo:

```text
date        commandes   pluie_mm   weekend
2026-05-01  120        0          0
2026-05-02  148        4          1
2026-05-03             12         1
```

El objetivo futuro está vacío, pero `pluie_mm` y `weekend` son conocidos. El modelo puede por tanto prever teniendo en cuenta estas condiciones futuras.

## Variables en los escenarios

En un escenario contextual, el usuario modifica las covariables futuras.

Ejemplos:

- subir `vix_close` un 20 %;
- poner `promo_active` a 1 durante una semana;
- bajar `temperature` 5 grados;
- simular un `breach_alert_level` alto;
- cambiar `trafic_indice` en los días futuros.

Forecast relanza entonces el modelo con este nuevo contexto para producir una nueva trayectoria.

## Diccionario de las variables finance

En un dataset financiero, las variables de contexto pueden tener nombres técnicos. Esta tabla explica las variables visibles en los escenarios finance.

| Variable | Qué representa | Lectura sencilla |
| --- | --- | --- |
| `nasdaq_return_pct` | Variación del Nasdaq en porcentaje | Mide si el mercado tech sube o baja |
| `vix_close` | Nivel del VIX al cierre | Mide el miedo o la volatilidad del mercado |
| `btc_close_usd` | Precio del Bitcoin en dólares | Sirve de señal de riesgo o de apetito especulativo |
| `usd_index_dxy` | Índice del dólar americano | Mide la fuerza del dólar frente a otras divisas |
| `treasury_10y_pct` | Tasa americana a 10 años | Representa el coste del dinero a largo plazo |
| `sector_etf_volume_musd` | Volumen negociado en un ETF sectorial, en millones de dólares | Mide la actividad en un sector concreto |
| `breach_alert_level` | Nivel de alerta ligado a una falla o un incidente cyber | Representa un estrés específico del sector ciberseguridad |
| `zero_day_news_score` | Puntuación de actualidad sobre fallas zero-day | Mide la intensidad de las noticias cyber críticas |
| `gov_contract_flow_index` | Índice de flujo de contratos públicos | Representa la dinámica de los contratos gubernamentales |
| `earnings_heat_index` | Índice de tensión en torno a los resultados financieros | Mide la importancia o la sensibilidad del periodo de resultados |
| `ai_capex_signal` | Señal de inversión ligada a los gastos IA | Representa la fuerza del tema inversión IA |
| `fed_event_flag` | Indicador de evento del banco central americano | Suele valer 1 cuando hay un evento Fed |
| `option_expiry_flag` | Indicador de expiración de opciones | Señala un día en el que las opciones pueden influir en los movimientos |
| `month_end_flag` | Indicador de fin de mes | Señala efectos de reequilibrio o cierre mensual |
| `weekend` | Indicador de fin de semana | Sirve sobre todo para datos diarios con efecto calendario |

## Cómo leer estas variables

Estas variables no predicen solas. Dan contexto al modelo.

Ejemplos:

- si `vix_close` sube, el mercado suele estar más nervioso;
- si `fed_event_flag` vale 1, el día puede ser más sensible a los anuncios de tasas;
- si `zero_day_news_score` sube, los valores cyber pueden reaccionar;
- si `sector_etf_volume_musd` sube, la actividad del sector es más fuerte;
- si `month_end_flag` vale 1, algunos movimientos pueden provenir de cierres mensuales.

El modelo aprende en el historial si estas variables ya acompañaron movimientos del objetivo.

## Uso en un escenario

En la pestaña Escenarios, modificar estas variables equivale a plantear una hipótesis.

Ejemplos:

| Hipótesis | Modificación posible |
| --- | --- |
| Mercado más estresado | `vix_close` +20 % |
| Dólar más fuerte | `usd_index_dxy` +2 % |
| Día Fed | `fed_event_flag` = 1 |
| Fuerte actualidad cyber | `zero_day_news_score` +30 % |
| Fin de mes sensible | `month_end_flag` = 1 |

Tras el relanzamiento, Forecast recalcula la trayectoria con este nuevo contexto futuro.

## Trampas que evitar

Una covariable puede degradar el resultado si está mal preparada.

A evitar:

- variable vacía en el futuro conocido;
- variable constante que no aporta ninguna información;
- texto libre no transformado en valor explotable;
- variable que contiene indirectamente el objetivo futuro;
- unidades mezcladas en una misma columna;
- valor futuro inventado sin hipótesis clara.
