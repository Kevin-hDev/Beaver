# Modelli

Un modello è il motore che calcola la previsione. Forecast offre famiglie locali e cloud e ne verifica capacità, stato e compatibilità con le risorse prima dell'esecuzione.

## Famiglie disponibili

| Famiglia | Editore | Uso principale |
| --- | --- | --- |
| Chronos / Chronos-Bolt | Amazon | Previsioni locali rapide e probabilistiche |
| TimesFM | Google | Previsione generale di serie temporali |
| Toto 2.0 | Datadog | Metriche e monitoraggio |
| MOIRAI 2.0 | Salesforce | Multi-serie e variabili contestuali |
| FlowState | IBM | Previsione locale probabilistica |
| TabPFN-TS, TiRex, Kairos, Sundial | Vari | Modelli locali specializzati o sperimentali |
| TimeGPT | Nixtla | Previsione cloud con chiave API |

Il catalogo dell'app è il riferimento per frequenze, orizzonte, covariate, multi-serie e intervalli.

## Modalità Manuale

In Manuale scegli il modello e Forecast impone la scelta. Se non è pronto o non è compatibile con dati o confidenza esatta, il LLM chiede un'altra scelta senza sostituirlo in silenzio.

## Modalità Auto

In Auto il LLM sceglie un solo modello da una lista breve già filtrata. Forecast esclude modelli non pronti, incompatibili, troppo pesanti o cloud non autorizzati.

Le informazioni hardware vengono esposte al LLM solo durante questa selezione Forecast. Senza backtest comparabili, Auto parla di compatibilità o raccomandazione per capacità, mai del modello migliore.

## Installazione e preparazione

Prepara scarica il modello, installa il runtime ed esegue una verifica reale prima della prima previsione. Più preparazioni entrano in coda e varianti della stessa famiglia possono condividere il runtime.

| Stato | Significato |
| --- | --- |
| Non installato | Mancano i file |
| Aggiornamento richiesto | Runtime o validazione devono essere aggiornati |
| Non valido | Installazione incompleta o non validata |
| Pronto | Modello e runtime verificati |
| Provider richiesto | Manca la chiave del servizio cloud |

Un modello locale è selezionabile solo quando è pronto. Il runtime condiviso viene rimosso solo se nessun altro modello ne ha bisogno.

## Modelli cloud

Un modello cloud invia i dati necessari al provider configurato. Auto lo usa solo con autorizzazione, provider pronto e politica dati compatibile. Forecast non passa mai silenziosamente da locale a cloud.
