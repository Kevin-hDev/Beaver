# Auswertung und Vergleich

Die Auswertung misst ein Modell auf historischen Perioden, die es bei der Berechnung nicht gesehen hat. Sie vergleicht Ergebnisse in identischen Zeitfenstern.

## Rollierender zeitlicher Backtest

Forecast teilt den Verlauf in mehrere Fenster. Pro Fenster nutzt das Modell nur die Vergangenheit und prognostiziert den nächsten Abschnitt. So werden Zukunftsinformationen ausgeschlossen.

Bei kurzem Verlauf können Fenster oder Horizont reduziert werden; die Oberfläche zeigt eine Warnung.

## Baselines

| Baseline | Prinzip |
| --- | --- |
| Naive | Wiederholt den letzten bekannten Wert |
| Saisonale Naive | Wiederholt den vergleichbaren saisonalen Wert |
| Drift | Verlängert den mittleren Trend |
| ETS | Modelliert Niveau, Trend und Saisonalität, wenn möglich |

## Metriken

| Metrik | Bedeutung |
| --- | --- |
| MASE | Fehler gegenüber Naive; kleiner ist besser |
| sMAPE | Symmetrischer relativer Fehler; kleiner ist besser |
| MAE | Mittlerer absoluter Fehler in Zieleinheiten |
| Abdeckung | Tatsächliche Werte innerhalb des Intervalls |
| Dauer | Beobachtete Laufzeit |
| Speicher | Beobachtete Spitze, wenn verfügbar |

Vergleiche gemessene Abdeckung mit dem angeforderten Niveau. Ein 80%-Intervall mit nur 40% Abdeckung ist schlecht kalibriert.

## Auswertung und Vergleich

Auswertung startet den Backtest und zeigt Details. Vergleich verwendet homogene Ergebnisse und zeigt Kompromisse zwischen Genauigkeit, Abdeckung, Geschwindigkeit und Ressourcen.

Präsentiere einen teilweisen Lauf niemals als vollständige Validierung.

## Bestes Modell

Nenne ein Modell nur dann das beste, wenn dieselben Fenster verwendet wurden, das Ergebnis vollständig ist, relevante Metriken besser sind, eine glaubwürdige Baseline geschlagen wird und Benutzerbedingungen erfüllt bleiben.

Ohne vergleichbare Backtests sprich nur von Kompatibilität oder einer Empfehlung nach Fähigkeiten.

## Modell-Ensemble

Nach einem erfolgreichen Mehrmodell-Backtest kann Vergleich zwei bis vier Modelle mit inverser MASE-Gewichtung kombinieren. Das Ensemble gilt als nicht unabhängig ausgewertet.
