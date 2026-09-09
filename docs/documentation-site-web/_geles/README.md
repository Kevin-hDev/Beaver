# Chantiers gelés

Briefs mis de côté parce que la fonctionnalité correspondante n'est pas finalisée.

**Ne pas les publier.** Documenter un comportement qui va changer produit une documentation fausse le jour de la sortie, et personne ne pense à la relire.

Les fichiers portent l'extension `.gele` pour qu'aucun outil de génération ne les ramasse par erreur.

---

## Mode Plan — `plan-mode.md.gele`

**Raison du gel** : le mode Plan doit être modifié. Dans son état actuel, il n'accorde pas assez d'autorisations au modèle : trop de choses sont bloquées pour qu'une exploration se déroule correctement.

**Ce que contient le brief** : le fonctionnement observé au moment de la rédaction — 19 outils autorisés, 3 conditionnels, les 7 états du parcours, les limites, le mécanisme de correction automatique.

**À refaire après modification** : la liste des outils autorisés est le cœur du sujet et va changer. Le reste du brief — parcours, limites, articulation avec les modes de permission — restera probablement valable.

**Ce qui reste vrai quoi qu'il arrive** : le mode Plan protège même en Accès complet, les deux mécanismes se cumulent. C'est l'argument à conserver.

---

## Extensions — quatre briefs non écrits

**Raison du gel** : l'implémentation a été interrompue en cours de route et doit être finalisée.

**Briefs prévus** :

- `extensions-centre.md` — découverte, catalogue, installation, mise à jour, désactivation, suppression
- `extensions-remplacer-un-outil.md` — substitution d'un outil natif, masquage, priorité, diagnostics
- `extensions-prompt-systeme.md` — réécriture du prompt système par une extension, portée, précédence
- `extensions-ecrire.md` — structure, hôte, canal de communication, source Git, limites et sécurité

**Niveau d'exigence attendu** : le plus élevé du site. Installer du code tiers qui remplace des outils et réécrit le prompt système engage la sécurité de l'utilisateur. Il devra être guidé pas à pas, avec les risques énoncés explicitement plutôt que mentionnés en note de bas de page.

---

## Comment reprendre un chantier gelé

1. Vérifier que la fonctionnalité est finalisée.
2. Relire le brief gelé : ce qui reste vrai, ce qui a changé.
3. **Relire le code**, sans se fier au brief — c'est précisément parce que le code allait changer qu'on a gelé.
4. Réécrire, retirer l'extension `.gele`, remettre le fichier à sa place et cocher la case dans `differents-points-a-traiter.md`.

---

## Compression du contexte — brief non écrit

**Raison du gel** : la compression va être revue.

**Brief prévu** : `04-agent/compression.md` — quand la compression se déclenche, ce qu'elle résume, ce qu'elle conserve, ce que l'utilisateur voit, et ce qu'il peut régler.

**À noter** : le brief `04-agent/contexte.md` reste rédigeable et couvre le budget de contexte, l'élagage et l'écran d'usage. Il ne doit pas décrire la compression, seulement y renvoyer.
