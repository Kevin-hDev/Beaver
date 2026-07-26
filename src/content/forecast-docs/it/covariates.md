# Covariate

Le covariate sono le variabili di contesto che possono influenzare il target. Descrivono l'ambiente attorno al dato da prevedere.

## Definizione

Una covariata risponde alla domanda: "cosa può aiutare il modello a capire perché il target sale, scende o cambia ritmo?"

Esempi:

| Dominio | Target | Covariate possibili |
| --- | --- | --- |
| Ristorante | Ordini | meteo, festivo, evento locale, promo |
| Finanza | Prezzo o volume | VIX, tassi, news score, Bitcoin, indice dollaro |
| SaaS | Carico server | utenti attivi, campagna, incidente, release |
| Retail | Vendite | stock, sconto, stagione, traffico negozio |

## Perché sono importanti

Senza covariate, il modello guarda soprattutto il passato del target.

Con le covariate, il modello può anche usare il contesto:

- "le vendite salgono spesso quando una promo è attiva";
- "gli ordini scendono quando piove forte";
- "il prezzo reagisce quando il VIX aumenta";
- "il carico aumenta dopo una campagna marketing".

Permettono quindi di produrre una previsione più contestualizzata rispetto a un semplice prolungamento del passato.

## Tipi di variabili di contesto

Una covariata può essere:

| Tipo | Esempio | Lettura |
| --- | --- | --- |
| Numerica | `temperature = 28` | Valore misurato |
| Percentuale | `discount_pct = 15` | Intensità di un effetto |
| Binaria | `weekend = 1` | Sì / no |
| Eventuale | `concert_local = 1` | Evento presente |
| Punteggio | `news_score = 0.72` | Indicatore calcolato |
| Calendario | `jour_ferie = 1` | Contesto temporale |

## Storico e futuro noto

Una covariata è più utile se esiste in due zone:

- storico: il modello apprende come ha influenzato il target;
- futuro noto: il modello usa i suoi valori futuri per prevedere il target.

Esempio:

```text
date        commandes   pluie_mm   weekend
2026-05-01  120        0          0
2026-05-02  148        4          1
2026-05-03             12         1
```

Il target futuro è vuoto, ma `pluie_mm` e `weekend` sono noti. Il modello può quindi prevedere tenendo conto di queste condizioni future.

## Variabili negli scenari

In uno scenario contestuale, l'utente modifica le covariate future.

Esempi:

- aumentare `vix_close` del 20%;
- impostare `promo_active` a 1 per una settimana;
- abbassare `temperature` di 5 gradi;
- simulare un `breach_alert_level` alto;
- cambiare `trafic_indice` sui giorni futuri.

Forecast rilancia allora il modello con questo nuovo contesto per produrre una nuova traiettoria.

## Dizionario delle variabili finanza

In un dataset finanziario, le variabili di contesto possono avere nomi tecnici. Questa tabella spiega le variabili visibili negli scenari finanza.

| Variabile | Cosa rappresenta | Lettura semplice |
| --- | --- | --- |
| `nasdaq_return_pct` | Variazione del Nasdaq in percentuale | Misura se il mercato tech sale o scende |
| `vix_close` | Livello del VIX alla chiusura | Misura la paura o la volatilità del mercato |
| `btc_close_usd` | Prezzo del Bitcoin in dollari | Serve come segnale di rischio o di appetito speculativo |
| `usd_index_dxy` | Indice del dollaro americano | Misura la forza del dollaro rispetto ad altre valute |
| `treasury_10y_pct` | Tasso americano a 10 anni | Rappresenta il costo del denaro a lungo termine |
| `sector_etf_volume_musd` | Volume scambiato su un ETF di settore, in milioni di dollari | Misura l'attività su un settore specifico |
| `breach_alert_level` | Livello di allerta legato a una falla o a un incidente cyber | Rappresenta uno stress specifico del settore cybersicurezza |
| `zero_day_news_score` | Punteggio di attualità attorno alle falle zero-day | Misura l'intensità delle notizie cyber critiche |
| `gov_contract_flow_index` | Indice di flusso di contratti pubblici | Rappresenta la dinamica dei contratti governativi |
| `earnings_heat_index` | Indice di tensione attorno ai risultati finanziari | Misura l'importanza o la sensibilità del periodo dei risultati |
| `ai_capex_signal` | Segnale di investimento legato alle spese IA | Rappresenta la forza del tema investimento IA |
| `fed_event_flag` | Indicatore di evento della banca centrale americana | Vale spesso 1 quando è presente un evento Fed |
| `option_expiry_flag` | Indicatore di scadenza opzioni | Segnala una giornata in cui le opzioni possono influenzare i movimenti |
| `month_end_flag` | Indicatore di fine mese | Segnala effetti di ribilanciamento o chiusura mensile |
| `weekend` | Indicatore weekend | Serve soprattutto per dati giornalieri con effetto calendario |

## Come leggere queste variabili

Queste variabili non prevedono nulla da sole. Danno contesto al modello.

Esempi:

- se `vix_close` sale, il mercato è spesso più nervoso;
- se `fed_event_flag` vale 1, la giornata può essere più sensibile agli annunci sui tassi;
- se `zero_day_news_score` sale, i valori cyber possono reagire;
- se `sector_etf_volume_musd` sale, l'attività del settore è più forte;
- se `month_end_flag` vale 1, alcuni movimenti possono derivare da chiusure mensili.

Il modello apprende dallo storico se queste variabili hanno già accompagnato dei movimenti del target.

## Utilizzo in uno scenario

Nella scheda Scenari, modificare queste variabili equivale a porre un'ipotesi.

Esempi:

| Ipotesi | Modifica possibile |
| --- | --- |
| Mercato più stressato | `vix_close` +20% |
| Dollaro più forte | `usd_index_dxy` +2% |
| Giornata Fed | `fed_event_flag` = 1 |
| Forte attualità cyber | `zero_day_news_score` +30% |
| Fine mese sensibile | `month_end_flag` = 1 |

Dopo il rilancio, Forecast ricalcola la traiettoria con questo nuovo contesto futuro.

## Trappole da evitare

Una covariata può degradare il risultato se è mal preparata.

Da evitare:

- variabile vuota sul futuro noto;
- variabile costante che non apporta alcuna informazione;
- testo libero non trasformato in valore sfruttabile;
- variabile che contiene indirettamente il target futuro;
- unità mescolate in una stessa colonna;
- valore futuro inventato senza un'ipotesi chiara.
