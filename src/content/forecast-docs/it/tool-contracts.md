# Tool Forecast

I sette tool Forecast formano un flusso controllato. I risultati grandi restano nello storage Forecast e il LLM scambia identificatori compatti.

## Ordine consigliato

```text
forecast_data_audit
  → forecast_models
  → forecast
  → forecast_read
  → forecast_backtest
  → forecast_compare_models
```

Usa `forecast_analyze` dopo per note, scenari o ensemble.

## `forecast_data_audit`

Chiama questo tool prima della prima previsione di ogni dataset. Fornisci dati o file, target, data, frequenza, orizzonte e confidenza esatta.

Valida date, duplicati, periodi mancanti, valori non validi, storico, serie, futuro e anomalie. Una risposta valida restituisce `data_profile_id`.

## `forecast_models`

Controlla politica e intervalli. In Manuale verifica il modello imposto. In Auto passa `data_profile_id`, scegli un candidato e conserva `selection_id`.

Le informazioni hardware compaiono solo qui. Non arrotondare la confidenza.

## `forecast`

Esegui la previsione con profilo, target, data, orizzonte, frequenza e confidenza invariata. Aggiungi serie e covariate solo se supportate.

In Auto passa anche modello, `selection_id`, origine e motivi autorizzati. La risposta restituisce `analysis_id`.

## `forecast_read`

Ometti `analysis_id` per elencare analisi o forniscilo per leggerne una. Usa `offset` e `limit`, massimo 200 punti per pagina.

Può restituire decomposizione, anomalie residue, importanza per permutazione cronologica e drift. Non inventare sostituti.

## `forecast_backtest`

Esegui una validazione temporale limitata su un'analisi salvata. Valuta modelli e baseline Naive, Naive stagionale, Drift ed ETS negli stessi periodi.

Controlla sempre stato e fallimenti.

## `forecast_compare_models`

Leggi la classifica salvata con errori, copertura, durata, memoria osservata e stato delle baseline. Definisci migliore un modello solo con un risultato completo.

## `forecast_analyze`

Usa `annotate`, `scenario`, `scenario_update`, `scenario_delete` o `ensemble`. Crea un ensemble solo dopo un backtest multi-modello riuscito e indica ponderazione inversa al MASE e assenza di valutazione indipendente.

## Riavviare il flusso

Ripeti `forecast_data_audit` e `forecast_models` quando cambiano dati, mappatura, target, frequenza, orizzonte, confidenza, covariate, struttura delle serie o risorse.
