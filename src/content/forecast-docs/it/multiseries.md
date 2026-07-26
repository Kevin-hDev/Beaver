# Multi-serie

Il multi-serie permette di prevedere più serie in una stessa analisi. Invece di lanciare una previsione separata per ciascun oggetto, Forecast riceve un solo dataset con una colonna che identifica ogni serie.

## Esempio

Un file può contenere le vendite di più negozi:

```text
date        magasin   ventes   promo
2026-05-01  paris    120      0
2026-05-01  lyon     92       1
2026-05-02  paris    132      1
2026-05-02  lyon     95       0
```

Qui, `magasin` è la colonna serie.

Forecast capisce che deve prevedere le vendite per `paris` e per `lyon`.

## A cosa serve

Il multi-serie è utile quando più serie condividono una logica comune.

Esempi:

- vendite per negozio;
- ordini per ristorante;
- traffico per server;
- prezzo per asset;
- incidenti per regione.

Il modello può sfruttare più informazioni rispetto a una previsione isolata, soprattutto se le serie si somigliano o condividono variabili di contesto.

## Orizzonte per serie

Ciascuna serie deve fornire una struttura temporale coerente.

Se l'orizzonte è `31`, ogni serie deve avere 31 punti futuri da prevedere.

Esempio:

```text
paris -> 31 righe future
lyon  -> 31 righe future
```

Un orizzonte incoerente rende difficile il confronto e può bloccare il modello.

## Lettura in Forecast

Nell'interfaccia, l'utente può selezionare la serie visualizzata.

Gli scenari possono poi essere letti:

- su una serie precisa;
- su più serie;
- in confronto con la previsione base.

Il multi-serie non cambia il principio di Forecast. Aggiunge semplicemente una dimensione: "quale serie sta venendo prevista in questo momento?"
