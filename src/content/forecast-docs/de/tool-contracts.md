# Forecast-Tools

Die Forecast-Tools ermöglichen es LLM-Agenten, den Vorhersage-Motor aus dem Chat heraus zu nutzen. Sie müssen mit präzisen Parametern aufgerufen werden, denn eine falsche Spalte oder ein inkohärenter Horizont erzeugt eine nutzlose Prognose.

## `forecast`

`forecast` startet eine neue Vorhersage.

Haupteingaben:

| Parameter | Rolle |
| --- | --- |
| `file_path` | Zu lesende Excel-, CSV- oder JSON-Datei |
| `data` | Bereits vorbereitete JSON-Daten |
| `date_column` | Spalte mit den Daten |
| `target_column` | Zu prognostizierende Spalte |
| `series_column` | Spalte, die die Serien identifiziert |
| `covariate_columns` | Zu verwendende Kontextvariablen |
| `frequency` | Zeitlicher Rhythmus |
| `horizon` | Anzahl zukünftiger Punkte |
| `model` | Zu verwendender Motor |

Hauptausgabe:

- `analysis_id`, der Identifikator des Forecast-Ergebnisses.

## `forecast_read`

`forecast_read` liest ein Forecast-Ergebnis.

Es dient dem Abruf von:

- der Prognose;
- der Historie;
- der Unsicherheit;
- den Szenarien;
- den verfügbaren Variablen;
- den Modell-Metadaten.

Wenn kein `analysis_id` angegeben wird, kann der Agent ihn nutzen, um die verfügbaren Ergebnisse aufzulisten.

## `forecast_analyze`

`forecast_analyze` fügt Elemente rund um eine Prognose hinzu oder verändert sie.

Es dient insbesondere dazu:

- eine Annotation zu erstellen;
- ein Szenario zu erstellen;
- ein kontextuelles Szenario neu zu starten;
- ein Szenario zu ändern;
- ein Szenario zu löschen.

## Was der Agent prüfen muss

Vor dem Aufruf eines Tools muss der Agent prüfen:

- die Zielgröße existiert;
- das Datum ist lesbar;
- der Horizont entspricht den zukünftigen Zeilen;
- die Kovariablen existieren tatsächlich;
- die erstellten oder im Web gefundenen Daten sind identifiziert;
- das gewählte Modell unterstützt das Bedürfnis.

Der Agent muss seine Entscheidungen erklären, anstatt einen undurchsichtigen Aufruf abzusetzen.
