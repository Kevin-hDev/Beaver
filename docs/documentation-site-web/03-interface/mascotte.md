# La mascotte

**Emplacement site** — Interface › Mascotte
**Répond à** — « C'est quoi ce personnage, à quoi il sert, et comment je le change ou l'enlève ? »
**Sources** — `src/services/mascot.ts:8-12`, `:29`, `:56`, `:83`, `:104` (réglages, activation, bornes de taille), `src-tauri/src/services/mascot/activity.rs:5` et `:9-17`, `event_mapping.rs:5-7`, `mascot/mod.rs`, `lifecycle.rs`, `src/components/settings/mascot-settings.tsx:25-62`, `src/components/mascot/`, `src/mascot-main.tsx`
**Vérification** — Vérifié dans le code : l'activation désactivée par défaut, les bornes de taille, les huit personnages, les huit animations, le suivi d'activité et les durées d'état

---

## Plan de page proposé

1. À quoi elle sert
2. Les huit personnages
3. Les états
4. La fenêtre dédiée
5. Les réglages

---

## Contenu

### 1. À quoi elle sert

**La mascotte est éteinte par défaut.** C'est le fait le plus important de la page : elle ne s'affiche pas tant que l'utilisateur ne l'active pas dans **Réglages › Mascotte** (`src/services/mascot.ts:12`, `enabled: false` dans l'état par défaut). La page doit donc expliquer comment l'**allumer**, pas comment s'en débarrasser.

Une fois activée : un personnage animé qui **reflète l'état de l'agent** : au repos, au travail, tâche réussie, tâche échouée.

Ce n'est pas seulement décoratif. Sur une tâche longue, il indique d'un coup d'œil si l'agent travaille encore ou s'il a terminé, sans revenir dans la conversation. C'est l'argument à mettre en avant : présenté comme un simple ornement, il paraît gratuit.

### 2. Les huit personnages

| Identifiant | 
|---|
| `cl-go-beaver` (le castor, personnage par défaut) |
| `circuit` |
| `kova` |
| `nival` |
| `mokai` |
| `volt` |
| `raku` |
| `pico` |

Voir *Points à confirmer* : les noms affichés en français n'ont pas été relevés.

### 3. Les états

Les états suivent les événements des conversations en cours.

**Huit animations** existent (`src-tauri/src/services/mascot/activity.rs:9-17`) : `Idle` (au repos), `Thinking`, `ExploreBook`, `WorkLaptop`, `Waiting`, `Success`, `Failed`, `Alert`.

- **Jusqu'à 32 conversations** sont suivies simultanément (`activity.rs:5`).
- Un état de **réussite** s'affiche pendant **2,2 secondes** (`event_mapping.rs:5`).
- Un état d'**échec** s'affiche pendant **2,6 secondes** (`event_mapping.rs:6`).
- Un état d'**alerte** s'affiche pendant **1,8 seconde** (`event_mapping.rs:7`).
- L'état tient compte du fait que l'application a le focus ou non.

### 4. La fenêtre dédiée

La mascotte dispose de sa **propre fenêtre**, séparée de la fenêtre principale. Elle peut donc rester visible pendant qu'on travaille dans une autre application — utile pour surveiller une tâche longue sans garder Beaver au premier plan.

### 5. Les réglages

Onglet dédié : **Réglages › Mascotte**. **Trois** réglages, pas deux :

- **Activation** — `enabled`, **faux par défaut** (`src/services/mascot.ts:12`, champ déclaré en `:29`, accepté en écriture en `:56`, relu en `:83`). Rien ne s'affiche tant qu'il n'est pas mis à vrai.
- Choix du personnage parmi les huit.
- **Taille réglable en pourcentage**, de **70 %** à **140 %** (`src/services/mascot.ts:8-9`), avec aperçu en direct. Une valeur hors de cet intervalle est ramenée dedans (`:104`) ; le défaut est **100 %**.

---

## Encadrés

**Encadré « Elle est éteinte au départ »** — à placer en tête de page.
> La mascotte n'apparaît pas tant que vous ne l'activez pas dans **Réglages › Mascotte**. C'est un choix, pas une panne.

**Encadré « Surveiller sans rester devant »**
> La mascotte a sa propre fenêtre. Laissez-la visible pendant que vous travaillez ailleurs : elle vous indique quand l'agent a terminé.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| La mascotte n'apparaît nulle part | Elle est éteinte par défaut | L'activer dans Réglages › Mascotte |
| La mascotte ne réagit pas | Aucune conversation active, ou plus de 32 suivies | Vérifier qu'une conversation travaille |
| L'état de réussite disparaît trop vite | Affiché 2,2 secondes | Comportement voulu |
| La fenêtre est perdue de vue | Fenêtre séparée | La retrouver via le gestionnaire de fenêtres du système |

---

## Renvois

- *Interface › Thèmes et apparence*
- *Réglages › Général et préférences*

---

## Points à confirmer

- **Les noms affichés des huit personnages** et leur apparence. Seuls les identifiants techniques ont été relevés. Une page sur un choix esthétique sans description des options n'a guère d'intérêt.
- ~~Peut-on désactiver complètement la mascotte ?~~ **Tranché** : oui, et c'est même l'état par défaut (`src/services/mascot.ts:12`).
- ~~Les bornes de la taille en pourcentage.~~ **Tranché** : **70 % à 140 %** (`src/services/mascot.ts:8-9`).
- ~~Le comportement au repos et les états.~~ **Tranché** : huit animations, dont `Idle` au repos et `WorkLaptop`/`Thinking` au travail (`mascot/activity.rs:9-17`) ; l'alerte dure **1,8 seconde** (`mascot/event_mapping.rs:7`).
- **La fenêtre dédiée est-elle affichée par défaut** une fois la mascotte activée, et comment l'ouvrir ou la fermer ?
- **Le rendu à l'écran de chacune des huit animations** — à relever pendant la passe d'interface, pour pouvoir les décrire autrement que par leur identifiant technique.
- **La mascotte est-elle disponible sur les trois systèmes ?** Une fenêtre secondaire peut se comporter différemment selon l'environnement de bureau, en particulier sous Linux.
