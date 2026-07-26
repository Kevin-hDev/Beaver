# LLM-Agenten

LLM-Agenten können Forecast als spezialisierten Motor nutzen. Ihre Rolle beschränkt sich nicht darauf, eine Datei zu lesen und auf ein Tool zu klicken: Sie können Daten vorbereiten, Kontext im Web suchen, ein Dataset aufbauen, die Vorhersage starten und das Ergebnis erklären.

## Was der Agent tun kann

Ein Agent kann an mehreren Stellen eingreifen:

| Schritt | Rolle des Agenten |
| --- | --- |
| Vorbereitung | Excel, CSV oder JSON lesen |
| Recherche | Externe Informationen im Web abrufen |
| Dataset | Nützliche Spalten erstellen oder ergänzen |
| Start | `forecast` mit den richtigen Parametern aufrufen |
| Lesen | `forecast_read` verwenden, um das Ergebnis abzurufen |
| Szenario | Hypothesen erstellen und das Modell neu starten |
| Erklärung | Trend, Unsicherheit, Variablen und Grenzen zusammenfassen |

Beispiel: Für eine Finanzprognose kann der Agent die lokale Datei lesen, den aktuellen Marktkontext suchen, Spalten wie `news_score` oder `event_flag` erzeugen und dann Forecast starten.

## Empfohlener Workflow

Der Agent sollte dieser Reihenfolge folgen:

1. die Anfrage des Nutzers verstehen;
2. die verfügbaren Daten prüfen;
3. die vorherzusagende Zielgröße identifizieren;
4. Daten, Frequenz und Horizont identifizieren;
5. bei Bedarf nützliche Kontextvariablen suchen oder erstellen;
6. prüfen, dass die zukünftigen Zeilen kohärent sind;
7. ein kompatibles Modell wählen;
8. `forecast` starten;
9. das Ergebnis mit `forecast_read` prüfen;
10. die Prognose erklären und nützliche Szenarien vorschlagen.

## Datenerstellung durch den Agenten

Der Agent kann Daten erstellen, wenn der Nutzer es verlangt oder die Vorhersage es erfordert.

Beispiele:

- eine `weekend`-Spalte aus dem Datum hinzufügen;
- `month_end_flag` erstellen;
- ein Web-Ereignis in einen numerischen Score umwandeln;
- einen Zukunftsbereich mit Wetter-Hypothesen füllen;
- ein Test-Dataset erstellen, um einen Workflow zu validieren;
- eine Excel-Datei in nutzbares JSON umwandeln.

Der Agent muss stets erklären, welche Spalten er erstellt hat und warum.

## Sicherheits- und Qualitätsregeln

Der Agent darf keine wichtige Daten stillschweigend erfinden.

Wenn er eine Variable erstellt, muss er unterscheiden zwischen:

- aus einer Datei gelesenen Daten;
- im Web gefundenen Daten;
- berechneten Daten;
- einer Simulations-Hypothese.

Diese Trennung ist wesentlich, damit der Nutzer weiß, was real, berechnet oder angenommen ist.

## Slash-Befehle

Slash-Befehle dienen als schnelle Leitfäden für Agenten und Nutzer.

Beispiele:

- `/forecast`: das Forecast-Modul verstehen;
- `/forecast-predict`: eine Vorhersage vorbereiten und starten;
- `/forecast-dataset`: ein sauberes Dataset aufbauen;
- `/forecast-scenarios`: nützliche Hypothesen erstellen;
- `/forecast-cmd`: die verfügbaren Tools verstehen.

Diese Befehle sollen ein kurzes, klares und direkt ausführbares Verfahren geben.
