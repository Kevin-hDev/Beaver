# Diagnóstico

Esta sección ayuda a entender por qué una previsión no da el resultado esperado.

## Datos JSON no válidos

Este error significa que Forecast no ha recibido una tabla explotable.

Puede provenir de:

- JSON mal formado;
- archivo no convertido correctamente;
- campo `data` vacío o truncado;
- formato de línea incorrecto;
- columnas ausentes.

Si el usuario proporciona un archivo, el agente debe verificar que el archivo se lee bien antes de convertir los datos.

## Modelo no disponible

Este error puede provenir de:

- modelo local no instalado;
- sidecar detenido;
- clave API ausente;
- modelo incompatible con los parámetros;
- datos demasiado cortos para el modelo solicitado.

El buen reflejo es verificar el modelo y luego probar con un dataset mínimo.

## Variables de contexto ignoradas

Una variable puede ser ignorada o inútil si:

- no existe en el historial;
- está vacía en el futuro;
- es constante;
- está mal tipada;
- no corresponde al horizonte;
- contiene texto no transformado en número o categoría explotable.

En este caso, hay que inspeccionar el dataset antes de acusar al modelo.

## Resultado plano

Una previsión plana puede ser normal si el objetivo es estable.

También puede indicar:

- historial demasiado corto;
- frecuencia mal elegida;
- contexto ausente;
- objetivo poco variable;
- modelo demasiado sencillo;
- futuro conocido poco informativo.

## Escenario sin efecto visible

Un escenario contextual puede mostrar poca diferencia si:

- la variable modificada tiene poca influencia;
- la modificación es demasiado débil;
- el modelo no usa esta variable como señal fuerte;
- la variable futura no se ha transmitido realmente;
- la curva está oculta en los filtros.

Hay que verificar el gráfico, los filtros, el tooltip y los datos del escenario.
