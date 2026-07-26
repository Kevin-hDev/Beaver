# Kovariablen

Kovariablen sind die Kontextvariablen, die die Zielgröße beeinflussen können. Sie beschreiben die Umgebung rund um die zu prognostizierende Größe.

## Definition

Eine Kovariable beantwortet die Frage: "Was kann dem Modell helfen zu verstehen, warum die Zielgröße steigt, fällt oder den Rhythmus wechselt?"

Beispiele:

| Bereich | Zielgröße | Mögliche Kovariablen |
| --- | --- | --- |
| Restaurant | Bestellungen | Wetter, Feiertag, lokales Ereignis, Promo |
| Finanzen | Preis oder Volumen | VIX, Zinsen, News-Score, Bitcoin, Dollar-Index |
| SaaS | Serverlast | aktive Nutzer, Kampagne, Vorfall, Release |
| Retail | Verkäufe | Bestand, Rabatt, Saison, Verkehr im Geschäft |

## Warum sie wichtig sind

Ohne Kovariable betrachtet das Modell vor allem die Vergangenheit der Zielgröße.

Mit Kovariablen kann das Modell auch den Kontext nutzen:

- "die Verkäufe steigen oft, wenn eine Promo aktiv ist";
- "die Bestellungen sinken, wenn es stark regnet";
- "der Preis reagiert, wenn der VIX steigt";
- "die Last steigt nach einer Marketingkampagne".

Sie erlauben somit eine kontextualisierte Prognose, die über eine einfache Fortführung der Vergangenheit hinausgeht.

## Arten von Kontextvariablen

Eine Kovariable kann sein:

| Typ | Beispiel | Lesart |
| --- | --- | --- |
| Numerisch | `temperature = 28` | Gemessener Wert |
| Prozent | `discount_pct = 15` | Stärke eines Effekts |
| Binär | `weekend = 1` | Ja / nein |
| Ereignis | `concert_local = 1` | Ereignis vorhanden |
| Score | `news_score = 0.72` | Berechneter Indikator |
| Kalender | `jour_ferie = 1` | Zeitlicher Kontext |

## Historie und bekannte Zukunft

Eine Kovariable ist nützlicher, wenn sie in zwei Bereichen vorhanden ist:

- Historie: das Modell lernt, wie sie die Zielgröße beeinflusst hat;
- bekannte Zukunft: das Modell nutzt ihre zukünftigen Werte, um die Zielgröße vorherzusagen.

Beispiel:

```text
date        commandes   pluie_mm   weekend
2026-05-01  120        0          0
2026-05-02  148        4          1
2026-05-03             12         1
```

Die zukünftige Zielgröße ist leer, aber `pluie_mm` und `weekend` sind bekannt. Das Modell kann also unter Berücksichtigung dieser zukünftigen Bedingungen prognostizieren.

## Variablen in Szenarien

In einem kontextuellen Szenario verändert der Nutzer die zukünftigen Kovariablen.

Beispiele:

- `vix_close` um 20 % erhöhen;
- `promo_active` für eine Woche auf 1 setzen;
- `temperature` um 5 Grad senken;
- ein hohes `breach_alert_level` simulieren;
- `trafic_indice` an zukünftigen Tagen ändern.

Forecast startet dann das Modell mit diesem neuen Kontext neu, um eine neue Trajektorie zu erzeugen.

## Glossar der Finanzvariablen

In einem Finanz-Dataset können Kontextvariablen technische Namen haben. Diese Tabelle erklärt die in Finanzszenarien sichtbaren Variablen.

| Variable | Was sie darstellt | Einfache Lesart |
| --- | --- | --- |
| `nasdaq_return_pct` | Veränderung des Nasdaq in Prozent | Misst, ob der Tech-Markt steigt oder fällt |
| `vix_close` | VIX-Stand beim Schlusskurs | Misst die Angst oder Volatilität des Marktes |
| `btc_close_usd` | Bitcoin-Preis in Dollar | Dient als Risikosignal oder Zeichen spekulativen Appetits |
| `usd_index_dxy` | US-Dollar-Index | Misst die Stärke des Dollars gegenüber anderen Währungen |
| `treasury_10y_pct` | US-Zinssatz über 10 Jahre | Stellt die langfristigen Geldkosten dar |
| `sector_etf_volume_musd` | Handelsvolumen eines Sektor-ETFs, in Millionen Dollar | Misst die Aktivität in einem bestimmten Sektor |
| `breach_alert_level` | Alarmstufe bei einer Schwachstelle oder einem Cyber-Vorfall | Stellt einen spezifischen Stress im Cybersicherheitssektor dar |
| `zero_day_news_score` | Nachrichten-Score rund um Zero-Day-Schwachstellen | Misst die Intensität kritischer Cyber-News |
| `gov_contract_flow_index` | Index der öffentlichen Auftragsströme | Stellt die Dynamik der Regierungsaufträge dar |
| `earnings_heat_index` | Spannungsindex rund um Finanzergebnisse | Misst die Bedeutung oder Sensibilität der Ergebnisphase |
| `ai_capex_signal` | Investitionssignal für KI-Ausgaben | Stellt die Stärke des KI-Investitionsthemas dar |
| `fed_event_flag` | Indikator für ein Ereignis der US-Zentralbank | Oft 1, wenn ein Fed-Ereignis vorliegt |
| `option_expiry_flag` | Indikator für Optionsverfall | Signalisiert einen Tag, an dem Optionen Bewegungen beeinflussen können |
| `month_end_flag` | Indikator für Monatsende | Signalisiert Rebalancing- oder Monatsabschluss-Effekte |
| `weekend` | Wochenend-Indikator | Dient vor allem bei täglichen Daten mit Kalendereffekt |

## Wie man diese Variablen liest

Diese Variablen sagen allein nichts voraus. Sie geben dem Modell Kontext.

Beispiele:

- wenn `vix_close` steigt, ist der Markt oft nervöser;
- wenn `fed_event_flag` 1 ist, kann der Tag sensibler auf Zinsansagen reagieren;
- wenn `zero_day_news_score` steigt, können Cyber-Werte reagieren;
- wenn `sector_etf_volume_musd` steigt, ist die Sektoraktivität stärker;
- wenn `month_end_flag` 1 ist, können Bewegungen aus Monatsabschlüssen stammen.

Das Modell lernt in der Historie, ob diese Variablen bereits mit Bewegungen der Zielgröße einhergingen.

## Verwendung in einem Szenario

Im Reiter Szenarien bedeutet das Ändern dieser Variablen, eine Hypothese aufzustellen.

Beispiele:

| Hypothese | Mögliche Änderung |
| --- | --- |
| Stressigerer Markt | `vix_close` +20 % |
| Stärkerer Dollar | `usd_index_dxy` +2 % |
| Fed-Tag | `fed_event_flag` = 1 |
| Starke Cyber-News | `zero_day_news_score` +30 % |
| Heikles Monatsende | `month_end_flag` = 1 |

Nach dem Neustart berechnet Forecast die Trajektorie mit diesem neuen zukünftigen Kontext neu.

## Fehler, die man vermeiden sollte

Eine Kovariable kann das Ergebnis verschlechtern, wenn sie schlecht vorbereitet ist.

Zu vermeiden:

- Variable leer in der bekannten Zukunft;
- konstante Variable, die keine Information liefert;
- Freitext, der nicht in einen nutzbaren Wert umgewandelt wurde;
- Variable, die indirekt die zukünftige Zielgröße enthält;
- gemischte Einheiten in einer Spalte;
- zukünftiger Wert erfunden ohne klare Hypothese.
