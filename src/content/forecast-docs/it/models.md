# Modelli

Un modello è il motore che calcola la previsione. Forecast propone diverse famiglie di modelli, locali (il calcolo resta sulla macchina) o cloud (i dati utili vengono inviati al provider configurato).

## Famiglie locali

| Famiglia | Editore | Dettaglio |
| --- | --- | --- |
| Chronos / Chronos-Bolt | Amazon | Modello locale veloce, adatto a un primo test o a un target semplice |
| TimesFM | Google | Modello locale di previsione di serie temporali |
| Toto 2.0 | Datadog | Modello locale orientato al monitoring e alle metriche |
| MOIRAI 2.0 | Salesforce | Modello locale, gestisce il multi-serie e le covariate |
| FlowState | IBM | Modello locale per serie temporali |
| TabPFN-TS | PriorLabs | Modello locale sperimentale |
| TiRex | NX-AI | Modello locale sperimentale |
| Kairos | Foundation Model Research | Modello locale sperimentale |
| Sundial | THUML | Modello locale sperimentale |

## Famiglia cloud

| Famiglia | Editore | Dettaglio |
| --- | --- | --- |
| TimeGPT-2 / TimeGPT-2.1 | Nixtla | Motore cloud specializzato in serie temporali. Richiede una chiave API e invia i dati utili al provider. |

I modelli cloud possono essere più potenti, ma comportano una dipendenza esterna e un invio di dati. Per dati sensibili, preferire un modello locale.

## Scegliere un modello

La scelta dipende soprattutto dal dataset e dal caso d'uso:

- **Test rapido, target semplice**: Chronos-Bolt.
- **Dati sensibili, calcolo locale**: qualsiasi famiglia locale.
- **Covariate e contesto futuro**: un modello che gestisce le variabili di contesto (MOIRAI 2.0, Chronos-2, TimeGPT).
- **Multi-serie**: un modello che gestisce più serie (MOIRAI 2.0, Chronos-2, TimeGPT).
- **Qualità cloud avanzata**: TimeGPT, accettando l'invio dei dati.

Un modello avanzato non compensa dati mal strutturati. Prima di cambiare modello, verificare la qualità del dataset, la frequenza, l'orizzonte e le variabili di contesto.

## Installare un modello locale

I modelli locali devono essere installati dal gestore dei modelli (Impostazioni → Forecast) o tramite la scheda modelli dello spazio Forecast. Vengono scaricati da Hugging Face o GitHub a seconda della famiglia, poi memorizzati localmente in `~/.local/share/cl-go-dash/forecast-models/`.
