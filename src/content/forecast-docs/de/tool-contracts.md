# Forecast-Tools

Die sieben Forecast-Tools bilden einen kontrollierten Ablauf. Große Ergebnisse bleiben im Forecast-Speicher; das LLM tauscht kompakte IDs aus.

## Empfohlene Reihenfolge

```text
forecast_data_audit
  → forecast_models
  → forecast
  → forecast_read
  → forecast_backtest
  → forecast_compare_models
```

Nutze `forecast_analyze` danach für Notizen, Szenarien oder Ensembles.

## `forecast_data_audit`

Rufe dieses Tool vor der ersten Prognose jedes Datensatzes auf. Übergib Daten oder Datei, Ziel, Datum, Frequenz, Horizont und exakte Konfidenz.

Es prüft Daten, Duplikate, fehlende Perioden, ungültige Werte, Verlauf, Serien, Zukunft und Ausreißer. Eine gültige Antwort liefert `data_profile_id`.

## `forecast_models`

Prüfe aktive Richtlinie und Intervalle. Kontrolliere im manuellen Modus das erzwungene Modell. Übergib in Auto `data_profile_id`, wähle einen Kandidaten und behalte `selection_id`.

Hardwareinformationen erscheinen nur in dieser Antwort. Runde die Konfidenz nicht.

## `forecast`

Starte die Prognose mit Profil, Ziel, Datum, Horizont, Frequenz und unveränderter Konfidenz. Ergänze Serie und Kovariaten nur bei Unterstützung.

Übergib in Auto auch Modell, `selection_id`, Quelle und erlaubte Gründe. Die Antwort liefert `analysis_id`.

## `forecast_read`

Lasse `analysis_id` weg, um Analysen aufzulisten, oder gib sie für eine Analyse an. Nutze `offset` und `limit`, höchstens 200 Punkte pro Seite.

Die Antwort kann Zerlegung, Residuenanomalien, chronologische Permutationsbedeutung und Drift enthalten. Erfinde keinen Ersatz.

## `forecast_backtest`

Führe eine begrenzte rollierende Prüfung auf einer gespeicherten Analyse aus. Modelle und Baselines Naive, saisonale Naive, Drift und ETS werden auf gleichen Perioden bewertet.

Prüfe immer Status und Fehler.

## `forecast_compare_models`

Lies die gespeicherte Rangliste mit Fehlern, Abdeckung, Dauer, beobachtetem Speicher und Baseline-Status. Nenne ein Modell nur bei vollständigem Nachweis das beste.

## `forecast_analyze`

Nutze `annotate`, `scenario`, `scenario_update`, `scenario_delete` oder `ensemble`. Erstelle ein Ensemble nur nach erfolgreichem Mehrmodell-Backtest und erkläre inverse MASE-Gewichtung sowie fehlende unabhängige Auswertung.

## Ablauf neu starten

Wiederhole `forecast_data_audit` und `forecast_models`, wenn sich Daten, Zuordnung, Ziel, Frequenz, Horizont, Konfidenz, Kovariaten, Serienstruktur oder Ressourcen ändern.
