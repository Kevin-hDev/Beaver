# Choix interactif — `ask_user_choice`

**Emplacement site** — Outils › Choix interactif
**Répond à** — « Pourquoi l'agent me pose parfois une question à choix multiples, et que se passe-t-il si je l'ignore ? »
**Sources** — `tool_interactive.rs`, `tool_interactive_parse.rs`, `tool_definitions_interactive.rs`, `interactive_choice_gate.rs`, `types_interactive.rs`, `tool_prompt_filter.rs`
**Vérification** — Vérifié dans le code, sauf la présentation à l'écran

---

## Plan de page proposé

1. Ce que fait l'outil
2. Quand l'agent est censé s'en servir
3. À quoi ressemble une question
4. Répondre, ou ne pas répondre
5. Désactiver les questions

---

## Contenu

### Ce que fait l'outil

`ask_user_choice` permet à l'agent d'**interrompre son travail pour poser une question à choix**, et d'attendre la réponse avant de continuer.

Il appartient au groupe **Choix utilisateur**, optionnel et **actif par défaut**.

La conversation est réellement suspendue : l'agent ne devine pas, ne poursuit pas en parallèle. Il attend.

### Quand l'agent est censé s'en servir

La définition de l'outil encadre son usage de façon assez stricte, et le site gagne à reprendre cette logique : elle explique pourquoi les questions sont rares.

**L'agent doit demander quand** :

- plusieurs approches sont valables et le choix appartient à l'utilisateur ;
- la demande est ambiguë et la suite dépend de sa préférence ;
- en mode Plan, une question de conception reste ouverte avant de proposer le plan.

**L'agent ne doit pas demander quand** :

- une option par défaut raisonnable existe — il la prend, la mentionne, et continue ;
- il cherche seulement un accord sur un plan — c'est le rôle du mode Plan ;
- la réponse se trouve en lisant le code ou la documentation.

C'est le point à retenir sur le site : **une question interactive signifie que la décision vous appartient vraiment**. Ce n'est pas une demande de validation, et ce n'est pas un aveu d'ignorance.

### À quoi ressemble une question

Le format est contraint, ce qui garantit une présentation homogène :

- **1 à 5 questions** en une seule fois ;
- **2 à 4 options** par question ;
- un intitulé très court par question — **30 caractères maximum** ;
- la question elle-même — **500 caractères maximum** ;
- pour chaque option, un libellé court (**80 caractères**) et **une phrase** qui explique ce qu'elle implique (**1 500 caractères**) ;
- **exactement une option recommandée par question** — ni zéro, ni deux. C'est vérifié, et une question qui n'en a pas est refusée.
- une option peut porter un **aperçu** : un extrait de code, une maquette en texte, un exemple de configuration, pour comparer visuellement.
- une question peut autoriser **plusieurs réponses** à la fois quand les options ne s'excluent pas.

La recommandation est signalée par l'interface, pas par le texte : l'agent n'a pas le droit d'écrire « (recommandé) » dans un libellé.

Une option **« Autre »** est toujours disponible et permet d'écrire une réponse libre — jusqu'à **1 500 caractères**.

### Répondre, ou ne pas répondre

Trois issues, et elles n'ont pas le même effet :

| Ce que fait l'utilisateur | Ce qui se passe |
|---|---|
| Il répond | Les réponses sont transmises à l'agent, qui reprend son travail |
| **Il ferme la question sans répondre** | **L'agent s'arrête** — il ne poursuit pas la tâche |
| Il arrête la réponse en cours | L'appel est annulé comme n'importe quel autre outil |

Le deuxième cas mérite un encadré : fermer la question **n'équivaut pas à « choisis pour moi »**. C'est interprété comme « je ne veux pas continuer », et l'agent s'arrête là. Pour qu'il tranche seul, la bonne réponse est de sélectionner une option — celle qui est recommandée, en général.

Les réponses sont validées avant d'être transmises : une réponse incomplète, une option inconnue, un choix multiple sur une question qui n'en accepte qu'un, ou une réponse libre sans avoir choisi « Autre » sont refusés.

### Désactiver les questions

Couper le groupe **Choix utilisateur** dans les réglages a deux effets, et le second est peu évident :

1. l'outil disparaît — l'agent ne peut plus poser de question ;
2. **toute la section du prompt système qui décrit les choix interactifs est retirée**. Le modèle ne sait même pas que ce mode d'interaction existe.

Résultat : l'agent tranche seul systématiquement, et mentionne son choix dans sa réponse plutôt que de le soumettre.

C'est le bon réglage pour qui préfère un agent qui avance sans s'arrêter — au prix de décisions prises à sa place.

---

## Tableaux

### Les limites de format

| Élément | Limite |
|---|---|
| Questions en une fois | **1 à 5** |
| Options par question | **2 à 4** |
| Options recommandées par question | **Exactement 1** |
| Intitulé court | **30 caractères** |
| Texte de la question | **500 caractères** |
| Libellé d'une option | **80 caractères** |
| Description d'une option | **1 500 caractères** |
| Aperçu d'une option | **1 500 caractères** |
| Réponse libre | **1 500 caractères** |

### Les refus de format

| Message | Cause |
|---|---|
| `questions` doit contenir entre 1 et 5 éléments | Trop ou pas assez de questions |
| Chaque question doit avoir 2 à 4 options | Nombre d'options hors bornes |
| Chaque question doit avoir exactement une option recommandée | Zéro ou plusieurs recommandations |
| Réponse interactive incomplète | Une question sans réponse |
| Choix multiple non autorisé | Plusieurs réponses sur une question à choix unique |
| Choix inconnu | Option absente de celles proposées |
| Réponse autre invalide | Réponse libre sans avoir choisi « Autre », ou l'inverse |

---

## Encadrés

> **Fermer la question arrête l'agent.**
> Ce n'est pas « décide toi-même » : c'est « n'y va pas ». Pour laisser l'agent trancher, il faut choisir une option — la recommandée fait l'affaire.

> **Une seule option est recommandée, toujours.**
> L'agent est obligé d'avoir un avis. Il ne peut pas présenter quatre options sur le même plan et laisser l'utilisateur seul face au choix.

> **Une question signifie que la décision vous appartient.**
> L'agent a pour consigne de trancher seul dès qu'une option par défaut raisonnable existe. S'il demande, c'est que le choix ne se déduit pas du code.

> **Désactiver ce groupe rend l'agent autonome, pas silencieux.**
> Il continuera à annoncer ses décisions dans ses réponses ; il cessera simplement de les soumettre.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| « L'agent s'est arrêté après ma question fermée » | Fermer une question interactive arrête l'agent | Choisir une option, ou relancer avec un message |
| « L'agent ne me demande jamais rien » | Groupe Choix utilisateur désactivé, ou aucune décision réellement ouverte | Vérifier les réglages |
| « L'agent me demande trop souvent mon avis » | Le modèle applique mal la consigne de trancher par défaut | Désactiver le groupe, ou le lui demander dans le message |
| « Je voulais répondre autre chose que les options » | L'option « Autre » existe toujours | La choisir et écrire la réponse |

---

## Renvois

- `04-agent/permissions.md` — la différence entre une question interactive et une demande d'approbation
- `05-outils/vue-densemble.md` — l'effet de la désactivation sur le prompt système
- `04-agent/fonctionnement.md` — l'arrêt de l'agent
- `10-reglages/agent.md`

---

## Points à confirmer

- Je n'ai **pas vu la question à l'écran**. Le format est connu par le code — intitulé, question, options, recommandation, aperçu — mais la mise en page ne l'est pas : disposition côte à côte quand il y a un aperçu, enchaînement des questions quand il y en a plusieurs, apparence de la marque « recommandé ». **La page a besoin d'une capture** et d'une vérification avant publication.
- Le comportement quand **plusieurs questions** sont posées d'un coup n'est pas clair depuis le code : le champ « question courante » suggère un enchaînement une par une plutôt qu'un affichage groupé. À vérifier à l'écran.
- L'effet exact de l'arrêt après fermeture d'une question — l'agent s'arrête-t-il net, ou écrit-il une phrase de conclusion ? — demande un essai.
- Un second usage de ce mécanisme existe pour le **mode Plan**, chantier gelé. Ne pas le documenter maintenant.
