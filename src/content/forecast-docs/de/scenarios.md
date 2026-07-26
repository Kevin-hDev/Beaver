# Szenarien

Ein Szenario untersucht eine Annahme aus einer bestehenden Analyse. Es ersetzt weder beobachtete Daten noch die ursprüngliche Prognose.

## Globale Anpassung

Eine prozentuale Anpassung erzeugt eine abgeleitete Kurve, etwa Nachfrage +10%, Umsatz -5% oder Kapazität +15%. Das Modell wird dabei nicht neu ausgeführt.

## Kontextuelles Szenario

Ein kontextuelles Szenario ändert zukünftige Kovariaten und führt das Modell erneut aus, wenn es diese unterstützt. Geändert werden können Budget, Preis, Wetter, Kapazität oder eine bestimmte Serie.

Geänderte Werte bleiben Annahmen.

## Erstellen und Bearbeiten

Der Forecast-Arbeitsbereich bündelt Erstellen, Bearbeiten und Löschen. Das Panel behält die schnelle Anzeige. Das LLM kann Szenarien auf Anfrage mit `forecast_analyze` verwalten.

## Kurven vergleichen

Vergleiche Ursprung und Szenarien im selben Zeitraum. Prüfe Beginn und Größe der Abweichung, Unsicherheit, betroffene Serien und tatsächlich geänderte Kovariaten.

## Modell-Ensemble

Ein Ensemble ist kein Geschäftsszenario. Es kombiniert zwei bis vier erfolgreich getestete Modelle mit inverser MASE-Gewichtung und ist als nicht unabhängig ausgewertet markiert.

## Gute Nutzung

Gib jedem Szenario einen klaren Namen, eine messbare Annahme, einen Zeitraum, eine Wertquelle, eine Erklärung und den Vergleich zur ursprünglichen Prognose.
