# Übersicht

Forecast dient dazu, die zukünftige Entwicklung einer messbaren Größe vorherzusagen. Das Modul analysiert die Historie, aktuelle Trends und Kontextvariablen, um eine datierte Vorhersage mit einer Unsicherheitsmarge und vergleichbaren Szenarien zu erzeugen.

## Einfache Definition von Forecasting

Beim Forecasting werden vergangene und aktuelle Daten beobachtet, um zu schätzen, was als Nächstes geschehen kann.

Beispiele:

- die Verkäufe der nächsten 30 Tage vorhersagen;
- den Umsatz des folgenden Monats schätzen;
- die Serverlast der nächsten Stunden voraussehen;
- den Preis oder das Volumen eines Werts projizieren;
- die Auswirkung eines bekannten zukünftigen Kontexts simulieren.

Das Modell liest nicht die Zukunft. Es berechnet eine wahrscheinliche Trajektorie aus in den Daten sichtbaren Mustern.

## Was Forecast zu einem LLM-Chat hinzufügt

Ein LLM kann eine Tabelle lesen und eine Erklärung schreiben. Forecast fügt einen spezialisierten Motor hinzu, der tatsächlich eine zukünftige Reihe berechnet.

Der Unterschied ist wichtig:

| LLM-Chat allein | Forecast |
| --- | --- |
| Erklärt eine Datei | Berechnet datierte zukünftige Punkte |
| Kann qualitativ schlussfolgern | Erzeugt eine numerische Kurve |
| Kann erfinden, wenn die Daten vage sind | Nutzt einen strengen Datenvertrag |
| Fasst einen Trend zusammen | Erzeugt eine Prognose, Schranken und Szenarien |

Das LLM bleibt rund um den Motor nützlich: Es bereitet die Daten vor, wählt die Spalten, kann Informationen im Web suchen, erstellt ein Dataset, startet Forecast und erklärt anschließend das Ergebnis.

## Hauptobjekt: die Zielgröße

Die Zielgröße ist die Spalte, die Forecast vorhersagen soll.

Beispiele:

- `ventes`
- `ca_total_eur`
- `commandes_total`
- `temperature`
- `stock_price`
- `incidents_count`

Die gesamte Vorhersage dreht sich um diese Zielgröße: Das Modell lernt ihr vergangenes Verhalten und schätzt dann ihre zukünftigen Werte.

## Was ein Forecast-Ergebnis enthält

Ein Forecast-Ergebnis enthält:

- die verwendeten historischen Werte;
- die zukünftige Prognose Punkt für Punkt;
- eine Unsicherheitsmarge;
- die verwendeten Kontextvariablen;
- die aus dieser Prognose erstellten Szenarien;
- die Metadaten, die nötig sind, um das Ergebnis zu prüfen, zu vergleichen und zu exportieren.

Es ist keine passive Datei. Es ist ein vollständiges Arbeitsobjekt, um zu verstehen, was wahrscheinlich, was riskant und was veränderlich ist, wenn sich der Kontext entwickelt.

## Allgemeine Logik

Der Standard-Workflow ist:

1. ein Dataset bereitstellen;
2. Datum, Zielgröße und gegebenenfalls die Serien wählen;
3. die nützlichen Kontextvariablen auswählen;
4. ein Vorhersagemodell starten;
5. die zukünftige Kurve und die Unsicherheit lesen;
6. Szenarien erstellen, um Hypothesen zu testen;
7. das LLM bitten, die Ergebnisse zu erklären oder neue Daten vorzubereiten.
