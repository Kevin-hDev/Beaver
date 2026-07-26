# Scenari

Uno scenario serve a testare un'ipotesi sul futuro. Confronta la previsione base con una traiettoria modificata.

## Perché creare uno scenario

La previsione base risponde a: "cosa prevede il modello con i dati attuali?"

Uno scenario risponde a: "cosa succede se il contesto cambia?"

Esempi:

- cosa diventano le vendite se viene lanciata una promozione?
- cosa diventa il fatturato se il traffico scende?
- cosa diventa un asset se il VIX aumenta?
- cosa diventa la domanda se viene aggiunto un evento locale?

Lo scenario trasforma Forecast in uno strumento di simulazione, non solo in un grafico di previsione.

## Adeguamento globale

L'adeguamento globale applica una variazione semplice alla curva.

Esempio:

```text
Previsione base   : 100, 110, 120
Scenario +10%     : 110, 121, 132
```

Questa modalità è rapida e leggibile. Non rilancia il modello, quindi non comprende le relazioni tra variabili.

## Scenario contestuale

Lo scenario contestuale modifica le variabili future, poi rilancia il modello.

Esempio:

```text
Ipotesi: vix_close +20%
Effetto atteso: il modello ricalcola il target con questo contesto di mercato più stressato.
```

Questa modalità è più importante per Chronos-2 e TimeGPT, perché usa le covariate come veri segnali di previsione.

## Variabili modificabili

Le variabili disponibili dipendono dal dataset.

Possono rappresentare:

- ambiente: meteo, traffico, eventi;
- finanza: volatilità, tassi, indici, news score;
- calendario: weekend, festivo, fine mese;
- business: promo, stock, budget, campagna;
- rischio: allerta, incidente, pressione concorrenziale.

Ogni modifica deve avere un senso aziendale. Modificare una variabile a caso produce uno scenario difficile da interpretare.

## Lettura nel grafico

Quando uno scenario è selezionato, il grafico deve permettere di confrontare:

- storico reale;
- previsione base;
- previsione dello scenario;
- variabili di contesto visualizzate;
- differenza tra base e scenario.

Schema tipico di un confronto:

```text
valore
  ^
  |              ╭───── scenario (VIX +20%)
  |           ╭──╯
  |       · ·─·      ← previsione base
  |     ·
  |   ·
  | ·
  ──────────────────────────────> tempo
       storico       │   futuro
                     │
                orizzonte
```

La domanda principale non è "la curva è diversa?", ma "quale ipotesi ha spostato la traiettoria, in quale data e di quanto?"

## Buon uso

Un buon scenario deve essere nominato chiaramente.

Esempi:

- `VIX +20% per 30 giorni`
- `Promo weekend attiva`
- `Pioggia forte settimana 2`
- `Traffico -15% dopo incidente`

Un nome vago come `test` o `crash` rende il confronto inutile quando esistono più scenari.
