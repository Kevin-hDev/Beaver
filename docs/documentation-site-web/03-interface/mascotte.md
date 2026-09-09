# La mascotte

**Emplacement site** — Interface › Mascotte
**Répond à** — « C'est quoi ce personnage, à quoi il sert, et comment je le change ou l'enlève ? »
**Sources** — `src-tauri/src/services/mascot/mod.rs`, `activity.rs` (ligne 5), `event_mapping.rs` (lignes 5-6), `lifecycle.rs`, `src/components/settings/mascot-settings.tsx` (lignes 18-64), `src/components/mascot/`, `src/mascot-main.tsx`
**Vérification** — Vérifié dans le code : les huit personnages, le suivi d'activité et les durées d'état

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

Un personnage animé qui **reflète l'état de l'agent** : au repos, au travail, tâche réussie, tâche échouée.

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

- **Jusqu'à 32 conversations** sont suivies simultanément.
- Un état de **réussite** s'affiche pendant **2,2 secondes**.
- Un état d'**échec** s'affiche pendant **2,6 secondes**.
- L'état tient compte du fait que l'application a le focus ou non.

### 4. La fenêtre dédiée

La mascotte dispose de sa **propre fenêtre**, séparée de la fenêtre principale. Elle peut donc rester visible pendant qu'on travaille dans une autre application — utile pour surveiller une tâche longue sans garder Beaver au premier plan.

### 5. Les réglages

Onglet dédié : **Réglages › Mascotte**.

- Choix du personnage parmi les huit.
- **Taille réglable en pourcentage**, avec aperçu en direct.

---

## Encadrés

**Encadré « Surveiller sans rester devant »**
> La mascotte a sa propre fenêtre. Laissez-la visible pendant que vous travaillez ailleurs : elle vous indique quand l'agent a terminé.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
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
- **Peut-on désactiver complètement la mascotte ?** Aucun réglage d'activation n'a été repéré, seulement le choix et la taille. À vérifier : c'est la première question de qui n'en veut pas.
- **La fenêtre dédiée est-elle affichée par défaut**, et comment l'ouvrir ou la fermer ?
- **Les bornes de la taille en pourcentage** — minimum et maximum non relevés.
- **Le comportement au repos.** Les durées de réussite et d'échec sont connues ; l'état par défaut et l'état « au travail » n'ont pas été décrits.
- **La mascotte est-elle disponible sur les trois systèmes ?** Une fenêtre secondaire peut se comporter différemment selon l'environnement de bureau, en particulier sous Linux.
