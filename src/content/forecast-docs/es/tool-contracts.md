# Tools Forecast

Las tools Forecast permiten a los agentes LLM usar el motor de predicción desde el chat. Deben llamarse con parámetros precisos, ya que una columna errónea o un horizonte incoherente produce una previsión inútil.

## `forecast`

`forecast` lanza una nueva predicción.

Entradas principales:

| Parámetro | Rol |
| --- | --- |
| `file_path` | Archivo Excel, CSV o JSON a leer |
| `data` | Datos ya preparados en JSON |
| `date_column` | Columna que contiene las fechas |
| `target_column` | Columna a predecir |
| `series_column` | Columna que identifica las series |
| `covariate_columns` | Variables de contexto a usar |
| `frequency` | Ritmo temporal |
| `horizon` | Número de puntos futuros |
| `model` | Motor a usar |

Salida principal:

- `analysis_id`, el identificador del resultado Forecast.

## `forecast_read`

`forecast_read` revisa un resultado Forecast.

Sirve para recuperar:

- la previsión;
- el historial;
- la incertidumbre;
- los escenarios;
- las variables disponibles;
- los metadatos del modelo.

Si no se proporciona ningún `analysis_id`, el agente puede usarlo para listar los resultados disponibles.

## `forecast_analyze`

`forecast_analyze` añade o modifica elementos en torno a una previsión.

Sirve en particular para:

- crear una anotación;
- crear un escenario;
- relanzar un escenario contextual;
- modificar un escenario;
- suprimir un escenario.

## Lo que el agente debe verificar

Antes de llamar a una tool, el agente debe verificar:

- que el objetivo existe;
- que la fecha es legible;
- que el horizonte corresponde a las líneas futuras;
- que las covariables existen realmente;
- que los datos creados o encontrados en la web están identificados;
- que el modelo elegido soporta la necesidad.

El agente debe explicar sus elecciones en lugar de enviar una llamada opaca.
