# Unsicherheit

Eine Prognose ist nie eine absolute Wahrheit. Forecast zeigt daher einen zentralen Wert und eine Unsicherheitsmarge an, um das Risiko rund um die Trajektorie aufzuzeigen.

## Zentraler Wert

Der zentrale Wert ist die Hauptschätzung des Modells.

Beispiel:

```text
2026-06-01 -> 142 Bestellungen prognostiziert
```

Er stellt das wahrscheinlichste Szenario gemäß den verwendeten Daten dar.

## Unsicherheitsschranken

Die Schranken geben eine wahrscheinliche Zone rund um den zentralen Wert an.

Beispiel:

```text
Prognose:    142
Untere Schranke: 128
Obere Schranke:  157
```

Einfache Lesart: das Modell schätzt 142, hält aber einen Wert zwischen 128 und 157 für plausibel.

## Quantile

Die Modelle können Quantile zurückgeben.

| Feld | Bedeutung |
| --- | --- |
| q10 | Wahrscheinlicher niedriger Wert |
| q50 | Zentraler oder medianer Wert |
| q90 | Wahrscheinlicher hoher Wert |

Je größer die Spanne zwischen q10 und q90, desto unsicherer ist das Modell.

## Warum die Unsicherheit wächst

Die Unsicherheit kann wachsen, wenn:

- die Historie kurz ist;
- die Zielgröße stark schwankt;
- der Horizont lang ist;
- die Kontextvariablen fehlen;
- ein aktueller Bruch in den Daten auftritt;
- mehrere zukünftige Szenarien möglich sind.

## Korrekte Verwendung

Der zentrale Wert dient dazu, den Trend zu lesen.

Die Unsicherheitsmarge dient dazu, das Risiko zu lesen.

Eine ernsthafte Entscheidung muss beide betrachten.
