# Le moteur local — comment Beaver fait tourner Ollama

**Emplacement site** — Modèles › Moteur local
**Répond à** — « Beaver installe-t-il Ollama ? Et si j'ai déjà Ollama sur ma machine ? »
**Sources** — `services/ollama_lifecycle.rs`, `services/ollama_port.rs`, `services/ollama_env.rs`, `services/ollama_kill.rs`, `services/gpu_detect.rs`, `services/gpu_vram.rs`, `ollama_polling.rs`, `models/config.rs`
**Vérification** — Vérifié dans le code

---

## Plan de page proposé

1. Ce que Beaver installe
2. Si vous avez déjà Ollama
3. Le port choisi
4. Les réglages appliqués au moteur
5. La taille de contexte s'adapte au matériel
6. L'arrêt et le nettoyage
7. Les traces

---

## Contenu

### Ce que Beaver installe

Beaver embarque son propre moteur Ollama, téléchargé au premier lancement dans son dossier de données, sous `ollama-bundle/`.

Deux conséquences à énoncer clairement :

- **Aucune installation système n'est nécessaire.** L'utilisateur n'a pas à installer Ollama séparément, ni à connaître son existence.
- **Le moteur de Beaver n'interfère pas avec une installation existante.** Il vit dans son propre dossier, avec ses propres réglages.

Le binaire est cherché à deux endroits dans le dossier du moteur : dans un sous-dossier `bin/`, puis à la racine. S'il est absent des deux, Beaver signale que le moteur n'est pas installé.

### Si vous avez déjà Ollama

C'est le comportement le plus utile de cette page, et il est invisible pour qui ne le cherche pas.

**Beaver détecte un moteur Ollama déjà en cours d'exécution et le réutilise** plutôt que d'en lancer un second. La détection ne se contente pas de constater qu'un port est occupé : elle **interroge réellement l'interface d'Ollama** pour vérifier que c'est bien lui qui répond, et non un autre programme.

Conséquences pratiques :

- Les modèles déjà téléchargés par l'application Ollama officielle sont **immédiatement disponibles** dans Beaver. Rien à retélécharger.
- Il n'y a **jamais deux moteurs en mémoire** — donc pas de modèle chargé deux fois, ni de mémoire vidéo occupée en double.
- **Les réglages de Beaver ne s'appliquent pas** à un moteur qu'il n'a pas lancé. Le moteur existant garde sa propre configuration. Ce point doit figurer sur le site : un utilisateur qui règle la persistance des modèles dans Beaver ne verra aucun effet si Beaver réutilise le moteur du système.

### Le port choisi

Beaver ne s'impose pas sur le port habituel d'Ollama.

1. Il cherche un port libre entre **11500 et 11599**.
2. Si un moteur Ollama répond déjà sur le port habituel **11434**, il le réutilise.
3. En dernier recours, il retombe sur **11434**.

Le moteur n'écoute que sur la machine locale. **Il n'est joignable depuis aucun autre appareil du réseau.**

### Les réglages appliqués au moteur

Quand Beaver lance son propre moteur, il lui impose une configuration. Le tableau complet est plus bas ; les points qui méritent une explication sur le site :

- **Les modèles distants d'Ollama sont désactivés.** Ollama propose d'exécuter certains modèles sur ses propres serveurs ; Beaver coupe cette possibilité. Quand une conversation utilise un modèle local dans Beaver, **elle est réellement locale** — rien ne part sur le réseau.
- **Un seul modèle est chargé en mémoire à la fois**, sauf si l'utilisateur active le multi-modèle dans les réglages avancés. Sur une machine ordinaire, deux modèles chargés simultanément saturent la mémoire vidéo.
- **Une seule requête est traitée à la fois.** Beaver est une application de bureau, pas un serveur partagé.
- **Dix minutes sont accordées au chargement d'un modèle.** C'est long exprès : sur un disque lent ou avec un modèle volumineux, le premier chargement peut réellement prendre plusieurs minutes.
- **Un gigaoctet de mémoire vidéo est laissé libre** quand une carte graphique est détectée, pour que le système d'exploitation et l'affichage continuent de fonctionner.
- **Combien de temps un modèle reste en mémoire** après usage est configurable, y compris « indéfiniment ». Un modèle qui reste chargé répond instantanément à la question suivante, au prix de la mémoire occupée.

### La taille de contexte s'adapte au matériel

Beaver mesure la mémoire disponible au démarrage et en déduit la taille de contexte des modèles locaux :

| Mémoire détectée | Contexte accordé |
|---|---|
| **24 Go ou plus** | **32 768 jetons** |
| **12 Go à 24 Go** | **24 576 jetons** |
| Moins de 12 Go, ou indétectable | **8 192 jetons** |

Quand la mémoire ne peut pas être mesurée, Beaver prend la valeur la plus basse. C'est le bon réflexe : mieux vaut un contexte modeste qui fonctionne qu'un contexte ambitieux qui fait échouer le chargement.

Sur les Mac à puce Apple, la mémoire mesurée est la **mémoire unifiée** — celle que le processeur et la partie graphique se partagent. Un Mac de 16 Go entre donc dans le palier intermédiaire.

### L'arrêt et le nettoyage

- À la fermeture de Beaver, le moteur qu'il a lancé est arrêté, **avec tout son arbre de processus**.
- Un moteur **réutilisé** n'est jamais arrêté : Beaver ne coupe pas un processus qu'il n'a pas démarré.
- L'identifiant du processus lancé est enregistré sur le disque. **Au démarrage suivant, un moteur orphelin** — laissé par un arrêt brutal ou une panne de courant — est détecté et arrêté avant qu'un nouveau ne démarre.
- Avant de tuer un processus orphelin, Beaver **vérifie deux fois que c'est bien un moteur Ollama**. Un identifiant de processus est réutilisé par le système : sans cette vérification, Beaver risquerait de tuer un programme sans rapport qui aurait hérité du même numéro.
- Beaver sait aussi demander au moteur de **libérer la mémoire vidéo** sans l'arrêter, quand elle doit servir ailleurs.

### Les traces

Les messages du moteur sont écrits dans `logs/ollama-sidecar.log`, dans le dossier de données de Beaver.

**Ce fichier est écrasé à chaque démarrage.** Il contient donc les traces de la session en cours, pas un historique. C'est le premier endroit où regarder quand un modèle refuse de se charger.

Beaver écrit de son côté, dans ses propres traces, le port retenu, la carte graphique détectée et les réglages appliqués — **en filtrant explicitement** ce qui pourrait être sensible : seules les variables d'une liste autorisée sont écrites.

---

## Tableaux

### Les réglages imposés au moteur lancé par Beaver

| Réglage | Valeur | Pourquoi |
|---|---|---|
| Écoute | Machine locale uniquement | Rien n'est exposé au réseau |
| Attention optimisée | Activée | Vitesse et mémoire |
| Compression du cache | Activée | Contexte plus long à mémoire égale |
| Requêtes simultanées | **1** | Application de bureau, pas serveur |
| Modèles distants d'Ollama | **Désactivés** | Le local reste local |
| Délai de chargement | **10 minutes** | Tolérance aux machines lentes |
| Taille de contexte | **8 192 à 32 768** | Selon la mémoire détectée |
| Modèles chargés en même temps | **1**, ou illimité si le multi-modèle est activé | Éviter la saturation mémoire |
| Persistance en mémoire | Configurable, « indéfiniment » possible | Compromis vitesse / mémoire |
| Mémoire vidéo réservée au système | **1 Go** | Garder l'affichage réactif |
| Mode processeur seul | Sur demande | Machines sans carte graphique utilisable |
| Accélération Vulkan | Windows uniquement | Cartes non NVIDIA |

### La détection de la carte graphique

| Plateforme | Méthode | Fabricants reconnus |
|---|---|---|
| macOS | Interrogation du processeur | Apple |
| Linux | Lecture des identifiants matériels du système | AMD, NVIDIA, Intel |
| Windows | Interrogation du gestionnaire de périphériques | NVIDIA, AMD, Intel |

### La détection de la mémoire

| Plateforme | Source, par ordre de préférence |
|---|---|
| macOS (puce Apple) | Mémoire unifiée du système |
| Linux | Outil NVIDIA, puis informations mémoire du pilote graphique |
| Windows | Outil NVIDIA, puis registre système, puis compteurs de performance |

Sur un Mac à processeur Intel, la mémoire n'est pas mesurée : Beaver prend le palier le plus bas.

---

## Encadrés

> **Vous n'avez pas à installer Ollama.**
> Beaver télécharge son propre moteur au premier lancement. Aucune manipulation, aucun terminal.

> **Si vous avez déjà Ollama, Beaver le réutilise.**
> Vos modèles déjà téléchargés sont immédiatement disponibles, et il n'y a jamais deux moteurs en mémoire. En contrepartie, les réglages de moteur de Beaver ne s'appliquent pas à cette installation-là.

> **Un modèle local est vraiment local.**
> Beaver désactive les modèles distants proposés par Ollama. Quand une conversation utilise un modèle local, rien ne quitte la machine.

> **Le moteur n'est joignable que depuis votre machine.**
> Il n'écoute pas sur le réseau. Aucun autre appareil ne peut s'y connecter.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| « Le moteur n'est pas installé » | Téléchargement du premier lancement interrompu | Relancer l'installation depuis les réglages |
| « Mes réglages de moteur ne changent rien » | Beaver réutilise un moteur système | Quitter l'application Ollama officielle, puis relancer Beaver |
| « Le chargement du modèle échoue » | Mémoire insuffisante pour le modèle choisi | Modèle plus petit, ou version plus compressée |
| « Le premier message est très lent » | Chargement du modèle en mémoire | Normal ; augmenter la persistance en mémoire pour éviter le rechargement |
| « Ma carte graphique n'est pas utilisée » | Fabricant non reconnu, ou pilotes absents | Vérifier les traces du moteur ; forcer le mode processeur si besoin |
| « Beaver a laissé un processus en arrière-plan » | Arrêt brutal | Il est détecté et arrêté au démarrage suivant |
| « Deux modèles saturent ma mémoire » | Multi-modèle activé | Le désactiver dans les réglages avancés |

---

## Renvois

- `06-modeles/ollama-modeles.md` — installer et supprimer des modèles
- `06-modeles/ollama-personnalisation.md` — les réglages par modèle
- `06-modeles/materiel-et-vram.md` — choisir une taille de modèle
- `02-installation/premier-lancement.md` — le téléchargement initial
- `10-reglages/modeles.md` — les réglages avancés du moteur
- `12-reference/journaux.md` — les traces du moteur
- `13-depannage/ollama.md`

---

## Points à confirmer

- **La liste des réglages du moteur exposés à l'utilisateur** dans l'écran des réglages avancés n'a pas été relevée. Le code applique une douzaine de valeurs ; seules quelques-unes semblent configurables (multi-modèle, persistance, accélération matérielle). À compléter en écrivant `10-reglages/modeles.md`.
- **Le comportement exact quand un moteur système est réutilisé** mérite une vérification produit : Beaver affiche-t-il quelque part qu'il réutilise un moteur existant ? Sans indication visible, un utilisateur ne peut pas comprendre pourquoi ses réglages restent sans effet. **Recommandation à l'équipe : le signaler dans l'interface.**
- Le **téléchargement du moteur au premier lancement** — taille, durée, reprise après interruption, vérification d'intégrité — est traité dans `02-installation/premier-lancement.md`. À relire pour éviter que les deux pages divergent.
- Sur **Mac Intel**, la mémoire n'est pas mesurée et le contexte tombe au palier minimal. À confirmer que c'est voulu et non un effet de bord.
