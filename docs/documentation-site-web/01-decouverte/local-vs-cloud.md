# Modèles locaux, clés API et comptes web

**Emplacement site** — Démarrage › Choisir son modèle (à placer avant la page Fournisseurs & comptes web du mockup)
**Répond à** — « Je dois choisir entre faire tourner un modèle chez moi ou payer un service. Quelle différence, et laquelle pour mon cas ? »
**Sources** — `README.md`, `src-tauri/src/services/llm/catalog.rs`, `src-tauri/src/services/llm/`, `src-tauri/src/services/oauth_providers/`, `src/components/agent-local/chat-input-actions-row.tsx`
**Vérification** — Liste des fournisseurs vérifiée dans le code (`services/llm/`, identifiants `groq`, `gemini`/`google`, `mistral`, `cerebras`, `openrouter`, `openai`, `deepseek`, `xai`, `kimi`/`moonshot`, `glm`/`zai`) ; comparatif issu du README et du fonctionnement général

---

## Plan de page proposé

1. Les trois voies
2. Tableau comparatif
3. Quand choisir le local
4. Quand choisir un modèle distant
5. Les fournisseurs disponibles
6. Combiner les deux
7. Changer de modèle en cours de conversation

---

## Contenu

### 1. Les trois voies

Point à poser d'emblée : **le choix n'est pas définitif**. Il se fait conversation par conversation, et se change en cours de route sans perdre l'historique.

- **Modèle local via Ollama** — installé sur le disque, exécuté sur le processeur ou la carte graphique. Rien ne sort de la machine.
- **Modèle distant via clé API** — clé créée chez le fournisseur, saisie une fois dans Beaver, requêtes envoyées à ses serveurs, facturation à l'usage.
- **Modèle distant via compte web** — pour trois fournisseurs seulement, connexion avec un compte existant au lieu d'une clé. Consommation imputée à ce compte selon ses propres règles.

### 2. Tableau comparatif

Voir la section Tableaux ci-dessous. C'est l'élément central de la page : le placer haut, avant les développements.

### 3. Quand choisir le local

Critères à énumérer :

- les fichiers contiennent des données qui ne doivent être transmises à personne ;
- travail sans connexion ou avec une connexion incertaine ;
- refus d'une facturation à l'usage ;
- machine dotée de mémoire vidéo confortable.

**La contrainte réelle est le matériel.** Un modèle qui ne tient pas en mémoire vidéo bascule sur le processeur, et le temps de réponse s'allonge nettement. Renvoyer vers la page *Matériel et VRAM* plutôt que de donner des chiffres ici — ils vieillissent vite.

### 4. Quand choisir un modèle distant

- la tâche demande un raisonnement long ou une grande fenêtre de contexte ;
- besoin de réponses rapides sans immobiliser la machine ;
- matériel insuffisant pour un modèle assez capable ;
- contenu dont le passage chez un fournisseur ne pose pas de problème.

### 5. Les fournisseurs disponibles

Tableau complet en section Tableaux. Deux mentions obligatoires en dessous :

- Les modèles proposés, les quotas et les tarifs sont décidés par le fournisseur et **changent sans préavis**. Ne jamais publier de prix sur le site.
- Beaver affiche les informations de compte **que le fournisseur accepte de communiquer** — la couverture varie de l'un à l'autre.

### 6. Combiner les deux

Décrire une organisation type plutôt qu'imposer une règle :

- modèle local pour les tâches courantes, les fichiers sensibles et le travail hors ligne ;
- modèle distant pour les tâches qui demandent du raisonnement, sur du contenu non sensible ;
- recherche web par **SearXNG local** quand on veut éviter d'envoyer les requêtes à un tiers ; par **Brave, Exa ou Firecrawl** quand on veut de meilleurs résultats.

### 7. Changer de modèle en cours de conversation

- Le sélecteur se trouve **dans la barre de saisie, sous la zone de message**. ✓ Vérifié : `model-selector.tsx` est monté par `model-controls.tsx`, lui-même dans `chat-input-actions-row.tsx`.
- Le changement ne perd pas l'historique de la conversation.
- Ne pas écrire « en haut de la conversation » : c'est faux, et c'est l'erreur que fait une lecture rapide de l'interface.

---

## Tableaux

### Tableau 1 — Comparatif des trois voies

| | Local (Ollama) | Clé API | Compte web |
|---|---|---|---|
| Données envoyées à un tiers | Aucune | Messages et contenu des fichiers lus | Identique à la clé API |
| Coût | Aucun, hors électricité | Facturé au jeton | Selon l'abonnement du compte |
| Fonctionne hors ligne | Oui | Non | Non |
| Vitesse | Dépend du matériel | Généralement élevée | Généralement élevée |
| Capacité de raisonnement | Limitée par la taille du modèle qui tient en mémoire | Accès aux modèles les plus capables | Identique à la clé API |
| Mise en place | Télécharger un modèle | Créer une clé chez le fournisseur | Se connecter |
| Limites d'usage | Aucune | Selon le crédit | Selon les quotas du compte |

### Tableau 2 — Fournisseurs disponibles

| Type | Fournisseur | Connexion |
|---|---|---|
| LLM | Groq | Clé API |
| LLM | Google Gemini | Clé API |
| LLM | Mistral | Clé API |
| LLM | Cerebras | Clé API |
| LLM | OpenRouter | Clé API |
| LLM | OpenAI | Clé API ou compte web OpenAI/Codex |
| LLM | DeepSeek | Clé API |
| LLM | xAI | Clé API ou compte web Grok |
| LLM | Moonshot Kimi | Clé API ou compte web Kimi (expérimental) |
| LLM | Z.ai GLM | Clé API |
| Recherche | Brave Search | Clé API |
| Recherche | Exa | Clé API |
| Recherche et extraction | Firecrawl | Clé API |
| Recherche | SearXNG | Aucune clé, exécution locale |
| Prévision | Nixtla TimeGPT | Clé API |

Les URL de création de clé figurent dans le README et doivent être reprises sur la page *Fournisseurs*, pas ici — elles changent, et les dupliquer sur deux pages garantit qu'une des deux sera périmée.

---

## Encadrés

**Encadré « Le compte web Kimi est expérimental »** — le README le signale explicitement. Ne pas le présenter au même niveau de fiabilité que les deux autres connexions par compte.

**Encadré « Aucun prix sur le site »** — note interne pour le rédacteur, pas pour la page : les tarifs des fournisseurs changent trop souvent pour être maintenus. Renvoyer vers leurs pages de tarification.

---

## Pièges et erreurs fréquentes

**Croire que « local » veut dire « gratuit et équivalent ».** Un modèle local suffisamment petit pour tourner sur une machine ordinaire est nettement moins capable qu'un modèle distant récent. Le dire franchement évite la déception après installation.

**Croire qu'un modèle local exige de désinstaller Ollama s'il est déjà présent.** C'est l'inverse : Beaver détecte le démon existant sur `localhost:11434` et le réutilise, et les modèles sont partagés.

**Croire que configurer une clé API envoie des données immédiatement.** Rien ne part tant qu'une conversation n'utilise pas ce fournisseur.

---

## Renvois

- *Fournisseurs et comptes web* — la procédure de configuration détaillée
- *Ollama — runtime géré* — l'installation et la gestion du démon
- *Matériel et VRAM* — quelle taille de modèle pour quelle machine
- *Recherche web* — le choix entre Brave, Exa, Firecrawl et SearXNG
- *Confidentialité des données* — ce que voit exactement un fournisseur

---

## Points à confirmer

- **Le statut « expérimental » du compte web Kimi.** Repris du README ; vérifier qu'il est toujours d'actualité au moment de publier.
- **La couverture réelle des informations d'usage par fournisseur.** Le README annonce limites, crédits, jetons, requêtes et estimation de coût « quand le fournisseur les expose ». Établir la liste exacte de ce qui s'affiche pour chacun, sinon la page *Usage et coûts* promettra plus que ce qui existe.
- **Le comportement du changement de modèle en cours de conversation.** Confirmé pour l'historique, mais non vérifié pour les cas limites : que se passe-t-il si le nouveau modèle a une fenêtre de contexte plus petite que l'historique déjà accumulé ? À trancher avant de rédiger la section 7.
- **Les identifiants internes `google`/`gemini` et `zai`/`glm`.** Le code emploie les deux formes. Sans incidence sur la documentation utilisateur, mais à garder en tête si le site affiche un jour des identifiants techniques.
