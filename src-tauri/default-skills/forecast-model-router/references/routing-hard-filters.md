# Filtres durs et abstention

## Applique les filtres dans l'ordre

1. Tu vérifies que `forecast_models` a retourné le candidat pour le profil courant.
2. Tu vérifies la licence pour l'usage réel.
3. Tu vérifies la politique locale ou cloud.
4. Tu vérifies provider, checkpoint, runtime et état de préparation.
5. Tu vérifies le niveau de confiance exact.
6. Tu vérifies série unique, lot, panel ou joint.
7. Tu vérifies les covariables réellement câblées.
8. Tu vérifies fréquence, contexte et horizon.
9. Tu vérifies quantiles, intervalles ou trajectoires.
10. Tu vérifies RAM, VRAM, stockage, OS et durée.
11. Tu vérifies la maturité exigée par le risque.

Tu exclus dès qu'un besoin obligatoire échoue ou reste inconnu.

## Distingue les séries

| Sémantique | Exigence |
| --- | --- |
| série unique | une cible à partir de son historique |
| lot indépendant | plusieurs séries prédites séparément |
| panel | modèle partagé entre séries |
| multivarié joint | dépendances croisées dans la prévision |
| hiérarchie | contraintes entre agrégats et détails |

Tu ne routes pas un besoin joint vers un simple lot.

## Vérifie les covariables

Tu autorises une covariable future seulement si :

- elle est réellement connue sur l'horizon ;
- son type est accepté ;
- l'adapter la transmet ;
- le smoke test ou test de capacité la couvre ;
- le backtest utilise sa version disponible au cutoff.

Tu scénarises une variable future incertaine.

## Vérifie l'horizon

Tu distingues :

- horizon direct ;
- prolongation récursive ;
- limite catalogue ;
- limite dynamique d'une API ;
- horizon réellement backtesté.

Tu exiges un backtest à l'horizon réel lorsqu'une récursion intervient.

## Vérifie les intervalles

Tu n'arrondis pas la confiance. Tu exclues un modèle qui ne sait pas produire le niveau exact demandé selon le backend.

Tu vérifies couverture et largeur. Tu ne confonds pas neuf quantiles natifs avec une calibration démontrée.

## Adapte la preuve au risque

| Risque | Minimum |
| --- | --- |
| exploratoire | candidat prêt et compatible |
| standard | test de capacité + backtest local souhaité |
| critique | backtest comparable, test final, validation humaine, plan de repli |

Tu n'autorises pas une fiche amont à remplacer une preuve locale.

## Abstien-toi proprement

Tu t'abstiens lorsque :

- la licence est ambiguë ;
- le cloud est interdit ;
- aucune capacité exacte n'existe ;
- le modèle est non préparé ;
- le matériel est insuffisant ;
- la vérité terrain est trop faible ;
- le backtest est partiel ;
- aucun candidat ne bat une baseline utile.

Tu proposes l'action minimale qui débloque : changer la confiance, préparer un modèle, corriger les données, lancer un tournoi, utiliser une baseline ou demander une validation.
