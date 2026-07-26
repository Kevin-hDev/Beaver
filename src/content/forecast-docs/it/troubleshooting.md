# Diagnostica

Questa sezione distingue il comportamento normale dai problemi che richiedono un'azione.

## Preparazione del modello

Un modello può essere Non installato, Aggiornamento richiesto, Non valido, Pronto o Provider richiesto. Prepara i modelli mancanti o vecchi, reinstalla quelli non validi e configura il provider cloud.

Più preparazioni entrano in coda e i file validi vengono riutilizzati.

## Ciclo del sidecar

Il runtime locale si avvia per una previsione o un backtest e può fermarsi subito dopo. È normale e libera risorse.

C'è un problema solo se non diventa pronto, la richiesta fallisce o Forecast restituisce un errore.

## Audit rifiutato

Può dipendere da colonne mancanti, date non valide o duplicate, frequenza incoerente, storico insufficiente, futuro errato o limiti superati.

Correggi il problema e ripeti l'audit. Non usare un vecchio profilo dopo una modifica dei dati.

## Confidenza incompatibile

I modelli continui accettano valori interi dal 50% al 99%; alcuni modelli fissi solo 60% o 80%.

In Manuale cambia livello o modello. In Auto ripeti la selezione con il valore esatto. Non arrotondarlo.

## Selezione Auto scaduta

La selezione è legata a dataset, sessione e risorse. Se scade, richiama `forecast_models`, ottieni un nuovo identificatore e ripeti `forecast`.

## Risultato assente

Controlla che Forecast abbia restituito `analysis_id`, seleziona l'analisi nello storico, verifica la sessione e rileggila. Un output rifiutato non viene mostrato come valido.

## Backtest parziale

Controlla stato generale e fallimenti individuali. Non considerare completa la classifica finché i modelli confrontati non hanno risultati omogenei.

## Covariate ignorate

Una covariata può mancare, essere vuota nel futuro, costante, mal tipizzata, disallineata o non supportata. Controlla Dati, modello e valori futuri.

## Risultato piatto o scenario debole

Una curva piatta può riflettere target stabile, storico corto, frequenza errata o contesto assente. Uno scenario può avere poco effetto se la modifica è piccola o il livello è nascosto.
