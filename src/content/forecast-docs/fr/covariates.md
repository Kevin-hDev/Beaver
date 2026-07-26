# Covariables

Les covariables sont les variables de contexte qui peuvent influencer la cible. Elles décrivent l'environnement autour de la donnée à prédire.

## Définition

Une covariable répond à la question : "qu'est-ce qui peut aider le modèle à comprendre pourquoi la cible monte, baisse ou change de rythme ?"

Exemples :

| Domaine | Cible | Covariables possibles |
| --- | --- | --- |
| Restaurant | Commandes | météo, jour férié, événement local, promo |
| Finance | Prix ou volume | VIX, taux, news score, Bitcoin, indice dollar |
| SaaS | Charge serveur | utilisateurs actifs, campagne, incident, release |
| Retail | Ventes | stock, remise, saison, trafic magasin |

## Pourquoi elles sont importantes

Sans covariable, le modèle regarde surtout le passé de la cible.

Avec covariables, le modèle peut aussi utiliser le contexte :

- "les ventes montent souvent quand une promo est active" ;
- "les commandes baissent quand il pleut fortement" ;
- "le prix réagit quand le VIX augmente" ;
- "la charge augmente après une campagne marketing".

Elles permettent donc de produire une prévision plus contextualisée qu'une simple prolongation du passé.

## Types de variables de contexte

Une covariable peut être :

| Type | Exemple | Lecture |
| --- | --- | --- |
| Numérique | `temperature = 28` | Valeur mesurée |
| Pourcentage | `discount_pct = 15` | Intensité d'un effet |
| Binaire | `weekend = 1` | Oui / non |
| Événementielle | `concert_local = 1` | Événement présent |
| Score | `news_score = 0.72` | Indicateur calculé |
| Calendrier | `jour_ferie = 1` | Contexte temporel |

## Historique et futur connu

Une covariable est plus utile si elle existe dans deux zones :

- historique : le modèle apprend comment elle a influencé la cible ;
- futur connu : le modèle utilise ses valeurs futures pour prédire la cible.

Exemple :

```text
date        commandes   pluie_mm   weekend
2026-05-01  120        0          0
2026-05-02  148        4          1
2026-05-03             12         1
```

La cible future est vide, mais `pluie_mm` et `weekend` sont connus. Le modèle peut donc prévoir en tenant compte de ces conditions futures.

## Variables dans les scénarios

Dans un scénario contextuel, l'utilisateur modifie les covariables futures.

Exemples :

- augmenter `vix_close` de 20 % ;
- passer `promo_active` à 1 pendant une semaine ;
- baisser `temperature` de 5 degrés ;
- simuler un `breach_alert_level` élevé ;
- changer `trafic_indice` sur les jours futurs.

Forecast relance alors le modèle avec ce nouveau contexte pour produire une nouvelle trajectoire.

## Dictionnaire des variables finance

Dans un dataset financier, les variables de contexte peuvent avoir des noms techniques. Cette table explique les variables visibles dans les scénarios finance.

| Variable | Ce que ça représente | Lecture simple |
| --- | --- | --- |
| `nasdaq_return_pct` | Variation du Nasdaq en pourcentage | Mesure si le marché tech monte ou baisse |
| `vix_close` | Niveau du VIX à la clôture | Mesure la peur ou la volatilité du marché |
| `btc_close_usd` | Prix du Bitcoin en dollars | Sert de signal de risque ou d'appétit spéculatif |
| `usd_index_dxy` | Indice du dollar américain | Mesure la force du dollar face à d'autres devises |
| `treasury_10y_pct` | Taux américain à 10 ans | Représente le coût de l'argent à long terme |
| `sector_etf_volume_musd` | Volume échangé sur un ETF sectoriel, en millions de dollars | Mesure l'activité sur un secteur précis |
| `breach_alert_level` | Niveau d'alerte lié à une faille ou un incident cyber | Représente un stress spécifique au secteur cybersécurité |
| `zero_day_news_score` | Score d'actualité autour des failles zero-day | Mesure l'intensité des nouvelles cyber critiques |
| `gov_contract_flow_index` | Indice de flux de contrats publics | Représente la dynamique des contrats gouvernementaux |
| `earnings_heat_index` | Indice de tension autour des résultats financiers | Mesure l'importance ou la sensibilité de la période de résultats |
| `ai_capex_signal` | Signal d'investissement lié aux dépenses IA | Représente la force du thème investissement IA |
| `fed_event_flag` | Indicateur d'événement de la banque centrale américaine | Vaut souvent 1 lorsqu'un événement Fed est présent |
| `option_expiry_flag` | Indicateur d'expiration d'options | Signale une journée où les options peuvent influencer les mouvements |
| `month_end_flag` | Indicateur de fin de mois | Signale les effets de rééquilibrage ou clôture mensuelle |
| `weekend` | Indicateur week-end | Sert surtout pour les données quotidiennes avec effet calendrier |

## Comment lire ces variables

Ces variables ne prédisent pas toutes seules. Elles donnent du contexte au modèle.

Exemples :

- si `vix_close` monte, le marché est souvent plus nerveux ;
- si `fed_event_flag` vaut 1, la journée peut être plus sensible aux annonces de taux ;
- si `zero_day_news_score` monte, les valeurs cyber peuvent réagir ;
- si `sector_etf_volume_musd` monte, l'activité du secteur est plus forte ;
- si `month_end_flag` vaut 1, certains mouvements peuvent venir de clôtures mensuelles.

Le modèle apprend dans l'historique si ces variables ont déjà accompagné des mouvements de la cible.

## Utilisation dans un scénario

Dans l'onglet Scénarios, modifier ces variables revient à poser une hypothèse.

Exemples :

| Hypothèse | Modification possible |
| --- | --- |
| Marché plus stressé | `vix_close` +20 % |
| Dollar plus fort | `usd_index_dxy` +2 % |
| Journée Fed | `fed_event_flag` = 1 |
| Forte actualité cyber | `zero_day_news_score` +30 % |
| Fin de mois sensible | `month_end_flag` = 1 |

Après relance, Forecast recalcule la trajectoire avec ce nouveau contexte futur.

## Pièges à éviter

Une covariable peut dégrader le résultat si elle est mal préparée.

À éviter :

- variable vide sur le futur connu ;
- variable constante qui n'apporte aucune information ;
- texte libre non transformé en valeur exploitable ;
- variable qui contient indirectement la cible future ;
- unités mélangées dans une même colonne ;
- valeur future inventée sans hypothèse claire.
