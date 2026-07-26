# Dataset

Un dataset è la tabella che Forecast utilizza per apprendere il passato e prevedere il futuro. Deve essere strutturato in modo che il modello capisca cosa prevedere, in quali date e con quale contesto.

## Struttura minima

Un dataset Forecast contiene come minimo:

| Colonna | Ruolo |
| --- | --- |
| Data | Indica quando ha avuto luogo ciascuna osservazione |
| Target | Valore da prevedere |

Esempio semplice:

```text
date        commandes
2026-05-01  120
2026-05-02  135
2026-05-03  128
```

Con questa tabella, Forecast può apprendere la dinamica degli ordini e prevedere le prossime date.

## Zona storica

La zona storica contiene i valori reali già noti.

Serve al modello per rilevare:

- tendenza;
- stagionalità;
- ritmo;
- picchi;
- cali;
- variazioni normali.

Il target deve essere compilato in questa zona.

## Zona futura

La zona futura contiene le date da prevedere.

In questa zona, il target è vuoto, perché è proprio ciò che Forecast deve calcolare.

Esempio:

```text
date        commandes
2026-05-01  120
2026-05-02  135
2026-05-03
2026-05-04
```

Qui, Forecast deve prevedere `commandes` per il 3 e il 4 maggio.

## Futuro noto

Il futuro noto aggiunge variabili di contesto sulle righe future.

Esempio:

```text
date        commandes   pluie_mm   promo
2026-05-01  120        0          0
2026-05-02  135        4          1
2026-05-03             12         0
2026-05-04             0          1
```

Il target futuro è vuoto, ma la pioggia e la promo sono già note o ipotizzate. Il modello può usare queste informazioni per produrre una previsione più realistica.

## Colonna serie

La colonna serie serve quando uno stesso file contiene più oggetti da prevedere.

Esempi:

- più negozi;
- più prodotti;
- più città;
- più asset finanziari;
- più server.

Esempio:

```text
date        magasin   ventes   promo
2026-05-01  paris    120      0
2026-05-01  lyon     92       1
2026-05-02  paris    132      1
2026-05-02  lyon     95       0
```

Forecast può così prevedere ciascuna serie tenendo conto del gruppo a cui appartiene.

## Dataset creato da un agente

Un agente LLM può creare o arricchire un dataset.

Può per esempio:

- convertire un Excel in JSON;
- aggiungere una colonna `weekend`;
- recuperare eventi sul web;
- trasformare un'informazione testuale in punteggio;
- compilare le righe future con delle ipotesi;
- pulire date o colonne.

L'agente deve indicare chiaramente quali dati provengono dal file, dal web, da un calcolo o da un'ipotesi.
