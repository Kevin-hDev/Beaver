# Scenari

Uno scenario esplora un'ipotesi da un'analisi esistente. Non sostituisce dati osservati o previsione originale.

## Regolazione globale

Una variazione percentuale crea una curva derivata, ad esempio domanda +10%, ricavi -5% o capacità +15%. Non riesegue il modello.

## Scenario contestuale

Uno scenario contestuale modifica covariate future e riesegue il modello quando supportate. Può cambiare budget, prezzo, meteo, capacità o una serie specifica.

I valori modificati restano ipotesi.

## Creazione e modifica

Lo spazio Forecast raggruppa creazione, modifica ed eliminazione. Il pannello mantiene la lettura rapida. Il LLM può gestire gli scenari con `forecast_analyze`.

## Confrontare le curve

Confronta previsione originale e scenari nello stesso periodo. Controlla inizio e ampiezza della divergenza, incertezza, serie coinvolte e covariate modificate.

## Ensemble di modelli

Un ensemble non è uno scenario aziendale. Combina da due a quattro modelli con backtest riuscito, ponderati con l'inverso del MASE, ed è indicato come non valutato indipendentemente.

## Buon uso

Assegna a ogni scenario nome chiaro, ipotesi misurabile, periodo, fonte dei valori, spiegazione e confronto con la previsione originale.
