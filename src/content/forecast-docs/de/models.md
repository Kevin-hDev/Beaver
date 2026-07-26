# Modelle

Ein Modell ist der Motor der Prognose. Forecast bietet lokale und Cloud-Familien und prüft vor jeder Ausführung Fähigkeiten, Bereitschaft und Ressourcen.

## Verfügbare Familien

| Familie | Anbieter | Haupteinsatz |
| --- | --- | --- |
| Chronos / Chronos-Bolt | Amazon | Schnelle lokale probabilistische Prognosen |
| TimesFM | Google | Allgemeine Zeitreihenprognose |
| Toto 2.0 | Datadog | Metriken und Monitoring |
| MOIRAI 2.0 | Salesforce | Mehrere Serien und Kontextvariablen |
| FlowState | IBM | Lokale probabilistische Prognose |
| TabPFN-TS, TiRex, Kairos, Sundial | Verschiedene | Spezialisierte oder experimentelle lokale Modelle |
| TimeGPT | Nixtla | Cloud-Prognose mit API-Schlüssel |

Der App-Katalog ist die Referenz für Frequenzen, Horizont, Kovariaten, Mehrserien und Intervalle.

## Manueller Modus

Im manuellen Modus wählst du das Modell und Forecast erzwingt diese Wahl. Ist es nicht bereit oder nicht exakt kompatibel, fordert das LLM eine andere Auswahl an, statt sie still zu ersetzen.

## Auto-Modus

Im Auto-Modus wählt das LLM genau ein Modell aus einer von Forecast gefilterten Liste. Nicht bereite, inkompatible, zu große oder nicht erlaubte Cloud-Modelle werden ausgeschlossen.

Hardwareinformationen erhält das LLM nur während dieser Forecast-Auswahl. Ohne vergleichbare Backtests spricht Auto von Kompatibilität oder einer Empfehlung nach Fähigkeiten, nie vom besten Modell.

## Installation und Vorbereitung

Vorbereiten lädt das Modell, installiert seine Laufzeit und führt vor der ersten Prognose eine echte Prüfung aus. Mehrere Vorbereitungen werden eingereiht; Varianten einer Familie können eine Laufzeit teilen.

| Status | Bedeutung |
| --- | --- |
| Nicht installiert | Modelldateien fehlen |
| Aktualisierung erforderlich | Laufzeit oder Prüfung muss erneuert werden |
| Ungültig | Installation ist unvollständig oder nicht validiert |
| Bereit | Modell und Laufzeit sind geprüft |
| Anbieter erforderlich | Cloud-API-Schlüssel fehlt |

Ein lokales Modell ist nur im Status Bereit wählbar. Eine gemeinsam genutzte Laufzeit wird beim Entfernen nur gelöscht, wenn kein anderes Modell sie benötigt.

## Cloud-Modelle

Ein Cloud-Modell sendet notwendige Daten an den konfigurierten Anbieter. Auto nutzt es nur mit Erlaubnis, bereitem Anbieter und passender Datenrichtlinie. Forecast wechselt nie still von lokal zu Cloud.
