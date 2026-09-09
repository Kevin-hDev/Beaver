# Prérequis

**Emplacement site** — Démarrage › Installation › Prérequis (ou en tête de la page Installation)
**Répond à** — « Est-ce que ça tourne sur ma machine ? »
**Sources** — `install.sh` (lignes 166-170), `install.ps1`, `CROSS-PLATFORM.md`, `.github/workflows/release.yml`, `src-tauri/tauri.conf.json`
**Vérification** — Architectures vérifiées dans `install.sh` ; dépendances issues de `CROSS-PLATFORM.md` avec une correction majeure signalée ci-dessous

---

## Avertissement au rédacteur

`CROSS-PLATFORM.md` affirme que le script d'installation gère **Fedora/RHEL via `dnf`**. C'est **faux dans le code actuel** : `install.sh` n'appelle que `apt-get` (ligne 153) et n'accepte que les couples `Darwin:arm64`, `Darwin:aarch64`, `Linux:x86_64`, `Linux:amd64` (lignes 166-170).

Ne pas reprendre l'affirmation Fedora/RHEL sur le site tant que ce n'est pas retesté. Voir *Points à confirmer*.

---

## Plan de page proposé

1. Tableau de compatibilité
2. macOS
3. Windows
4. Linux
5. Matériel recommandé
6. Réseau

---

## Contenu

### 1. Tableau de compatibilité

Voir section Tableaux. C'est l'élément principal de la page.

Point à énoncer avant le tableau : **une seule architecture par système** est distribuée. Il n'y a pas de build macOS Intel, pas de build Linux ARM, pas de build Windows ARM.

### 2. macOS

- **Apple Silicon uniquement** (puces M1 et suivantes). ✓ `install.sh` n'accepte que `Darwin:arm64` et `Darwin:aarch64`.
- **Les Mac Intel ne sont pas pris en charge.** Le script refuse l'installation ; il n'existe pas d'asset x86_64.
- Aucune dépendance à installer : `curl` et `hdiutil` sont fournis par le système.
- L'application n'est pas signée. macOS Gatekeeper peut la bloquer si le `.dmg` est téléchargé par un navigateur — c'est la raison d'être du script `install.sh`, qui passe par `curl` et évite la mise en quarantaine.

### 3. Windows

- **x64 uniquement.** Pas de build ARM.
- Installeur NSIS, aucune dépendance à installer manuellement en principe.
- **Réserve connue** : l'installeur NSIS ne gère pas explicitement WebView2 et le runtime Visual C++. S'ils manquent sur une machine vierge, l'application peut ne pas démarrer. Le sujet est ouvert dans `CROSS-PLATFORM.md`, non résolu.
- **Windows Defender — accès contrôlé aux dossiers** : au premier lancement, `ollama.exe` peut être bloqué par la protection anti-rançongiciel quand il écrit les modèles dans `~/.ollama/models/`. L'utilisateur doit cliquer « Autoriser » dans la notification ; la demande ne se répète pas.

### 4. Linux

- **x64 uniquement**, paquet `.deb`.
- **Debian et Ubuntu.** Le paquet est construit sur **Ubuntu 22.04**, ce qui garantit la compatibilité avec 22.04, 24.04 et 25.04 (glibc rétro-compatible vers le haut).
- Dépendances installées automatiquement par le script via `apt-get` :
  - `libwebkit2gtk-4.1-0` — moteur de rendu web de Tauri 2
  - `libgtk-3-0` — boîte à outils graphique
- **Les distributions hors famille Debian ne sont pas prises en charge par le script.**
- **Le navigateur intégré n'est pas disponible sur Linux.** À mentionner ici, pas seulement dans la page navigateur : c'est un critère de choix avant installation.
- Problème d'affichage connu : fenêtre au fond transparent au lieu d'opaque, cause non identifiée, possiblement liée au compositeur (Wayland contre X11) ou au thème GTK.

### 5. Matériel recommandé

Distinguer deux usages, sinon la recommandation n'a pas de sens :

**Avec des modèles distants uniquement** (clé API ou compte web) : n'importe quelle machine récente convient. L'application ne fait que de l'affichage et du réseau.

**Avec des modèles locaux** : c'est la mémoire vidéo qui décide. Un modèle qui n'y tient pas bascule sur le processeur et devient très lent.

- macOS Apple Silicon : Metal est intégré au binaire Ollama, rien à configurer.
- Linux NVIDIA : pilote NVIDIA **531 ou supérieur** requis ; CUDA est fourni dans le bundle.
- Linux AMD : l'archive ROCm est sélectionnée automatiquement à la détection du GPU.
- Windows NVIDIA : CUDA fourni dans le bundle, détection automatique.
- Windows AMD : Vulkan, activé automatiquement.

Renvoyer vers *Matériel et VRAM* pour les correspondances taille de modèle / mémoire, et ne pas les dupliquer ici.

### 6. Réseau

- **Connexion requise au premier lancement** pour télécharger Ollama (plusieurs centaines de Mo).
- Le dépôt GitHub doit être joignable : la recherche de mises à jour et le téléchargement d'Ollama passent par l'API GitHub Releases.
- **Après installation, l'application fonctionne hors ligne** avec un modèle local déjà téléchargé.
- Les modèles distants et la recherche web exigent évidemment une connexion.

---

## Tableaux

### Tableau — Compatibilité

| Système | Architecture | Format | Dépendances | Navigateur intégré |
|---|---|---|---|---|
| macOS | Apple Silicon (arm64) uniquement | `.dmg` | Aucune | Oui |
| Windows | x64 uniquement | Installeur NSIS `.exe` | WebView2, VC++ Runtime (voir réserve) | Oui |
| Linux | x64 uniquement | `.deb` (Debian/Ubuntu) | `libwebkit2gtk-4.1-0`, `libgtk-3-0` | **Non** |

### Tableau — Accélération matérielle par système

| Système | GPU | Moteur | Configuration |
|---|---|---|---|
| macOS | Apple Silicon | Metal | Aucune, intégré au binaire |
| Linux | NVIDIA | CUDA | Pilote NVIDIA ≥ 531 requis |
| Linux | AMD | ROCm | Archive ROCm sélectionnée automatiquement |
| Windows | NVIDIA | CUDA | Aucune, détection automatique |
| Windows | AMD | Vulkan | Aucune, activé automatiquement |

---

## Encadrés

**Encadré « Mac Intel »** — style avertissement, très visible.
> Beaver est distribué uniquement pour les Mac à puce Apple Silicon (M1 et suivantes). Les Mac à processeur Intel ne sont pas pris en charge.

**Encadré « Linux et navigateur intégré »**
> Le navigateur intégré n'est pas disponible sur Linux. Toutes les autres fonctionnalités le sont.

**Encadré « Premier lancement et connexion »**
> Le premier lancement télécharge Ollama depuis GitHub. Prévoyez une connexion et quelques centaines de mégaoctets. Ensuite, l'application fonctionne hors ligne avec un modèle local.

---

## Pièges et erreurs fréquentes

**Croire qu'un Mac Intel fonctionnera quand même.** Le script échoue avec un message d'architecture non prise en charge. Le dire avant que la personne télécharge.

**Installer sur Fedora ou Arch en suivant la page Linux.** Le script ne gère que `apt-get`. Voir l'avertissement en tête.

**Lancer l'application sans connexion au premier démarrage.** L'écran de configuration d'Ollama ne peut pas aboutir. Ce n'est pas un blocage définitif — on peut relancer plus tard — mais le message doit être clair.

**Compter sur le GPU AMD sous Windows pour de la performance maximale.** Vulkan fonctionne mais Ollama s'appuie sur une version datée de llama.cpp, avec une perte de performance importante documentée en amont.

---

## Renvois

- *Installation macOS*, *Installation Windows*, *Installation Linux* — les procédures
- *Matériel et VRAM* — quelle taille de modèle pour quelle machine
- *Premier lancement* — ce qui se passe après l'installation
- *Dépannage › Installation* — quand ça échoue

---

## Points à confirmer

- **Fedora et RHEL.** `CROSS-PLATFORM.md` annonce un support via `dnf` que `install.sh` n'implémente pas. Trancher : soit le support est ajouté, soit la mention disparaît du dépôt. Ne rien publier avant.
- **Version minimale de macOS.** `tauri.conf.json` ne définit pas `minimumSystemVersion` ; la valeur par défaut de Tauri s'applique. Déterminer la version réellement requise et l'afficher — un prérequis système sans numéro de version est incomplet.
- **WebView2 et VC++ Runtime sous Windows.** Le sujet est ouvert dans `CROSS-PLATFORM.md` : le bootstrapper WebView2 est censé être inclus par Tauri, sans test sur machine vierge. À vérifier avant de promettre « aucune dépendance ».
- **Le GPU AMD sous Linux n'a jamais été testé en conditions réelles** d'après `CROSS-PLATFORM.md`. La détection fonctionne, l'archive ROCm est bien téléchargée, la suite est inconnue. Formuler prudemment.
- **Le problème de fenêtre transparente sous Linux.** Toujours ouvert, cause non identifiée. Décider s'il figure dans les prérequis, dans le dépannage, ou dans les deux.
- **Fraîcheur de `CROSS-PLATFORM.md`.** Le document mentionne « avril 2026 » et des versions v0.6.x–v0.7.x, alors que la version courante est 1.1.2. Une partie des constats GPU et des problèmes connus peut être périmée. Faire une passe de vérification avant publication.
