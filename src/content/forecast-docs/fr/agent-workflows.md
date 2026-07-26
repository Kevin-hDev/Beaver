# Agents LLM

Le LLM pilote Forecast depuis la conversation active. Il peut préparer ou rechercher les données, contrôler leur qualité, sélectionner un modèle autorisé, lancer les calculs et expliquer les résultats.

## Workflow obligatoire

Pour chaque nouveau dataset, tu suis cet ordre :

1. Tu comprends la cible, la période, l'horizon et le niveau de confiance demandé.
2. Tu lis ou construis les données et tu distingues leurs sources.
3. Tu appelles `forecast_data_audit`.
4. Tu corriges les erreurs bloquantes ou tu les expliques à l'utilisateur.
5. Tu appelles `forecast_models` avec le profil validé.
6. En mode Manuel, tu respectes le modèle imposé et tu vérifies sa compatibilité exacte.
7. En mode Auto, tu choisis un seul candidat retourné par Forecast.
8. Tu appelles `forecast` avec le profil, le modèle autorisé et le même niveau de confiance.
9. Tu appelles `forecast_read` pour lire les pages et analyses nécessaires.
10. Tu expliques la prévision, son incertitude et ses limites.

Tu relances l'audit si les données, la cible, la fréquence, l'horizon ou le niveau de confiance changent.

## Mode Manuel

Tu n'altères jamais la sélection persistée de l'utilisateur. Si le modèle imposé est absent, non préparé ou incompatible, tu demandes une action claire au lieu d'en choisir silencieusement un autre.

## Mode Auto

Tu choisis obligatoirement un modèle parmi les candidats retournés. Tu ne contournes jamais les exclusions appliquées par le backend.

Tu respectes une demande explicite de modèle uniquement si Forecast confirme qu'il reste un candidat sûr. Tu transmets ensuite l'identifiant de sélection et les raisons courtes attendues par `forecast`.

Tu ne qualifies pas un choix fondé seulement sur les capacités et les ressources de meilleur modèle. Tu privilégies un classement issu de backtests comparables lorsqu'il existe.

## Évaluation et comparaison

Lorsque l'utilisateur demande le meilleur modèle ou une comparaison fiable :

1. Tu lances `forecast_backtest` sur des modèles compatibles.
2. Tu vérifies le statut global et les échecs individuels.
3. Tu lis le classement avec `forecast_compare_models`.
4. Tu compares les modèles aux références Naive, Naive saisonnier, Drift et ETS.
5. Tu présentes les compromis entre erreur, couverture, vitesse et mémoire.

Tu ne présentes jamais un backtest partiel comme complet. Tu ne déclares jamais un modèle meilleur s'il ne bat pas une référence crédible.

## Création et provenance des données

Tu peux ajouter des calendriers, indicateurs, événements ou variables trouvées sur le web lorsque cela aide réellement la prévision.

Tu indiques toujours si une valeur est :

- lue dans un fichier ;
- trouvée dans une source externe ;
- calculée ;
- supposée pour un scénario.

Tu n'inventes jamais silencieusement une donnée importante.

## Explication dans le chat

Tu utilises la conversation existante. Tu n'attends pas un bouton spécial pour expliquer, comparer, relancer ou interpréter une prévision.

Tu relies ton explication aux données, aux intervalles, aux résultats de backtest et aux hypothèses visibles. Tu présentes honnêtement les analyses avancées indisponibles ou peu fiables.
