# Szenarien

Ein Szenario dient dazu, eine Hypothese über die Zukunft zu testen. Es vergleicht die Basis-Prognose mit einer veränderten Trajektorie.

## Warum ein Szenario erstellen

Die Basis-Prognose beantwortet: "Was sagt das Modell mit den aktuellen Daten voraus?"

Ein Szenario beantwortet: "Was passiert, wenn sich der Kontext ändert?"

Beispiele:

- was werden aus den Verkäufen, wenn eine Promotion gestartet wird?
- was wird aus dem Umsatz, wenn der Verkehr sinkt?
- was wird aus einem Wert, wenn der VIX steigt?
- was wird aus der Nachfrage, wenn ein lokales Ereignis hinzugefügt wird?

Das Szenario macht Forecast zu einem Simulationswerkzeug, nicht nur zu einem Prognose-Graphen.

## Globale Anpassung

Die globale Anpassung wendet eine einfache Veränderung auf die Kurve an.

Beispiel:

```text
Basis-Prognose:  100, 110, 120
Szenario +10 %:  110, 121, 132
```

Dieser Modus ist schnell und gut lesbar. Er startet das Modell nicht neu und versteht daher nicht die Beziehungen zwischen Variablen.

## Kontextuelles Szenario

Das kontextuelle Szenario verändert die zukünftigen Variablen und startet dann das Modell neu.

Beispiel:

```text
Hypothese: vix_close +20 %
Erwartete Wirkung: das Modell berechnet die Zielgröße mit diesem stressigeren Marktkontext neu.
```

Dieser Modus ist wichtiger für Chronos-2 und TimeGPT, da er die Kovariablen als echte Vorhersagesignale nutzt.

## Veränderbare Variablen

Die verfügbaren Variablen hängen vom Dataset ab.

Sie können darstellen:

- Umgebung: Wetter, Verkehr, Ereignisse;
- Finanzen: Volatilität, Zinsen, Indizes, News-Score;
- Kalender: Wochenende, Feiertag, Monatsende;
- Fachbereich: Promo, Bestand, Budget, Kampagne;
- Risiko: Alarm, Vorfall, Wettbewerbsdruck.

Jede Änderung muss eine fachliche Bedeutung haben. Eine zufällig geänderte Variable erzeugt ein schwer interpretierbares Szenario.

## Lesart im Graphen

Wenn ein Szenario ausgewählt ist, sollte der Graph Folgendes vergleichen:

- tatsächlicher Verlauf;
- Basis-Prognose;
- Szenario-Prognose;
- angezeigte Kontextvariablen;
- Differenz zwischen Basis und Szenario.

Typisches Schema eines Vergleichs:

```text
wert
  ^
  |              ╭───── Szenario (VIX +20 %)
  |           ╭──╯
  |       · ·─·      ← Basis-Prognose
  |     ·
  |   ·
  | ·
  ──────────────────────────────> zeit
       Verlauf         │   Zukunft
                       │
                  Horizont
```

Die Hauptfrage ist nicht "Ist die Kurve anders?", sondern "Welche Hypothese hat die Trajektorie verschoben, an welchem Datum und um wie viel?"

## Korrekte Verwendung

Ein gutes Szenario muss klar benannt sein.

Beispiele:

- `VIX +20% während 30 Tagen`
- `Wochenend-Promo aktiv`
- `Starker Regen Woche 2`
- `Verkehr -15% nach Vorfall`

Ein vager Name wie `test` oder `crash` macht den Vergleich nutzlos, wenn mehrere Szenarien existieren.
