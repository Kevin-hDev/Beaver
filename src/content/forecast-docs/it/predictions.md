# Previsioni

Una previsione è il risultato calcolato da Forecast per stimare i prossimi valori di un target. Risponde a una domanda concreta: "se i dati passati e il contesto noto restano coerenti, quali valori ci si può aspettare in seguito?"

## Cosa rappresenta una previsione

Una previsione contiene una serie di punti futuri.

Ciascun punto corrisponde a una data o a un periodo:

```text
2026-06-01 -> 142 ordini previsti
2026-06-02 -> 151 ordini previsti
2026-06-03 -> 149 ordini previsti
```

Questi valori non sono una certezza. Rappresentano la stima del modello.

## Input necessari

Per lanciare una previsione, Forecast ha bisogno di:

| Elemento | Ruolo |
| --- | --- |
| Data | Posiziona ciascuna riga nel tempo |
| Target | Valore da prevedere |
| Frequenza | Ritmo dei dati: giorno, ora, mese, ecc. |
| Orizzonte | Numero di punti futuri da prevedere |
| Modello | Motore utilizzato per calcolare la traiettoria |

Le variabili di contesto e il multi-serie non sono obbligatori, ma diventano importanti non appena si vuole spiegare o simulare il futuro.

## Orizzonte

L'orizzonte indica la profondità della previsione.

Esempi:

- orizzonte `24` con frequenza oraria: prevedere le prossime 24 ore;
- orizzonte `31` con frequenza quotidiana: prevedere i prossimi 31 giorni;
- orizzonte `12` con frequenza mensile: prevedere i prossimi 12 mesi.

Più l'orizzonte è lungo, più l'incertezza aumenta generalmente.

## Risultato e identificatore

Ogni lancio produce un identificatore `analysis_id`.

Questo identificatore non significa "file salvato". Serve a ritrovare il risultato calcolato: curva futura, incertezza, parametri, variabili, scenari e annotazioni.

L'applicazione lo utilizza per:

- riaprire una previsione;
- visualizzare il grafico;
- confrontare più risultati;
- creare o rilanciare scenari;
- permettere a un agente LLM di rileggere il risultato.

## Interpretazione corretta

Una previsione deve essere letta con tre domande:

- la tendenza sale, scende o resta stabile?
- l'incertezza è piccola o ampia?
- quali variabili di contesto possono spiegare il movimento?

Una curva da sola non basta. Forecast diventa utile quando la previsione è collegata al suo contesto.
