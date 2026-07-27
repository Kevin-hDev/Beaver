# Backtests, baselines et comparaison

## Reproduis l'usage réel

Tu construis des fenêtres temporelles qui imitent la production :

- même horizon que la décision ;
- mêmes informations disponibles à chaque origine ;
- même délai de publication ;
- mêmes transformations ajustées sur le passé ;
- mêmes séries et segments ;
- mêmes quantiles ;
- même budget de temps et de mémoire.

Tu préfères plusieurs origines glissantes à un seul découpage. Tu utilises une fenêtre croissante quand l'ancien historique reste pertinent et une fenêtre fixe quand les régimes changent.

## Conserve un test final

Tu sépares :

1. entraînement ;
2. validation et sélection ;
3. calibration ;
4. test final verrouillé.

Tu n'utilises pas le test final pour choisir modèle, paramètres, variables, poids d'ensemble ou largeur d'intervalle.

## Construis les baselines

Tu testes selon le profil :

- dernière valeur ;
- Naive saisonnier ;
- Drift ;
- moyenne ou médiane saisonnière ;
- ETS, Theta ou autre référence statistique disponible ;
- prévision métier existante.

Tu ne qualifies pas une méthode avancée d'utile si elle ne bat pas une référence crédible avec un gain stable et matériel.

## Compare honnêtement

Tu conserves les scores par pli avant agrégation. Tu présentes :

- moyenne et médiane ;
- dispersion ;
- biais ;
- pire segment ou régime pertinent ;
- stabilité du rang ;
- amélioration relative à la baseline ;
- durée et mémoire ;
- échecs de modèle.

Tu comptes une erreur d'exécution comme un échec. Tu ne supprimes pas les plis ratés pour améliorer le classement.

## Traite les horizons et panels

Tu mesures chaque horizon séparément lorsque son usage diffère. Tu compares modèles locaux, globaux, panels et joints selon la sémantique réelle des séries.

Tu ne confonds pas :

- plusieurs séries prédites séparément ;
- un modèle partagé entre séries ;
- une distribution jointe qui modélise leurs dépendances.

## Utilise les tests statistiques avec prudence

Lorsque le nombre de plis le permet :

- Tu utilises Diebold-Mariano pour deux erreurs appariées compatibles.
- Tu utilises Giacomini-White pour une comparaison conditionnelle pertinente.
- Tu utilises un Model Confidence Set pour plusieurs candidats.
- Tu utilises un bootstrap temporel ou des erreurs robustes à l'autocorrélation.
- Tu corriges les comparaisons multiples.

Tu ne fais pas un test `t` naïf sur des horizons qui se chevauchent. Tu présentes la taille de l'effet et son utilité, pas seulement une valeur `p`.

## Construis un ensemble

Tu commences par la moyenne simple ou la médiane comme baseline d'ensemble. Tu apprends des poids uniquement sur des prévisions hors échantillon.

Tu imposes :

- poids bornés et régularisés ;
- comparaison à budget égal ;
- calibration après combinaison ;
- aucun accès au test final ;
- stratégie explicite si un membre échoue.

Tu n'assembles pas plusieurs tailles presque identiques pour simuler de la diversité.

## Départage les modèles équivalents

Tu définis une bande d'équivalence ou un gain pratique minimal avant le classement final. Tu optimises d'abord la priorité déclarée : qualité maximale, équilibre ou rapidité. Tu sélectionnes le candidat le moins coûteux uniquement parmi les modèles dont la qualité est équivalente. Tu n'utilises pas cette règle pour imposer l'ordre initial des essais.
