# Choisir un modèle adapté à sa machine

**Emplacement site** — Modèles › Matériel
**Répond à** — « Quel modèle local puis-je faire tourner avec ma machine ? »
**Sources** — `src/components/settings/vram-table.tsx`, `services/gpu_vram.rs`, `services/gpu_detect.rs`, `services/ollama_env.rs`, `src/components/ollama/model-profile-specs.ts`
**Vérification** — Vérifié dans le code

---

## Plan de page proposé

1. La question à se poser
2. Les deux chiffres qui comptent
3. La table de correspondance
4. Comment lire la compression
5. Ce que Beaver détecte tout seul
6. Le cas des Mac à puce Apple
7. Quand ça ne tient pas

---

## Contenu

### La question à se poser

Un modèle de langage doit tenir **entièrement en mémoire** pour fonctionner correctement. Sur une carte graphique, c'est la mémoire vidéo ; sur un Mac récent, c'est la mémoire partagée entre le processeur et la partie graphique.

Quand un modèle ne tient pas, deux choses peuvent arriver : le chargement échoue, ou une partie du modèle bascule sur la mémoire ordinaire — et là, la génération devient dix à cinquante fois plus lente. Le second cas est le plus déroutant : rien n'échoue, tout est simplement inutilisable.

D'où la question à trancher avant de télécharger : **combien de mémoire ce modèle demande-t-il ?**

### Les deux chiffres qui comptent

Le nom d'un modèle local porte presque toujours ces deux informations.

**Le nombre de paramètres**, écrit en milliards : 3B, 7B, 13B, 30B, 70B. C'est la « taille » du modèle. Plus il est grand, meilleures sont ses réponses, et plus il demande de mémoire.

**Le niveau de compression** — noté Q4, Q5, Q8, f16. Un modèle est compressé pour tenir dans moins de mémoire, au prix d'une légère perte de qualité. C'est le levier le plus efficace : passer un modèle de 13 milliards de paramètres de la version non compressée à la version Q4 fait passer sa consommation de 28 Go à 8 Go.

### La table de correspondance

Beaver affiche cette table dans ses réglages avancés. Elle mérite de figurer aussi sur le site, c'est l'information la plus concrètement utile de toute la section.

**Mémoire nécessaire, en gigaoctets :**

| Taille | Q4_K_M | Q5_K_M | Q8_0 | f16 |
|---|---|---|---|---|
| **3 milliards** | ~2 | ~2,5 | ~3,5 | ~6 |
| **7 milliards** | ~4,5 | ~5,5 | ~8 | ~16 |
| **13 milliards** | ~8 | ~9,5 | ~14 | ~28 |
| **30 milliards** | ~20 | ~23 | ~34 | ~68 |
| **70 milliards** | ~40 | ~48 | ~70 | ~140 |

Ces valeurs sont des **estimations**, et le site doit le dire. Elles couvrent le modèle lui-même, pas le contexte de la conversation, qui occupe de la place en plus à mesure qu'elle s'allonge. Prévoir une marge d'un à deux gigaoctets.

**Lecture rapide par configuration :**

| Machine | Ce qui passe confortablement |
|---|---|
| 8 Go | Modèles de 3 à 7 milliards, compressés en Q4 |
| 16 Go | Jusqu'à 13 milliards en Q4, ou 7 milliards en Q8 |
| 24 Go | Jusqu'à 30 milliards en Q4 |
| 32 Go | 30 milliards en Q5, confortablement |
| 48 Go et plus | 70 milliards en Q4 |

### Comment lire la compression

| Notation | Ce que ça veut dire | Quand la choisir |
|---|---|---|
| **Q4_K_M** | Compression forte | Le choix par défaut. Le meilleur compromis dans la quasi-totalité des cas |
| **Q5_K_M** | Compression moyenne | Quand la mémoire le permet et que la qualité compte |
| **Q8_0** | Compression légère | Rarement justifié : double la mémoire pour un gain à peine perceptible |
| **f16** | Aucune compression | Réservé aux machines très bien dotées, ou à des mesures comparatives |

**Le conseil pratique à écrire sur le site** : entre un modèle plus grand fortement compressé et un modèle plus petit peu compressé, à mémoire égale, **le grand modèle compressé gagne presque toujours**. Un modèle de 13 milliards en Q4 donne de meilleures réponses qu'un modèle de 7 milliards en Q8, pour la même mémoire.

### Ce que Beaver détecte tout seul

Au démarrage, Beaver identifie la carte graphique et mesure la mémoire disponible, puis en tire la taille de contexte accordée aux modèles locaux :

| Mémoire détectée | Contexte accordé |
|---|---|
| 24 Go ou plus | **32 768 jetons** |
| 12 à 24 Go | **24 576 jetons** |
| Moins de 12 Go, ou indétectable | **8 192 jetons** |

Quand la mesure échoue, Beaver prend la valeur la plus prudente plutôt que d'espérer.

Il réserve par ailleurs **un gigaoctet de mémoire vidéo** au système : sans cette marge, un modèle qui remplit exactement la mémoire rend l'affichage saccadé ou fait échouer le chargement.

L'écran de détail d'un modèle affiche ses caractéristiques réelles avant installation : taille du fichier, nombre de paramètres, longueur de contexte, niveau de compression, architecture, et s'il s'agit d'un modèle à experts spécialisés.

### Le cas des Mac à puce Apple

Sur un Mac à puce Apple, il n'y a pas de mémoire vidéo séparée : processeur et partie graphique **partagent la même mémoire**. Un Mac de 16 Go dispose donc d'environ 16 Go pour un modèle — moins ce qu'utilisent le système et les applications ouvertes.

C'est un avantage réel : un Mac de 32 Go fait tourner des modèles qui demanderaient une carte graphique haut de gamme sur un PC.

Deux réserves à mentionner :

- il faut **laisser de la place au système** — compter 4 à 6 Go pris par macOS et les applications courantes ;
- sur un **Mac à processeur Intel**, la mémoire n'est pas mesurée et Beaver retombe sur le contexte minimal.

### Quand ça ne tient pas

Par ordre de préférence :

1. **Prendre une compression plus forte** du même modèle — Q8 vers Q4 divise la mémoire par deux.
2. **Descendre d'une taille** — 13 milliards vers 7 milliards.
3. **Fermer les autres applications**, surtout les navigateurs, très gourmands en mémoire vidéo.
4. **Vérifier qu'un seul modèle est chargé** — le multi-modèle se désactive dans les réglages avancés.
5. **Passer en mode processeur seul** — ça fonctionne, c'est très lent, mais ça débloque une machine sans carte graphique utilisable.
6. **Utiliser un modèle distant** pour cette tâche — voir `01-decouverte/local-vs-cloud.md`.

---

## Encadrés

> **Un modèle doit tenir entièrement en mémoire.**
> S'il déborde, il ne plante pas forcément : il devient dix à cinquante fois plus lent. C'est le symptôme le plus courant et le plus déroutant.

> **À mémoire égale, préférez un grand modèle compressé.**
> 13 milliards de paramètres en Q4 valent mieux que 7 milliards en Q8. La compression coûte beaucoup moins en qualité que la réduction de taille.

> **Q4_K_M est le bon choix par défaut.**
> Dans la quasi-totalité des cas. Les niveaux supérieurs doublent la mémoire pour un gain difficilement perceptible.

> **Prévoyez une marge.**
> Les valeurs de la table couvrent le modèle seul. Le contexte de la conversation s'y ajoute et grandit avec elle.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| « Le modèle répond extrêmement lentement » | Il déborde sur la mémoire ordinaire | Compression plus forte, ou modèle plus petit |
| « Le chargement du modèle échoue » | Mémoire insuffisante | Idem |
| « Ça marchait, et maintenant c'est lent » | Une autre application a pris de la mémoire | Fermer le navigateur, relancer |
| « Mon contexte est limité à 8 192 jetons » | Moins de 12 Go détectés, ou mesure impossible | Comportement attendu |
| « Ma carte graphique n'est pas utilisée » | Fabricant non reconnu, ou pilotes absents | Vérifier les traces du moteur |
| « J'ai 16 Go mais un modèle de 13 milliards ne passe pas » | Le système et les applications occupent déjà de la place | Fermer des applications, ou passer en Q4 |

---

## Renvois

- `06-modeles/ollama-runtime.md` — la détection matérielle et les réglages du moteur
- `06-modeles/ollama-modeles.md` — installer un modèle et lire ses caractéristiques
- `01-decouverte/local-vs-cloud.md` — quand renoncer au local
- `02-installation/prerequis.md` — la configuration minimale
- `13-depannage/ollama.md`

---

## Points à confirmer

- **La table de mémoire est codée en dur dans l'interface** (`vram-table.tsx`), avec cinq tailles et quatre niveaux de compression. Elle ne s'adapte pas aux formats plus récents qui pourraient apparaître. Sans conséquence aujourd'hui — ce sont des estimations et les ordres de grandeur restent valables — mais à surveiller.
- **La formule de calcul affichée sous la table** vient d'une clé de traduction que je n'ai pas relevée. À récupérer si le site reproduit la table.
- Les **recommandations par configuration** (8 Go, 16 Go, 24 Go…) sont ma déduction à partir de la table, pas une donnée du produit. À faire valider avant publication — c'est le tableau que les lecteurs utiliseront le plus.
- Le **conseil « grand modèle compressé plutôt que petit modèle peu compressé »** est un consensus du domaine, pas une affirmation vérifiée dans le code de Beaver. À conserver, mais sans le présenter comme une mesure faite par l'équipe.
- Affichage à vérifier lors de la passe d'interface : emplacement exact de la table dans les réglages, et présence d'un avertissement quand le modèle sélectionné dépasse la mémoire détectée.
