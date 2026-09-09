# Installer et gérer des modèles locaux

**Emplacement site** — Modèles › Modèles locaux
**Répond à** — « Comment j'installe un modèle sur ma machine, et comment je choisis ? »
**Sources** — `commands/ollama_setup.rs`, `commands/ollama_updates.rs`, `services/agent_local/ollama_registry_details.rs`, `services/agent_local/ollama_client.rs`, `src/components/ollama/` (`model-search.tsx`, `model-variants-list.tsx`, `model-profile.tsx`, `model-profile-specs.ts`, `model-install-button.tsx`), `services/ollama_lifecycle.rs`
**Vérification** — Vérifié dans le code pour les mécanismes ; parcours d'interface à confirmer

---

## Plan de page proposé

1. Où trouver des modèles
2. Choisir une variante
3. Ce que dit la fiche d'un modèle
4. Installer
5. Mettre à jour
6. Supprimer
7. Les modèles partagés avec Ollama

---

## Contenu

### Où trouver des modèles

Beaver interroge le catalogue public d'Ollama et permet d'y chercher un modèle par son nom, depuis l'application — sans passer par un navigateur ni par une ligne de commande.

Le catalogue est celui d'Ollama : les mêmes modèles, les mêmes noms.

### Choisir une variante

Un modèle n'existe pas en un seul exemplaire. Il se décline en **variantes**, qui combinent une taille et un niveau de compression. Chaque variante a son propre poids sur le disque et sa propre demande en mémoire.

C'est là que se joue le choix, et le site doit y renvoyer vers `06-modeles/materiel-et-vram.md`, qui donne la table de correspondance.

### Ce que dit la fiche d'un modèle

Avant d'installer, Beaver affiche les caractéristiques réelles de la variante :

| Information | Ce qu'elle apporte |
|---|---|
| **Capacités** | Ce que le modèle sait faire — outils, images, raisonnement |
| **Taille du fichier** | Ce qu'il occupera sur le disque |
| **Nombre de paramètres** | Sa « taille » — 3, 7, 13, 30, 70 milliards |
| **Longueur de contexte** | Combien il peut lire d'un coup |
| **Niveau de compression** | Le compromis mémoire / qualité |
| **Architecture** | La famille technique |
| **Experts spécialisés** | Si le modèle n'active qu'une partie de lui-même à chaque requête |
| **Empreinte** | L'identifiant exact de la version |

La ligne **Capacités** est la plus importante pour l'usage agentique et mérite un encadré : **un modèle qui ne sait pas utiliser d'outils ne pourra pas travailler dans Beaver au-delà de la conversation.** Il répondra, mais ne lira aucun fichier et ne lancera aucune commande.

### Installer

L'installation télécharge la variante choisie. C'est long — plusieurs gigaoctets — et la progression est affichée.

Points de comportement vérifiés dans le code :

- **Une seule installation à la fois.** Un verrou empêche deux téléchargements simultanés, qui se disputeraient la bande passante et le disque.
- **L'installation est annulable.** L'annulation est propre : si le moteur n'était pas encore installé et que le premier téléchargement est interrompu, le dossier partiel est supprimé plutôt que laissé en place.
- Le moteur redémarre et attend d'être prêt avant que l'installation soit déclarée réussie.

### Mettre à jour

Beaver sait détecter les modèles installés dont une version plus récente existe.

Le mécanisme, à décrire simplement : il regroupe les modèles installés par famille, interroge le catalogue pour chaque famille, et compare. Jusqu'à **100 familles** sont examinées.

Un modèle mis à jour garde son nom : c'est la version derrière qui change.

### Supprimer

Supprimer un modèle libère l'espace disque correspondant. C'est réversible au prix d'un nouveau téléchargement.

### Les modèles partagés avec Ollama

Point important, déjà abordé dans `06-modeles/ollama-runtime.md` et à rappeler ici :

**Si l'application Ollama officielle est installée et en cours d'exécution, Beaver réutilise son moteur — donc ses modèles.** Tout ce qui a été téléchargé d'un côté est disponible de l'autre, sans copie ni retéléchargement.

Conséquence à écrire : **supprimer un modèle depuis Beaver le supprime aussi pour l'application Ollama**, puisque c'est le même stockage. Ce n'est pas une copie.

---

## Encadrés

> **Vérifiez que le modèle sait utiliser des outils.**
> C'est la capacité qui décide de tout dans Beaver. Un modèle sans cette capacité tient une conversation mais ne lit aucun fichier et ne lance aucune commande.

> **Une variante = une taille + une compression.**
> Le même modèle existe en plusieurs versions dont la demande en mémoire va du simple au décuple. Voir la table dans `06-modeles/materiel-et-vram.md`.

> **Vos modèles sont partagés avec l'application Ollama.**
> Si vous l'avez installée, le stockage est commun : rien à retélécharger, mais une suppression vaut des deux côtés.

> **Une seule installation à la fois.**
> Un second téléchargement attend que le premier finisse.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| « Le modèle est installé mais n'utilise aucun outil » | Modèle sans capacité d'appel d'outils | Vérifier les capacités avant installation |
| « Le téléchargement est très lent » | Plusieurs gigaoctets | Normal ; il est annulable et reprenable |
| « Je n'arrive pas à lancer deux installations » | Verrou volontaire | Attendre la fin de la première |
| « Le modèle est installé mais ne se charge pas » | Mémoire insuffisante | Prendre une variante plus compressée |
| « J'ai supprimé un modèle et il a disparu d'Ollama aussi » | Stockage partagé | Comportement attendu |
| « Aucune mise à jour n'est proposée » | Le modèle est à jour, ou sa famille n'a pas été examinée | Au-delà de 100 familles installées, toutes ne sont pas vérifiées |

---

## Renvois

- `06-modeles/materiel-et-vram.md` — choisir la bonne variante
- `06-modeles/ollama-runtime.md` — le moteur et le partage avec Ollama
- `06-modeles/ollama-personnalisation.md` — régler un modèle installé
- `06-modeles/catalogue-et-favoris.md` — retrouver ses modèles
- `02-installation/premier-lancement.md`
- `13-depannage/ollama.md`

---

## Points à confirmer

- **Le parcours exact d'installation dans l'interface** n'est pas décrit : où se trouve la recherche, comment la liste des variantes se présente, ce qu'affiche la barre de progression. Le code des composants existe (`model-search`, `model-variants-list`, `model-install-button`) mais je n'ai pas reconstitué l'enchaînement des écrans. **À compléter avant rédaction du site.**
- **Le téléchargement est-il reprenable** après une interruption ? Le code gère l'annulation et le nettoyage, mais je n'ai pas vérifié la reprise d'un téléchargement partiel. Affirmation à valider — je l'ai indiquée dans le tableau des pièges, à retirer si elle est fausse.
- **La limite de 100 familles** pour la vérification des mises à jour est silencieuse : un utilisateur avec beaucoup de modèles ne saura pas que tous n'ont pas été examinés. À signaler à l'équipe.
- **La suppression d'un modèle** n'a pas été lue dans le code : je décris le comportement attendu. À vérifier, notamment s'il existe une confirmation.
- **Les modèles recommandés** — s'il en existe une liste mise en avant pour un premier usage — n'ont pas été identifiés. Ce serait très utile sur le site : un utilisateur qui découvre ne sait pas par où commencer.
