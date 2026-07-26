# Incertezza

Una previsione seria non è una sola curva. Forecast associa il valore centrale a un intervallo che rappresenta l'incertezza al livello di confidenza richiesto.

## Valore centrale

Il valore centrale è generalmente la mediana `q50`. Circa metà dei risultati possibili è sotto e metà sopra.

## Livello di confidenza

I modelli continui accettano dal 50% al 99% a passi di un punto percentuale. Senza preferenze, il LLM usa 80%.

Alcuni modelli offrono solo livelli fissi, attualmente 60% o 80%. Forecast conserva la richiesta esatta: Auto restituisce solo candidati compatibili, Manuale segnala l'incompatibilità e non arrotonda mai in silenzio.

## Limiti e quantili

Un intervallo centrale dell'80% usa generalmente `q10`, `q50` e `q90`; al 90% usa `q05`, `q50` e `q95`.

## Ventaglio d'incertezza

Il ventaglio mostra intervalli che si allargano o restringono. Limiti più larghi indicano minore precisione. Un intervallo stretto è utile solo se ben calibrato.

## Copertura misurata

Dopo il backtest, Forecast confronta il livello dichiarato con la quota realmente coperta. Uno storico breve può rendere instabile questa misura.

## Buon uso

Usa l'incertezza per confrontare rischi, distinguere tendenze robuste, preparare soglie prudenti, verificare la calibrazione e non confondere scenari e certezze.
