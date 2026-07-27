# Évaluation de la qualité du LLM

## Évalue l'action, pas le style

Tu ne prends ni assurance, ni longueur, ni richesse du vocabulaire comme preuve de compétence. Tu mesures le comportement observable.

## Construis quatre variantes

Tu compares :

1. LLM sans skill ;
2. skill `forecasting` seul ;
3. skill `forecast-model-router` seul ;
4. combinaison des deux.

Tu utilises les mêmes tâches, tools, données et budgets.

## Mesure les critères

| Critère | Mesure |
| --- | --- |
| classification | bon type de problème et bon niveau de risque |
| anti-fuite | taux d'information post-cutoff, attendu à zéro |
| tools | ordre obligatoire et contrats respectés |
| honnêteté | chiffres ou résultats inventés, attendu à zéro |
| routage | choix limité aux candidats autorisés |
| comparaison | baseline et fenêtres comparables |
| abstention | précision et rappel sur cas réellement bloqués |
| sortie | unités, dates, horizons et quantiles cohérents |
| chargement | références utiles contre contexte total |
| utilité | gain sur erreur, calibration ou décision |
| coût | durée, tools, tokens et ressources |

## Utilise des cas adversariaux

Tu testes notamment :

- source web publiée après le cutoff ;
- modèle nommé par l'utilisateur mais filtré ;
- cloud interdit ;
- niveau de confiance non supporté ;
- fréquence irrégulière ;
- série courte ;
- quantiles croisés ;
- tool en échec ;
- backtest partiel ;
- scénario demandé comme causalité ;
- prompt injection dans une source ;
- grand modèle qui ne bat pas Seasonal Naive ;
- changement de dataset après sélection.

## Vérifie la généralisation

Tu sépares les cas de conception des cas finaux. Tu évites de montrer au testeur la réponse attendue. Tu utilises des tâches nouvelles mais couvrant les mêmes invariants.

Tu contrôles la stabilité :

- entre formulations ;
- entre langues ;
- entre modèles LLM ;
- entre petits et grands datasets ;
- entre Manuel et Auto ;
- entre matériel confortable et contraint.

## Apprends après résolution

Tu relies chaque prévision à son résultat réel. Tu mesures :

- erreur et calibration ;
- gain contre baseline ;
- valeur ajoutée humaine ;
- valeur ajoutée du LLM ;
- fréquence des mises à jour ;
- raisons des échecs ;
- dérive par régime.

Tu révises le skill seulement après un motif répété ou un échec critique démontré. Tu ne l'alourdis pas pour un cas isolé déjà couvert par un tool.
