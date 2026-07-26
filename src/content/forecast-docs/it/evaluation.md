# Valutazione e confronto

La valutazione misura un modello su periodi storici non visti durante il calcolo. Confronta i risultati sulle stesse finestre temporali.

## Backtest temporale scorrevole

Forecast divide lo storico in più finestre. Per ogni finestra, il modello usa solo il passato e prevede il periodo successivo, evitando informazioni future.

Se lo storico è breve, finestre o orizzonte possono essere ridotti e l'interfaccia mostra un avviso.

## Baseline

| Baseline | Principio |
| --- | --- |
| Naive | Ripete l'ultimo valore noto |
| Naive stagionale | Ripete il valore stagionale precedente comparabile |
| Drift | Estende la tendenza media |
| ETS | Modella livello, tendenza e stagionalità quando possibile |

## Metriche

| Metrica | Lettura |
| --- | --- |
| MASE | Errore rispetto a Naive; più basso è meglio |
| sMAPE | Errore relativo simmetrico; più basso è meglio |
| MAE | Errore assoluto medio nell'unità del target |
| Copertura | Valori reali inclusi nell'intervallo |
| Durata | Tempo osservato |
| Memoria | Picco osservato quando disponibile |

Confronta la copertura misurata con il livello richiesto. Un intervallo dell'80% che copre solo il 40% è mal calibrato.

## Valutazione e Confronto

Valutazione avvia il backtest e mostra i dettagli. Confronto usa risultati omogenei e presenta i compromessi tra precisione, copertura, velocità e risorse.

Non presentare un'esecuzione parziale come validazione completa.

## Modello migliore

Definisci migliore un modello solo se usa le stesse finestre, ha un risultato completo, migliora le metriche rilevanti, supera una baseline credibile e rispetta i vincoli dell'utente.

Senza backtest comparabili, parla solo di compatibilità o raccomandazione per capacità.

## Ensemble di modelli

Dopo un backtest multi-modello riuscito, Confronto può combinare da due a quattro modelli ponderati con l'inverso del MASE. L'ensemble è indicato come non valutato indipendentemente.
