# Datasets

Die Qualität einer Prognose beginnt bei den Daten. Forecast trennt historische Zeilen, bereits bekannte Zukunftsinformationen und Szenarioannahmen.

## Mindeststruktur

Ein nutzbarer Datensatz enthält Datumsspalte, Zielspalte, Frequenz und Horizont. Eine optionale Serienspalte trennt Produkte, Regionen oder Sensoren; Kovariaten ergänzen Kontext.

## Historischer Bereich

Historische Zeilen enthalten Datum und beobachtetes Ziel. Sie müssen geordnet, ausreichend lang und mit der gewählten Frequenz konsistent sein.

Forecast prüft ungültige oder ungeordnete Daten, Duplikate, fehlende Perioden, leere oder nicht numerische Werte, Ausreißer, Verlaufslänge und Konsistenz zwischen Serien.

Ein struktureller Fehler blockiert die Ausführung. Ein nicht blockierendes Risiko bleibt als Warnung sichtbar.

## Zukünftiger Bereich

Zukünftige Zeilen dürfen das Ziel auslassen. Sie sind nützlich für bereits bekannte Informationen wie Kalender, geplante Preise, Budgets, Kampagnen, Wetterprognosen oder Kapazitäten.

Stelle unbekannte Zukunftsinformationen niemals als Fakten dar.

## Prüfung vor der Prognose

Jeder neue Datensatz durchläuft `forecast_data_audit`. Die Prüfung validiert Daten, Horizont, Frequenz und angefordertes Konfidenzniveau.

Eine gültige Prüfung erstellt ein wiederverwendbares Profil. Das LLM verwendet es für Modellauswahl und Prognose, ohne alle Daten erneut zu übertragen.

Wiederhole die Prüfung, wenn sich Daten, Ziel, Horizont, Frequenz oder Konfidenz ändern.

## Vom LLM erstellte Daten

Das LLM kann CSV, Tabellen oder JSON lesen, Kontext recherchieren und Spalten erstellen. Es muss klar zwischen Datei-, Web-, berechneten und angenommenen Werten unterscheiden.

## Vorschau

Der Bereich Daten zeigt Zeilen, historische Punkte, zukünftige Zeilen, Serien, fehlende Perioden, Ausreißer, Zuordnung und eine begrenzte Vorschau.
