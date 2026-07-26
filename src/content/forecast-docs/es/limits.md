# Límites

Forecast predice una trayectoria probable. No garantiza que el futuro llegue exactamente como se ha previsto.

## Lo que hay que recordar

Una previsión depende siempre:

- de la calidad del historial;
- de la estabilidad del fenómeno observado;
- de las variables de contexto disponibles;
- del horizonte solicitado;
- del modelo elegido.

Si los datos son débiles, incoherentes o incompletos, la previsión será frágil.

## Lo que Forecast no puede garantizar

Forecast no garantiza:

- un resultado futuro cierto;
- una causalidad real entre dos variables;
- una buena predicción tras una ruptura brutal;
- una interpretación de negocio automática;
- una fiabilidad idéntica en todos los dominios.

Ejemplo: si una crisis, un fallo, una guerra de precios o un evento excepcional ocurre sin estar representado en los datos, el modelo puede subestimar el cambio.

## Modelos locales y cloud

Los modelos locales mantienen el cálculo en la máquina.

Los modelos cloud requieren enviar los datos útiles al provider configurado. Conviene por tanto evitar enviar datos sensibles si no es aceptable para el caso de uso.

## Buena práctica

Una previsión debe usarse como ayuda a la decisión.

La buena lectura es:

- tendencia probable;
- horquilla de riesgo;
- variables que pueden explicar el movimiento;
- escenarios alternativos;
- decisión humana final.
