# Famille Toto 2.0

## Positionnement

Utilise Toto comme famille locale probabiliste lorsque plusieurs séries temporelles alignées doivent être modélisées conjointement.

| Propriété | Contrat Beaver |
| --- | --- |
| Fournisseur | Datadog |
| Exécution | locale, CPU ou GPU |
| Licence à vérifier | Apache-2.0 |
| Tailles | 4 M, 22 M, 313 M, 1 B, 2,5 B |
| Séries | unique, lot ou multivarié joint |
| Covariables | non câblées |
| Contexte | jusqu'à 4 096 points |
| Horizon catalogue | jusqu'à 2 048 points |
| Incertitude | intervalles centraux fixes de 60 % ou 80 % |

## Route la famille

Choisis-la lorsque :

- les séries sont alignées sur le même calendrier ;
- leurs dépendances croisées peuvent apporter de l'information ;
- le niveau de confiance vaut exactement 60 % ou 80 % ;
- les ressources permettent la variante sélectionnée.

Écarte-la si des covariables obligatoires sont fournies ou si les séries ne peuvent pas être alignées sans déformer le problème.

## Compare les tailles

En qualité maximale, inclus dès le premier tour la plus grande variante compatible avec le matériel, une variante intermédiaire et une baseline. En équilibre, compare 313 M à 22 M ou 4 M. En rapidité, commence par 4 M ou 22 M et monte si la qualité reste insuffisante.

Garde 1 B ou 2,5 B lorsque leur gain local est utile et stable. Choisis une variante moins coûteuse seulement si elle reste dans la bande d'équivalence.

Compare aussi mode joint et séries indépendantes. Un modèle joint ne gagne que si les interactions améliorent réellement les plis futurs.

## Régle le décodage

Traite `decode_block_size` comme un paramètre expérimental. Garde-le identique entre candidats pendant une comparaison ou inclus son réglage dans le protocole. Enregistre sa valeur avec la provenance.

## Sources vivantes

Vérifie le [dépôt officiel Toto](https://github.com/DataDog/toto) et la fiche Hugging Face de la variante exacte. Laisse l'adaptateur Beaver décider des options réellement disponibles.
