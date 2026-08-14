# Installation sur Linux

**Emplacement site** — Démarrage › Installation › Linux
**Répond à** — « Comment j'installe Beaver sur ma distribution, et laquelle est prise en charge ? »
**Sources** — `install.sh` (lignes 142-160, 166-175), `CROSS-PLATFORM.md`, `.github/workflows/release.yml`
**Vérification** — Vérifié dans le code, avec une correction importante par rapport à `CROSS-PLATFORM.md`

---

## Avertissement au rédacteur

`CROSS-PLATFORM.md` annonce un support **Fedora/RHEL via `dnf`**. Le code ne l'implémente pas : `install.sh` n'appelle que `apt-get`, ne construit que le suffixe `_amd64.deb`, et vérifie le paquet avec `dpkg-deb` et `dpkg-query`.

**Ne pas publier de procédure Fedora/RHEL.** Voir *Points à confirmer*.

---

## Plan de page proposé

1. Distributions prises en charge
2. Méthode recommandée — le script d'installation
3. Ce que fait le script
4. Méthode alternative — le paquet `.deb`
5. Dépendances
6. Limites connues sur Linux
7. Mettre à jour
8. Désinstaller

---

## Contenu

### 1. Distributions prises en charge

- **Debian et Ubuntu, architecture x64 uniquement.**
- Le paquet est construit sur **Ubuntu 22.04**, ce qui le rend compatible avec 22.04, 24.04 et 25.04 : la bibliothèque C standard est rétro-compatible vers le haut, donc construire sur la plus ancienne cible couvre les suivantes.
- Aucune autre famille de distribution n'est prise en charge par le script.
- Pas de build ARM.

### 2. Méthode recommandée — le script d'installation

Même commande que sur macOS ; le script détecte le système lui-même :

```bash
(
  installer="$(mktemp /tmp/beaver-bootstrap.XXXXXXXX)" &&
  trap 'rm -f "$installer"' EXIT &&
  curl --proto '=https' --tlsv1.2 --fail --silent --show-error \
    --output "$installer" \
    https://raw.githubusercontent.com/Kevin-hDev/Beaver/main/install.sh &&
  bash "$installer"
)
```

### 3. Ce que fait le script

1. Détecte le système et l'architecture. **N'accepte que `Linux:x86_64` ou `Linux:amd64`.**
2. Interroge l'API GitHub Releases, réponse plafonnée à **512 Kio**.
3. Exige l'asset `.deb` et le fichier `update-manifest.json`.
4. Suit **trois redirections au maximum**, uniquement vers `release-assets.githubusercontent.com`.
5. Télécharge le `.deb` avec un plafond de **2 Gio**, vérifie sa taille et son **SHA-256**.
6. **Refuse si un paquet `beaver` ou `cl-go` est déjà installé.**
7. **Contrôle les métadonnées du paquet avant installation** — c'est inhabituel et mérite d'être mentionné : nom `beaver`, architecture `amd64`, version identique à celle annoncée par la release, et champs `Provides`, `Conflicts`, `Replaces` valant tous `cl-go`. Un paquet qui ne correspond pas exactement est rejeté.
8. Installe via `apt-get install -y`, ce qui tire les dépendances manquantes.
9. Vérifie que `/usr/bin/cl-go-dash` existe et n'est pas un lien symbolique.
10. Lance l'application en arrière-plan.

Le script **demande une élévation de privilèges** pour installer le paquet.

### 4. Méthode alternative — le paquet `.deb`

- Le `.deb` est attaché à chaque release GitHub.
- Installation manuelle : `sudo apt install ./Beaver_X.Y.Z_amd64.deb`.
- Utiliser `apt` plutôt que `dpkg -i` : `apt` résout les dépendances, `dpkg` non et laisse un paquet à moitié configuré.
- **Le même `.deb` sert à la première installation et aux mises à jour intégrées.**

### 5. Dépendances

Installées automatiquement par `apt-get` :

- `libwebkit2gtk-4.1-0` — le moteur de rendu web utilisé par Tauri 2
- `libgtk-3-0` — la boîte à outils graphique

Aucune intervention manuelle en principe.

### 6. Limites connues sur Linux

À énoncer clairement sur cette page, pas seulement ailleurs :

- **Le navigateur intégré n'est pas disponible sur Linux.** Toutes les autres fonctionnalités le sont.
- **Fenêtre au fond transparent** — problème ouvert, sans cause identifiée. Pistes évoquées dans le dépôt : les réglages `decorations` et `transparent` de la configuration Tauri, le compositeur (Wayland contre X11), ou le thème GTK.
- **GPU AMD jamais testé en conditions réelles.** La détection fonctionne — lecture des identifiants constructeur dans `/sys/class/drm/*/device/vendor` — et l'archive ROCm est bien téléchargée, mais le résultat n'a pas été validé sur une machine équipée.
- **GPU NVIDIA** : pilote **531 ou supérieur** requis. CUDA est fourni dans le bundle Ollama, rien d'autre à installer.

### 7. Mettre à jour

- **Le script refuse de mettre à jour** : « Une application est déjà installée. Utilise sa mise à jour intégrée. »
- La détection couvre le paquet `beaver` et le paquet `cl-go`, l'ancien nom.
- Les mises à jour passent par la fonction intégrée, qui installe le même `.deb`.

### 8. Désinstaller

Voir *Points à confirmer*.

Ce qui est certain :
- Le paquet se retire avec `sudo apt remove beaver`.
- Les données restent dans `~/.local/share/cl-go-dash/`.
- La clé maîtresse du coffre reste dans le magasin de secrets (Secret Service).

---

## Tableaux

### Tableau — Dépendances installées automatiquement

| Paquet | Rôle |
|---|---|
| `libwebkit2gtk-4.1-0` | Moteur de rendu web de Tauri 2 |
| `libgtk-3-0` | Boîte à outils graphique |

### Tableau — Contrôles du paquet avant installation

| Champ vérifié | Valeur exigée |
|---|---|
| `Package` | `beaver` |
| `Architecture` | `amd64` |
| `Version` | Identique à la version annoncée par la release |
| `Provides` | `cl-go` |
| `Conflicts` | `cl-go` |
| `Replaces` | `cl-go` |

---

## Encadrés

**Encadré « Debian et Ubuntu uniquement »** — avertissement.
> Le script d'installation ne prend en charge que les distributions de la famille Debian, en x64. Fedora, RHEL, Arch et openSUSE ne sont pas gérés.

**Encadré « Navigateur intégré »** — information.
> Le navigateur intégré n'est pas disponible sur Linux. Le reste de l'application fonctionne normalement.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| « Système non pris en charge » | Architecture non x64, ou système non reconnu | Aucune : pas de build ARM |
| « Une application est déjà installée » | Paquet `beaver` ou `cl-go` présent | Utiliser la mise à jour intégrée, ou `apt remove` d'abord |
| « Paquet d'installation invalide » | Une métadonnée du `.deb` ne correspond pas | Signaler le problème : cela signifie que l'asset publié ne correspond pas à ce qu'attend le script |
| Dépendances manquantes après `dpkg -i` | `dpkg` ne résout pas les dépendances | `sudo apt install -f`, ou réinstaller avec `apt install ./fichier.deb` |
| Fenêtre transparente | Problème ouvert, cause inconnue | Aucune solution documentée ; essayer une session X11 si vous êtes sous Wayland |
| Modèles très lents malgré un GPU | GPU non exploité par Ollama | Consulter `~/.local/share/cl-go-dash/logs/ollama-sidecar.log` |

---

## Renvois

- *Prérequis* — compatibilité et matériel
- *Premier lancement* — la suite
- *Mise à jour* — la mise à jour intégrée
- *Dépannage › Ollama* — les problèmes de GPU
- *Interface › Navigateur intégré* — la restriction de plateforme

---

## Points à confirmer

- **Fedora et RHEL.** Annoncés dans `CROSS-PLATFORM.md`, absents du code. Trancher avant publication : ajouter le support, ou corriger le document interne. En l'état, la documentation interne du projet est fausse sur ce point.
- **La procédure de désinstallation complète.** Non documentée. Établir la commande de retrait, le sort du dossier de données, et le nettoyage de l'entrée dans le Secret Service.
- **Le fond de fenêtre transparent.** Reproduire et identifier : Wayland ou X11, quel environnement de bureau, quel thème. Sans cela, la page de dépannage ne peut rien proposer.
- **Le GPU AMD.** À tester réellement avant d'affirmer quoi que ce soit sur le site.
- **Le nom du binaire.** `/usr/bin/cl-go-dash`, pas `beaver`. Vérifier si la commande de lancement en terminal doit être documentée, et sous quel nom.
- **Fraîcheur de `CROSS-PLATFORM.md`** — document daté d'avril 2026 pour une version courante 1.1.2.
