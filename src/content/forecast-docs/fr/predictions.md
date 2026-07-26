# Prévisions

Une prévision prolonge une ou plusieurs séries à partir de leur historique, de leurs variables de contexte et du modèle sélectionné. Elle contient une estimation centrale ainsi que des bornes d'incertitude lorsque le modèle les supporte.

## Résultat sauvegardé

Chaque calcul valide crée un `analysis_id`. Cet identifiant relie la prévision au panneau, à l'espace Forecast, aux scénarios, aux notes, aux évaluations et aux exports.

Forecast vérifie avant la sauvegarde :

- le nombre de points et de séries attendu ;
- les dates futures et leur ordre ;
- l'absence de valeurs non numériques ;
- l'alignement de la médiane et des quantiles ;
- l'horizon réellement produit.

Une sortie partielle ou incohérente n'est pas enregistrée comme une analyse valide.

## Graphique principal

Le graphique principal distingue l'historique de la zone prévue. Les filtres permettent d'afficher ou masquer les séries, l'incertitude, les scénarios, les événements, les comparaisons, les anomalies et les signaux de qualité disponibles.

Vous pouvez :

- faire glisser le graphique pour vous déplacer ;
- utiliser la molette ou le pavé tactile pour zoomer ;
- utiliser les barres de saut pour changer rapidement de niveau de détail ;
- replier une carte pour alléger la page ;
- ouvrir le tableau des points lorsque des valeurs exactes sont nécessaires.

Le zoom ne bloque pas le défilement de la page lorsqu'aucun changement de zoom n'est possible.

## Graphiques complémentaires

L'espace Forecast peut afficher :

- un éventail d'incertitude pour voir comment les intervalles évoluent ;
- une vue saisonnière qui compare les périodes disponibles ;
- un graphique de fiabilité après un backtest.

Pour une analyse multi-séries, la série active reste synchronisée entre les graphiques.

## Tableau des prédictions

Le tableau reste replié par défaut pour ne pas alourdir l'affichage. Une fois ouvert, il présente les dates, la valeur centrale et les bornes disponibles dans une zone à hauteur limitée et défilable.

Pour une analyse très longue, `forecast_read` renvoie les points par pages bornées plutôt que de charger toute la série dans le contexte du LLM.

## Mise à jour en temps réel

Le panneau et l'espace Forecast lisent la même analyse sauvegardée. Une nouvelle prévision, une modification ou un changement d'analyse active actualise les vues concernées sans exiger de fermer puis rouvrir la fenêtre.

## Interprétation correcte

Lisez toujours la courbe avec :

- la qualité des données ;
- le niveau d'incertitude ;
- l'horizon ;
- les ruptures ou anomalies ;
- les résultats de backtest ;
- les modèles de référence ;
- les hypothèses utilisées.

Une courbe régulière n'est pas une preuve de précision.
