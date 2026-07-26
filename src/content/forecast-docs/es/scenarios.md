# Escenarios

Un escenario sirve para probar una hipótesis sobre el futuro. Compara la previsión base con una trayectoria modificada.

## Por qué crear un escenario

La previsión base responde a: "¿qué prevé el modelo con los datos actuales?"

Un escenario responde a: "¿qué ocurre si el contexto cambia?"

Ejemplos:

- ¿qué pasan a ser las ventas si se lanza una promoción?
- ¿qué pasa a ser la facturación si el tráfico baja?
- ¿qué pasa a ser un activo si el VIX sube?
- ¿qué pasa a ser la demanda si se añade un evento local?

El escenario transforma Forecast en una herramienta de simulación, no solo en un gráfico de previsión.

## Ajuste global

El ajuste global aplica una variación sencilla a la curva.

Ejemplo:

```text
Previsión base : 100, 110, 120
Escenario +10 %  : 110, 121, 132
```

Este modo es rápido y legible. No relanza el modelo, por lo que no entiende las relaciones entre variables.

## Escenario contextual

El escenario contextual modifica las variables futuras y luego relanza el modelo.

Ejemplo:

```text
Hipótesis : vix_close +20 %
Efecto esperado : el modelo recalcula el objetivo con este contexto de mercado más estresado.
```

Este modo es más importante para Chronos-2 y TimeGPT, ya que utiliza las covariables como verdaderas señales de predicción.

## Variables modificables

Las variables disponibles dependen del dataset.

Pueden representar:

- entorno: clima, tráfico, eventos;
- finanzas: volatilidad, tasas, índices, news score;
- calendario: weekend, festivo, fin de mes;
- negocio: promo, stock, presupuesto, campaña;
- riesgo: alerta, incidente, presión competitiva.

Cada modificación debe tener un sentido de negocio. Modificar una variable al azar produce un escenario difícil de interpretar.

## Lectura en el gráfico

Cuando se selecciona un escenario, el gráfico debe permitir comparar:

- historial real;
- previsión base;
- previsión del escenario;
- variables de contexto mostradas;
- diferencia entre base y escenario.

Esquema típico de una comparación:

```text
valor
  ^
  |              ╭───── escenario (VIX +20%)
  |           ╭──╯
  |       · ·─·      ← previsión base
  |     ·
  |   ·
  | ·
  ──────────────────────────────> tiempo
       historial     │   futuro
                      │
                 horizonte
```

La pregunta principal no es "¿es la curva diferente?", sino "¿qué hipótesis ha desplazado la trayectoria, en qué fecha y en cuánto?"

## Buen uso

Un buen escenario debe tener un nombre claro.

Ejemplos:

- `VIX +20% durante 30 días`
- `Promo fin de semana activa`
- `Lluvia fuerte semana 2`
- `Tráfico -15% tras incidente`

Un nombre vago como `test` o `crash` hace inútil la comparación cuando existen varios escenarios.
