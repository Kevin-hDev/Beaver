# Scénarios

Un scénario explore une hypothèse à partir d'une analyse existante. Il ne remplace ni les données observées ni la prévision d'origine.

## Ajustement global

Un ajustement en pourcentage crée une courbe dérivée.

Exemples :

- demande supérieure de 10 % ;
- revenus inférieurs de 5 % ;
- capacité augmentée de 15 %.

Ce scénario est rapide à lire, mais il ne relance pas le modèle et ne prouve pas qu'une cause réelle produirait cet effet.

## Scénario contextuel

Un scénario contextuel modifie une ou plusieurs covariables futures, puis relance le modèle lorsque celui-ci supporte ces variables.

Exemples :

- augmenter un budget publicitaire ;
- modifier un prix prévu ;
- appliquer une météo plus chaude ;
- réduire une capacité future ;
- cibler une série particulière.

Les valeurs modifiées restent des hypothèses et doivent être présentées comme telles.

## Création et modification

L'espace Forecast regroupe les scénarios dans son onglet dédié. Vous pouvez y créer, modifier ou supprimer une hypothèse sans surcharger le panneau latéral.

Le panneau conserve la lecture rapide des scénarios existants. Le LLM peut également les gérer avec `forecast_analyze` lorsque vous le demandez dans le chat.

## Comparer les courbes

Affichez la prévision d'origine et les scénarios utiles sur la même période. Vérifiez :

- la date où les courbes commencent à diverger ;
- l'amplitude de l'écart ;
- l'évolution de l'incertitude ;
- les séries concernées ;
- les covariables réellement modifiées.

Une faible différence peut être normale si la variable choisie influence peu le modèle.

## Ensemble de modèles

Un ensemble n'est pas un scénario métier. Il combine deux à quatre modèles ayant réussi un backtest multi-modèles, avec une pondération fondée sur l'inverse du MASE.

Forecast l'indique comme non évalué indépendamment tant qu'un backtest spécifique ne confirme pas ses performances.

## Bon usage

Donnez à chaque scénario :

- un nom clair ;
- une hypothèse mesurable ;
- une période ;
- la source des valeurs ;
- une explication de ce qui change ;
- une comparaison avec la prévision d'origine.
