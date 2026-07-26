# Agenti LLM

Gli agenti LLM possono usare Forecast come motore specializzato. Il loro ruolo non si limita a leggere un file e cliccare su un tool: possono preparare i dati, cercare contesto sul web, costruire un dataset, lanciare la previsione e spiegare il risultato.

## Cosa può fare l'agente

Un agente può intervenire in più momenti:

| Fase | Ruolo dell'agente |
| --- | --- |
| Preparazione | Leggere un file Excel, CSV o JSON |
| Ricerca | Andare a cercare informazioni esterne sul web |
| Dataset | Creare o completare colonne utili |
| Lancio | Chiamare `forecast` con i parametri giusti |
| Lettura | Usare `forecast_read` per recuperare il risultato |
| Scenario | Creare ipotesi e rilanciare il modello |
| Spiegazione | Riassumere tendenza, incertezza, variabili e limiti |

Esempio: per una previsione finanziaria, l'agente può leggere il file locale, cercare il contesto di mercato recente, produrre colonne come `news_score` o `event_flag`, poi lanciare Forecast.

## Workflow consigliato

L'agente deve seguire questo ordine:

1. capire la richiesta dell'utente;
2. ispezionare i dati disponibili;
3. identificare il target da prevedere;
4. identificare le date, la frequenza e l'orizzonte;
5. cercare o creare le variabili di contesto utili se necessario;
6. verificare che le righe future siano coerenti;
7. scegliere un modello compatibile;
8. lanciare `forecast`;
9. rileggere il risultato con `forecast_read`;
10. spiegare la previsione e proporre scenari utili.

## Creazione di dati da parte dell'agente

L'agente può creare dati se l'utente lo richiede o se la previsione lo richiede.

Esempi:

- aggiungere una colonna `weekend` a partire dalla data;
- creare `month_end_flag`;
- trasformare un evento web in punteggio numerico;
- compilare una zona futura con ipotesi meteo;
- costruire un dataset di test per validare un workflow;
- convertire un file Excel in JSON sfruttabile.

L'agente deve sempre spiegare quali colonne ha creato e perché.

## Regole di sicurezza e di qualità

L'agente non deve inventare silenziosamente un dato importante.

Se crea una variabile, deve distinguere:

- dato letto in un file;
- dato trovato sul web;
- dato calcolato;
- ipotesi di simulazione.

Questa separazione è essenziale affinché l'utente sappia cosa è reale, calcolato o supposto.

## Comandi slash

I comandi slash fungono da guide rapide per agenti e utenti.

Esempi:

- `/forecast`: capire il modulo Forecast;
- `/forecast-predict`: preparare e lanciare una previsione;
- `/forecast-dataset`: costruire un dataset pulito;
- `/forecast-scenarios`: creare ipotesi utili;
- `/forecast-cmd`: capire i tool disponibili.

Questi comandi devono fornire una procedura breve, chiara e direttamente azionabile.
