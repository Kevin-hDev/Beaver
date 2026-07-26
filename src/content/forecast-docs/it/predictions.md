# Previsioni

Una previsione estende una o più serie usando storico, variabili contestuali e modello selezionato. Include una stima centrale e limiti di incertezza quando disponibili.

## Risultato salvato

Ogni esecuzione valida crea un `analysis_id` che collega pannello, spazio Forecast, scenari, note, valutazioni ed esportazioni.

Prima del salvataggio, Forecast verifica quantità, date, ordine, valori finiti, quantili e orizzonte. Un output parziale o incoerente non viene salvato come analisi valida.

## Grafico principale

Il grafico separa storico e area prevista. I filtri controllano serie, incertezza, scenari, eventi, confronti, anomalie e segnali di qualità.

Puoi trascinare, usare rotella o trackpad per lo zoom, usare le barre di salto, comprimere le schede e aprire la tabella. Lo zoom non blocca lo scorrimento quando non può più cambiare.

## Grafici complementari

Lo spazio può mostrare un ventaglio d'incertezza, un confronto stagionale e, dopo un backtest, un grafico di affidabilità. Nelle analisi multi-serie, la serie attiva resta sincronizzata.

## Tabella delle previsioni

La tabella è chiusa per impostazione predefinita. Una volta aperta mostra date, valore centrale e limiti in un'area scorrevole limitata.

Per analisi lunghe, `forecast_read` restituisce pagine limitate invece dell'intera serie nel contesto del LLM.

## Aggiornamento in tempo reale

Pannello e spazio Forecast leggono la stessa analisi. Nuove previsioni, modifiche e cambi di analisi aggiornano le viste senza riaprire la finestra.

## Interpretazione corretta

Leggi la curva con qualità dei dati, incertezza, orizzonte, anomalie, backtest, baseline e ipotesi. Una curva regolare non dimostra precisione.
