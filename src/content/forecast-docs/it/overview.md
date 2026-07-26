# Panoramica

Forecast serve a prevedere l'evoluzione futura di un dato misurabile. Il modulo analizza lo storico, le tendenze recenti e le variabili di contesto per produrre una previsione datata, con un margine di incertezza e scenari confrontabili.

## Definizione semplice di forecasting

Il forecasting consiste nell'osservare dati passati e attuali per stimare cosa può accadere in seguito.

Esempi:

- prevedere le vendite dei prossimi 30 giorni;
- stimare il fatturato del mese successivo;
- anticipare il carico server delle prossime ore;
- proiettare il prezzo o il volume di un asset;
- simulare l'effetto di un contesto futuro noto.

Il modello non legge il futuro. Calcola una traiettoria probabile a partire da schemi visibili nei dati.

## Cosa Forecast aggiunge a una chat LLM

Un LLM può leggere una tabella e scrivere una spiegazione. Forecast aggiunge un motore specializzato che calcola realmente una serie futura.

La differenza è importante:

| Chat LLM da solo | Forecast |
| --- | --- |
| Spiega un file | Calcola punti futuri datati |
| Può ragionare qualitativamente | Produce una curva numerica |
| Può inventare se i dati sono confusi | Usa un contratto dati rigoroso |
| Riassume una tendenza | Genera una previsione, dei limiti e degli scenari |

L'LLM resta utile attorno al motore: prepara i dati, sceglie le colonne, può cercare informazioni sul web, costruisce un dataset, lancia Forecast, poi spiega il risultato.

## Oggetto principale: il target

Il target è la colonna che Forecast deve prevedere.

Esempi:

- `ventes`
- `ca_total_eur`
- `commandes_total`
- `temperature`
- `stock_price`
- `incidents_count`

Tutta la previsione ruota attorno a questo target: il modello apprende il suo comportamento passato, poi stima i suoi valori futuri.

## Cosa contiene un risultato Forecast

Un risultato Forecast contiene:

- i valori storici utilizzati;
- la previsione futura punto per punto;
- una fascia di incertezza;
- le variabili di contesto utilizzate;
- gli scenari creati a partire da questa previsione;
- i metadati necessari per rileggere, confrontare ed esportare il risultato.

Non è un file passivo. È un oggetto di lavoro completo per capire ciò che è probabile, ciò che è rischioso e ciò che cambia se il contesto evolve.

## Logica generale

Il workflow standard è:

1. fornire un dataset;
2. scegliere la data, il target ed eventualmente le serie;
3. selezionare le variabili di contesto utili;
4. lanciare un modello di previsione;
5. leggere la curva futura e l'incertezza;
6. creare scenari per testare delle ipotesi;
7. chiedere all'LLM di spiegare i risultati o di preparare nuovi dati.
