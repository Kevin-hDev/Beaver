# Multi-Serien

Multi-Serien erlaubt es, mehrere Serien in einer einzigen Analyse vorherzusagen. Anstatt für jedes Objekt eine separate Prognose zu starten, erhält Forecast ein einziges Dataset mit einer Spalte, die jede Serie identifiziert.

## Beispiel

Eine Datei kann die Verkäufe mehrerer Geschäfte enthalten:

```text
date        magasin   ventes   promo
2026-05-01  paris    120      0
2026-05-01  lyon     92       1
2026-05-02  paris    132      1
2026-05-02  lyon     95       0
```

Hier ist `magasin` die Serienspalte.

Forecast versteht, dass es die Verkäufe für `paris` und für `lyon` vorhersagen soll.

## Wozu es dient

Multi-Serien ist nützlich, wenn mehrere Serien eine gemeinsame Logik haben.

Beispiele:

- Verkäufe pro Geschäft;
- Bestellungen pro Restaurant;
- Verkehr pro Server;
- Preis pro Wert;
- Vorfälle pro Region.

Das Modell kann mehr Informationen nutzen als eine isolierte Prognose, insbesondere wenn sich die Serien ähneln oder Kontextvariablen teilen.

## Horizont pro Serie

Jede Serie muss eine kohärente zeitliche Struktur aufweisen.

Wenn der Horizont `31` ist, muss jede Serie 31 zukünftige Punkte zum Vorhersagen aufweisen.

Beispiel:

```text
paris -> 31 zukünftige Zeilen
lyon  -> 31 zukünftige Zeilen
```

Ein inkohärenter Horizont erschwert den Vergleich und kann das Modell blockieren.

## Lesart in Forecast

In der Oberfläche kann der Nutzer die angezeigte Serie auswählen.

Die Szenarien können dann gelesen werden:

- auf einer bestimmten Serie;
- auf mehreren Serien;
- im Vergleich zur Basis-Prognose.

Multi-Serien ändert nicht das Prinzip von Forecast. Es fügt lediglich eine Dimension hinzu: "Welche Serie wird gerade vorhergesagt?"
