# Comment utiliser ces fichiers

À lire avant de rédiger la documentation du site.

## À quoi servent ces fichiers

Ce dossier n'est **pas** la documentation finale. C'est la matière première : tout ce qu'il faut savoir pour écrire chaque page sans rien oublier et sans rien inventer.

Un fichier ici = une page (ou une section) du site. Il contient les faits vérifiés, les valeurs exactes, les procédures, les tableaux de référence et les pièges. Le travail de rédaction finale — le ton, les transitions, les phrases — reste à faire au moment de fabriquer la page.

## Pourquoi ne pas écrire directement la page

Parce que les faits et la rédaction se vérifient différemment. Une phrase mal tournée se corrige en dix secondes. Une valeur fausse — une limite, un chemin, un comportement d'outil — survit des mois et se propage dans les traductions.

Ces fichiers séparent les deux : ici on garantit l'exactitude, sur le site on garantit la lisibilité.

## Structure d'un fichier

Chaque fichier suit le même gabarit :

```
# Titre de la page

**Emplacement site** — où la page se range dans le sommaire
**Répond à** — la question de l'utilisateur à laquelle la page répond
**Sources** — les fichiers du dépôt qui font autorité sur le sujet
**Vérification** — d'où viennent les faits

## Plan de page proposé
La liste des titres de niveau 2 à créer.

## Contenu
Un bloc par titre, avec les faits sous forme de puces denses.

## Tableaux
Les tableaux de référence, déjà complets.

## Encadrés
Les avertissements et notes à mettre en évidence.

## Pièges et erreurs fréquentes
Ce qui bloque les utilisateurs, avec la cause et la résolution.

## Renvois
Les autres pages à lier.

## Points à confirmer
Ce que je n'ai pas pu vérifier. À trancher avant publication.
```

## Les conventions

**Les valeurs exactes sont en gras ou en `code`.** Un chiffre, un chemin, un nom de fichier, un nom d'outil : jamais approximatif. Si le fichier dit **32 flux actifs**, c'est 32, pas « une trentaine ».

**Le champ Sources donne les fichiers qui font foi.** Quand une information doit être revérifiée dans six mois, on sait où regarder. Ne pas les citer sur le site : ils sont là pour le rédacteur.

**Le champ Vérification indique le niveau de confiance :**

- *Vérifié dans le code* — j'ai lu la ligne qui implémente le comportement.
- *Issu du README* — repris de la documentation existante du dépôt, non recoupé avec le code.
- *À confirmer* — déduit, plausible, mais non prouvé. Ne pas publier tel quel.

## Hiérarchie des sources

Le code fait foi. Toujours. Dans cet ordre de confiance décroissante :

1. **Le code source** — `src-tauri/src/`, `src/`, les scripts d'installation, les workflows CI.
2. **`README.md`, `CHANGELOG.md`, `SECURITY.md`** — maintenus avec les releases, donc à jour, mais orientés vitrine : ils décrivent ce que fait le produit, pas comment on s'en sert.
3. **Le contenu de `docs/`** — **daté, non maintenu, partiellement faux**. À traiter comme une piste à vérifier, jamais comme une source.

Cette hiérarchie n'est pas théorique. Trois erreurs ont déjà été relevées en écrivant ces fichiers :

- `CROSS-PLATFORM.md` annonce un support Fedora/RHEL que `install.sh` n'implémente pas ;
- `CLAUDE.md` affirme que la release CI est publiée directement, alors que le workflow la crée en brouillon ;
- plusieurs constats matériels de `CROSS-PLATFORM.md` datent d'avril 2026, pour une version courante 1.1.2.

Quand un fichier de `docs/` est la seule source disponible, le signaler dans le champ *Vérification* et dans *Points à confirmer*. Ne pas publier sans vérification.

## Les libellés utilisateur des modes de permission

Le code et l'interface n'emploient pas les mêmes mots. **Toujours écrire les libellés de la colonne de droite** dans la documentation destinée aux utilisateurs.

| Identifiant dans le code | Libellé affiché |
|---|---|
| `auto` | Accès complet |
| `manual` | Demande d'approbation |
| `chat` | Chatbot |

Un quatrième mode, `subagent`, existe uniquement en interne : il s'applique aux sessions enfants et contourne la garde de permission. Il n'a pas de libellé parce qu'il n'est jamais proposé à l'utilisateur. Ne pas le présenter comme un quatrième choix ; le mentionner à sa place, dans la page sur les sous-agents.

**La section « Points à confirmer » n'est pas facultative.** Un fichier sans incertitude déclarée est suspect : ça veut dire soit que le sujet est trivial, soit que je n'ai pas cherché assez loin.

## La vérification à l'écran se fait à la fin

Ces fichiers sont rédigés à partir du code. Beaucoup décrivent donc un mécanisme sans dire comment il se présente à l'écran : disposition, libellés exacts, repli d'un résultat long, apparence d'une demande d'approbation.

**C'est voulu, et ce n'est pas bloquant.** Une passe d'interface est prévue en fin de parcours, fichier par fichier. Les mentions d'affichage non vérifié dans les « Points à confirmer » constituent la liste de contrôle de cette passe.

Deux conséquences pour qui reprend ces fichiers :

- ne pas lire « pas vérifié à l'écran » comme un doute sur les faits — les valeurs, les limites et les comportements viennent du code et sont sûrs ;
- **certains états ne seront jamais observables.** Les messages d'erreur supposent de provoquer la panne correspondante — disque plein, service injoignable, archive corrompue. Ils sont documentés d'après le code, avec leur source ; ils ne recevront pas de capture.

## Le rapport avec le mockup existant

Un mockup de page de documentation existe dans `docs/beaver-site/mockup/docs.html`. Il donne la mise en page cible : sommaire latéral groupé, fil d'Ariane, sommaire de page à droite, encadrés, tableaux, blocs terminal.

**Le mockup est un brouillon.** Il montre ce qui a déjà été pensé — la mise en page, l'esprit, les sujets à couvrir — mais **ses mots ne sont pas à reprendre** et son contenu comporte des approximations comme des éléments périmés.

Deux exemples relevés :

- sa page d'exemple sur les modes de permission décrit correctement les trois modes, mais son tableau simplifie le comportement réel — plusieurs outils ne déclenchent une approbation que sous condition ;
- il évoque une organisation en onglets de conversations qui **n'existe plus** dans le produit.

**Le code source fait foi**, sur le contenu comme sur le vocabulaire. En cas de désaccord entre le mockup et un fichier de ce dossier, le fichier fait foi ; en cas de doute sur un fichier de ce dossier, le code tranche.

Le mockup propose six groupes dans le sommaire : Démarrage, Agent, Outils, Extensions, Forecast, Référence. Ce dossier en compte quatorze, plus fins. Le regroupement final est une décision de conception du site ; la correspondance est donnée ci-dessous.

## Correspondance avec le sommaire du mockup

| Groupe du mockup | Sections de ce dossier |
|---|---|
| Démarrage | 01 Découverte, 02 Installation |
| Agent | 04 Agent local |
| Outils | 05 Outils, 03 Interface (terminal, navigateur, fichiers) |
| Extensions | 07 Intégrations |
| Forecast | 08 Forecast |
| Référence | 09 Automatisation, 10 Réglages, 11 Sécurité, 12 Référence, 13 Dépannage, 14 Projet |

Les sections 03 Interface et 10 Réglages n'ont pas d'équivalent direct dans le mockup. Elles sont pourtant nécessaires : une application de bureau avec seize onglets de réglages a besoin d'une page de référence qui les décrit tous.

## Langue

Ces fichiers sont en français. La traduction vers les six autres langues de l'application se fera une fois le contenu figé — traduire pendant que le fond bouge encore multiplie le travail par sept à chaque correction.

## Ce qui reste à décider par l'équipe du site

Ces points ne sont pas tranchés ici parce qu'ils relèvent de la conception du site, pas du produit :

- le regroupement final des pages dans le sommaire ;
- le tutoiement ou le vouvoiement — le mockup tutoie, ces fichiers vouvoient par neutralité ;
- les captures d'écran, absentes de ces fichiers ;
- la version de l'application à laquelle la documentation correspond, à afficher quelque part sur le site.
