# Suppression réelle des paramètres Ollama vides

## Problème

L’éditeur de paramètres retire correctement les champs vides du payload envoyé au backend. Le backend recrée toutefois le modèle en utilisant ce même modèle comme source. Ollama réhérite alors des paramètres absents de la requête.

Conséquence : une valeur renseignée peut être remplacée, mais elle ne peut pas être supprimée. Après sauvegarde, l’ancienne valeur réapparaît dans l’interface.

Ce comportement concerne tous les paramètres : `num_ctx`, les paramètres numériques, les séquences `stop` et les paramètres personnalisés.

## Contrat utilisateur

- Un champ rempli représente une personnalisation explicite.
- Un champ vide ne produit aucune directive `PARAMETER` correspondante.
- Après sauvegarde et rechargement, un champ supprimé reste visuellement vide.
- Ollama reste libre d’appliquer sa valeur interne par défaut pendant l’exécution.
- Une sauvegarde de paramètres ne doit modifier aucune autre directive du Modelfile.

## Approche retenue

Beaver utilisera le Modelfile complet renvoyé par Ollama comme base locale de reconstruction.

Le backend :

1. charge le Modelfile courant ;
2. conserve sa directive `FROM` réelle au lieu de la remplacer par le nom du modèle ;
3. retire uniquement les directives `PARAMETER` de premier niveau ;
4. ajoute uniquement les paramètres non vides transmis par l’éditeur ;
5. reconstruit le modèle avec la commande Ollama existante et ses arguments séparés.

La conservation du `FROM` réel empêche l’auto-héritage depuis le modèle personnalisé. La transformation textuelle préserve les directives que le parseur structuré actuel ne connaît pas, notamment `ADAPTER`, `MESSAGE`, `RENDERER` et `PARSER`.

## Transformation sûre du Modelfile

Une fonction pure et dédiée transformera le texte. Elle respectera les blocs multilignes délimités par des triples guillemets afin qu’une ligne commençant par `PARAMETER` à l’intérieur d’un prompt ou d’un template ne soit jamais supprimée.

Les règles sont :

- reconnaître les directives sans tenir compte de la casse ni de l’indentation ;
- retirer seulement les directives `PARAMETER` situées au niveau principal ;
- préserver toutes les autres lignes et leur ordre ;
- ajouter les nouveaux paramètres dans un bloc déterministe ;
- conserver les valeurs multiples, notamment les séquences `stop` ;
- refuser les entrées invalides avant toute reconstruction ;
- ne jamais écrire de contenu brut du Modelfile dans les journaux.

## Gestion des erreurs

La validation existante reste la première barrière. Si la lecture, la transformation ou la reconstruction échoue, l’opération échoue complètement et le marqueur de personnalisation précédent est restauré.

L’événement `modelfile-updated` n’est émis qu’après une reconstruction réussie. L’interface continue alors à recharger le Modelfile depuis Ollama, qui reste la source de vérité.

## Tests

Les tests seront écrits avant la correction et devront échouer avec l’implémentation actuelle.

Ils couvriront :

- définir puis supprimer `num_ctx` ;
- supprimer un paramètre numérique différent de `num_ctx` ;
- supprimer toutes les séquences `stop` ;
- conserver un mélange de paramètres remplis et vides ;
- conserver les paramètres personnalisés non vides ;
- préserver `FROM`, `SYSTEM`, `TEMPLATE`, `LICENSE`, `ADAPTER`, `MESSAGE`, `RENDERER`, `PARSER` et les commentaires ;
- préserver une ligne ressemblant à `PARAMETER` dans un bloc multiligne ;
- ne jamais remplacer `FROM` par le nom du modèle édité ;
- recharger un champ vide après la sauvegarde ;
- conserver les limites existantes sur le nombre et la taille des paramètres.

## Périmètre

Cette correction appartient à `codex/fix-ollama-context-diagnostics`, car elle est nécessaire pour rétablir proprement le contexte automatique après les tests Gemma 4.

Elle ne modifie pas la branche `codex/fix-windows-directoryless-streams`, dont les corrections de review restent à traiter séparément lorsque les retours seront fournis.

Le problème de fermeture en arrière-plan reste également un chantier distinct.
