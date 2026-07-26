# Diagnose

Dieser Abschnitt hilft zu verstehen, warum eine Prognose nicht das erwartete Ergebnis liefert.

## Ungültige JSON-Daten

Dieser Fehler bedeutet, dass Forecast keine verwendbare Tabelle erhalten hat.

Es kann liegen an:

- fehlerhaftem JSON;
- einer nicht korrekt konvertierten Datei;
- einem leeren oder abgeschnittenen Feld `data`;
- einem falschen Zeilenformat;
- fehlenden Spalten.

Wenn der Nutzer eine Datei bereitstellt, muss der Agent prüfen, ob die Datei korrekt gelesen wird, bevor die Daten konvertiert werden.

## Modell nicht verfügbar

Dieser Fehler kann liegen an:

- einem nicht installierten lokalen Modell;
- einem angehaltenen Sidecar;
- einem fehlenden API-Schlüssel;
- einem Modell, das nicht mit den Parametern kompatibel ist;
- Daten, die für das angeforderte Modell zu kurz sind.

Der richtige Reflex ist, das Modell zu prüfen und dann mit einem minimalen Dataset zu testen.

## Ignorierte Kontextvariablen

Eine Variable kann ignoriert oder nutzlos sein, wenn:

- sie in der Historie nicht existiert;
- sie in der Zukunft leer ist;
- sie konstant ist;
- sie falsch typisiert ist;
- sie nicht zum Horizont passt;
- sie Text enthält, der nicht in eine nutzbare Zahl oder Kategorie umgewandelt wurde.

In diesem Fall sollte das Dataset geprüft werden, bevor das Modell beschuldigt wird.

## Flaches Ergebnis

Eine flache Prognose kann normal sein, wenn die Zielgröße stabil ist.

Sie kann auch auf Folgendes hinweisen:

- Historie zu kurz;
- Frequenz falsch gewählt;
- Kontext abwesend;
- Zielgröße wenig variabel;
- Modell zu einfach;
- bekannte Zukunft wenig informativ.

## Szenario ohne sichtbare Wirkung

Ein kontextuelles Szenario kann wenig Unterschied zeigen, wenn:

- die veränderte Variable wenig Einfluss hat;
- die Änderung zu gering ist;
- das Modell diese Variable nicht als starkes Signal nutzt;
- die zukünftige Variable nicht wirklich übertragen wurde;
- die Kurve in den Filtern verdeckt ist.

Der Graph, die Filter, der Tooltip und die Daten des Szenarios sollten geprüft werden.
