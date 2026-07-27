# Famille FlowState

## Positionnement

Utilise FlowState comme famille locale compacte lorsque la saisonnalité est connue ou estimable de façon crédible.

| Propriété | Contrat Beaver |
| --- | --- |
| Fournisseur | IBM |
| Exécution | locale, CPU ou GPU |
| Licence à vérifier | Apache-2.0 |
| Variantes | R1 et R1.1 |
| Séries | une série ou plusieurs séries indépendantes |
| Covariables | non câblées |
| Dépendances jointes | non |
| Contexte | 2 048 pour R1 ; 4 096 pour R1.1 |
| Horizon catalogue | jusqu'à 4 096 points |
| Incertitude | intervalles centraux fixes de 60 % ou 80 % |

## Route la famille

Choisis-la lorsque le calendrier fournit une période saisonnière défendable et lorsque le niveau demandé vaut 60 % ou 80 %. Préfère R1.1 si le contexte supplémentaire est nécessaire et si le backtest justifie le léger surcoût.

Écarte-la si la saisonnalité est inconnue, instable ou inventée. Écarte-la pour une confiance non supportée, des covariables obligatoires ou une dépendance jointe.

## Vérifie la période

Déduis la période à partir du calendrier et de la connaissance métier, puis teste sa stabilité. Enregistre la période et l'échelle. Compare au moins une variante saisonnière et une variante non saisonnière si l'hypothèse reste incertaine.

## Compare

Compare R1 et R1.1 sur les mêmes plis, puis compare le gagnant à une baseline saisonnière et à un modèle d'une autre famille. Mesure les horizons courts et longs séparément.

## Sources vivantes

Vérifie le [dépôt officiel Granite TSFM](https://github.com/ibm-granite/granite-tsfm) et les fiches IBM liées à FlowState. Prends les paramètres exposés par Beaver comme contrat courant.
