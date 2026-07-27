# Famille Chronos-Bolt

## Positionnement

Utilise Chronos-Bolt comme famille probabiliste locale, simple et efficace pour des séries traitées indépendamment. Considère Tiny comme candidat très léger et Base comme candidat de forte capacité. Choisis leur ordre d'évaluation selon la priorité de l'utilisateur.

| Propriété | Contrat Beaver |
| --- | --- |
| Fournisseur | Amazon |
| Exécution | locale, CPU ou GPU |
| Licence à vérifier | Apache-2.0 |
| Séries | une série ou plusieurs séries indépendantes |
| Covariables | non câblées |
| Dépendances jointes | non |
| Contexte utile | jusqu'à 2 048 points dans l'adaptateur |
| Horizon catalogue | jusqu'à 1 000 points |
| Incertitude | quantiles ; confiance entière de 50 % à 99 % |

## Route la famille

Choisis-la lorsque :

- l'utilisateur exige une exécution locale ;
- la cible est une série unique ou un lot indépendant ;
- tu veux une baseline neuronale probabiliste rapide ;
- les ressources sont limitées ;
- aucune covariable n'est obligatoire.

Écarte-la lorsque des covariables doivent réellement influencer la prévision, lorsque les séries doivent interagir conjointement ou lorsque le contexte nécessaire dépasse le contexte effectif.

## Compare les tailles

En qualité maximale, teste Base dès le premier tour avec Small ou Mini et une baseline. En équilibre, compare Small ou Mini à Base. En rapidité, commence par Tiny ou Mini et monte si le backtest révèle un manque de qualité. Compare toutes les tailles retenues sur les mêmes plis et au même horizon.

Garde Base lorsque son gain est utile et stable. Garde le modèle le moins coûteux lorsque son écart avec le meilleur reste inférieur au seuil pratique défini. Ne suppose pas que Base gagne parce qu'il contient plus de paramètres, mais ne l'écarte pas du premier tour lorsque la qualité maximale est demandée. Mesure aussi la durée, la mémoire, la calibration et la stabilité.

Compare Chronos-Bolt à Chronos-2 comme une génération différente. Tu ne déduis pas leur qualité relative de leur nombre de paramètres.

## Contrôle l'horizon

La sortie native est plus courte que l'horizon catalogue. Traite toute extension récursive comme une propriété de l'adaptateur, puis backteste exactement à l'horizon demandé. Réduis ta confiance si l'erreur ou la largeur des intervalles croît fortement avec l'horizon.

## Sources vivantes

Vérifie les changements importants dans le [dépôt officiel Chronos](https://github.com/amazon-science/chronos-forecasting) et la fiche officielle du modèle exact sur Hugging Face. Prends toujours les capacités renvoyées par Beaver comme contrat exécutable.
