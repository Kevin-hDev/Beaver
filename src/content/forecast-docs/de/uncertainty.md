# Unsicherheit

Eine seriöse Prognose besteht nicht nur aus einer Kurve. Forecast verbindet den Zentralwert mit einem Intervall für das angeforderte Konfidenzniveau.

## Zentralwert

Der Zentralwert ist meist der Median `q50`. Ungefähr die Hälfte möglicher Ergebnisse liegt darunter und die andere Hälfte darüber.

## Konfidenzniveau

Kontinuierliche Modelle akzeptieren 50% bis 99% in ganzen Prozentpunkten. Ohne Benutzerwunsch verwendet das LLM 80%.

Einige Modelle liefern nur feste Niveaus, derzeit 60% oder 80%. Forecast erhält den exakten Wunsch: Auto liefert nur kompatible Kandidaten, Manuell meldet die Inkompatibilität und rundet nie still.

## Grenzen und Quantile

Ein zentrales 80%-Intervall nutzt meist `q10`, `q50` und `q90`; 90% meist `q05`, `q50` und `q95`.

## Unsicherheitsfächer

Das Fächerdiagramm zeigt breitere oder engere Intervalle über den Horizont. Breitere Grenzen bedeuten weniger Präzision. Ein enges Intervall ist nur bei guter Kalibrierung nützlich.

## Gemessene Abdeckung

Nach dem Backtest vergleicht Forecast das angegebene Niveau mit dem tatsächlich abgedeckten Anteil. Ein kurzer Verlauf kann diese Messung instabil machen.

## Gute Nutzung

Nutze Unsicherheit, um Risiken zu vergleichen, robuste Trends zu erkennen, vorsichtige Schwellen zu planen, Kalibrierung zu prüfen und Szenarien nicht mit Gewissheit zu verwechseln.
