export interface PersonalityFile {
  name: string;
  path: string;
  /* Clé de traduction, pas un libellé : le backend nomme la description, c'est
     l'interface qui la met dans la langue de l'utilisateur. */
  description_key: string;
}
