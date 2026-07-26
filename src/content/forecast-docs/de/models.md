# Modelle

Ein Modell ist der Motor, der die Prognose berechnet. Forecast bietet mehrere Modellfamilien, lokal (die Berechnung bleibt auf der Maschine) oder Cloud (die benötigten Daten werden an den konfigurierten Provider gesendet).

## Lokale Familien

| Familie | Herausgeber | Detail |
| --- | --- | --- |
| Chronos / Chronos-Bolt | Amazon | Schnelles lokales Modell, gut für einen ersten Test oder eine einfache Zielgröße |
| TimesFM | Google | Lokales Modell zur Zeitreihen-Prognose |
| Toto 2.0 | Datadog | Lokales Modell für Monitoring und Metriken |
| MOIRAI 2.0 | Salesforce | Lokales Modell, unterstützt Multi-Serien und Kovariablen |
| FlowState | IBM | Lokales Modell für Zeitreihen |
| TabPFN-TS | PriorLabs | Experimentelles lokales Modell |
| TiRex | NX-AI | Experimentelles lokales Modell |
| Kairos | Foundation Model Research | Experimentelles lokales Modell |
| Sundial | THUML | Experimentelles lokales Modell |

## Cloud-Familie

| Familie | Herausgeber | Detail |
| --- | --- | --- |
| TimeGPT-2 / TimeGPT-2.1 | Nixtla | Cloud-Motor spezialisiert auf Zeitreihen. Erfordert einen API-Schlüssel und sendet die benötigten Daten an den Provider. |

Cloud-Modelle können leistungsfähiger sein, bringen aber eine externe Abhängigkeit und eine Datenübertragung mit sich. Für sensible Daten ein lokales Modell bevorzugen.

## Ein Modell wählen

Die Wahl hängt vor allem vom Dataset und vom Anwendungsfall ab:

- **Schneller Test, einfache Zielgröße**: Chronos-Bolt.
- **Sensible Daten, lokale Berechnung**: eine beliebige lokale Familie.
- **Kovariablen und bekannter Zukunftskontext**: ein Modell, das Kontextvariablen unterstützt (MOIRAI 2.0, Chronos-2, TimeGPT).
- **Multi-Serien**: ein Modell, das mehrere Serien unterstützt (MOIRAI 2.0, Chronos-2, TimeGPT).
- **Fortgeschrittene Cloud-Qualität**: TimeGPT, bei Akzeptanz der Datenübertragung.

Ein fortgeschrittenes Modell gleicht schlecht strukturierte Daten nicht aus. Bevor das Modell gewechselt wird, sollten Dataset-Qualität, Frequenz, Horizont und Kontextvariablen geprüft werden.

## Ein lokales Modell installieren

Lokale Modelle müssen über den Modell-Manager (Settings → Forecast) oder über den Reiter Modelle im Forecast-Bereich installiert werden. Sie werden je nach Familie von Hugging Face oder GitHub heruntergeladen und lokal unter `~/.local/share/cl-go-dash/forecast-models/` gespeichert.
