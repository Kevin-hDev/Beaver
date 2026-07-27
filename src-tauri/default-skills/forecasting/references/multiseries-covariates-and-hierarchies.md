# Séries multiples, covariables et hiérarchies

## Définis la sémantique

| Terme | Sens |
| --- | --- |
| série unique | une cible utilise son propre historique |
| lot indépendant | plusieurs séries sont prévues séparément |
| panel | un modèle commun partage des motifs entre séries |
| multivarié joint | les dépendances croisées influencent la prévision |
| hiérarchie | les séries doivent respecter des sommes ou regroupements |

Tu n'utilises jamais `multivarié` comme synonyme de fichier contenant plusieurs séries.

## Contrôle les séries

Tu vérifies :

- identifiant stable ;
- dates et fréquence par série ;
- longueur d'historique ;
- alignement temporel ;
- unités compatibles ;
- trous et doublons ;
- séries nouvelles ou disparues ;
- relation métier réelle ;
- poids décisionnel de chaque série.

Tu ne regroupes pas des séries sans lien uniquement parce qu'elles partagent un fichier.

## Classe les covariables

Tu sépares :

- historiques seulement ;
- futures connues avec certitude ;
- futures prévues et versionnées ;
- statiques ;
- interventions décidées ;
- données indisponibles au cutoff.

Tu refuses une covariable future qui dépend du résultat à prédire ou qui ne sera publiée qu'après la décision.

## Compare les stratégies

Tu compares selon le volume et la relation entre séries :

- un modèle local par série ;
- un modèle global partagé ;
- un modèle par groupe homogène ;
- un panel ;
- un modèle joint ;
- une approche hybride.

Tu conserves une baseline par série et une baseline agrégée. Tu ne choisis pas un modèle joint sans démontrer qu'il améliore les séries importantes et la dépendance utile.

## Traite les covariables futures incertaines

Tu ne remplaces pas une variable future incertaine par sa valeur réalisée. Tu utilises :

- sa prévision disponible au cutoff ;
- plusieurs scénarios ;
- des échantillons joints ;
- une analyse de sensibilité.

Tu propages son incertitude lorsque la décision y est sensible.

## Préserve les hiérarchies

Pour une hiérarchie produit, magasin, région ou temps :

- Tu définis la matrice d'agrégation.
- Tu vérifies les contraintes à chaque niveau.
- Tu compares bottom-up, top-down et réconciliation.
- Tu utilises MinT ou une autre méthode seulement après comparaison.
- Tu évalues précision et calibration à tous les niveaux.
- Tu vérifies la cohérence des trajectoires probabilistes.

Tu ne sacrifie pas silencieusement un niveau critique pour améliorer la moyenne globale.

## Évalue par segment

Tu présentes au minimum :

- macro-moyenne entre séries ;
- score pondéré par importance métier ;
- distribution des erreurs ;
- pires séries ;
- séries nouvelles ou intermittentes ;
- stabilité par régime ;
- cohérence agrégée.

Tu signales tout gain provenant uniquement des grandes séries.
