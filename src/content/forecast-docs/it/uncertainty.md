# Incertezza

Una previsione non è mai una verità assoluta. Forecast mostra quindi un valore centrale e una fascia di incertezza per indicare il rischio attorno alla traiettoria.

## Valore centrale

Il valore centrale è la stima principale del modello.

Esempio:

```text
2026-06-01 -> 142 ordini previsti
```

Rappresenta lo scenario più probabile in base ai dati utilizzati.

## Limiti di incertezza

I limiti indicano una zona probabile attorno al valore centrale.

Esempio:

```text
Previsione: 142
Limite basso: 128
Limite alto: 157
```

Lettura semplice: il modello stima 142, ma considera che un valore attorno a 128-157 resti plausibile.

## Quantili

I modelli possono restituire dei quantili.

| Campo | Significato |
| --- | --- |
| q10 | Valore basso probabile |
| q50 | Valore centrale o mediano |
| q90 | Valore alto probabile |

Più lo scarto tra q10 e q90 è ampio, più il modello è incerto.

## Perché l'incertezza aumenta

L'incertezza può aumentare quando:

- lo storico è corto;
- il target varia fortemente;
- l'orizzonte è lungo;
- le variabili di contesto sono assenti;
- compare una rottura recente nei dati;
- sono possibili più scenari futuri.

## Buon uso

Il valore centrale serve a leggere la tendenza.

La fascia di incertezza serve a leggere il rischio.

Una decisione seria deve tenere conto di entrambi.
