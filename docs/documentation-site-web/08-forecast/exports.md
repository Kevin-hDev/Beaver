# Exporter une prévision

**Emplacement site** — Forecast › Exporter
**Répond à** — « Dans quels formats puis-je sortir ma prévision, où le fichier atterrit-il, et qu'est-ce qu'il contient exactement ? »
**Sources** — `src-tauri/src/services/forecast/export/` (`mod.rs`, `common.rs`, `csv.rs`, `xlsx.rs`, `xlsx_advanced.rs`, `xlsx_input.rs`, `xlsx_style.rs`, `chart.rs`, `chart_data.rs`, `pdf.rs`, `report_advanced.rs`, `advanced_rows.rs`, `quantile_labels.rs`, `spreadsheet_text.rs`, `xlsx_security_tests.rs`) ; `src-tauri/src/commands/forecast.rs` ; `src-tauri/src/services/forecast/intervals.rs`, `services/brand.rs` ; `src/components/forecast/widgets/export-dropdown.tsx`, `use-forecast-export.ts`, `forecast-panel.tsx`, `workbench/forecast-workbench-report.tsx` ; `src/i18n/fr.json`
**Vérification** — Vérifié dans le code le 10 septembre 2026, format par format. Aucun fichier exporté n'a été ouvert ; les points à contrôler sur un vrai fichier sont listés en fin de page.

---

## Plan de page proposé

1. Les sept sorties, et où les trouver
2. Où va le fichier, et comment il s'appelle
3. Ce qu'il y a dans tout export
4. CSV — le tableau à plat
5. Excel — un classeur, huit feuilles
6. JSON — tout, sans perte
7. PNG et SVG — l'image du graphique
8. PDF — le rapport texte
9. Copier — le collage rapide
10. Choisir le bon format

---

## Contenu

### 1. Les sept sorties, et où les trouver

Le bouton porte le libellé **« Export »** (`fr.json`, `forecast.export.title`) et ouvre un menu de sept entrées, dans cet ordre (`widgets/export-dropdown.tsx:19-27`) :

| Libellé affiché | Ce que ça produit |
|---|---|
| **CSV** | Un fichier `.csv` |
| **Excel (.xlsx)** | Un classeur `.xlsx` |
| **PNG** | Une image `.png` |
| **SVG** | Une image vectorielle `.svg` |
| **JSON** | Un fichier `.json` |
| **PDF (rapport)** | Un document `.pdf` |
| **Copier (clipboard)** | Rien sur le disque : le texte va dans le presse-papiers |

**Les sept fonctionnent.** Chacun a son écriture dans le moteur, appelée depuis un seul aiguillage (`services/forecast/export/mod.rs:67-75`) : CSV, Excel, JSON, SVG, PNG, PDF, et le cas du presse-papiers traité à part (`:53-60`).

Le bouton apparaît à **deux endroits**, et il s'agit du même composant : dans le pied du panneau Forecast dès qu'une analyse est ouverte (`forecast-panel.tsx:191-194`), et dans la barre d'outils de la section **Rapport** de la fenêtre Espace Forecast (`workbench/forecast-workbench-report.tsx:14-16`).

**Trois messages** possibles après l'action (`use-forecast-export.ts:26-33` ; `fr.json`, `forecast.export`) :

- **« Export enregistré »** — un fichier a été écrit ;
- **« Export copié »** — le texte est dans le presse-papiers ;
- **« Export impossible »** — l'export a échoué, sans autre précision.

### 2. Où va le fichier, et comment il s'appelle

**La destination** est le **dossier de téléchargements du système**. Quand celui-ci est introuvable, Beaver retombe sur `~/.local/share/cl-go-dash/forecast-exports/`, qu'il crée au besoin (`export/mod.rs:123-127`).

**Le nom du fichier** suit toujours la même forme :

```
<nom de l'analyse, nettoyé>-<8 premiers caractères de l'identifiant>.<extension>
```

Le nettoyage du nom est strict (`export/common.rs:130-153`) : seuls les **lettres et chiffres non accentués** sont conservés, en minuscules ; les espaces, tirets et traits soulignés deviennent des tirets ; **tout le reste est supprimé** — accents compris ; les tirets consécutifs sont réduits à un seul ; le résultat est coupé à **64 caractères** ; s'il ne reste rien, le nom devient `forecast`.

Une analyse nommée « Prévision CA — été 2026 » donne donc un fichier commençant par `prvision-ca-t-2026`. **C'est le comportement réel, et la page doit le dire** : les accents ne sont pas translittérés, ils disparaissent.

Les huit caractères d'identifiant à la fin évitent que deux analyses portant le même nom s'écrasent l'une l'autre.

**Une analyse ne peut être exportée que depuis la conversation à laquelle elle appartient** : l'autorisation est vérifiée avant toute lecture (`commands/forecast.rs:163`).

### 3. Ce qu'il y a dans tout export

Quel que soit le format, l'export part du même assemblage : **l'analyse complète et ses notes** (`export/mod.rs:49-51`). Les notes ne sont donc jamais laissées de côté, même dans un export de données brutes.

**Les en-têtes de fourchette s'adaptent au niveau de confiance demandé.** Ils ne sont pas figés à `q10`/`q50`/`q90` : ils sont calculés à partir du niveau retenu (`export/quantile_labels.rs:11-31`). Deux exemples vérifiés par les tests du fichier :

| Confiance demandée | En-têtes produits |
|---|---|
| **90 %** | `q05`, `q50`, `q95` |
| **99 %** | `q0050`, `q50`, `q9950` |

Le nombre de décimales, lui, change d'un format à l'autre — c'est un détail qui compte quand on compare deux exports :

| Format | Précision des valeurs |
|---|---|
| CSV, presse-papiers | **6 décimales** (`export/common.rs:155-161`) |
| PDF | **2 décimales** (`export/pdf.rs:155-161`) |
| Excel, JSON | La valeur telle quelle, sans arrondi (`export/xlsx.rs:98`, `common.rs:91-101`) |

Une valeur non finie — infinie ou indéfinie — sort **vide**, jamais sous forme de texte parasite.

### 4. CSV — le tableau à plat

Un seul tableau, **dix colonnes**, qui empile tout ce que l'analyse contient (`export/csv.rs:9-40`) :

`section`, `name`, `date`, `series_id`, `value`, puis les **trois en-têtes de fourchette**, puis `text` et `source`.

La colonne `section` dit ce qu'est chaque ligne (`export/common.rs:28-88`, `advanced_rows.rs:3-56`) :

| Valeur de `section` | Ce que la ligne porte |
|---|---|
| `history` | Un point de l'historique fourni |
| `prediction` | Un point prévu, avec sa fourchette |
| `scenario` | Un point d'un scénario, le nom du scénario étant dans `name` |
| `annotation` | Une annotation posée sur le graphique |
| `note` | Une note attachée à l'analyse |
| `residual_anomaly` | Un écart notable relevé après le calcul |
| `variable_importance` | Le poids d'une variable explicative |
| `drift` | Un constat de dérive sur une série |
| `ensemble_member` | Un modèle et son poids dans un ensemble |

**Le CSV est protégé contre l'injection de formule.** Toute cellule de texte commençant par `=`, `+`, `-`, `@`, une tabulation ou un retour chariot reçoit une apostrophe en tête, ce qui empêche le tableur de l'exécuter à l'ouverture (`export/spreadsheet_text.rs:1-11`, avec son test). C'est un point de sécurité qui mérite une phrase sur le site : un nom de note malicieux ne devient pas une formule active dans Excel.

**La conséquence visible** : une valeur négative saisie comme texte, ou un titre commençant par un tiret, apparaîtra avec une apostrophe en tête dans le fichier.

### 5. Excel — un classeur, huit feuilles

Le classeur contient jusqu'à **huit feuilles**, créées dans cet ordre (`export/xlsx.rs:7-41`) :

| Nom de la feuille | Contenu | Toujours présente |
|---|---|---|
| `Metadata` | Identifiant, nom, modèle, fournisseur, cible, fréquence, date de création, horizon | Oui |
| `Historique` | Les points fournis : date, série, valeur | Oui |
| `Prévisions` | Les points prévus, avec les trois colonnes de fourchette | Oui |
| `Scenarios` | Les points de chaque scénario, précédés de son nom | Oui — vide s'il n'y en a pas |
| `Notes` | Les notes : type, date, titre, texte, source | Oui — vide s'il n'y en a pas |
| `Advanced` | Décomposition, écarts notables, importance des variables, dérive | Oui — vide si l'analyse avancée est absente |
| `Ensemble` | Les points de la prévision d'ensemble | **Seulement** si un ensemble a été construit |
| `Input data` | **Le fichier d'origine, colonne par colonne, ligne par ligne** | Oui |

**La dernière feuille est l'argument le plus fort de ce format** : le classeur exporté contient vos données de départ à côté du résultat (`export/xlsx_input.rs:6-22`). Un fichier suffit pour refaire le travail ou le transmettre.

Chaque feuille reçoit une **ligne d'en-tête figée**, un **filtre automatique** et un ajustement de largeur des colonnes plafonné à 52 caractères (`export/xlsx_style.rs:3-15`).

**Le classeur est lui aussi protégé de l'injection de formule** : un titre de note écrit `=HYPERLINK(...)` reste du texte et n'est pas exécuté — un test dédié ouvre le fichier produit pour le vérifier (`export/xlsx_security_tests.rs:6-35`).

### 6. JSON — tout, sans perte

Le fichier contient trois clés à la racine (`export/common.rs:91-101`) :

| Clé | Contenu |
|---|---|
| `analysis` | L'analyse entière, telle qu'elle est enregistrée |
| `notes` | Les notes attachées |
| `quantile_labels` | Les trois en-têtes de fourchette correspondant au niveau de confiance |

Le fichier est **mis en forme lisiblement**, pas compacté. C'est le seul format qui ne perd rien : tous les autres sélectionnent, arrondissent ou mettent en page.

**Le format à conseiller pour alimenter un autre programme.**

### 7. PNG et SVG — l'image du graphique

Les deux sortent le **même dessin** : le SVG est écrit tel quel, le PNG en est le rendu (`export/chart.rs:6-26`).

| Caractéristique | Valeur | Source |
|---|---|---|
| Dimensions | **1 400 × 760 pixels** | `export/chart_data.rs:3-4` |
| Fond | Sombre, `#101012`, **quel que soit votre thème** | `export/chart.rs:40` |
| Titre | Le nom de l'analyse | `export/chart.rs:42` |
| Sous-titre | Modèle, colonne cible, horizon, nombre de points | `export/chart.rs:31-37` |
| Courbes | Historique en gris clair, prévision en orange avec un point par échéance, scénarios en bleu pointillé | `export/chart.rs:52-57` |
| Zone de confiance | Une bande orange transparente autour de la prévision | `export/chart.rs:137-151` |
| Repères | Axe horizontal daté et grille horizontale avec valeurs | `export/chart.rs:62-84` |
| Légende | Trois entrées | `export/chart.rs:86-93` |

**La légende est écrite en français en dur dans le code**, sans passer par les traductions : **« Historique »**, **« Prevision »** — sans accent — et **« Confiance »** (`export/chart.rs:87-91`). Voir « Points à confirmer ».

**Une particularité à signaler** : l'axe des valeurs ajoute un symbole **€** quand le nom de la colonne cible contient la suite de lettres `eur`, en majuscules ou en minuscules (`export/chart.rs:167-178`). C'est une commodité qui vise « chiffre_affaires_eur », mais elle se déclenche aussi sur des noms comme « valeur », « heures » ou « couleur ». Les valeurs au-delà de mille sont abrégées en milliers, avec une décimale.

Le rendu du PNG utilise les **polices installées sur votre machine** (`export/chart.rs:13`) ; l'image peut donc différer légèrement d'un ordinateur à l'autre.

### 8. PDF — le rapport texte

Le PDF n'est pas une mise en page graphique : c'est un **rapport en texte à espacement fixe**, écrit directement par Beaver sans bibliothèque de mise en page (`export/pdf.rs:120-142`).

| Caractéristique | Valeur | Source |
|---|---|---|
| Format de page | **595 × 842 points**, soit A4 | `export/pdf.rs:20` |
| Police | **Courier**, corps 9 | `export/pdf.rs:14`, `:112` |
| Lignes par page | **46** | `export/pdf.rs:6` |
| Pagination | Automatique, autant de pages que nécessaire | `export/pdf.rs:10` |

**Le contenu, dans l'ordre** (`export/pdf.rs:28-92`, `report_advanced.rs:3-95`) :

1. **L'en-tête** — « Beaver Forecast », puis le nom de l'analyse, le modèle et son fournisseur, la colonne cible, l'horizon et la fréquence, la période couverte par l'historique et son nombre de points.
2. **PREVISIONS** — un tableau daté : date, série, valeur, et les trois valeurs de fourchette.
3. **ANALYSE AVANCEE** — la décomposition par série, le nombre d'écarts notables puis **les dix premiers**, l'importance des variables avec sa méthode et sa fiabilité puis **les dix premières**, et les constats de dérive.
4. **BACKTEST** — le classement des modèles évalués, avec leurs mesures, et la mention `echec` pour ceux qui n'ont pas abouti.
5. **ENSEMBLE** — les modèles combinés et leur poids, précédés de l'état de validation de l'ensemble.
6. **SCENARIOS** — le nom de chaque scénario et son nombre de points.
7. **NOTES** — pour chaque note : date, type, titre, puis **les quatre premières lignes** de son contenu.

**Deux troncatures à annoncer** : dix écarts notables, dix variables, quatre lignes par note. Le PDF est un résumé lisible, pas une archive — pour l'exhaustivité, le JSON ou le classeur Excel.

**Les titres de section sont écrits sans accent** dans le code — `Modele`, `Frequence`, `PREVISIONS`, `ANALYSE AVANCEE`, `SCENARIOS` (`export/pdf.rs:33-47`, `report_advanced.rs:7`). Voir « Points à confirmer ».

### 9. Copier — le collage rapide

L'entrée **« Copier (clipboard) »** n'écrit aucun fichier. Le moteur produit un texte, l'interface le pose dans le presse-papiers (`export/mod.rs:53-60` ; `use-forecast-export.ts:26-30`).

**Le texte produit** (`export/common.rs:103-128`) : quatre lignes d'en-tête — nom de l'analyse, modèle, colonne cible, horizon et fréquence —, une ligne vide, puis un **tableau séparé par des tabulations** avec date, série, valeur et les trois valeurs de fourchette, une ligne par échéance. Les lignes d'analyse avancée, de backtest et d'ensemble suivent, dans la même forme que le PDF.

**Le séparateur par tabulation est ce qui rend ce format utile** : un collage direct dans un tableur remplit les colonnes toutes seules.

**Ce que le presse-papiers ne contient pas** : l'historique fourni, les scénarios détaillés, les annotations et les notes. C'est le seul export volontairement partiel.

Les cellules de texte y sont protégées de l'injection de formule comme dans le CSV (`export/common.rs:118-119`).

### 10. Choisir le bon format

| Ce que vous voulez faire | Format |
|---|---|
| Refaire des calculs, croiser avec d'autres données | **Excel (.xlsx)** — il contient aussi vos données d'origine |
| Ouvrir dans n'importe quel tableur, ou traiter par script | **CSV** |
| Alimenter un autre programme sans rien perdre | **JSON** |
| Mettre le graphique dans une présentation | **PNG** |
| Mettre le graphique dans un document imprimé, ou le retoucher | **SVG** |
| Envoyer un compte rendu lisible sans logiciel particulier | **PDF (rapport)** |
| Coller trois chiffres dans un message ou un tableur ouvert | **Copier (clipboard)** |

---

## Tableaux

### Ce que chaque format contient

| | CSV | Excel | JSON | PNG / SVG | PDF | Copier |
|---|---|---|---|---|---|---|
| Historique fourni | Oui | Oui | Oui | Oui, dessiné | Bornes seulement | **Non** |
| Points prévus et fourchette | Oui | Oui | Oui | Oui, dessinés | Oui | Oui |
| Scénarios | Oui | Oui | Oui | Oui, dessinés | Nom et nombre de points | Non |
| Annotations | Oui | Non | Oui | Non | Non | Non |
| Notes | Oui | Oui | Oui | Non | Oui, 4 lignes chacune | Non |
| Analyse avancée | Oui | Oui | Oui | Non | Oui, tronquée à 10 | Oui |
| Résultats d'évaluation | Non | Non | Oui | Non | Oui | Oui |
| Ensemble | Oui, ses membres | Oui, ses points | Oui | Non | Oui, ses membres | Oui |
| **Données d'origine** | Non | **Oui**, feuille `Input data` | Oui | Non | Non | Non |

---

## Encadrés

> **ℹ À placer en tête de page — Les sept sorties existent et fonctionnent.**
> CSV, Excel, PNG, SVG, JSON, PDF et copie dans le presse-papiers. Le fichier part dans votre dossier de téléchargements, nommé d'après l'analyse.

> **ℹ À placer dans « Excel » — Le classeur emporte vos données de départ.**
> Sa dernière feuille, `Input data`, contient le fichier d'origine ligne par ligne. C'est le seul export qui permet de refaire le travail à partir d'un fichier unique.

> **⚠ À placer dans « Où va le fichier » — Les accents disparaissent du nom de fichier.**
> Seules les lettres et chiffres non accentués sont conservés : « Prévision CA — été 2026 » devient `prvision-ca-t-2026`. Le contenu du fichier, lui, garde les accents. Les huit caractères ajoutés à la fin du nom évitent que deux analyses homonymes s'écrasent.

> **ℹ À placer dans « CSV » — Vos exports ne peuvent pas exécuter de formule.**
> Une cellule de texte commençant par `=`, `+`, `-` ou `@` est neutralisée avant écriture, dans le CSV comme dans le classeur Excel. Un tableur qui ouvre le fichier affiche le texte au lieu de l'exécuter. C'est pourquoi certaines cellules apparaissent avec une apostrophe en tête.

> **⚠ À placer dans « PDF » — Le PDF est un résumé, pas une archive.**
> Il ne garde que les dix premiers écarts notables, les dix premières variables et les quatre premières lignes de chaque note. Pour l'intégralité, prenez le JSON ou le classeur Excel.

> **ℹ À placer dans « PNG et SVG » — L'image est toujours sur fond sombre.**
> Le graphique exporté ne suit pas le thème de l'application : il est dessiné sur fond sombre dans tous les cas. À savoir avant de le poser sur une page blanche.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| « Export impossible » | Message unique : écriture refusée, dossier inaccessible, analyse introuvable | Vérifier la place disque et les droits sur le dossier de téléchargements |
| Je ne retrouve pas le fichier | Il est dans le dossier de téléchargements du système, pas dans un dossier Beaver | Chercher un nom commençant par celui de l'analyse |
| Le nom du fichier est méconnaissable | Accents et ponctuation supprimés, minuscules imposées, coupé à 64 caractères | Comportement attendu ; renommer le fichier après coup |
| Des cellules commencent par une apostrophe | Protection contre l'injection de formule | Comportement voulu ; retirer l'apostrophe manuellement si nécessaire |
| Le graphique exporté est sur fond sombre alors que mon thème est clair | Le dessin d'export a ses propres couleurs | Utiliser le SVG et changer le fond dans un éditeur |
| Un symbole € apparaît sur l'axe sans raison | Le nom de la colonne cible contient les lettres `eur` — « valeur », « heures », « couleur » | Renommer la colonne cible avant le calcul |
| Le PDF est incomplet par rapport à l'écran | Troncatures volontaires : 10 anomalies, 10 variables, 4 lignes par note | Prendre le JSON ou le classeur Excel |
| Le collage ne contient pas mes notes | Le presse-papiers ne porte que la prévision et les analyses, pas l'historique ni les notes | Utiliser CSV ou Excel |
| Le PNG ne rend pas pareil sur une autre machine | Le rendu utilise les polices installées localement | Utiliser le SVG, ou vérifier le rendu sur la machine cible |

---

## Renvois

- `08-forecast/vue-densemble.md` — où se trouve le bouton d'export dans le panneau et dans la fenêtre
- `08-forecast/scenarios-notes-rapports.md` — les scénarios, les notes et la section Rapport, qui alimentent tous les exports
- `08-forecast/analyse-avancee.md` — ce que recouvrent décomposition, écarts notables, importance des variables et dérive dans les exports
- `08-forecast/evaluation-et-comparaison.md` — les résultats d'évaluation, présents dans le JSON, le PDF et le presse-papiers
- `12-reference/emplacement-des-donnees.md` — le dossier de repli `forecast-exports/`

---

## Points à confirmer

**Écarts relevés dans le code — à arbitrer avant publication**

1. **Les textes des exports ne sont pas traduits.** Trois endroits, tous en dur dans le moteur : la légende du graphique — **« Historique »**, **« Prevision »**, **« Confiance »** (`export/chart.rs:87-91`) —, les noms de feuilles du classeur, qui mélangent français et anglais — `Metadata`, `Historique`, `Prévisions`, `Scenarios`, `Notes`, `Advanced`, `Ensemble`, `Input data` (`export/xlsx.rs:13-37`, `:44`, `:115`, `:155` ; `xlsx_advanced.rs:6` ; `xlsx_input.rs:7`) —, et les titres de section du PDF (`export/pdf.rs:33-47`). Un utilisateur en anglais, en allemand ou en japonais reçoit donc des exports partiellement en français. À arbitrer : est-ce assumé — un export est un artefact technique — ou faut-il faire passer ces textes par les traductions ? Dans le second cas, c'est une trentaine de chaînes en sept langues.
2. **« Prevision » est écrit sans accent dans la légende du graphique** (`export/chart.rs:89`), probablement pour éviter un problème de police au rendu. Si la traduction est mise en place, ce contournement disparaîtra ; en attendant, la faute est visible sur chaque image exportée.
3. **Le symbole € se déclenche sur une correspondance de trois lettres.** `target.to_ascii_lowercase().contains("eur")` (`export/chart.rs:168`) marque aussi « valeur », « heures », « couleur », « secteur ». C'est un faux repère sur un axe, exactement le cas que la règle d'interface du projet interdit. À signaler comme correctif à faire ; la page doit en attendant décrire le comportement réel.
4. **Le message d'échec est unique.** « Export impossible » couvre toutes les causes (`use-forecast-export.ts:32-33`), alors que le moteur en distingue plusieurs — « Format d'export invalide », « Impossible de créer le dossier d'export », « Export CSV impossible », « Export PDF impossible » (`export/mod.rs:106`, `:127` ; `csv.rs:8` ; `pdf.rs:25`). Décision produit.

**Non vérifié — suppose d'ouvrir un fichier exporté**

5. **Le rendu des caractères accentués dans le PDF.** Le document déclare une police Courier standard sans préciser d'encodage (`export/pdf.rs:14`), et l'échappement ne traite que `\`, `(` et `)` (`export/pdf.rs:144-149`). Le nom d'une analyse, le titre d'une note ou un nom de colonne contenant des accents pourraient s'afficher de travers. **À vérifier en produisant un PDF depuis une analyse au nom accentué avant publication** — c'est le point le plus susceptible d'être visible par un utilisateur français.
6. **Le rendu réel du PNG** — lisibilité des libellés d'axe, chevauchement des dates quand l'horizon est long, apparence quand plusieurs séries sont présentes. Aucun fichier n'a été produit.
7. **Le comportement du classeur Excel quand une analyse porte beaucoup de lignes.** L'ajustement automatique des largeurs est plafonné à 52 caractères (`xlsx_style.rs:13`), mais rien n'a été ouvert.
8. **Le dossier de téléchargements sous Windows et sous Linux.** La fonction utilisée suit les conventions de chaque système (`export/mod.rs:124`), mais seuls les chemins ont été lus, aucun export n'a été fait sur ces systèmes. À confirmer avant d'écrire un chemin précis sur le site.
9. **Le comportement du presse-papiers quand la fenêtre n'a pas le focus.** L'écriture passe par l'interface (`use-forecast-export.ts:27`) et peut être refusée par le système dans certains cas ; non provoqué.

**Affichage non vérifié — liste de contrôle pour la passe d'interface**

10. Le menu **Export** : ses icônes par format, sa position quand le panneau est étroit, et le fait qu'il reste entièrement visible — c'est une couche flottante posée dans un portail (`export-dropdown.tsx:123`).
11. Le bouton Export dans la section **Rapport** de la fenêtre Espace Forecast, et sa cohérence avec celui du panneau.
12. Les trois messages de retour (« Export enregistré », « Export copié », « Export impossible ») : leur durée d'affichage et leur lisibilité dans les deux thèmes.
