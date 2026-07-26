# Tools Forecast

Les sept tools Forecast forment un parcours contrôlé. Ils gardent les résultats volumineux dans le stockage Forecast et échangent principalement des identifiants compacts avec le LLM.

## Ordre recommandé

Pour un nouveau dataset, tu utilises généralement :

```text
forecast_data_audit
  → forecast_models
  → forecast
  → forecast_read
  → forecast_backtest
  → forecast_compare_models
```

Tu utilises `forecast_analyze` ensuite pour les notes, scénarios ou ensembles.

## `forecast_data_audit`

Tu appelles ce tool avant chaque première prévision sur un nouveau dataset.

Tu fournis les données ou le fichier, la cible, la date, la fréquence, l'horizon et le niveau de confiance exact. Le tool contrôle notamment les dates, doublons, périodes manquantes, valeurs invalides, longueur d'historique, séries, lignes futures et valeurs atypiques.

Une réponse valide fournit un `data_profile_id`. Tu réutilises cet identifiant pour la sélection et la prévision.

## `forecast_models`

Tu inspectes la politique active et les capacités d'intervalle.

En mode Manuel, tu vérifies le modèle imposé et sa compatibilité avec le niveau de confiance exact.

En mode Auto, tu fournis le `data_profile_id`. Tu choisis un seul candidat retourné et tu conserves le `selection_id`. Les informations matérielles utiles sont exposées uniquement dans cette réponse Forecast.

Tu n'arrondis jamais le niveau de confiance pour adapter un modèle.

## `forecast`

Tu lances la prévision validée avec :

- le `data_profile_id` ;
- la cible et la date ;
- l'horizon et la fréquence ;
- le niveau de confiance inchangé ;
- la série et les covariables si elles sont supportées ;
- en mode Auto, le modèle, le `selection_id`, la source et les raisons autorisées.

La réponse fournit un `analysis_id`. Tu n'utilises pas un modèle différent de celui imposé en Manuel ou autorisé en Auto.

## `forecast_read`

Tu omets `analysis_id` pour obtenir une liste bornée des analyses. Tu le fournis pour lire une analyse précise.

Les prédictions sont paginées avec `offset` et `limit`. Une page contient au maximum 200 points.

La lecture peut aussi fournir :

- la décomposition tendance, saisonnalité et résidu ;
- les anomalies fondées sur les résidus ;
- une importance chronologique par permutation ;
- les signaux de dérive.

Tu présentes honnêtement une analyse absente ou peu fiable. Tu ne la remplaces pas par une approximation inventée.

## `forecast_backtest`

Tu lances une validation temporelle glissante sur une analyse sauvegardée. Tu peux demander plusieurs modèles compatibles et un nombre borné de fenêtres.

Le tool évalue les modèles et tente les références Naive, Naive saisonnier, Drift et ETS sur les mêmes périodes.

Tu vérifies toujours le statut et les échecs de modèles. Tu ne présentes pas un résultat partiel comme complet.

## `forecast_compare_models`

Tu lis le classement sauvegardé après le backtest. La réponse résume les erreurs, la couverture, la durée, la mémoire observée et l'état des références.

Tu ne qualifies un modèle de meilleur que lorsqu'un résultat comparable et complet le justifie.

## `forecast_analyze`

Tu modifies une analyse existante avec des actions autorisées :

| Action | Rôle |
| --- | --- |
| `annotate` | Ajoute une note datée |
| `scenario` | Crée un ajustement global ou contextuel |
| `scenario_update` | Modifie un scénario |
| `scenario_delete` | Supprime un scénario |
| `ensemble` | Combine deux à quatre modèles ayant réussi un backtest |

Tu crées un ensemble uniquement après un backtest multi-modèles réussi. Tu indiques qu'il est pondéré par l'inverse du MASE et qu'il n'a pas été évalué indépendamment.

## Quand recommencer le parcours

Tu rappelles `forecast_data_audit` et `forecast_models` si :

- le dataset ou son mapping change ;
- la cible, la fréquence ou l'horizon change ;
- le niveau de confiance change ;
- les besoins en covariables ou multi-séries changent ;
- Forecast signale une sélection expirée ou une évolution des ressources.
