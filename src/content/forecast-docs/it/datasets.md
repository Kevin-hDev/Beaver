# Dataset

La qualità della previsione parte dai dati. Forecast separa righe storiche, informazioni future già note e ipotesi create per gli scenari.

## Struttura minima

Un dataset utilizzabile contiene colonna data, colonna target, frequenza e orizzonte. Una colonna serie opzionale separa prodotti, regioni o sensori; le covariate aggiungono contesto.

## Area storica

Le righe storiche contengono data e target osservato. Devono essere ordinate, abbastanza numerose e coerenti con la frequenza.

Forecast verifica date non valide o disordinate, duplicati, periodi mancanti, valori vuoti o non numerici, anomalie, lunghezza dello storico e coerenza tra serie.

Un errore strutturale blocca l'esecuzione. Un rischio non bloccante resta visibile come avviso.

## Area futura

Le righe future possono omettere il target. Sono utili per informazioni già note come calendario, prezzi pianificati, budget, campagne, meteo previsto o capacità.

Non presentare come fatto un'informazione futura sconosciuta.

## Audit prima della previsione

Ogni nuovo dataset passa da `forecast_data_audit`. L'audit valida dati, orizzonte, frequenza e livello di confidenza richiesto.

Un audit valido crea un profilo riutilizzabile. Il LLM lo usa per selezionare il modello ed eseguire la previsione senza rinviare tutti i dati.

Ripeti l'audit se cambiano dati, target, orizzonte, frequenza o confidenza.

## Dati creati dal LLM

Il LLM può leggere CSV, fogli di calcolo o JSON, cercare contesto e creare colonne. Deve distinguere valori letti da file, trovati online, calcolati o ipotizzati.

## Anteprima

La sezione Dati mostra righe, punti storici, righe future, serie, periodi mancanti, anomalie, mappatura e un'anteprima limitata.
