# Tools Forecast

Les tools Forecast permettent aux agents LLM d'utiliser le moteur de prédiction depuis le chat. Ils doivent être appelés avec des paramètres précis, car une mauvaise colonne ou un horizon incohérent produit une prévision inutile.

## `forecast`

`forecast` lance une nouvelle prédiction.

Entrées principales :

| Paramètre | Rôle |
| --- | --- |
| `file_path` | Fichier Excel, CSV ou JSON à lire |
| `data` | Données déjà préparées en JSON |
| `date_column` | Colonne qui contient les dates |
| `target_column` | Colonne à prédire |
| `series_column` | Colonne qui identifie les séries |
| `covariate_columns` | Variables de contexte à utiliser |
| `frequency` | Rythme temporel |
| `horizon` | Nombre de points futurs |
| `model` | Moteur à utiliser |

Sortie principale :

- `analysis_id`, l'identifiant du résultat Forecast.

## `forecast_read`

`forecast_read` relit un résultat Forecast.

Il sert à récupérer :

- la prévision ;
- l'historique ;
- l'incertitude ;
- les scénarios ;
- les variables disponibles ;
- les métadonnées du modèle.

Si aucun `analysis_id` n'est fourni, l'agent peut l'utiliser pour lister les résultats disponibles.

## `forecast_analyze`

`forecast_analyze` ajoute ou modifie des éléments autour d'une prévision.

Il sert notamment à :

- créer une annotation ;
- créer un scénario ;
- relancer un scénario contextuel ;
- modifier un scénario ;
- supprimer un scénario.

## Ce que l'agent doit vérifier

Avant d'appeler un tool, l'agent doit vérifier :

- la cible existe ;
- la date est lisible ;
- l'horizon correspond aux lignes futures ;
- les covariables existent vraiment ;
- les données créées ou trouvées sur le web sont identifiées ;
- le modèle choisi supporte le besoin.

L'agent doit expliquer ses choix au lieu d'envoyer un appel opaque.
