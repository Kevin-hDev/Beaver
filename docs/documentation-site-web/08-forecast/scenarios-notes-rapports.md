# Scénarios, notes et rapport

**Emplacement site** — Forecast › Scénarios, notes et rapport
**Répond à** — « Comment tester une hypothèse sans perdre ma prévision, où écrire ce que je comprends de mes données, et qu'est-ce qui est conservé quand je referme Beaver ? »
**Sources** — `src-tauri/src/services/forecast/scenarios.rs`, `scenario_percent.rs`, `scenario_context.rs`, `scenario_context_run.rs` ; `notes.rs`, `notes_types.rs`, `notes_validation.rs`, `notes_format.rs`, `notes_paths.rs`, `notes_files.rs`, `notes_annotations.rs`, `notes_cleanup.rs`, `notes_transaction.rs` ; `types.rs`, `storage_index.rs`, `limits.rs` ; `export/mod.rs` ; `src/components/forecast/sections/` (`forecast-scenarios.tsx`, `forecast-scenario-form.tsx`, `forecast-scenario-context-fields.tsx`, `forecast-notes.tsx`, `forecast-notes-timeline.tsx`) ; `src/components/forecast/workbench/` (`forecast-workbench-nav.tsx`, `forecast-workbench-section.tsx`, `forecast-workbench-scenarios.tsx`, `forecast-workbench-notes.tsx`, `forecast-workbench-report.tsx`) ; `src-tauri/src/services/agent_local/tool_definitions_forecast.rs`, `tool_catalog.rs` ; `src/i18n/fr.json`
**Vérification** — Vérifié dans le code le 10 septembre 2026. Aucun scénario ni aucune note n'a été créé dans l'application : les seuils, les chemins et les libellés viennent de la lecture du code et de ses tests.

> **Les formats d'export ne sont pas décrits ici.** La section Rapport contient le menu d'export ; ce qu'il produit est dans `08-forecast/exports.md`.

---

## Plan de page proposé

1. Les trois sections, et où elles vivent
2. Les scénarios : deux façons de poser une hypothèse
3. L'ajustement global
4. L'ajustement de contexte
5. Les notes : ce que c'est, où c'est écrit
6. La frise et la liste
7. La section Rapport
8. Ce qui est conservé, et ce qui disparaît

---

## Contenu

### 1. Les trois sections, et où elles vivent

Scénarios, Notes et Rapport sont trois des **sept sections de l'espace Forecast** (`forecast-workbench-nav.tsx:4-12`) : Données, Prévision, Évaluation, Comparaison, Scénarios, Notes, Rapport (`src/i18n/fr.json`, `forecast.workbench.sections`).

Ce n'est pas le menu principal de Forecast, qui n'a que trois entrées — Vue principale, Comparaisons, Historique (`src/i18n/fr.json`, `forecast.nav`). L'espace Forecast est une fenêtre à part, qu'on ouvre depuis une analyse ; son titre est **« Espace Forecast »** (`src/i18n/fr.json`, `forecast.workbench.windowTitle`).

Les trois sections sont attachées à **une analyse précise** : sans analyse sélectionnée, elles affichent « Aucune analyse à évaluer » et « Lancez d'abord une prévision depuis la conversation. » (`forecast-workbench-section.tsx:24-30`).

### 2. Les scénarios : deux façons de poser une hypothèse

Un scénario est une **variante de la prévision** enregistrée à côté d'elle, sans la remplacer. Le sous-titre de la section le dit : « Créer et modifier des hypothèses sans perdre la prévision de référence. » (`src/i18n/fr.json`, `forecast.workbench.sectionDescriptions.scenarios`).

Deux types existent, et un seul est disponible par défaut (`scenarios.rs:134-138`) :

| Type affiché | Identifiant interne | Relance le modèle ? | Conditions |
|---|---|---|---|
| **Ajustement global** | `percent_adjustment` | **non** | aucune |
| **Contexte** | `context_adjustment` | **oui** | il faut des variables de contexte |

Le bouton « Contexte » est **désactivé** quand l'analyse n'utilise aucune variable de contexte (`forecast-scenario-form.tsx:60`).

L'interface affiche une phrase d'explication sous les deux boutons (`forecast-scenario-form.tsx:75-79` ; `src/i18n/fr.json`, `forecast.scenarios`) :
- Ajustement global : « Crée une courbe dérivée à partir de la prévision actuelle, sans relancer le modèle. »
- Contexte : « Relance le modèle avec les nouvelles valeurs futures des variables sélectionnées. »

**Champs communs** : un **nom** obligatoire, au plus 80 caractères, et une **description** facultative, au plus 500 (`scenarios.rs:8-9`, `:124-133` ; les mêmes bornes sont posées côté formulaire, `forecast-scenario-form.tsx:36`, `:44`).

**Cinquante scénarios au maximum** par analyse (`types.rs:11`, `scenarios.rs:54-56`).

### 3. L'ajustement global

Le plus simple, et le plus rapide : Beaver **multiplie** la prévision par un facteur, sans rien recalculer (`scenario_percent.rs:10-30`).

Le pourcentage va de **−95 % à +500 %** (`scenarios.rs:10-11`, `:147-149` ; formulaire `forecast-scenario-form.tsx:84-87`, pas de 0,1).

Ce qui est multiplié : les prédictions **et les trois quantiles** — la borne basse, la médiane et la borne haute (`scenario_percent.rs:24`, `:32-38`). La plage de confiance se déplace donc avec la courbe, et sa largeur relative reste identique.

**Le modèle n'est pas rappelé.** Aucun calcul, aucun réseau, aucun démarrage de moteur : c'est une transformation arithmétique de la prévision déjà obtenue. C'est ce qui rend ce type instantané, et c'est aussi sa limite — voir l'encadré.

Le bouton porte **« Lancer le scénario »** à la création et **« Mettre à jour »** en modification (`forecast-scenario-form.tsx:98-104`).

### 4. L'ajustement de contexte

Celui-ci **relance vraiment le modèle** avec d'autres valeurs futures pour vos variables explicatives (`scenario_context.rs:35` puis `scenario_context_run::rerun`).

**Comment on le règle** (`forecast-scenario-context-fields.tsx`) : on ajoute une ou plusieurs lignes, chacune composée d'une **variable**, d'un **mode** — « % » ou « Valeur » — et d'un **nombre**. Le bouton d'ajout s'appelle « Ajouter une variable », celui de retrait « Retirer la variable ». Une nouvelle ligne démarre à +10 % (`forecast-scenario-context-fields.tsx:99`).

**Douze ajustements au maximum** (`scenario_context.rs:8`, `:56-58`).

**Sur quelles lignes ça s'applique** : uniquement les lignes **futures**, c'est-à-dire celles dont la colonne cible est vide (`scenario_context.rs:118-120`). Les données historiques ne sont jamais modifiées. Si aucune ligne future ne correspond, le scénario est refusé avec « Aucune ligne future modifiée » (`scenario_context.rs:127-129`).

**Une série ou toutes.** Quand l'analyse contient plusieurs séries, un sélecteur permet de viser une série précise ; sa valeur par défaut est **« Toutes les séries »** (`forecast-scenario-context-fields.tsx:29-40`, `scenario_context.rs:152-168`).

**Quatre conditions doivent être réunies**, sinon le scénario est refusé (`scenario_context.rs:51-83`) : le moteur doit accepter les variables de contexte futures, les données source doivent être disponibles, l'analyse doit utiliser au moins une variable, et chaque variable citée doit faire partie de celles utilisées. La série visée doit exister.

**La relance passe par le même chemin qu'une prévision normale** (`scenario_context_run.rs:16-62`) : validation de la requête, contrôle de qualité des données, vérification des capacités du moteur, puis appel local ou distant. Toutes les conditions habituelles s'appliquent donc — modèle installé, ressources suffisantes, clé API pour un modèle distant.

**Conséquence à dire clairement sur le site : un scénario de contexte avec un modèle distant renvoie vos données au fournisseur.** Voir `08-forecast/modele-cloud-timegpt.md`.

Le bouton porte **« Relancer le scénario »** en modification, puisque le calcul est refait (`forecast-scenario-form.tsx:101-103`).

### 5. Les notes : ce que c'est, où c'est écrit

Une note est un **texte daté**, attaché à une analyse, rangé dans un vrai fichier Markdown sur votre disque.

**Cinq types** (`src/i18n/fr.json`, `forecast.notes.types`) : **Contexte**, **Risque**, **Décision**, **Anomalie**, **Hypothèse**. Le type par défaut d'une nouvelle note est « Contexte » (`forecast-notes.tsx:86`).

**Deux sources possibles**, `user` et `llm` (`notes_validation.rs:35-41`) : une note écrite par vous, ou une note posée par l'agent de Beaver. L'interface les distingue par la couleur du point sur la frise (`forecast-notes-timeline.tsx:61`) et affiche « Utilisateur » ou « LLM » (`src/i18n/fr.json`, `forecast.notes.sources`).

**Où c'est rangé.** Un fichier `.md` par note, dans `forecast-notes/<identifiant de l'analyse>/<identifiant de la note>.md`, sous le dossier de données de Beaver (`notes_paths.rs:5`, `:33-45`). L'interface affiche ce chemin sous la frise, raccourci au dossier de l'application (`forecast-notes.tsx:163-167`), et le bouton **« Ouvrir la note »** ouvre le fichier dans l'éditeur du système (`notes.rs:115-125`).

**Ce que contient le fichier** (`notes_format.rs:4-17`) : un en-tête entre deux lignes de tirets — identifiant, identifiant d'analyse, date, type, source, dates de création et de mise à jour — puis le titre en titre de niveau 1, puis le contenu.

```
---
id: "…"
analysis_id: "…"
date: "2026-07-23"
type: "context"
source: "user"
created_at: "…"
updated_at: "…"
---

# Le titre de la note

Le contenu, en Markdown.
```

**Les limites** (`notes_validation.rs:4-8`) : titre 120 caractères, date 80, contenu 60 kilo-octets, fichier entier 64 kilo-octets. Le titre et la date refusent les guillemets doubles et les caractères de contrôle (`notes_validation.rs:58-68`) — c'est ce qui garantit que l'en-tête reste lisible. Le contenu accepte les retours à la ligne et les tabulations, rien d'autre en caractères de contrôle (`notes_validation.rs:47-56`).

**Deux cents notes au maximum** par analyse (`types.rs:10`, `notes.rs:42-44`, `notes_files.rs:48-50`).

**L'écriture est sûre.** Chaque fichier est écrit en deux temps — fichier temporaire puis renommage (`notes_files.rs:79`) — et l'enregistrement de la note et celui de l'analyse forment une transaction : si l'analyse ne peut pas être sauvegardée, la note écrite est annulée (`notes.rs:60-62`, `:87-89`, `:109-111`). Un verrou empêche deux opérations sur les notes de se croiser (`notes.rs:14`, `:40`, et suivants).

**Les chemins sont vérifiés à chaque accès** : identifiant filtré par une expression régulière stricte, chemin canonicalisé, appartenance au bon dossier contrôlée, liens symboliques refusés (`notes_paths.rs:104-136`, `notes_validation.rs:9`).

**Les notes existent en deux endroits à la fois** : le fichier Markdown, et une copie dans l'analyse elle-même sous forme d'annotation (`notes_annotations.rs:7-31`). Beaver les remet d'accord à chaque ouverture de la liste : les fichiers manquants sont réécrits depuis les annotations, et les annotations sont mises à jour depuis les fichiers (`notes.rs:16-24`, `notes_files.rs:102-123`).

### 6. La frise et la liste

La section Notes affiche trois zones (`forecast-notes.tsx:142-194`) :

**Une frise** — « Timeline annotée » — où chaque note est un point placé selon sa date, sur un axe qui va du début de l'historique à la fin de la prévision (`forecast-notes-timeline.tsx`). La hauteur de la frise se règle en la tirant, et un double-clic la remet à sa taille d'origine (`forecast-notes.tsx:162`). Vide, elle affiche « Aucune note placée sur cette prévision. »

**Une liste** des notes, triées par date puis par ordre de création (`notes_files.rs:52`).

**Un panneau de détail**, avec l'aperçu Markdown de la note sélectionnée et les actions : Modifier, Supprimer, Ouvrir la note. Sans note, il affiche « Sélectionne une note ou crée une nouvelle note. »

Le bouton de création s'appelle **« Nouvelle note »** (`src/i18n/fr.json`, `forecast.notes.new`).

**La liste se recharge quand la fenêtre reprend le focus** (`forecast-notes.tsx:69-73`) : si vous modifiez un fichier de note dans un éditeur externe, revenir sur Beaver suffit à voir le changement.

### 7. La section Rapport

Sa description annonce : « Rassembler la provenance, les notes et les exports avancés. » (`src/i18n/fr.json`, `forecast.workbench.sectionDescriptions.report`).

**Ce qu'elle contient réellement** est plus court (`forecast-workbench-report.tsx:12-21`) : un **menu d'export** en haut, et en dessous la **section Analyse** — les sept blocs décrits dans `08-forecast/analyse-avancee.md`. Ni provenance, ni notes. Voir « Points à confirmer ».

**Le menu d'export** propose sept entrées (`export/mod.rs:96-121` ; `src/i18n/fr.json`, `forecast.export`) : CSV, Excel (.xlsx), PNG, SVG, JSON, PDF (rapport), Copier (clipboard).

**Où atterrissent les fichiers** : le dossier de téléchargements du système, et à défaut `forecast-exports/` sous le dossier de données de Beaver (`export/mod.rs:124-125`). Le nom du fichier est celui de l'analyse, nettoyé, suivi des **huit premiers caractères** de son identifiant (`export/mod.rs:128-130`).

**Les notes partent dans l'export** : elles sont chargées et jointes à l'analyse avant toute écriture (`export/mod.rs:49-51`). Le détail de ce que contient chaque format est dans `08-forecast/exports.md`.

### 8. Ce qui est conservé, et ce qui disparaît

**Tout est enregistré sur votre disque, tout de suite.** Scénarios et annotations vivent dans le fichier de l'analyse (`types.rs:79-81`), les notes dans leurs fichiers Markdown. Il n'y a pas de bouton « enregistrer l'analyse » : chaque création, modification et suppression déclenche une écriture (`scenarios.rs:66`, `:107`, `:120` ; `notes.rs:59-62`).

**Cinq cents analyses au maximum.** Au-delà, Beaver supprime la plus ancienne **du même espace de travail** ; s'il n'y en a aucune, il refuse d'enregistrer avec « Le stockage Forecast est plein. Supprime d'anciennes analyses puis réessaie. » (`limits.rs:19`, `storage_index.rs:178-191` ; `src/i18n/fr.json`, `forecast.errors.capacityReached`).

**Supprimer une analyse supprime ses notes**, et l'opération est reprennable : le dossier de notes est d'abord renommé avec un préfixe `.delete-`, puis supprimé une fois l'analyse effacée ; si l'effacement échoue, le dossier est remis en place (`notes_cleanup.rs:8-28`). Un dossier resté en attente après une coupure est traité au démarrage suivant : remis en place si l'analyse existe encore, supprimé sinon (`notes_cleanup.rs:35-64`).

**Les scénarios ne survivent pas à une nouvelle prévision** : ils appartiennent à l'analyse. Une nouvelle prévision crée une nouvelle analyse, donc une liste de scénarios vide.

**Les notes sont vos fichiers.** Elles restent lisibles dans n'importe quel éditeur de texte, sans Beaver.

---

## Encadrés

> **ℹ Un ajustement global ne prévient pas le modèle.**
> Il multiplie la courbe déjà calculée, y compris sa plage de confiance. C'est immédiat et sans risque, mais ça ne répond pas à « que se passerait-il si… » : ça répond à « à quoi ressemblerait cette courbe 10 % plus haut ».

> **ℹ Un scénario de contexte relance vraiment le modèle.**
> Les nouvelles valeurs de vos variables explicatives sont envoyées au moteur, qui recalcule. Avec un modèle distant, cela renvoie vos données au fournisseur.

> **⚠ Un scénario de contexte ne modifie que les lignes futures.**
> Ce sont celles dont la colonne cible est vide. Si votre fichier n'en contient pas, le scénario est refusé.

> **ℹ Vos notes sont de vrais fichiers Markdown.**
> Un fichier par note, dans le dossier de données de Beaver, lisible et modifiable dans n'importe quel éditeur. Revenir sur Beaver suffit à voir le changement.

> **⚠ Supprimer une analyse supprime ses notes.**
> L'opération est faite de façon à ne rien laisser à moitié fait : le dossier est mis de côté, l'analyse est effacée, puis le dossier est supprimé. Une coupure au milieu est rattrapée au démarrage suivant.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| Le bouton « Contexte » est grisé | l'analyse n'utilise aucune variable de contexte | relancer une prévision en désignant des colonnes explicatives |
| « Aucune ligne future modifiée » | le fichier ne contient pas de lignes futures pour cette série | ajouter des lignes avec la date future et la cible vide |
| « Covariable future manquante » | une ligne future n'a pas de valeur pour la variable ajustée | compléter la colonne sur toute la période à prévoir |
| « Scénario contextuel non supporté par ce moteur » | le modèle de l'analyse ne lit pas les variables de contexte futures | voir `08-forecast/modeles-locaux.md` |
| Le scénario de contexte échoue avec un modèle distant | clé API absente, ou service indisponible | voir `08-forecast/modele-cloud-timegpt.md` |
| « Trop de scénarios » | 50 scénarios déjà enregistrés sur cette analyse | en supprimer |
| « Limite de notes atteinte » | 200 notes déjà attachées à cette analyse | en supprimer |
| Le titre de la note est refusé | il est vide, dépasse 120 caractères, ou contient un guillemet double | le raccourcir, retirer les guillemets |
| « Le stockage Forecast est plein. » | 500 analyses, et aucune à évincer dans cet espace de travail | supprimer d'anciennes analyses |
| La note modifiée dans un éditeur externe n'apparaît pas | la liste se recharge au retour du focus | cliquer sur la fenêtre de Beaver |

---

## Renvois

- `08-forecast/exports.md` — ce que contient chacun des sept formats du menu Rapport
- `08-forecast/analyse-avancee.md` — les sept blocs affichés sous le menu d'export
- `08-forecast/modeles-locaux.md` — quels modèles acceptent les variables de contexte futures
- `08-forecast/modele-cloud-timegpt.md` — ce qui sort de la machine quand un scénario de contexte relance un modèle distant
- `08-forecast/vue-densemble.md` — l'espace Forecast et ses sept sections
- `04-agent/fonctionnement.md` — l'outil `forecast_analyze`, par lequel l'agent crée notes et scénarios

---

## Points à confirmer

**Écarts relevés dans le code — à arbitrer avant publication**

1. **La section Rapport ne contient pas ce que sa description annonce.** Le texte affiché promet « la provenance, les notes et les exports avancés » (`src/i18n/fr.json`, `forecast.workbench.sectionDescriptions.report`), le composant n'affiche que le menu d'export et la section Analyse (`forecast-workbench-report.tsx:12-21`). Deux corrections possibles : ajouter la provenance et les notes, ou réécrire la description. **Tant que ce n'est pas tranché, le site ne doit pas décrire une section Rapport qui rassemblerait la provenance.** C'est le point le plus important de ce fichier.
2. **La borne −95 / +500 s'applique aussi au mode « Valeur ».** En mode absolu, le nombre saisi devient directement la valeur de la variable, mais il reste borné comme s'il s'agissait d'un pourcentage (`scenario_context.rs:99-103` ; formulaire `forecast-scenario-context-fields.tsx:65-67`). Une variable dont les valeurs normales sont des milliers ne peut donc pas être fixée à sa vraie échelle. À vérifier avec l'équipe : bug ou limite assumée ? En attendant, ne pas présenter le mode « Valeur » comme permettant n'importe quelle valeur.
3. **Le mode « Valeur » n'a pas de libellé explicite.** Le sélecteur propose « % » et « Valeur » (`src/i18n/fr.json`, `forecast.scenarios.percentMode` et `absoluteMode`), sans indiquer que le second remplace la valeur au lieu de l'ajuster. Le site devra l'expliquer ; à proposer aussi comme évolution de l'infobulle.
4. **Un ajustement global s'applique à toutes les séries.** Le champ `target_series_id` existe dans la requête (`scenarios.rs:25`) mais n'est utilisé que par le scénario de contexte (`scenarios.rs:163-182`). Avec plusieurs séries, un ajustement de +10 % les déplace toutes. À confirmer : voulu, ou manque ?
5. **La description d'un scénario n'est affichée nulle part de vérifié.** Elle est saisie, validée, enregistrée (`scenarios.rs:129-133`, `:151-155`), mais la lecture des composants de liste n'a pas permis de confirmer où elle est relue. À vérifier à l'écran.
6. **L'agent peut créer notes et scénarios, et le site doit décider s'il en parle ici.** L'outil `forecast_analyze` porte les actions `annotate`, `scenario`, `scenario_update`, `scenario_delete` et `ensemble` (`tool_definitions_forecast.rs:19-83`), ce qui explique le `llm` accepté comme source de note (`notes_validation.rs:35-41`). Mais **les sept outils Forecast de l'agent sont désactivés par défaut** (`tool_catalog.rs:73-79`). À arbitrer : décrire ce chemin dans cette page, ou le laisser à la page de l'agent avec un simple renvoi. Ne pas écrire « l'agent peut annoter vos prévisions » sans dire qu'il faut d'abord activer ces outils.
7. **Le chemin du fichier de note est affiché à l'écran** (`forecast-notes.tsx:163-167`). C'est utile, mais c'est aussi un chemin interne visible dans l'interface. À confirmer que c'est voulu — le projet a par ailleurs une règle qui écarte les chemins de fichiers des messages destinés à l'utilisateur.

**Non vérifié — hors de portée d'une lecture du code**

8. **Aucun scénario ni aucune note n'a été créé.** Le rendu de la frise, le comportement du redimensionnement et l'aperçu Markdown n'ont pas été observés.
9. **Le temps que prend un scénario de contexte** dépend du modèle et de la taille des données ; il n'a pas été mesuré. C'est pourtant la différence pratique la plus visible entre les deux types de scénario.

**Affichage non vérifié — liste de contrôle pour la passe d'interface finale**

10. La frise avec 200 notes : superposition des points, lisibilité de l'axe, comportement des repères de date.
11. Les deux couleurs de points — note d'utilisateur et note d'agent — dans les deux thèmes : elles doivent se distinguer autrement que par une nuance voisine.
12. Le formulaire de scénario avec 12 lignes d'ajustement de contexte : hauteur, défilement, tenue du bouton d'envoi.
13. Le panneau de détail d'une note vide, en cours d'enregistrement, en erreur, et avec contenu — les quatre états.
14. La section Rapport sur une analyse sans analyse avancée : que voit-on sous le menu d'export ?
