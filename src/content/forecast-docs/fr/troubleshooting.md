# Diagnostic

Cette rubrique distingue les comportements normaux des problèmes qui nécessitent une action.

## Préparation d'un modèle

Un modèle peut afficher :

- **Non installé** : lancez Préparer ;
- **Mise à jour requise** : relancez la préparation pour actualiser le moteur ou la validation ;
- **Invalide** : désinstallez puis préparez de nouveau le modèle ;
- **Provider requis** : configurez la clé du fournisseur cloud ;
- **Prêt** : le modèle peut être sélectionné.

Les préparations multiples entrent dans une file d'attente. Les fichiers déjà valides sont réutilisés lorsque c'est possible.

## Cycle du sidecar

Le moteur local démarre pour une prévision ou un backtest, puis peut s'arrêter juste après l'opération. Cet arrêt est normal et libère les ressources.

Il y a un problème uniquement si le moteur ne devient pas prêt, si la requête échoue ou si Forecast retourne une erreur.

## Audit des données refusé

L'audit peut bloquer pour :

- une colonne requise absente ;
- des dates invalides ou dupliquées ;
- une fréquence incohérente ;
- un historique insuffisant ;
- des lignes futures incorrectes ;
- trop de données pour les limites autorisées.

Corrigez l'erreur indiquée puis relancez l'audit. Ne lancez pas la prévision avec un ancien profil si le dataset a changé.

## Niveau de confiance incompatible

Les modèles continus acceptent des niveaux entiers entre 50 % et 99 %. Certains modèles à grille fixe acceptent uniquement 60 % ou 80 %.

En mode Manuel, choisissez un niveau supporté ou un autre modèle. En mode Auto, relancez la sélection avec le niveau exact demandé. Forecast ne doit jamais l'arrondir silencieusement.

## Sélection Auto expirée

Une sélection Auto est liée au dataset, à la session et aux ressources disponibles. Si elle expire ou si les conditions changent, le LLM rappelle `forecast_models`, récupère un nouvel identifiant puis relance `forecast`.

## Résultat absent du panneau

Une analyse valide ouvre normalement le panneau et synchronise l'espace Forecast. Si rien n'apparaît :

1. vérifiez que Forecast a retourné un `analysis_id` ;
2. sélectionnez l'analyse dans l'historique Forecast ;
3. vérifiez que la session active est la bonne ;
4. relancez la lecture de l'analyse.

Une sortie rejetée pendant la validation n'est pas affichée comme une prévision valide.

## Backtest partiel

Un backtest peut réussir pour les références et échouer pour un ou plusieurs modèles. Consultez le statut et les échecs individuels.

Ne considérez pas le classement comme complet tant que les modèles que vous voulez comparer n'ont pas tous produit des résultats homogènes.

## Variables de contexte ignorées

Une covariable peut être inutilisable si elle est absente, vide dans le futur, constante, mal typée, mal alignée avec l'horizon ou non supportée par le modèle.

Vérifiez l'onglet Données, le modèle sélectionné et les valeurs futures.

## Résultat plat ou scénario peu visible

Une courbe plate peut refléter une cible stable, un historique court, une fréquence incorrecte ou un manque de contexte.

Un scénario peut avoir peu d'effet si la modification est faible, si la variable influence peu le modèle ou si la couche est masquée dans les filtres. Comparez les données et les hypothèses avant de conclure à une erreur.
