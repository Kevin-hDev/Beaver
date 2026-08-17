import { open } from "@tauri-apps/plugin-shell";

const ALLOWED_PROTOCOLS = new Set(["http:", "https:", "mailto:"]);
const MAX_LINK_LENGTH = 2048;

/**
 * Ouvre un lien externe dans le navigateur du système, uniquement si son
 * protocole est inoffensif. Tout contenu affiché dans le chat peut venir
 * d'un LLM, d'une page web ou d'une sortie de commande : un href n'est
 * jamais une valeur de confiance. Le nettoyage markdown retire déjà les
 * protocoles dangereux du DOM ; cette fonction est la seconde porte, et la
 * seule autorité sur la liste des protocoles autorisés.
 */
export function openExternalLink(href: string): boolean {
  if (!href || href.length > MAX_LINK_LENGTH) return false;
  try {
    const url = new URL(href);
    if (!ALLOWED_PROTOCOLS.has(url.protocol)) return false;
    void open(url.toString()).catch(() => undefined);
    return true;
  } catch {
    /* Les liens invalides ou relatifs restent inertes dans le chat desktop. */
    return false;
  }
}
