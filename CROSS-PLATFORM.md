# Cross-plateforme — Installation, Build, Mise à jour

## Architecture du système de mise à jour

### Détection des mises à jour (identique sur les 3 OS)

- **Backend Rust** : `commands/app_update.rs`
  - `check_app_update()` — appelle exactement `api.github.com/repos/Kevin-hDev/Beaver/releases/latest`
  - Compare la version locale (`env!("CARGO_PKG_VERSION")`) avec le tag de la release
  - Cherche le nom d'asset Beaver exact pour l'OS et l'architecture : `.dmg` (macOS), `.deb` (Linux), `-setup.exe` (Windows NSIS)
  - Vérifie que `update-manifest.json` contient la même version, le même nom, la même taille et le SHA-256 attendu
  - Retourne les métadonnées validées ou `None`

- **Backend Rust** : `commands/check_ollama_updates`
  - Liste les modèles installés, fetch les tags registry pour chaque famille
  - Compare les digests SHA256 — retourne la liste des modèles avec update dispo

- **Frontend** : `hooks/use-update-checker.ts`
  - Check au lancement + toutes les heures
  - Écoute aussi `ollama-models-changed` pour re-vérifier après un pull

- **Frontend** : `components/layout/update-notifications.tsx`
  - Bulles style macOS, animation slide-down 500ms staggerée
  - Barre de progression orange pour le téléchargement/pull

### Téléchargement + auto-install (par OS)

**Backend Rust** :
- `commands/app_update_install.rs` — téléchargement borné et vérification SHA-256 de l'asset
- `services/app_update_helper.rs` — copie le helper autonome dans un fichier temporaire unique
- `updater_worker/` — vérifie l'installation actuelle, attend la fermeture de l'app, installe puis vérifie le redémarrage

| OS | Asset | Processus |
|---|---|---|
| **macOS** | `.dmg` | Monte le DMG en lecture seule → vérifie `Beaver.app` → prépare la copie → conserve l'ancienne app → bascule → vérifie le redémarrage → restaure l'ancienne app en cas d'échec |
| **Linux** | `.deb` | Lance `pkexec /usr/bin/apt-get install -y <asset>` avec des arguments séparés → vérifie le redémarrage |
| **Windows** | `-setup.exe` (NSIS) | Lance l'installeur avec les arguments séparés `/S` et `/D=<dossier actuel>` → vérifie le redémarrage |

L'app se ferme automatiquement (`app.exit(0)`) après avoir lancé le helper autonome. Le helper attend sa fermeture avant toute modification et n'utilise aucun shell intermédiaire.

L'auto-update ne réinstalle pas dans un chemin par défaut : il remplace l'installation réellement lancée. Ça évite les doublons quand l'utilisateur a choisi un dossier personnalisé pendant la première installation.

### Récupération après un échec

- **macOS** : le helper conserve l'ancien bundle jusqu'à la confirmation du
  redémarrage et le restaure automatiquement si cette confirmation échoue.
- **Linux et Windows** : le helper détecte un redémarrage défaillant, mais le
  gestionnaire de paquets ou l'installeur a déjà remplacé les fichiers. Il
  n'effectue donc pas de retour arrière automatique.
- Dans ce cas, réinstalle manuellement la dernière version connue comme stable
  depuis sa release GitHub. Les données utilisateur restent dans le dossier
  historique et ne sont pas remplacées par l'installeur.

### Passage depuis l'ancien nom

Les installations CL-GO `v1.0.1` récupèrent d'abord la version-pont CL-GO
`v1.0.2` depuis le dépôt historique. Cette version-pont, puis toutes les
versions Beaver, récupèrent leurs mises à jour uniquement depuis
`Kevin-hDev/Beaver`. Le dépôt historique reste disponible pour les anciennes
installations.

---

## Première installation

### macOS / Linux

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

Le script `install.sh` :
1. Détecte l'OS (`uname -s`) et l'architecture (`uname -m`)
2. Accepte uniquement macOS Apple Silicon et Linux Debian/Ubuntu x64
3. Appelle l'API GitHub Releases avec une réponse limitée à 512 Kio
4. Exige l'asset Beaver exact et `update-manifest.json`
5. Suit au maximum trois redirections GitHub autorisées
6. Limite le téléchargement à 2 Gio et vérifie sa taille et son SHA-256
7. Installe :
   - macOS : monte le DMG en lecture seule → vérifie `Beaver.app` → copie → démonte → lance
   - Linux : installe le paquet `.deb` avec `apt-get`

Ce script est réservé à une première installation. Sur macOS et Linux, il
refuse de remplacer une installation Beaver ou CL-GO existante : utilise la
mise à jour intégrée de l'application.

Le même paquet `.deb` est utilisé pour la première installation et les mises à
jour intégrées sous Linux.

### Windows

```powershell
$installer = Join-Path ([IO.Path]::GetTempPath()) "beaver-install-$([Guid]::NewGuid().ToString('N')).ps1"
try {
  Invoke-WebRequest https://raw.githubusercontent.com/Kevin-hDev/Beaver/main/install.ps1 -OutFile $installer -ErrorAction Stop
  & $installer
} finally {
  Remove-Item -LiteralPath $installer -Force -ErrorAction SilentlyContinue
}
```

Le script `install.ps1` exige l'asset
`Beaver_<version>_x64-setup.exe`, vérifie sa taille et son SHA-256 avec le
manifeste borné, puis lance l'installeur avec `/S` et `/D=...`.

---

## Build et release

### Versions — 3 fichiers à synchroniser

| Fichier | Champ |
|---|---|
| `src-tauri/tauri.conf.json` | `"version": "X.Y.Z"` |
| `src-tauri/Cargo.toml` | `version = "X.Y.Z"` |
| `package.json` | `"version": "X.Y.Z"` |

### CI GitHub Actions

**Workflow** : `.github/workflows/release.yml`

Se déclenche automatiquement quand on push un tag `v*`, ou manuellement via GitHub Actions.

**Builds** :
- `macos-latest` → ARM64 (Apple Silicon) → `.dmg`
- `ubuntu-22.04` → x64 → `.deb`
- `windows-latest` → x64 → NSIS installer (`.exe`)

**Processus pour publier une release** :

```bash
# 1. Bumper la version dans les 3 fichiers
# 2. Commit + push
git add src-tauri/tauri.conf.json src-tauri/Cargo.toml package.json
git commit -m "chore: bump version to X.Y.Z"
git push

# 3. Créer et pusher le tag → déclenche le CI automatiquement
git tag vX.Y.Z
git push origin vX.Y.Z
```

Le CI crée une **release draft** avec tous les assets. Il reste à la publier sur GitHub.

### Build local (macOS uniquement)

```bash
npm run tauri build
# → src-tauri/target/release/bundle/macos/Beaver.app
# → src-tauri/target/release/bundle/dmg/Beaver_X.Y.Z_aarch64.dmg
```

---

## Dépendances par OS

### macOS
- Rien à installer côté utilisateur. `curl` et `hdiutil` sont pré-installés.

### Linux (Ubuntu/Debian)
- `libwebkit2gtk-4.1-0` — moteur web pour Tauri 2
- `libgtk-3-0` — toolkit graphique
- Installées automatiquement par `install.sh` via `apt-get`

### Linux (Fedora/RHEL)
- `webkit2gtk4.1`
- `gtk3`
- Installées automatiquement par `install.sh` via `dnf`

### Windows
- Rien à installer. L'installeur NSIS gère tout.

---

## Accélération matérielle (GPU)

### État des lieux (avril 2026)

Le bundle Ollama (`ollama-windows-amd64.zip` / `ollama-linux-amd64.tar.zst`) inclut les libs GPU dans `lib/ollama/`. Ollama est téléchargé au premier lancement dans `~/.local/share/cl-go-dash/ollama-bundle/`.

### Par OS et GPU

| OS | GPU | Backend | Comment ça marche |
|---|---|---|---|
| **macOS** | Apple Silicon | Metal | Intégré dans le binaire `ollama-darwin.tgz`, rien à configurer |
| **Linux** | NVIDIA | CUDA | Le bundle générique inclut `lib/ollama/libggml-cuda.so` — seul le driver NVIDIA ≥531 est requis côté système |
| **Linux** | AMD | ROCm | Le bundle `ollama-linux-amd64-rocm.tar.zst` est sélectionné automatiquement par `select_archive_name()` quand un GPU AMD est détecté |
| **Windows** | NVIDIA | CUDA | Le bundle inclut `lib/ollama/cuda_v12/` et `cuda_v13/` — Ollama auto-détecte CUDA, rien à configurer |
| **Windows** | AMD | Vulkan | `OLLAMA_VULKAN=1` injecté automatiquement par `ollama_lifecycle.rs` quand un GPU AMD est détecté |

### Détection GPU (`services/gpu_detect.rs`)

- **Linux** : lit les vendor IDs dans `/sys/class/drm/*/device/vendor` (PCI : `0x10de` = NVIDIA, `0x1002` = AMD, `0x8086` = Intel)
- **Windows** : PowerShell `Get-CimInstance Win32_VideoController`, match sur le nom ("nvidia", "geforce", "amd", "radeon", etc.)
- **macOS** : retourne `Unknown` — Metal est toujours disponible via le binaire macOS

### Sélecteur CPU/GPU (`Settings > Avancé`)

L'utilisateur peut choisir entre CPU et GPU dans les paramètres avancés (masqué sur macOS). Le choix est stocké dans `config.json` → `advanced.hardware_accel` (`"cpu"` ou `"gpu"`, défaut `"gpu"`).

Au démarrage du sidecar (`ollama_lifecycle.rs`) :
- `"cpu"` → injecte `OLLAMA_LLM_LIBRARY=cpu` (force le CPU)
- `"gpu"` → injecte `OLLAMA_VULKAN=1` si AMD sur Windows, sinon laisse Ollama auto-détecter

Un bouton "Restart Ollama" apparaît dans les settings après changement pour appliquer immédiatement.

### Pourquoi Vulkan et pas ROCm sur Windows ?

**ROCm Windows** (7.2.2, avril 2026) n'est pas encore production-ready pour Ollama :
- Auto-détection GPU instable (issues ollama #14686, #12573, #15715)
- Les RX 9000 nécessitent des hacks manuels (`ROCBLAS_TENSILE_LIBPATH`)
- Le zip ROCm (`ollama-windows-amd64-rocm.zip`) est un complément, pas un standalone

**Vulkan** est déjà bundlé (`lib/ollama/vulkan/ggml-vulkan.dll`), couvre tous les GPU AMD (RX 5000 à 9000) et fonctionne sans dépendance externe. Seule limitation : Ollama utilise un llama.cpp daté (b7437, déc. 2025) avec ~56% de perf en moins que le potentiel (issue ollama #15601). Ça se comblera quand Ollama mettra à jour.

**Gemma 4 + Vulkan** : le modèle produit des réponses incohérentes (LaTeX, boucles) car son architecture (KV cache partagée) n'est pas correctement gérée par le vieux backend Vulkan d'Ollama (issues ollama #15261, llama.cpp #21516). Workaround : passer en CPU via le sélecteur.

### Logs sidecar

Les logs stderr d'Ollama sont écrits dans `~/.local/share/cl-go-dash/logs/ollama-sidecar.log` (écrasé à chaque démarrage). Utile pour diagnostiquer les problèmes GPU.

---

## Corrections Windows (v0.6.8 → v0.7.1)

| Version | Problème | Cause | Fix |
|---|---|---|---|
| v0.6.8 | Ollama sur CPU malgré GPU AMD | Aucune variable GPU injectée au sidecar | Injection `OLLAMA_VULKAN=1` + logs stderr |
| v0.6.9 | Bouton mise à jour invisible | Code cherchait le mauvais type d'asset Windows, le CI produit NSIS `.exe` | `platform_extension()` → `-setup.exe` + lancement direct NSIS `/S` |
| v0.7.0 | Toggles personnalité cassés | `path.split("/")` ne sépare pas les backslash Windows | `path.split(/[\\/]/)` |
| v0.7.0 | Pas de choix CPU/GPU | — | Sélecteur dans Settings/Avancé + `restart_ollama_sidecar` |
| v0.7.1 | Contour fenêtre invisible | Padding 1px insuffisant sur Windows/Linux | Padding 3px sur `.os-other` uniquement |

---

## Fichiers concernés

```
install.sh                                  # Script première installation macOS/Linux
install.ps1                                 # Script première installation Windows
.github/workflows/release.yml              # CI build cross-plateforme
src-tauri/src/commands/app_update.rs        # Check version GitHub Releases
src-tauri/src/commands/app_update_install.rs # Download + auto-install par OS
src-tauri/src/commands/ollama_updates.rs    # Check mises à jour modèles Ollama
src-tauri/src/commands/ollama_setup.rs      # Download Ollama + restart sidecar
src-tauri/src/services/ollama_lifecycle.rs  # Spawn/kill sidecar + injection env GPU
src-tauri/src/services/gpu_detect.rs        # Détection GPU (sysfs Linux, PowerShell Windows)
src-tauri/src/models/config.rs              # AdvancedSettings.hardware_accel
src/hooks/use-update-checker.ts             # Hook React — check toutes les heures
src/components/layout/update-notifications.tsx  # UI bulles de notification
src/components/layout/update-notifications.css  # Styles des bulles
src/components/layout/window-toolbar.tsx    # Icône update dans le toolbar
src/components/settings/advanced-settings.tsx   # Sélecteur CPU/GPU
src/components/settings/hardware-accel-control.tsx # Composant bouton restart
src/components/personality/personality-tab.tsx   # Toggles injection contexte
```

---

## Ollama — téléchargement au premier lancement

Depuis v0.6.5, Ollama n'est plus bundlé dans l'app. Il est téléchargé au premier lancement :

1. L'app appelle `is_ollama_installed()` → vérifie si le port 11434 est ouvert OU si le binaire existe dans `~/.local/share/cl-go-dash/ollama-bundle/`
2. Le sidecar n'est lancé au setup que si le binaire existe (`ollama_binary_path().is_ok()`) — évite une erreur inutile au premier lancement
3. Si non installé → affiche l'écran de setup → `download_ollama()` télécharge l'archive depuis GitHub Releases
3. Archive extraite dans `~/.local/share/cl-go-dash/ollama-bundle/`
4. Validation post-extraction : le binaire `ollama` (ou `ollama.exe`) doit exister, sinon erreur

### Choix de l'archive par OS

| OS | Archive | Détection GPU |
|---|---|---|
| macOS | `ollama-darwin.tgz` | Non (Metal intégré) |
| Windows | `ollama-windows-amd64.zip` | Non (archive standard uniquement) |
| Linux | `ollama-linux-amd64.tar.zst` ou `-rocm.tar.zst` | Oui (sysfs `/sys/class/drm/card*/device/vendor`) |

**Fichiers** : `commands/ollama_setup.rs`, `commands/ollama_extract.rs`, `services/gpu_detect.rs`

### Validations du download (v0.6.7)

- Taille minimum 10 MB (détecte les pages HTML ou downloads incomplets)
- Refus si Content-Type `text/html`
- Vérification que le binaire existe après extraction
- Cleanup automatique en cas d'échec (archive temp + dossier dest)
- PowerShell : `CREATE_NO_WINDOW` pour ne pas flasher de terminal

---

## Data directory

Depuis v0.6.6, le chemin est unifié sur les 3 OS : `~/.local/share/cl-go-dash/`

`cl-go-dash` est un identifiant historique conservé pour la compatibilité :
Beaver ne déplace, ne copie et ne recrée pas les données existantes.

- macOS/Linux : `~/.local/share/cl-go-dash/`
- Windows : `C:\Users\<user>\.local\share\cl-go-dash\`

**Migration automatique** : si des données existent dans l'ancien emplacement (`%APPDATA%\cl-go-dash` sur Windows, `~/Library/Application Support/cl-go-dash` sur macOS), elles sont copiées automatiquement.

**Initialisation** : au premier lancement, `init_base_structure()` crée tous les dossiers et fichiers par défaut (memory/core, skills, inbox, config.json, AGENTS.md, etc.).

**Fichiers** : `services/paths.rs`, `storage_migration.rs`

---

## Problèmes connus — GPU

### Windows : GPU AMD non utilisé

**Symptôme** : Ollama tourne en mode CPU malgré un GPU AMD (ex: Radeon RX 7800 XT).

**Ce qui a été tenté** :
- v0.6.6 : détection GPU via PowerShell `Get-CimInstance Win32_VideoController` + téléchargement de `ollama-windows-amd64-rocm.zip`
- **Résultat** : le zip ROCm n'est PAS un bundle autonome sur Windows — c'est un complément au bundle principal. Le télécharger seul ne fonctionne pas (ollama.exe absent).
- v0.6.7 : revert — Windows utilise toujours `ollama-windows-amd64.zip` (archive standard)

**Pistes non explorées** :
- Télécharger les DEUX archives (standard + ROCm) et merger dans le même dossier
- Vérifier si l'archive standard inclut déjà le support ROCm (comme elle inclut CUDA)
- Vérifier la documentation Ollama sur le support AMD Windows — peut nécessiter l'installation d'Ollama via l'installeur officiel (`OllamaSetup.exe`) qui gère les drivers ROCm
- Tester si les drivers AMD ROCm sont installés sur la machine Windows

### Linux : GPU AMD non testé

**Symptôme** : la détection GPU fonctionne (lecture sysfs), l'archive ROCm est téléchargée, mais pas encore testé en conditions réelles.

### macOS : pas de problème GPU

Metal est intégré nativement dans l'archive `ollama-darwin.tgz`. Aucune détection nécessaire.

---

## Problèmes connus — Dépendances

### Windows : dépendances non installées automatiquement

**Symptôme** : l'installateur NSIS installe l'app mais ne gère pas les dépendances système (WebView2, VC++ Runtime). Si manquantes, l'app peut ne pas se lancer.

**Ce qui a été tenté** : rien encore — le NSIS généré par Tauri est censé inclure WebView2 bootstrapper.

**Pistes** :
- Vérifier si `tauri.conf.json` > `bundle` > `windows` a les bons paramètres pour WebView2
- Tester sur une machine Windows vierge (sans Visual Studio ni dev tools)
- Vérifier les logs du NSIS installer

### Linux : dépendances gérées par install.sh

Le script `install.sh` installe `libwebkit2gtk-4.1-0` et `libgtk-3-0` automatiquement via apt/dnf. Pas de problème signalé.

### macOS : aucune dépendance externe

Tout est inclus dans le `.app`.

---

## Problèmes connus — Affichage

### Linux : fenêtre transparente

**Symptôme** : le fond de l'application est transparent au lieu d'être opaque.

**Ce qui a été tenté** : rien encore.

**Pistes** :
- Vérifier les paramètres `decorations` et `transparent` dans `tauri.conf.json`
- Peut être lié au compositor (Wayland vs X11) ou au thème GTK

---

## Single instance (v0.7.6)

Plugin `tauri-plugin-single-instance` — empêche d'ouvrir l'app en double sur les 3 OS. Si une instance tourne déjà, la deuxième tentative remet le focus sur la fenêtre existante.

**Fichier** : `src-tauri/src/lib.rs` (plugin init)

---

## Comportement bouton fermer (v0.7.6)

| OS | Bouton croix | Effet |
|---|---|---|
| **macOS** | ❌ rouge | Hide la fenêtre (l'app reste dans le Dock). Clic Dock → re-show. Cmd+Q → quitte vraiment |
| **Linux** | ❌ | Ferme l'app + cleanup sidecar |
| **Windows** | ❌ | Ferme l'app + cleanup sidecar |

Sur macOS, `CloseRequested` est intercepté dans `on_window_event` → `win.hide()` + `api.prevent_close()`. Le cleanup (kill sidecar, kill PTY) ne se déclenche que sur `Exit`/`ExitRequested`, pas sur `CloseRequested`. Le clic Dock est géré via `RunEvent::Reopen` → `win.show()` + `set_focus()`.

**Fichiers** : `src-tauri/src/lib.rs` (on_window_event), `src-tauri/src/app_events.rs` (RunEvent handler)

---

## Splash screen (v0.7.6)

Au lancement, un écran avec l'icône de l'app s'affiche pendant que React charge :

- Fond blur + transparent adapté au thème (dark/light), détecté via `localStorage("clgo-theme")`
- Castor SVG unique coloré via CSS selon le thème sombre ou clair
- SVG préchargé via `<link rel="preload">`
- Retiré instantanément (pas de fade) quand `ollamaReady` résout — couvre les deux chemins : setup screen (premier lancement) et app normale

**Fichiers** : `index.html` (splash HTML/CSS/script), `src/App.tsx` (useEffect removal), `public/castor.svg`

---

## Points d'attention

- **Repo public** : obligatoire pour que l'API GitHub Releases fonctionne sans token d'authentification
- **macOS** : le script `curl` et le téléchargement direct depuis l'app évitent la quarantaine ajoutée par certains navigateurs
- **Ollama téléchargé au premier lancement** : le CI ne bundle PAS Ollama — il est téléchargé dans `~/.local/share/cl-go-dash/ollama-bundle/` au premier lancement
- **Ubuntu 22.04** : builder sur la plus vieille version cible garantit la compatibilité avec 22.04, 24.04, 25.04 (glibc forward-compatible)
- **Chemins Windows** : toujours utiliser `split(/[\\/]/)` côté frontend pour extraire un nom de fichier d'un path — `split("/")` ne fonctionne pas avec les backslash Windows
- **ROCm Linux** : production-ready (v7.2.2), auto-détecté par `select_archive_name()`. Le bundle ROCm est téléchargé automatiquement pour les GPU AMD
- **ROCm Windows** : pas fiable pour Ollama (avril 2026), Vulkan utilisé à la place
- **Windows Defender — Accès contrôlé aux dossiers** : au premier lancement, `ollama.exe` peut être bloqué par la protection anti-ransomware quand il essaie d'écrire sur le disque (modèles dans `~/.ollama/models/`). L'utilisateur doit cliquer "Autoriser" dans la notification — ça ne redemande plus ensuite.
