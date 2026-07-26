# Datasets

Un dataset est le tableau que Forecast utilise pour apprendre le passé et prédire le futur. Il doit être structuré pour que le modèle comprenne quoi prédire, à quelles dates, et avec quel contexte.

## Structure minimale

Un dataset Forecast contient au minimum :

| Colonne | Rôle |
| --- | --- |
| Date | Indique quand chaque observation a eu lieu |
| Cible | Valeur à prédire |

Exemple simple :

```text
date        commandes
2026-05-01  120
2026-05-02  135
2026-05-03  128
```

Avec ce tableau, Forecast peut apprendre la dynamique des commandes et prédire les prochaines dates.

## Zone historique

La zone historique contient les valeurs réelles déjà connues.

Elle sert au modèle pour détecter :

- tendance ;
- saisonnalité ;
- rythme ;
- pics ;
- baisses ;
- variations normales.

La cible doit être remplie dans cette zone.

## Zone future

La zone future contient les dates à prédire.

Dans cette zone, la cible est vide, parce que c'est justement ce que Forecast doit calculer.

Exemple :

```text
date        commandes
2026-05-01  120
2026-05-02  135
2026-05-03
2026-05-04
```

Ici, Forecast doit prédire `commandes` pour les 3 et 4 mai.

## Futur connu

Le futur connu ajoute des variables de contexte sur les lignes futures.

Exemple :

```text
date        commandes   pluie_mm   promo
2026-05-01  120        0          0
2026-05-02  135        4          1
2026-05-03             12         0
2026-05-04             0          1
```

La cible future est vide, mais la pluie et la promo sont déjà connues ou supposées. Le modèle peut utiliser ces informations pour produire une prévision plus réaliste.

## Colonne série

La colonne série sert quand un même fichier contient plusieurs objets à prédire.

Exemples :

- plusieurs magasins ;
- plusieurs produits ;
- plusieurs villes ;
- plusieurs actifs financiers ;
- plusieurs serveurs.

Exemple :

```text
date        magasin   ventes   promo
2026-05-01  paris    120      0
2026-05-01  lyon     92       1
2026-05-02  paris    132      1
2026-05-02  lyon     95       0
```

Forecast peut alors prédire chaque série en tenant compte du groupe auquel elle appartient.

## Dataset créé par un agent

Un agent LLM peut créer ou enrichir un dataset.

Il peut par exemple :

- convertir un Excel en JSON ;
- ajouter une colonne `weekend` ;
- récupérer des événements sur le web ;
- transformer une information textuelle en score ;
- remplir les lignes futures avec des hypothèses ;
- nettoyer des dates ou des colonnes.

L'agent doit indiquer clairement quelles données viennent du fichier, du web, d'un calcul ou d'une hypothèse.
