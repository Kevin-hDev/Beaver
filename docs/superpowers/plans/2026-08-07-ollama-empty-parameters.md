# Ollama Empty Parameters Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Supprimer réellement toute personnalisation Ollama dont le champ est vidé, sans altérer les autres directives du Modelfile.

**Architecture:** Une fonction Rust pure réécrit le Modelfile complet en retirant uniquement les directives `PARAMETER` de premier niveau, puis ajoute les seules valeurs non vides. `OllamaClient::update_parameters` reconstruit ensuite le modèle à partir du `FROM` réel avec le lanceur CLI existant, ce qui supprime l’auto-héritage.

**Tech Stack:** Rust, Tauri, Ollama CLI, React 19, TypeScript, Vitest.

## Global Constraints

- Un champ vide reste vide après rechargement et ne produit aucune directive `PARAMETER`.
- Préserver toutes les directives autres que `PARAMETER`, y compris les blocs multilignes et les directives inconnues.
- Valider les clés, longueurs, types numériques et caractères de contrôle avant transformation.
- Ne jamais journaliser le contenu brut du Modelfile.
- Écrire chaque test avant le code qui le fait passer.
- Garder chaque fichier source sous 230 lignes.

---

### Task 1: Transformation sûre des paramètres

**Files:**
- Create: `src-tauri/src/services/agent_local/ollama_modelfile_parameters.rs`
- Create: `src-tauri/src/services/agent_local/ollama_modelfile_parameters_tests.rs`
- Modify: `src-tauri/src/services/agent_local/agent_local_modules_core.rs`

**Interfaces:**
- Consumes: `ollama_parameter_validation::validate_parameter_entries` et `modelfile_parser::parse_param_value`.
- Produces: `rewrite(content: &str, entries: &[(String, String)]) -> Result<String, String>`.

- [ ] Écrire des tests qui exigent la suppression de tous les anciens paramètres, la conservation de `FROM` et des directives inconnues, la conservation d’un faux `PARAMETER` dans un bloc multiligne, les `stop` multiples et le rendu sûr des chaînes.
- [ ] Lancer `cargo test ollama_modelfile_parameters --lib` et vérifier que les tests échouent parce que le module n’existe pas encore.
- [ ] Implémenter le scanner de directives de premier niveau et le rendu déterministe des valeurs.
- [ ] Relancer les tests et vérifier qu’ils passent.

### Task 2: Reconstruction sans auto-héritage

**Files:**
- Modify: `src-tauri/src/services/agent_local/ollama_client.rs:130`
- Modify: `src-tauri/src/services/agent_local/ollama_parameter_validation.rs`
- Delete: `src-tauri/src/services/agent_local/ollama_create_payload.rs`
- Modify: `src-tauri/src/services/agent_local/agent_local_modules_core.rs`

**Interfaces:**
- Consumes: `ollama_modelfile_parameters::rewrite`.
- Produces: `OllamaClient::update_parameters` qui appelle `ollama_modelfile_create::create_from_modelfile` avec le texte transformé.

- [ ] Écrire un test de validation qui refuse les retours à la ligne réels dans une valeur afin d’empêcher l’injection d’une directive.
- [ ] Lancer `cargo test ollama_parameter_validation --lib` et observer l’échec attendu.
- [ ] Ajouter la validation des retours à la ligne et brancher la reconstruction sur le Modelfile transformé.
- [ ] Supprimer le constructeur JSON devenu inutilisé et son module.
- [ ] Relancer les tests Rust ciblés et vérifier qu’ils passent.

### Task 3: Contrat du bouton de remise à zéro

**Files:**
- Modify: `src/components/ollama/__tests__/model-parameters-editor.test.tsx`

**Interfaces:**
- Consumes: bouton `ollama.useDefaultValue` et commande Tauri `update_parameters`.
- Produces: test utilisateur garantissant qu’un champ vidé est absent du payload de sauvegarde.

- [ ] Écrire un test qui charge `num_ctx`, clique sur sa remise à zéro, sauvegarde et attend un payload vide.
- [ ] Vérifier que le test protège le comportement réel du composant.
- [ ] Lancer les tests frontend ciblés et vérifier qu’ils passent avec le contrat existant.

### Task 4: Validation et livraison

**Files:**
- Modify: `graphify-out/` uniquement via la commande de maintenance, sans l’ajouter au commit.

- [ ] Exécuter `cargo fmt --check`, les tests Rust ciblés, les tests frontend ciblés, `npx tsc --noEmit`, `npm run lint` et `cargo check --lib`.
- [ ] Exécuter les suites complètes proportionnellement au risque si les validations ciblées sont propres.
- [ ] Exécuter `graphify update .` ou consigner précisément l’impossibilité si le graphe du worktree est absent.
- [ ] Vérifier le diff, l’absence de secrets et l’absence de fichiers hors périmètre.
- [ ] Commiter la correction et compléter la note Git du commit avec la cause, les décisions et la matrice de tests restante.
