# Vue d'ensemble de l'interface

**Emplacement site** — Interface › Vue d'ensemble (première page de la section)
**Répond à** — « Je viens d'ouvrir l'application. Qu'est-ce que je regarde ? »
**Sources** — `src/components/layout/nav-items.ts`, `app-layout.tsx`, `window-toolbar.tsx`, `window-controls.tsx`, `panel-slots.tsx`, `sidebar-compact-state.ts`, `src/components/settings/settings-sections.ts`
**Vérification** — Vérifié dans le code : les quatre sections de navigation et l'organisation des réglages

---

## Rôle de cette page

Page d'orientation. Elle nomme chaque zone de l'écran et renvoie vers sa page détaillée. Elle ne décrit aucune procédure.

Contrainte de rédaction : **une seule page, un seul écran de référence**. Si le site n'a qu'une capture d'écran annotée, c'est ici qu'elle va.

---

## Plan de page proposé

1. Les quatre sections de l'application
2. L'organisation de l'écran
3. La barre d'outils
4. Les réglages
5. Les commandes de fenêtre

---

## Contenu

### 1. Les quatre sections de l'application

La navigation principale compte **quatre destinations**, dans cet ordre :

| Section | Contenu |
|---|---|
| **Sessions** | Les conversations avec l'agent — le cœur de l'application |
| **Réveils** | Les instructions programmées et leur historique |
| **Personnalité** | Les fichiers qui définissent le comportement de l'agent |
| **Réglages** | La configuration complète |

### 2. L'organisation de l'écran

De gauche à droite :

- **La barre latérale** — la liste des éléments de la section active : conversations, réveils, fichiers de personnalité. Largeur ajustable, masquable avec **⌘B**.
- **La zone centrale** — la conversation, ou le contenu de la section.
- **Le panneau latéral** — prévisualisation, navigateur ou Forecast, au choix. Voir *Panneau latéral*.
- **Le terminal** — s'ouvre en bas avec **⌘J**.

L'application replie automatiquement la barre latérale et compacte la conversation quand la fenêtre devient trop étroite.

### 3. La barre d'outils

En haut de la fenêtre :

- navigation avant et arrière dans l'historique de consultation ;
- recherche (**⌘G**) ;
- nouvelle conversation (**⌥⌘N**) ;
- notifications de mise à jour, avec barre de progression pendant un téléchargement ;
- indicateur de téléchargement de modèle en cours.

### 4. Les réglages

**Cinq sections, seize onglets.** Le regroupement suit ce qu'on configure, pas l'usage qu'on en fait.

| Section | Onglets |
|---|---|
| Préférences | Général, Mascotte, Raccourcis |
| Agent | Mémoire, Prompt système, Outils, Avancé |
| Modèles | Ollama, Forecast, LLM |
| Intégrations | Fournisseurs, Connecteurs, Canaux, Extensions |
| Application | Conversations archivées, À propos |

Point de conception intéressant à mentionner : Ollama et Forecast voisinent parce qu'on y installe et paramètre tous deux des modèles locaux, même si l'un produit du texte et l'autre des prévisions. Les fournisseurs sont rangés dans les intégrations parce qu'on y saisit une clé et une connexion, pas un modèle.

### 5. Les commandes de fenêtre

L'application dessine ses propres commandes de fenêtre, adaptées au système.

Différence importante déjà signalée ailleurs, à rappeler brièvement : **sur macOS, fermer la fenêtre ne quitte pas l'application** — elle reste dans le Dock. Sur Windows et Linux, la croix quitte réellement.

Renvoyer vers *Premier lancement* pour le détail.

---

## Encadrés

**Encadré « Chaque conversation a son environnement »**
> Modèle, permissions, répertoire de travail et outils actifs sont propres à chaque conversation.

**Encadré « Sur macOS, la pastille rouge ne quitte pas »**
> Fermer la fenêtre la masque : l'application reste dans le Dock. Utilisez ⌘Q pour quitter.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| La barre latérale disparaît | Repli automatique sur fenêtre étroite | Élargir la fenêtre, ou ⌘B |
| Un réglage attendu est introuvable | Seize onglets répartis en cinq sections | Voir le tableau ci-dessus |
| L'application semble fermée mais tourne (macOS) | La fermeture masque la fenêtre | ⌘Q pour quitter |

---

## Renvois

Cette page renvoie vers toute la section Interface, ainsi que vers *Réglages › Référence complète*.

---

## Points à confirmer

- **Le contenu de la section Personnalité.** Repérée dans la navigation, non explorée. À traiter dans *Agent › Personnalité et AGENTS.md*.
- **L'historique de navigation avant/arrière** — porte-t-il sur les conversations, les sections, les deux ?
- **La liste complète des éléments de la barre d'outils.** Les principaux sont relevés ; une vérification à l'écran est nécessaire avant publication.
- **Le comportement de la barre latérale par section.** Elle affiche une liste différente selon la section active ; le détail n'a pas été vérifié pour Réveils et Personnalité.
- **La bannière d'erreur de coffre.** Un composant existe pour signaler un problème d'accès au coffre. Décrire ce que voit l'utilisateur et ce qu'il doit faire — à traiter dans *Sécurité › Coffre*.
