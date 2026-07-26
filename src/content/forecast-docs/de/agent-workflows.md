# LLM-Agenten

Das LLM steuert Forecast aus der aktiven Unterhaltung. Es kann Daten vorbereiten oder recherchieren, Qualität prüfen, ein erlaubtes Modell wählen, rechnen und Ergebnisse erklären.

## Verbindlicher Ablauf

Gehe bei jedem neuen Datensatz so vor:

1. Verstehe Ziel, Zeitraum, Horizont und gewünschte Konfidenz.
2. Lies oder erstelle die Daten und unterscheide ihre Quellen.
3. Rufe `forecast_data_audit` auf.
4. Behebe blockierende Fehler oder erkläre sie.
5. Rufe `forecast_models` mit dem geprüften Profil auf.
6. Respektiere im manuellen Modus das erzwungene Modell und prüfe die exakte Kompatibilität.
7. Wähle im Auto-Modus genau einen zurückgegebenen Kandidaten.
8. Rufe `forecast` mit Profil, erlaubtem Modell und unveränderter Konfidenz auf.
9. Nutze `forecast_read` für benötigte Seiten und Analysen.
10. Erkläre Prognose, Unsicherheit und Grenzen.

Wiederhole die Prüfung, wenn sich Daten, Ziel, Frequenz, Horizont oder Konfidenz ändern.

## Manueller Modus

Ändere niemals die gespeicherte Benutzerauswahl. Fehlt das Modell oder ist es nicht bereit oder inkompatibel, fordere eine klare Aktion statt einer stillen Ersetzung.

## Auto-Modus

Wähle einen zurückgegebenen Kandidaten und umgehe keine Backend-Ausschlüsse. Respektiere einen ausdrücklichen Modellwunsch nur, wenn Forecast ihn als sicher bestätigt.

Übermittle Auswahl-ID und erlaubte kurze Gründe an `forecast`. Bezeichne eine reine Fähigkeiten- und Ressourcenwahl nicht als bestes Modell.

## Auswertung und Vergleich

Wenn der Benutzer das beste Modell verlangt:

1. Führe `forecast_backtest` mit kompatiblen Modellen aus.
2. Prüfe Status und einzelne Fehler.
3. Lies die Rangliste mit `forecast_compare_models`.
4. Vergleiche mit Naive, saisonaler Naive, Drift und ETS.
5. Zeige Fehler, Abdeckung, Geschwindigkeit und Speicher.

Präsentiere keinen teilweisen Backtest als vollständig und kein Modell als bestes, wenn es keine glaubwürdige Baseline schlägt.

## Herkunft und Erklärung

Kennzeichne Werte aus Dateien, externen Quellen, Berechnungen oder Annahmen. Erfinde wichtige Daten niemals still.

Nutze den bestehenden Chat für Erklärung, Vergleich oder Neustart. Stelle fehlende oder wenig zuverlässige Analysen ehrlich dar.
