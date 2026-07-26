# Diagnostica

Questa sezione aiuta a capire perché una previsione non dà il risultato atteso.

## Dati JSON non validi

Questo errore significa che Forecast non ha ricevuto una tabella sfruttabile.

Può dipendere da:

- JSON mal formato;
- file non convertito correttamente;
- campo `data` vuoto o troncato;
- formato di riga errato;
- colonne assenti.

Se l'utente fornisce un file, l'agente deve verificare che il file sia letto correttamente prima di convertire i dati.

## Modello non disponibile

Questo errore può dipendere da:

- modello locale non installato;
- sidecar fermato;
- chiave API assente;
- modello incompatibile con i parametri;
- dati troppo corti per il modello richiesto.

La reazione giusta è verificare il modello, poi testare con un dataset minimale.

## Variabili di contesto ignorate

Una variabile può essere ignorata o inutile se:

- non esiste nello storico;
- è vuota nel futuro;
- è costante;
- è mal tipizzata;
- non corrisponde all'orizzonte;
- contiene testo non trasformato in numero o categoria sfruttabile.

In questo caso, ispezionare il dataset prima di accusare il modello.

## Risultato piatto

Una previsione piatta può essere normale se il target è stabile.

Può anche indicare:

- storico troppo corto;
- frequenza mal scelta;
- contesto assente;
- target poco variabile;
- modello troppo semplice;
- futuro noto poco informativo.

## Scenario senza effetto visibile

Uno scenario contestuale può mostrare poca differenza se:

- la variabile modificata ha poca influenza;
- la modifica è troppo debole;
- il modello non usa questa variabile come segnale forte;
- la variabile futura non è stata realmente trasmessa;
- la curva è nascosta nei filtri.

Bisogna verificare il grafico, i filtri, il tooltip e i dati dello scenario.
