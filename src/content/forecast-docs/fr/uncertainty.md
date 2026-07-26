# Incertitude

Une prévision sérieuse ne se limite pas à une seule courbe. Forecast associe la valeur centrale à un intervalle qui représente l'incertitude du modèle pour le niveau de confiance demandé.

## Valeur centrale

La valeur centrale correspond généralement à la médiane, souvent notée `q50`. Elle sépare les résultats possibles en deux groupes :

- environ la moitié en dessous ;
- environ la moitié au-dessus.

Elle ne garantit pas que la valeur réelle suivra exactement cette trajectoire.

## Niveau de confiance

Le niveau de confiance accepté se situe entre 50 % et 99 %, par pas d'un point de pourcentage pour les modèles continus. Si l'utilisateur ne précise rien, le LLM utilise 80 %.

Certains modèles ne fournissent honnêtement que des niveaux fixes, actuellement 60 % ou 80 %. Forecast respecte toujours la valeur exacte demandée :

- en mode Auto, il ne propose que des modèles compatibles ;
- en mode Manuel, il signale l'incompatibilité ;
- il n'arrondit jamais silencieusement la demande vers une valeur supportée.

## Bornes et quantiles

Pour un niveau central de 80 %, Forecast utilise généralement :

- `q10` comme borne basse ;
- `q50` comme médiane ;
- `q90` comme borne haute.

Pour 90 %, les bornes deviennent généralement `q05` et `q95`. Les libellés s'adaptent aux niveaux réellement calculés.

## Éventail d'incertitude

Le graphique en éventail aide à voir l'élargissement ou le resserrement des intervalles au fil de l'horizon. Plus les bornes s'écartent, moins le modèle est précis sur cette période.

Un intervalle étroit n'est utile que s'il est bien calibré.

## Couverture mesurée

Après un backtest, Forecast compare le niveau annoncé à la proportion de valeurs réellement couvertes. Par exemple, un intervalle théorique de 80 % devrait contenir environ 80 % des observations de validation.

Un historique trop court peut rendre cette mesure instable. L'interface le signale.

## Bon usage

Utilisez l'incertitude pour :

- comparer les risques entre plusieurs périodes ;
- distinguer une tendance robuste d'une trajectoire fragile ;
- préparer des seuils prudents ;
- vérifier la calibration avec un backtest ;
- comparer des scénarios sans confondre hypothèse et certitude.
