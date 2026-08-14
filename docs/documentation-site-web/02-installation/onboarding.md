# Le parcours d'accueil

**Emplacement site** — Démarrage › Premier lancement › Parcours d'accueil
**Répond à** — « Qu'est-ce qu'on me demande au démarrage, et que se passe-t-il si je passe une étape ? »
**Sources** — `src/components/onboarding/onboarding-screen.tsx`, `onboarding-welcome.tsx`, `onboarding-preferences.tsx`, `onboarding-agent-import.tsx`, `onboarding-api.tsx`, `onboarding-provider-grid.tsx`, `src/components/ollama/ollama-setup-screen.tsx`
**Vérification** — Vérifié dans le code : l'ordre des étapes, leur contenu et la condition d'affichage de la dernière

---

## Plan de page proposé

1. Vue d'ensemble du parcours
2. Étape 1 — Bienvenue
3. Étape 2 — Préférences
4. Étape 3 — Import depuis un autre assistant
5. Étape 4 — Connexion d'un fournisseur
6. Étape 5 — Ollama (conditionnelle)
7. Revenir en arrière, passer une étape
8. Refaire ces réglages plus tard

---

## Contenu

### 1. Vue d'ensemble du parcours

- **Quatre ou cinq étapes**, présentées en diapositives horizontales.
- La cinquième — Ollama — **n'apparaît que si Ollama n'est pas déjà disponible** sur la machine.
- Aucune étape n'est bloquante : on peut traverser le parcours sans rien configurer et tout faire plus tard dans les réglages.

### 2. Étape 1 — Bienvenue

Écran de présentation avec l'illustration du castor, un titre, une description, un bouton pour commencer. Rien à saisir.

### 3. Étape 2 — Préférences

Deux réglages, et deux seulement :

- **Thème** — parmi les thèmes disponibles.
- **Langue** — parmi les sept langues de l'interface. Le choix est mémorisé côté navigateur sous la clé `clgo-language`.

Ces deux réglages se retrouvent ensuite dans Réglages › Général.

### 4. Étape 3 — Import depuis un autre assistant

Propose de reprendre les instructions, skills et règles depuis une autre application d'agent déjà installée. Neuf sources sont reconnues.

Cette étape a sa page dédiée : renvoyer vers *Import depuis un autre assistant* plutôt que de tout détailler ici.

### 5. Étape 4 — Connexion d'un fournisseur

- Une **grille de cartes**, une par fournisseur de modèles, avec son icône, son nom et une courte description.
- Seuls les fournisseurs de catégorie **LLM** sont affichés, **32 au maximum**.
- Les fournisseurs déjà configurés portent la mention « Connecté ».
- On sélectionne une carte, on saisit la clé, on enregistre. Un lien mène à la page de création de clé du fournisseur.
- La clé saisie est effacée du champ dès qu'on quitte l'étape.

**Cette étape est facultative.** Sans aucune clé, l'application fonctionne avec un modèle local.

### 6. Étape 5 — Ollama (conditionnelle)

- **N'apparaît pas** si un démon Ollama est déjà disponible ou si le binaire est déjà présent dans le dossier de données.
- Sinon, propose le téléchargement et l'installation d'Ollama.
- **Peut être passée.** L'application reste utilisable avec des modèles distants uniquement.

Le détail du téléchargement est dans *Premier lancement* — ne pas le dupliquer.

### 7. Revenir en arrière, passer une étape

- Les étapes Import et Fournisseur proposent un retour en arrière.
- L'enchaînement s'adapte : si l'étape Ollama n'est pas nécessaire, valider l'étape Fournisseur termine directement le parcours.

### 8. Refaire ces réglages plus tard

Tableau de correspondance en section Tableaux. C'est l'information la plus utile de la page : quelqu'un qui a traversé le parcours trop vite veut savoir où retrouver chaque réglage.

---

## Tableaux

### Tableau — Les étapes

| Ordre | Étape | Ce qu'on y fait | Facultative |
|---|---|---|---|
| 1 | Bienvenue | Rien, écran de présentation | — |
| 2 | Préférences | Thème et langue | Oui |
| 3 | Import | Reprendre instructions, skills et règles d'un autre assistant | Oui |
| 4 | Fournisseur | Saisir une clé API | Oui |
| 5 | Ollama | Télécharger le moteur local | Oui, et affichée seulement si nécessaire |

### Tableau — Où refaire ces réglages ensuite

| Réglage | Emplacement |
|---|---|
| Thème | Réglages › Général |
| Langue | Réglages › Général |
| Import depuis un autre assistant | Réglages › Extensions, ou l'assistant d'import |
| Clés de fournisseurs | Réglages › Fournisseurs |
| Ollama | Réglages › Ollama |

---

## Encadrés

**Encadré « Rien n'est définitif »** — en tête de page.
> Aucune étape du parcours d'accueil n'est obligatoire. Tout ce qui y est proposé se retrouve dans les réglages.

**Encadré « Sans clé API »** — étape 4.
> Vous pouvez traverser cette étape sans rien saisir. Beaver fonctionne avec un modèle local, sans compte ni clé.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| La grille de fournisseurs reste vide | Le catalogue n'a pas pu être chargé | Passer l'étape et configurer plus tard dans les réglages |
| L'étape Ollama n'apparaît pas | Ollama déjà disponible sur la machine | Normal : Beaver réutilise l'installation existante |
| Le parcours ne réapparaît pas au lancement suivant | Il ne s'affiche qu'une fois | Tous les réglages sont accessibles dans Réglages |
| La clé saisie semble perdue en revenant en arrière | Le champ est vidé au changement de fournisseur et en fin d'étape | Vérifier dans Réglages › Fournisseurs que la clé est bien enregistrée |

---

## Renvois

- *Premier lancement* — le contexte : stockage, migration, Ollama
- *Import depuis un autre assistant* — le détail de l'étape 3
- *Fournisseurs et comptes web* — configurer une clé après coup
- *Interface › Thèmes* et *Interface › Langues*

---

## Points à confirmer

- **Le parcours peut-il être relancé ?** Le code ne montre pas de commande pour le rejouer. Confirmer, et si ce n'est pas possible, le dire explicitement sur le site.
- **Le comportement quand aucune étape n'est validée.** Vérifier qu'on arrive bien dans une application utilisable, et ce qui s'affiche alors dans une conversation neuve sans modèle disponible.
- **L'emplacement exact de l'assistant d'import dans les réglages.** À vérifier avant de publier le tableau de correspondance : l'onglet exact n'est pas confirmé.
- **La limite de 32 fournisseurs affichés.** Sans effet aujourd'hui puisqu'il y en a dix, mais à garder en tête si le catalogue s'étoffe.
- **Le mode de permission n'est pas proposé pendant le parcours.** L'application démarre en **Accès complet**. Envisager de le mentionner à l'étape de bienvenue, et en attendant, le signaler sur cette page.
