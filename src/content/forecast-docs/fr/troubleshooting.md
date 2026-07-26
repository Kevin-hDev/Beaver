# Diagnostic

Cette rubrique aide à comprendre pourquoi une prévision ne donne pas le résultat attendu.

## Données JSON invalides

Cette erreur signifie que Forecast n'a pas reçu un tableau exploitable.

Ça peut venir de :

- JSON mal formé ;
- fichier non converti correctement ;
- champ `data` vide ou tronqué ;
- mauvais format de ligne ;
- colonnes absentes.

Si l'utilisateur fournit un fichier, l'agent doit vérifier que le fichier est bien lu avant de convertir les données.

## Modèle non disponible

Cette erreur peut venir de :

- modèle local non installé ;
- sidecar arrêté ;
- clé API absente ;
- modèle incompatible avec les paramètres ;
- données trop courtes pour le modèle demandé.

Le bon réflexe est de vérifier le modèle, puis de tester avec un dataset minimal.

## Variables de contexte ignorées

Une variable peut être ignorée ou inutile si :

- elle n'existe pas dans l'historique ;
- elle est vide dans le futur ;
- elle est constante ;
- elle est mal typée ;
- elle ne correspond pas à l'horizon ;
- elle contient du texte non transformé en nombre ou catégorie exploitable.

Dans ce cas, il faut inspecter le dataset avant d'accuser le modèle.

## Résultat plat

Une prévision plate peut être normale si la cible est stable.

Elle peut aussi indiquer :

- historique trop court ;
- fréquence mal choisie ;
- contexte absent ;
- cible peu variable ;
- modèle trop simple ;
- futur connu peu informatif.

## Scénario sans effet visible

Un scénario contextuel peut afficher peu de différence si :

- la variable modifiée a peu d'influence ;
- la modification est trop faible ;
- le modèle n'utilise pas cette variable comme signal fort ;
- la variable future n'a pas été réellement transmise ;
- la courbe est masquée dans les filtres.

Il faut vérifier le graphe, les filtres, le tooltip et les données du scénario.
