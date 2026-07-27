# Métriques, calibration et incertitude

## Relie la métrique à la décision

| Besoin | Mesures principales | Vigilance |
| --- | --- | --- |
| coût linéaire | MAE | dépend de l'échelle |
| grosses erreurs coûteuses | RMSE | dominée par les extrêmes |
| comparaison entre séries | MASE ou RMSSE | saisonnalité du dénominateur |
| lecture relative | sMAPE | instable près de zéro |
| volume agrégé | WAPE | masque les petites séries |
| direction de l'erreur | biais ou ME | peut s'annuler entre segments |
| coût asymétrique | pinball ou perte métier | coûts à définir avant |

Tu ne choisis jamais MAPE seule avec des zéros, valeurs proches de zéro, négatives ou intermittentes.

## Évalue les sorties probabilistes

| Sortie | Contrôles |
| --- | --- |
| quantiles | pinball loss, WIS si disponible, non-croisement |
| intervalles | couverture, largeur, score d'intervalle |
| distribution | CRPS ou log score si disponible |
| événement binaire | Brier, log score, calibration |
| multi-classe | Brier multi-classe, log score, somme à 100 % |
| trajectoires jointes | energy score, variogram score, cohérence |

Une distribution utile doit être calibrée et précise. Tu refuses une méthode qui obtient une bonne couverture uniquement en produisant des intervalles inutilisables.

## Sépare les incertitudes

Tu distingues :

- variabilité du phénomène ;
- incertitude des paramètres ;
- incertitude du modèle ;
- incertitude des variables futures ;
- erreurs de mesure et révisions ;
- changement de régime ;
- ignorance profonde.

Tu ne compresses pas une rupture structurelle dans un intervalle artificiellement précis.

## Contrôle la calibration

Tu mesures la couverture globale et, si le volume le permet :

- par horizon ;
- par saison ;
- par série ;
- par segment ;
- par niveau hiérarchique ;
- par régime.

Tu ajustes une calibration sur les résidus de validation, jamais sur le test final. Tu indiques si la couverture est marginale, locale, conditionnelle ou seulement approximative.

## Utilise les méthodes adaptées

Tu peux utiliser :

- distribution native du modèle ;
- bootstrap de résidus ou trajectoires ;
- régression quantile ;
- ensemble de modèles ;
- post-traitement sur résidus ;
- méthode conforme adaptée à la dépendance temporelle ;
- scénarios pour les ruptures hors historique.

Tu ne présentes jamais une méthode conforme comme une garantie universelle sous dérive ou dépendance non couverte.

## Vérifie la cohérence

Tu contrôles :

- valeurs finies ;
- quantiles ordonnés ;
- médiane entre les bornes ;
- intervalle correspondant au niveau demandé ;
- bornes métier ;
- non-négativité si requise ;
- sommes et contraintes hiérarchiques ;
- même nombre de dates et de prédictions.

## Communique sans surpromettre

Tu présentes :

1. valeur centrale ;
2. intervalle ou distribution ;
3. couverture réellement mesurée ;
4. hypothèses ;
5. facteurs d'élargissement ;
6. régimes absents ;
7. déclencheurs de mise à jour.
