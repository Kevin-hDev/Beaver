# Agents LLM

Les agents LLM peuvent utiliser Forecast comme un moteur spécialisé. Leur rôle ne se limite pas à lire un fichier et cliquer sur un tool : ils peuvent préparer les données, chercher du contexte sur le web, construire un dataset, lancer la prédiction et expliquer le résultat.

## Ce que l'agent peut faire

Un agent peut intervenir à plusieurs moments :

| Étape | Rôle de l'agent |
| --- | --- |
| Préparation | Lire un Excel, CSV ou JSON |
| Recherche | Aller chercher des informations externes sur le web |
| Dataset | Créer ou compléter des colonnes utiles |
| Lancement | Appeler `forecast` avec les bons paramètres |
| Lecture | Utiliser `forecast_read` pour récupérer le résultat |
| Scénario | Créer des hypothèses et relancer le modèle |
| Explication | Résumer tendance, incertitude, variables et limites |

Exemple : pour une prévision financière, l'agent peut lire le fichier local, chercher le contexte marché récent, produire des colonnes comme `news_score` ou `event_flag`, puis lancer Forecast.

## Workflow recommandé

L'agent doit suivre cet ordre :

1. comprendre la demande de l'utilisateur ;
2. inspecter les données disponibles ;
3. identifier la cible à prédire ;
4. identifier les dates, la fréquence et l'horizon ;
5. chercher ou créer les variables de contexte utiles si nécessaire ;
6. vérifier que les lignes futures sont cohérentes ;
7. choisir un modèle compatible ;
8. lancer `forecast` ;
9. relire le résultat avec `forecast_read` ;
10. expliquer la prévision et proposer des scénarios utiles.

## Création de données par l'agent

L'agent peut créer des données si l'utilisateur le demande ou si la prédiction le nécessite.

Exemples :

- ajouter une colonne `weekend` à partir de la date ;
- créer `month_end_flag` ;
- transformer un événement web en score numérique ;
- remplir une zone future avec des hypothèses météo ;
- construire un dataset de test pour valider un workflow ;
- convertir un fichier Excel en JSON exploitable.

L'agent doit toujours expliquer quelles colonnes il a créées et pourquoi.

## Règles de sécurité et de qualité

L'agent ne doit pas inventer silencieusement une donnée importante.

S'il crée une variable, il doit distinguer :

- donnée lue dans un fichier ;
- donnée trouvée sur le web ;
- donnée calculée ;
- hypothèse de simulation.

Cette séparation est essentielle pour que l'utilisateur sache ce qui est réel, calculé ou supposé.

## Commandes slash

Les commandes slash servent de guides rapides pour les agents et les utilisateurs.

Exemples :

- `/forecast` : comprendre le module Forecast ;
- `/forecast-predict` : préparer et lancer une prédiction ;
- `/forecast-dataset` : construire un dataset propre ;
- `/forecast-scenarios` : créer des hypothèses utiles ;
- `/forecast-cmd` : comprendre les tools disponibles.

Ces commandes doivent donner une procédure courte, claire et directement actionnable.
