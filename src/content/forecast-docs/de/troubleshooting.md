# Diagnose

Dieser Abschnitt trennt normales Verhalten von Problemen, die eine Aktion erfordern.

## Modellvorbereitung

Ein Modell kann Nicht installiert, Aktualisierung erforderlich, Ungültig, Bereit oder Anbieter erforderlich anzeigen. Bereite fehlende oder veraltete Modelle vor, installiere ungültige neu und konfiguriere bei Bedarf den Cloud-Anbieter.

Mehrere Vorbereitungen werden eingereiht; gültige Dateien werden wiederverwendet.

## Sidecar-Lebenszyklus

Die lokale Laufzeit startet für Prognose oder Backtest und kann direkt danach stoppen. Das ist normal und gibt Ressourcen frei.

Ein Problem besteht nur, wenn sie nicht bereit wird, die Anfrage fehlschlägt oder Forecast einen Fehler meldet.

## Abgelehnte Datenprüfung

Ursachen können fehlende Spalten, ungültige oder doppelte Daten, falsche Frequenz, zu kurzer Verlauf, falsche Zukunftszeilen oder überschrittene Grenzen sein.

Behebe das Problem und wiederhole die Prüfung. Nutze nach Datenänderungen kein altes Profil.

## Inkompatible Konfidenz

Kontinuierliche Modelle akzeptieren ganze Werte von 50% bis 99%, einige feste Modelle nur 60% oder 80%.

Ändere im manuellen Modus Niveau oder Modell. Starte in Auto die Auswahl mit dem exakten Wert neu. Runde niemals still.

## Abgelaufene Auto-Auswahl

Die Auswahl ist an Datensatz, Sitzung und Ressourcen gebunden. Rufe bei Ablauf `forecast_models` erneut auf, hole eine neue ID und wiederhole `forecast`.

## Fehlendes Ergebnis

Prüfe `analysis_id`, wähle die Analyse im Verlauf, kontrolliere die Sitzung und lies sie erneut. Eine bei der Validierung verworfene Ausgabe wird nicht als gültig angezeigt.

## Teilweiser Backtest

Prüfe Gesamtstatus und einzelne Fehler. Betrachte die Rangliste erst als vollständig, wenn die verglichenen Modelle homogene Ergebnisse haben.

## Ignorierte Kovariaten

Eine Kovariate kann fehlen, in der Zukunft leer, konstant, falsch typisiert, falsch ausgerichtet oder nicht unterstützt sein. Prüfe Daten, Modell und Zukunftswerte.

## Flache Kurve oder schwaches Szenario

Eine flache Kurve kann ein stabiles Ziel, kurzen Verlauf, falsche Frequenz oder fehlenden Kontext zeigen. Ein Szenario kann wenig wirken, wenn die Änderung klein oder die Ebene ausgeblendet ist.
