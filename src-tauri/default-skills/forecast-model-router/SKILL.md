---
name: forecast-model-router
description: "Use when selecting, comparing, routing, sizing, or explaining Beaver Forecast models in Manual or Auto mode. Triggers: forecast model, modèle de prévision, model selector, meilleur modèle, Auto, Chronos, TimesFM, TimeGPT, Toto, MOIRAI, FlowState."
---

# Routeur des modèles Forecast

Tu sélectionnes un modèle de prévision parmi les candidats que Beaver autorise. Tu relies le profil de tâche, les capacités réellement testées, les backtests locaux et les contraintes de ressources. Tu ne transformes jamais une fiche statique ou un nom commercial en preuve de supériorité.

<critical_constraints>
- Tu choisis uniquement un modèle renvoyé par `forecast_models` pour le `data_profile_id` courant.
- Tu ne réactives jamais un candidat filtré par le backend.
- Tu ne contournes jamais licence, permission cloud, niveau de confiance, capacité, runtime ou ressources.
- Tu ne qualifies jamais un modèle de meilleur sans backtest local complet et comparable.
- Tu traites les fiches de ce skill comme connaissances stables ; tu traites le backend comme source de vérité vivante.
- Tu distingues série unique, lot indépendant, panel et multivarié joint.
- Tu ne compenses jamais un filtre obligatoire par un bon score secondaire.
- Tu n'utilises jamais la taille, le suffixe Pro/Base ou un leaderboard comme preuve unique.
- Tu ne privilégies ni le plus petit ni le plus gros modèle avant une comparaison adaptée à l'objectif.
- Tu utilises le coût pour départager uniquement les modèles dont la qualité appartient à la bande d'équivalence utile.
- Tu t'abstiens lorsque la capacité, la licence ou la preuve nécessaire manque.
- Tu ne bascules jamais du local vers le cloud sans autorisation explicite.
- Tu ne réutilises jamais un ticket expiré, consommé ou lié à un autre profil.
</critical_constraints>

## Démarrage rapide

1. Tu reçois le `ForecastTaskProfile` produit par `$forecasting` et le `data_profile_id` de `forecast_data_audit`.
2. Tu appelles `forecast_models` avec le profil courant et `requested_model_id` uniquement si l'utilisateur demande un modèle exact.
3. Tu élimines mentalement toute option qui ne répond pas à l'intention, sans ajouter de candidat à la liste du backend.
4. Tu privilégies `selection_basis=rolling_backtest`; sinon tu qualifies le choix de compatible par capacités.
5. Tu choisis un candidat, puis tu transmets exactement son modèle et son `selection_id` à `forecast`.

## Applique les responsabilités

| Composant | Responsabilité |
| --- | --- |
| skill `forecasting` | cible, cutoff, données, protocole, métriques et décision |
| backend Beaver | installation, capacités, confiance, cloud, ressources, liste courte et ticket |
| ce routeur | interprétation du besoin, choix entre candidats et explication |
| tools d'évaluation | baselines, plis, métriques, calibration, mémoire et durée |

Tu ne redéfinis pas le protocole scientifique général. Tu lis le profil préparé par `$forecasting`.

## Vérifie le profil

Tu contrôles :

- cible, unité et bornes ;
- fréquence et calendrier ;
- longueur d'historique ;
- horizon demandé ;
- série unique, lot, panel, joint ou hiérarchie ;
- covariables passées et futures réellement connues ;
- point, quantiles, intervalle ou trajectoires ;
- perte métier et niveau de risque ;
- priorité qualité maximale, équilibre ou rapidité ;
- local ou cloud ;
- licence d'usage ;
- RAM, VRAM, CPU, GPU, OS et temps.

Tu poses une question uniquement si la réponse change le filtre ou le choix.

## Applique les filtres durs

Tu appliques dans cet ordre :

1. modèle retourné par le backend ;
2. licence compatible ;
3. politique locale ou cloud ;
4. runtime et état de préparation ;
5. niveau de confiance exact ;
6. sémantique des séries ;
7. covariables câblées ;
8. fréquence, contexte et horizon ;
9. sortie probabiliste ou trajectoires ;
10. ressources avec marge ;
11. preuve suffisante pour le niveau de risque.

Tu lis [routing-hard-filters.md](references/routing-hard-filters.md) pour un cas ambigu, critique ou incompatible.

## Classe les candidats

Lorsque des backtests comparables existent, tu respectes l'ordre du backend :

1. baseline battue ;
2. MASE ;
3. perte quantile ;
4. écart de couverture ;
5. sMAPE ;
6. MAE ;
7. RMSE ;
8. biais absolu ;
9. stabilité ;
10. mémoire ;
11. durée.

Sans backtest, tu privilégies compatibilité, diversité de famille, preuves disponibles et objectif déclaré. Tu ne réduis pas systématiquement la taille et tu ne construis pas un score sur 100.

Tu traites cet ordre comme un classement technique générique. Pour une tâche critique avec coût asymétrique, tu ne l'utilises pas seul pour dire `meilleur` : tu exiges ensuite une comparaison sur la perte métier ou le quantile décisionnel défini avant les résultats.

Tu lis [ranking-and-tournaments.md](references/ranking-and-tournaments.md) avant une comparaison, un ensemble ou un choix entre tailles.

## Choisis selon la sémantique

| Besoin | Familles à examiner si le backend les retourne |
| --- | --- |
| CPU rapide et mono-série | Chronos-Bolt, Toto compact, FlowState, Kairos |
| long contexte | TimesFM, Chronos-2, FlowState |
| covariables locales | Chronos-2, TimesFM pour variables numériques compatibles |
| dépendances jointes locales | Toto 2.0 sur séries alignées |
| trajectoires échantillonnées | Sundial |
| cloud géré | TimeGPT avec consentement et accès |
| saisonnalité connue | FlowState |
| modèle compact probabiliste | TiRex, Kairos, Chronos-Bolt, Toto |

Cette table ne crée aucun candidat. Le résultat vivant de `forecast_models` reste prioritaire.

## Choisis la taille

Dans Chronos-Bolt, Toto ou Kairos :

1. Tu fixes la priorité avant de choisir les tailles.
2. En qualité maximale, tu testes dès le premier tour la variante compatible disposant de la plus forte capacité ou des meilleures preuves, avec une variante plus légère et une baseline.
3. En équilibre, tu testes un candidat de forte qualité et un candidat moins coûteux.
4. En rapidité, tu commences par une variante compacte et tu montes uniquement si elle reste insuffisante.
5. Tu compares les candidats sur les mêmes plis et tu définis un gain pratique minimal.
6. Tu gardes le modèle le plus qualitatif lorsque son avantage est utile et stable.
7. Tu choisis le moins coûteux uniquement lorsque les modèles sont équivalents selon le seuil défini.

Tu ne confonds jamais taille et qualité. Tu compares les générations et architectures différentes comme des familles distinctes : un modèle plus récent et plus petit peut battre une ancienne variante plus grande.

## Gère Manuel et Auto

### Manuel

- Tu respectes le modèle imposé.
- Tu vérifies son état et sa capacité exacte.
- Tu demandes à l'utilisateur de changer modèle, données ou confiance si nécessaire.
- Tu ne proposes pas un remplacement silencieux.

### Auto

- Tu choisis un seul candidat retourné.
- Tu interprètes `meilleure prévision`, usage critique ou coût d'erreur élevé comme une priorité de qualité maximale.
- Tu utilises l'équilibre lorsque l'utilisateur ne donne aucune priorité et que le risque n'impose pas la qualité maximale.
- Tu respectes une priorité explicite de rapidité, de mémoire ou de coût.
- Tu conserves `selection_basis`, `selection_id` et raisons autorisées.
- Tu respectes une demande explicite seulement si le backend renvoie ce modèle comme sûr.
- Tu relances `forecast_models` après expiration, consommation ou changement de profil.
- Tu ne modifies pas la préférence persistée.

## Abstention et repli

Tu t'abstiens si :

- aucun candidat ne bat une baseline avec un gain utile ;
- une capacité obligatoire n'est pas testée ;
- la licence ou la politique de données reste ambiguë ;
- les backtests sont trop faibles ou partiels ;
- une rupture rend l'historique non représentatif ;
- le besoin critique n'a pas de validation humaine.

Tu proposes alors baseline, clarification ciblée, nouveau tournoi, correction des données, scénario, modèle Manuel compatible ou validation humaine.

## Charge uniquement les références nécessaires

### Doctrine transversale

- [routing-hard-filters.md](references/routing-hard-filters.md) — compatibilité, exclusions et abstention.
- [ranking-and-tournaments.md](references/ranking-and-tournaments.md) — classement, baselines, tailles et ensembles.
- [capability-evidence-and-freshness.md](references/capability-evidence-and-freshness.md) — vocabulaire, états C0–C6 et fiabilité S0–S4.
- [licenses-cloud-and-hardware.md](references/licenses-cloud-and-hardware.md) — licences, cloud, OS, RAM et VRAM.

### Fiches de familles

| Famille | Fiche |
| --- | --- |
| Chronos-Bolt | [family-chronos-bolt.md](references/family-chronos-bolt.md) |
| Chronos-2 | [family-chronos-2.md](references/family-chronos-2.md) |
| TimesFM | [family-timesfm.md](references/family-timesfm.md) |
| TimeGPT | [family-timegpt.md](references/family-timegpt.md) |
| Toto | [family-toto.md](references/family-toto.md) |
| MOIRAI | [family-moirai.md](references/family-moirai.md) |
| FlowState | [family-flowstate.md](references/family-flowstate.md) |
| TabPFN-TS | [family-tabpfn-ts.md](references/family-tabpfn-ts.md) |
| TiRex | [family-tirex.md](references/family-tirex.md) |
| Kairos | [family-kairos.md](references/family-kairos.md) |
| Sundial | [family-sundial.md](references/family-sundial.md) |

### Fiches des modèles exacts

| Famille | Fiches |
| --- | --- |
| Chronos-Bolt | [Tiny](references/model-chronos-bolt-tiny.md), [Mini](references/model-chronos-bolt-mini.md), [Small](references/model-chronos-bolt-small.md), [Base](references/model-chronos-bolt-base.md) |
| Chronos-2 | [Chronos-2](references/model-chronos-2.md) |
| TimesFM | [2.5 200M](references/model-timesfm-2.5-200m.md) |
| TimeGPT | [Mini](references/model-timegpt-2-mini.md), [Standard](references/model-timegpt-2-standard.md), [Pro](references/model-timegpt-2-pro.md), [2.1](references/model-timegpt-2.1.md) |
| Toto | [4M](references/model-toto-2.0-4m.md), [22M](references/model-toto-2.0-22m.md), [313M](references/model-toto-2.0-313m.md), [1B](references/model-toto-2.0-1b.md), [2.5B](references/model-toto-2.0-2.5b.md) |
| MOIRAI | [R Small](references/model-moirai-2.0-r-small.md) |
| FlowState | [R1](references/model-flowstate-r1.md), [R1.1](references/model-flowstate-r1.1.md) |
| TabPFN-TS | [TS-3](references/model-tabpfn-ts-3.md) |
| TiRex | [35M](references/model-tirex-35m.md) |
| Kairos | [10M](references/model-kairos-10m.md), [23M](references/model-kairos-23m.md), [50M](references/model-kairos-50m.md) |
| Sundial | [128M](references/model-sundial-128m.md) |

Tu lis uniquement les fiches des candidats retournés par le backend.

## Explique le choix

Tu présentes :

```text
Modèle choisi :
Pourquoi il est admissible :
Base de sélection : rolling_backtest | capabilities_resources
Baseline comparée :
Finalistes écartés :
Compromis qualité / calibration / vitesse / mémoire / coût :
Capacités non validées :
Condition de repli ou de nouvelle sélection :
```

Tu utilises `compatible` si la preuve locale manque. Tu utilises `meilleur sur ce protocole` uniquement après un backtest complet, comparable et daté.
