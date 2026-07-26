# Incertidumbre

Una previsión seria no se limita a una curva. Forecast asocia el valor central a un intervalo que representa la incertidumbre para el nivel de confianza solicitado.

## Valor central

El valor central suele ser la mediana `q50`. Aproximadamente la mitad de los resultados posibles queda por debajo y la otra mitad por encima.

## Nivel de confianza

Los modelos continuos aceptan entre 50% y 99%, por pasos de un punto porcentual. Sin preferencia del usuario, el LLM utiliza 80%.

Algunos modelos solo ofrecen niveles fijos, actualmente 60% u 80%. Forecast conserva la solicitud exacta: Auto devuelve candidatos compatibles, Manual informa de la incompatibilidad y nunca se redondea en silencio.

## Límites y cuantiles

Un intervalo central del 80% suele usar `q10`, `q50` y `q90`. Para 90% suele usar `q05`, `q50` y `q95`.

## Abanico de incertidumbre

El abanico muestra cómo los intervalos se amplían o reducen. Límites más anchos significan menor precisión. Un intervalo estrecho solo es útil si está bien calibrado.

## Cobertura medida

Tras el backtest, Forecast compara el nivel anunciado con la proporción de valores realmente cubiertos. Un historial corto puede volver inestable esta medida.

## Buen uso

Usa la incertidumbre para comparar riesgos, distinguir tendencias robustas, preparar umbrales prudentes, verificar la calibración y comparar escenarios sin confundir hipótesis y certeza.
