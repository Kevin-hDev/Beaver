# Datasets

Ein Dataset ist die Tabelle, die Forecast verwendet, um die Vergangenheit zu lernen und die Zukunft vorherzusagen. Es muss so strukturiert sein, dass das Modell versteht, was vorhergesagt werden soll, zu welchen Daten und mit welchem Kontext.

## Mindeststruktur

Ein Forecast-Dataset enthält mindestens:

| Spalte | Rolle |
| --- | --- |
| Datum | Gibt an, wann jede Beobachtung stattfand |
| Zielgröße | Zu prognostizierender Wert |

Einfaches Beispiel:

```text
date        commandes
2026-05-01  120
2026-05-02  135
2026-05-03  128
```

Mit dieser Tabelle kann Forecast die Dynamik der Bestellungen lernen und die nächsten Daten vorhersagen.

## Historischer Bereich

Der historische Bereich enthält die bereits bekannten realen Werte.

Er dient dem Modell, um Folgendes zu erkennen:

- Trend;
- Saisonalität;
- Rhythmus;
- Spitzen;
- Rückgänge;
- normale Schwankungen.

Die Zielgröße muss in diesem Bereich gefüllt sein.

## Zukunftsbereich

Der Zukunftsbereich enthält die Daten, die vorhergesagt werden sollen.

In diesem Bereich ist die Zielgröße leer, denn genau das soll Forecast berechnen.

Beispiel:

```text
date        commandes
2026-05-01  120
2026-05-02  135
2026-05-03
2026-05-04
```

Hier muss Forecast `commandes` für den 3. und 4. Mai vorhersagen.

## Bekannte Zukunft

Die bekannte Zukunft fügt Kontextvariablen in den zukünftigen Zeilen hinzu.

Beispiel:

```text
date        commandes   pluie_mm   promo
2026-05-01  120        0          0
2026-05-02  135        4          1
2026-05-03             12         0
2026-05-04             0          1
```

Die zukünftige Zielgröße ist leer, aber Regen und Promo sind bereits bekannt oder angenommen. Das Modell kann diese Informationen nutzen, um eine realistischere Prognose zu erzeugen.

## Serienspalte

Die Serienspalte dient, wenn eine einzige Datei mehrere Objekte enthält, die vorhergesagt werden sollen.

Beispiele:

- mehrere Geschäfte;
- mehrere Produkte;
- mehrere Städte;
- mehrere Finanzwerte;
- mehrere Server.

Beispiel:

```text
date        magasin   ventes   promo
2026-05-01  paris    120      0
2026-05-01  lyon     92       1
2026-05-02  paris    132      1
2026-05-02  lyon     95       0
```

Forecast kann dann jede Serie vorhersagen und dabei die Gruppe berücksichtigen, zu der sie gehört.

## Von einem Agent erstelltes Dataset

Ein LLM-Agent kann ein Dataset erstellen oder anreichern.

Er kann zum Beispiel:

- ein Excel in JSON umwandeln;
- eine `weekend`-Spalte hinzufügen;
- Ereignisse im Web abrufen;
- eine textuelle Information in einen Score umwandeln;
- die zukünftigen Zeilen mit Hypothesen füllen;
- Daten oder Spalten bereinigen.

Der Agent muss klar angeben, welche Daten aus der Datei, aus dem Web, aus einer Berechnung oder aus einer Annahme stammen.
