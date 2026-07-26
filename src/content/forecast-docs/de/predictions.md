# Prognosen

Eine Prognose ist das von Forecast berechnete Ergebnis, um die nächsten Werte einer Zielgröße zu schätzen. Sie beantwortet eine konkrete Frage: "Wenn die vergangenen Daten und der bekannte Kontext kohärent bleiben, welche Werte kann man dann als Nächstes erwarten?"

## Was eine Prognose darstellt

Eine Prognose enthält eine Folge zukünftiger Punkte.

Jeder Punkt entspricht einem Datum oder einer Periode:

```text
2026-06-01 -> 142 Bestellungen prognostiziert
2026-06-02 -> 151 Bestellungen prognostiziert
2026-06-03 -> 149 Bestellungen prognostiziert
```

Diese Werte sind keine Gewissheit. Sie stellen die Schätzung des Modells dar.

## Benötigte Eingaben

Um eine Prognose zu starten, benötigt Forecast:

| Element | Rolle |
| --- | --- |
| Datum | Verortet jede Zeile in der Zeit |
| Zielgröße | Zu prognostizierender Wert |
| Frequenz | Rhythmus der Daten: Tag, Stunde, Monat usw. |
| Horizont | Anzahl der vorherzusagenden zukünftigen Punkte |
| Modell | Motor, der die Trajektorie berechnet |

Kontextvariablen und Multi-Serien sind nicht zwingend, werden aber wichtig, sobald man die Zukunft erklären oder simulieren will.

## Horizont

Der Horizont gibt die Tiefe der Prognose an.

Beispiele:

- Horizont `24` bei stündlicher Frequenz: die nächsten 24 Stunden vorhersagen;
- Horizont `31` bei täglicher Frequenz: die nächsten 31 Tage vorhersagen;
- Horizont `12` bei monatlicher Frequenz: die nächsten 12 Monate vorhersagen.

Je länger der Horizont, desto größer wird im Allgemeinen die Unsicherheit.

## Ergebnis und Identifikator

Jeder Start erzeugt einen Identifikator `analysis_id`.

Dieser Identifikator bedeutet nicht "gespeicherte Datei". Er dient dazu, das berechnete Ergebnis wiederzufinden: zukünftige Kurve, Unsicherheit, Parameter, Variablen, Szenarien und Annotationen.

Die Anwendung nutzt ihn, um:

- eine Prognose wieder zu öffnen;
- den Graphen anzuzeigen;
- mehrere Ergebnisse zu vergleichen;
- Szenarien zu erstellen oder erneut zu starten;
- einem LLM-Agent zu ermöglichen, das Ergebnis zu prüfen.

## Korrekte Interpretation

Eine Prognose muss mit drei Fragen gelesen werden:

- steigt der Trend, fällt er oder bleibt er stabil?
- ist die Unsicherheit gering oder groß?
- welche Kontextvariablen können die Bewegung erklären?

Eine Kurve allein reicht nicht. Forecast wird nützlich, wenn die Prognose mit ihrem Kontext verbunden wird.
