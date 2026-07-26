# Tool Forecast

I tool Forecast permettono agli agenti LLM di usare il motore di previsione dalla chat. Devono essere chiamati con parametri precisi, perché una colonna sbagliata o un orizzonte incoerente produce una previsione inutile.

## `forecast`

`forecast` lancia una nuova previsione.

Input principali:

| Parametro | Ruolo |
| --- | --- |
| `file_path` | File Excel, CSV o JSON da leggere |
| `data` | Dati già preparati in JSON |
| `date_column` | Colonna che contiene le date |
| `target_column` | Colonna da prevedere |
| `series_column` | Colonna che identifica le serie |
| `covariate_columns` | Variabili di contesto da usare |
| `frequency` | Ritmo temporale |
| `horizon` | Numero di punti futuri |
| `model` | Motore da usare |

Output principale:

- `analysis_id`, l'identificatore del risultato Forecast.

## `forecast_read`

`forecast_read` rilegge un risultato Forecast.

Serve a recuperare:

- la previsione;
- lo storico;
- l'incertezza;
- gli scenari;
- le variabili disponibili;
- i metadati del modello.

Se non viene fornito alcun `analysis_id`, l'agente può usarlo per elencare i risultati disponibili.

## `forecast_analyze`

`forecast_analyze` aggiunge o modifica elementi attorno a una previsione.

Serve in particolare a:

- creare un'annotazione;
- creare uno scenario;
- rilanciare uno scenario contestuale;
- modificare uno scenario;
- eliminare uno scenario.

## Cosa deve verificare l'agente

Prima di chiamare un tool, l'agente deve verificare:

- il target esiste;
- la data è leggibile;
- l'orizzonte corrisponde alle righe future;
- le covariate esistono davvero;
- i dati creati o trovati sul web sono identificati;
- il modello scelto supporta il bisogno.

L'agente deve spiegare le sue scelte invece di inviare una chiamata opaca.
