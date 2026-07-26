# Datasets

La qualité d'une prévision dépend d'abord des données. Forecast sépare les lignes historiques, les informations futures déjà connues et les hypothèses créées pour un scénario.

## Structure minimale

Un dataset exploitable contient au minimum :

| Élément | Rôle |
| --- | --- |
| Colonne de date | Place chaque observation dans le temps |
| Colonne cible | Contient la valeur à prévoir |
| Fréquence | Définit le rythme : heure, jour, semaine, mois, trimestre ou année |
| Horizon | Définit le nombre de prochaines périodes à prévoir |

Une colonne de série peut séparer plusieurs produits, régions ou capteurs. Des covariables peuvent ajouter du contexte.

## Zone historique

Les lignes historiques contiennent une date et une cible observée. Elles doivent être ordonnées, suffisamment nombreuses et cohérentes avec la fréquence choisie.

Forecast vérifie notamment :

- les dates invalides ou désordonnées ;
- les doublons ;
- les périodes manquantes ;
- les valeurs absentes ou non numériques ;
- les valeurs atypiques ;
- la longueur de l'historique par rapport à l'horizon ;
- la cohérence de chaque série.

Une erreur structurelle bloque la prévision. Un risque non bloquant reste visible comme avertissement.

## Zone future

Les lignes futures peuvent omettre la cible, puisque c'est la valeur à prévoir. Elles sont utiles lorsqu'elles contiennent des informations déjà connues pour les prochaines périodes.

Exemples :

- calendrier et jours fériés ;
- prix planifié ;
- budget prévu ;
- campagne programmée ;
- météo prévisionnelle ;
- capacité ou stock attendu.

Une information future inconnue ne doit pas être présentée comme un fait.

## Audit avant prévision

Chaque nouveau dataset passe par `forecast_data_audit` avant le calcul. L'audit vérifie les données, l'horizon, la fréquence et le niveau de confiance demandé.

Quand l'audit est valide, Forecast crée un profil réutilisable. Le LLM utilise ensuite ce profil pour sélectionner un modèle et lancer la prévision sans renvoyer inutilement toutes les données dans la conversation.

Si les données, la cible, l'horizon, la fréquence ou le niveau de confiance changent, un nouvel audit est nécessaire.

## Données créées ou enrichies par le LLM

Le LLM peut lire un CSV, un tableur ou du JSON, rechercher du contexte et créer des colonnes utiles. Il doit distinguer clairement :

- une donnée lue dans un fichier ;
- une donnée trouvée sur le web ;
- une donnée calculée ;
- une hypothèse de simulation.

Cette provenance permet de comprendre ce qui est réel, dérivé ou supposé.

## Aperçu dans l'espace Forecast

L'onglet Données affiche le nombre de lignes, les points historiques, les lignes futures, les séries, les périodes manquantes et les valeurs atypiques. Il montre aussi la cible, la date, la fréquence, les covariables et un aperçu borné du dataset.
