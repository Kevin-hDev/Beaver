# Audit des données et prévention des fuites

## Contrôle la structure

Tu vérifies avant tout calcul :

- types, unités et plage de valeurs ;
- ordre chronologique et dates invalides ;
- fréquence déclarée contre fréquence observée ;
- fuseau horaire, heure d'été et calendrier métier ;
- doublons ;
- périodes absentes contre vraies valeurs nulles ;
- valeurs manquantes, infinies ou non numériques ;
- valeurs atypiques et ruptures de définition ;
- longueur d'historique par rapport à l'horizon et aux saisons ;
- définition du calendrier saisonnier, notamment semaines ISO, années de 52 ou 53 semaines et jours ouvrés ;
- séries, identifiants et hiérarchies ;
- lignes futures et covariables.

Tu bloques la prévision si une erreur empêche d'identifier la cible, le temps, l'unité ou la disponibilité réelle des entrées.

## Fige le temps

Tu distingues trois dates :

1. date du phénomène ;
2. date de publication ;
3. date de révision.

Tu utilises uniquement les valeurs qui auraient réellement été disponibles à l'origine simulée. Pour une statistique révisée, tu utilises son vintage historique. Si aucun vintage n'existe, tu signales que le backtest est optimiste ou non reconstructible.

## Recherche les fuites

Tu cherches explicitement :

- une normalisation ajustée sur tout le dataset ;
- une imputation qui consulte le futur ;
- une fenêtre centrée ou un lissage bidirectionnel ;
- une sélection de variables effectuée avec le test ;
- une météo observée après coup à la place de la prévision météo disponible ;
- une promotion décrite par ses ventes finales ;
- une demande nulle causée par une rupture de stock ;
- un catalogue d'actifs ou produits contenant seulement les survivants ;
- une source web publiée après le cutoff ;
- une contamination possible du pré-entraînement d'un modèle fondationnel.

Tu ajustes chaque transformation uniquement dans la fenêtre d'entraînement, puis tu l'appliques à la validation.

## Classe les variables externes

| Classe | Exemple | Usage |
| --- | --- | --- |
| observée passée | météo historique | historique seulement |
| future connue | calendrier, tarif signé | horizon futur autorisé |
| future prévue | météo prévue | entrée versionnée avec incertitude |
| indisponible au cutoff | ventes consolidées tardives | interdite |
| intervention | campagne décidée | projection conditionnelle, pas causalité |
| statique | catégorie produit | autorisée si stable et connue |

Tu construis l'inventaire `variable -> disponibilité -> source -> date de publication -> révisions`.

## Traite les cas difficiles

- Pour les nombreux zéros, tu distingues absence de demande, indisponibilité et données manquantes.
- Pour une série irrégulière, tu n'inventes pas une régularité sans expliquer l'agrégation.
- Pour une série courte, tu réduis la complexité et tu renforces les baselines.
- Pour une rupture, tu enregistres sa date, le nombre d'observations et d'origines post-rupture, puis tu classes la preuve du régime courant `sufficient`, `weak` ou `unknown`.
- Avant les résultats, tu définis le nombre minimal d'origines post-rupture complètes selon l'horizon, la fréquence et la perte principale.
- Tu classes la preuve `sufficient` uniquement si ce minimum est atteint, si chaque origine reproduit l'horizon réel et si les sensibilités de fenêtre ne renversent pas la conclusion ; sinon tu utilises `weak` ou `unknown`.
- Pour une preuve post-rupture `weak` ou `unknown`, tu compares plusieurs fenêtres, tu renforces les scénarios et tu bloques l'expression `meilleur modèle`.
- Pour des observations récentes révisables, tu sépares `nowcast` et `forecast`, puis tu backtestes la chaîne sur les vintages disponibles.
- Pour une contrainte de non-négativité, tu la valides après inversion des transformations.
- Pour une hiérarchie, tu contrôles les sommes à chaque niveau.

## Évalue la qualité de l'audit

Tu conserves :

- l'empreinte du dataset ;
- le mapping date, cible, série et covariables ;
- les erreurs bloquantes ;
- les avertissements ;
- les corrections appliquées ;
- la fréquence et l'horizon retenus ;
- le niveau de confiance exact ;
- le `data_profile_id`.

Tu relances l'audit après toute modification matérielle. Tu ne réutilises pas un profil lié à une autre empreinte.
