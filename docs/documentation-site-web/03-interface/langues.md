# Langues

**Emplacement site** — Référence › Thèmes et langues, ou Interface › Langues
**Répond à** — « Dans quelles langues est l'application, et comment je fais répondre l'agent dans ma langue ? »
**Sources** — `src/components/settings/general-settings-options.ts`, `src/i18n/` (sept fichiers de traduction), `src/components/onboarding/onboarding-preferences.tsx`
**Vérification** — Vérifié dans le code : les deux listes de langues et la clé de stockage

---

## Le point le plus important de cette page

**Il y a deux réglages de langue distincts, et ils ne font pas la même chose.**

- **La langue de l'interface** — les menus, les boutons, les messages de l'application.
- **La langue des réponses** — celle dans laquelle l'agent rédige ses réponses.

Changer l'un ne change pas l'autre. Quelqu'un qui met l'interface en français et se demande pourquoi l'agent répond en anglais cherche le second réglage. C'est la question de support la plus prévisible de toute cette section.

---

## Plan de page proposé

1. La langue de l'interface
2. La langue des réponses de l'agent
3. Différence entre les deux
4. Ce qui n'est pas traduit

---

## Contenu

### 1. La langue de l'interface

**Sept langues.** Liste en section Tableaux.

- Le changement est immédiat, sans redémarrage.
- Le choix est mémorisé côté application, sous la clé `clgo-language`.
- Il est proposé pendant le parcours d'accueil, et se modifie ensuite dans Réglages › Général.

### 2. La langue des réponses de l'agent

- **Les sept mêmes langues**, plus une option vide affichée « — ».
- L'option vide signifie **aucune consigne** : l'agent répond dans la langue qui lui semble appropriée, généralement celle de votre message.
- Quand une langue est choisie, la consigne est transmise au modèle dans ses instructions.

À préciser : c'est une **consigne**, pas une garantie. Un modèle local de petite taille peut l'ignorer, surtout sur des réponses longues ou techniques.

### 3. Différence entre les deux

Voir le tableau comparatif en section Tableaux. C'est l'élément à mettre en avant.

### 4. Ce qui n'est pas traduit

À vérifier et compléter — voir *Points à confirmer*. Ce qui est certain par nature :

- les réponses des modèles, qui dépendent du modèle et de la consigne de langue ;
- les messages d'erreur renvoyés par les fournisseurs distants ;
- les noms de modèles, de fournisseurs et de connecteurs ;
- le contenu de vos propres fichiers d'instructions et de mémoire.

---

## Tableaux

### Tableau — Les sept langues

| Langue | Interface | Réponses de l'agent |
|---|---|---|
| Français | Oui | Oui |
| Anglais | Oui | Oui |
| Allemand | Oui | Oui |
| Espagnol | Oui | Oui |
| Italien | Oui | Oui |
| Chinois | Oui | Oui |
| Japonais | Oui | Oui |
| *Aucune consigne* | — | Oui, option « — » |

### Tableau — Les deux réglages

| | Langue de l'interface | Langue des réponses |
|---|---|---|
| Ce que ça change | Menus, boutons, messages de l'application | La langue dans laquelle l'agent écrit |
| Effet | Immédiat et garanti | Consigne transmise au modèle, non garantie |
| Option « aucune » | Non | Oui |
| Emplacement | Réglages › Général | Réglages › Général |

---

## Encadrés

**Encadré « Deux réglages, deux effets »** — à placer en tête.
> La langue de l'interface et la langue des réponses de l'agent se règlent séparément. Mettre l'interface en français ne fait pas répondre l'agent en français : choisissez aussi la langue des réponses.

**Encadré « Une consigne, pas une garantie »**
> La langue des réponses est transmise au modèle sous forme d'instruction. Les modèles les plus petits peuvent s'en écarter, surtout sur des réponses longues.

---

## Pièges et erreurs fréquentes

| Symptôme | Cause | Résolution |
|---|---|---|
| L'interface est en français, l'agent répond en anglais | Deux réglages distincts | Régler aussi la langue des réponses |
| L'agent dérive vers l'anglais en cours de conversation | Le modèle s'écarte de la consigne | Essayer un modèle plus grand, ou le rappeler dans le message |
| Un texte reste en anglais dans l'interface | Élément non traduit ou provenant d'un service externe | Voir la section « Ce qui n'est pas traduit » |
| La langue revient à l'anglais après réinstallation | Le choix est stocké côté application | Le refaire dans les réglages |

---

## Renvois

- *Thèmes et apparence* — l'autre moitié de Réglages › Général
- *Parcours d'accueil* — le choix initial
- *Agent › Prompts système* — où s'insère la consigne de langue

---

## Points à confirmer

- **Où s'applique exactement la consigne de langue des réponses.** Vérifier si elle est injectée dans le prompt système, dans chaque message, ou ailleurs — cela change ce qu'on peut promettre à l'utilisateur.
- **La complétude des traductions.** Sept fichiers existent, de tailles nettement différentes : le chinois fait 99 Ko, le japonais 125 Ko, le français 113 Ko. L'écart peut refléter la densité des langues, ou des traductions incomplètes. À vérifier avant d'affirmer que l'interface est intégralement traduite.
- **Ce qui reste en anglais dans l'interface.** Établir la liste réelle plutôt que de la déduire.
- **Le stockage du choix de langue.** La clé `clgo-language` semble vivre côté navigateur et non dans le dossier de données. Confirmer, et en déduire si le réglage survit à une réinstallation.
- **La langue par défaut au premier lancement.** Détectée depuis le système, ou anglais d'office ? Non vérifié.
