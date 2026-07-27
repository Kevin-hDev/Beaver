# Tools Beaver et provenance

## Respecte l'ordre des tools

Pour un nouveau dataset :

```text
forecast_data_audit
  → forecast_models
  → forecast
  → forecast_read
  → forecast_backtest
  → forecast_compare_models
```

Tu utilises `forecast_analyze` ensuite pour notes, scénarios et ensembles.

## Utilise `forecast_data_audit`

Tu fournis données ou fichier, mapping, fréquence, horizon et confiance exacte. Tu corriges toute erreur bloquante. Tu conserves le `data_profile_id`.

Tu réaudites après changement de données, mapping, cible, fréquence, horizon, confiance, séries ou covariables.

## Utilise `forecast_models`

En Manuel, tu vérifies le modèle forcé et son `interval_capability`.

En Auto :

- Tu fournis le `data_profile_id`.
- Tu choisis un seul candidat retourné.
- Tu conserves `selection_id`, `selection_basis` et raisons autorisées.
- Tu utilises `requested_model_id` seulement pour une demande explicite.
- Tu ne contournes aucun filtre du backend.

Tu ne qualifies pas un choix fondé sur capacités et ressources de meilleur modèle.

## Utilise `forecast`

Tu transmets profil, cible, date, horizon, fréquence, confiance inchangée, série et covariables compatibles. En Auto, tu transmets exactement modèle, sélection et source autorisés.

Tu conserves l'`analysis_id`. Tu ne remplaces pas une erreur par une estimation verbale.

## Utilise `forecast_read`

Tu lis une analyse précise avec son identifiant. Tu pagines les prévisions ; une page contient au plus 200 points.

Tu lis décomposition, anomalies résiduelles, importance chronologique et dérive seulement si elles existent. Tu ne les reconstruis pas par intuition.

## Utilise `forecast_backtest`

Tu évalues sur l'analyse sauvegardée. Tu demandes uniquement des modèles compatibles avec le même profil. Tu lis statut global et `model_failures`.

Tu ne présentes pas un run partiel comme complet.

## Utilise `forecast_compare_models`

Tu lis le classement sauvegardé. Tu compares erreur, biais, perte quantile, couverture, durée, mémoire et baselines.

Tu n'appelles meilleur qu'un candidat dont le résultat complet et comparable bat une référence crédible.

## Utilise `forecast_analyze`

| Action | Condition |
| --- | --- |
| `annotate` | note datée et sourcée |
| `scenario` | ajustement explicitement conditionnel |
| `scenario_update` | scénario existant |
| `scenario_delete` | suppression demandée |
| `ensemble` | deux à quatre modèles ayant réussi un backtest commun |

Tu précises qu'un ensemble pondéré par l'inverse du MASE n'est pas automatiquement évalué comme nouveau modèle.

## Conserve la provenance

Tu archives :

- empreinte des données et `data_profile_id` ;
- cutoff et origine ;
- modèle, fournisseur, checkpoint et révision ;
- runtime et configuration effective ;
- sélection Manuel ou Auto ;
- `selection_basis` et raisons ;
- matériel ;
- baselines, plis et métriques ;
- durée, mémoire et échecs ;
- sources externes et dates ;
- hypothèses, annotations et scénarios.

Tu rends une erreur utilisateur générique. Tu n'exposes jamais chemin interne, stack trace, secret, clé ou payload sensible.
