# Prognosen

Eine Prognose verlängert eine oder mehrere Serien aus Verlauf, Kontextvariablen und ausgewähltem Modell. Sie enthält eine zentrale Schätzung und, wenn verfügbar, Unsicherheitsgrenzen.

## Gespeichertes Ergebnis

Jede gültige Ausführung erstellt eine `analysis_id`. Sie verbindet Panel, Arbeitsbereich, Szenarien, Notizen, Auswertungen und Exporte.

Forecast validiert vor dem Speichern Anzahl, Daten, Reihenfolge, endliche Werte, Quantile und Horizont. Eine teilweise oder inkonsistente Ausgabe wird nicht als gültige Analyse gespeichert.

## Hauptdiagramm

Das Diagramm trennt Verlauf und Prognosebereich. Filter steuern Serien, Unsicherheit, Szenarien, Ereignisse, Vergleiche, Anomalien und Qualitätssignale.

Du kannst ziehen, mit Mausrad oder Trackpad zoomen, Sprungleisten nutzen, Karten einklappen und die Punkttabelle öffnen. Ist kein weiterer Zoom möglich, bleibt das Scrollen der Seite frei.

## Ergänzende Diagramme

Der Arbeitsbereich kann Unsicherheitsfächer, saisonale Vergleiche und nach einem Backtest ein Zuverlässigkeitsdiagramm zeigen. Bei mehreren Serien bleibt die aktive Serie synchronisiert.

## Prognosetabelle

Die Tabelle ist standardmäßig eingeklappt. Geöffnet zeigt sie Datum, Zentralwert und Grenzen in einem begrenzten scrollbaren Bereich.

Bei langen Analysen liefert `forecast_read` begrenzte Seiten statt der ganzen Serie im LLM-Kontext.

## Echtzeit-Aktualisierung

Panel und Arbeitsbereich lesen dieselbe Analyse. Neue Prognosen, Änderungen und Analysewechsel aktualisieren die Ansichten ohne erneutes Öffnen.

## Richtige Interpretation

Lies die Kurve zusammen mit Datenqualität, Unsicherheit, Horizont, Anomalien, Backtests, Baselines und Annahmen. Eine glatte Kurve beweist keine Genauigkeit.
