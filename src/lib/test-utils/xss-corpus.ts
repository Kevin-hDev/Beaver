/**
 * Corpus de vecteurs XSS connus (inspiré OWASP / cheat sheets publiques) pour
 * vérifier que le pipeline de rendu de Beaver ne laisse passer aucune forme
 * d'exécution. Chaque payload est classé : "html" (HTML brut dans le markdown)
 * ou "markdown" (syntaxe markdown piégée).
 */

export interface XssPayload {
  name: string;
  input: string;
}

export const XSS_PAYLOADS: XssPayload[] = [
  // --- Balises d'exécution directes ---
  { name: "script simple", input: "<script>alert(1)</script>" },
  { name: "script majuscules", input: "<ScRiPt>alert(1)</ScRiPt>" },
  { name: "script avec espaces", input: "<script\n>alert(1)</script\n>" },
  { name: "svg onload", input: "<svg onload=alert(1)>" },
  { name: "svg avec script", input: "<svg><script>alert(1)</script></svg>" },
  { name: "img onerror", input: '<img src=x onerror=alert(1)>' },
  { name: "img onerror entre quotes", input: '<img src="x" onerror="alert(1)">' },
  { name: "img onerror backticks", input: "<img src=x onerror=`alert(1)`>" },
  { name: "img onerror sans espace", input: "<img/src=x/onerror=alert(1)>" },
  { name: "body onload", input: "<body onload=alert(1)>" },
  { name: "input autofocus", input: "<input onfocus=alert(1) autofocus>" },
  { name: "details ontoggle", input: "<details open ontoggle=alert(1)>" },
  { name: "marquee onstart", input: "<marquee onstart=alert(1)>x</marquee>" },
  { name: "video source onerror", input: "<video><source onerror=alert(1)></video>" },
  { name: "audio onerror", input: '<audio src=x onerror=alert(1)>' },
  { name: "iframe javascript", input: '<iframe src="javascript:alert(1)"></iframe>' },
  { name: "iframe data html", input: '<iframe src="data:text/html,<script>alert(1)</script>"></iframe>' },
  { name: "object javascript", input: '<object data="javascript:alert(1)"></object>' },
  { name: "embed data html", input: '<embed src="data:text/html,<script>alert(1)</script>">' },
  { name: "form formaction", input: '<form><button formaction="javascript:alert(1)">x</button></form>' },
  { name: "base href javascript", input: '<base href="javascript:alert(1)//">' },
  { name: "meta refresh", input: '<meta http-equiv="refresh" content="0;url=javascript:alert(1)">' },
  { name: "math mtext script", input: "<math><mtext><script>alert(1)</script></mtext></math>" },
  { name: "noscript script", input: "<noscript><script>alert(1)</script></noscript>" },
  { name: "template script", input: "<template><script>alert(1)</script></template>" },

  // --- Protocoles piégés dans les liens HTML ---
  { name: "a javascript", input: '<a href="javascript:alert(1)">x</a>' },
  { name: "a javascript majuscules", input: '<a href="JaVaScRiPt:alert(1)">x</a>' },
  { name: "a javascript tabulation", input: '<a href="java\tscript:alert(1)">x</a>' },
  { name: "a javascript saut de ligne", input: '<a href="java\nscript:alert(1)">x</a>' },
  { name: "a entité deux-points", input: '<a href="javascript&#58;alert(1)">x</a>' },
  { name: "a entité &colon;", input: '<a href="javascript&colon;alert(1)">x</a>' },
  { name: "a encodage entités complet", input: '<a href="&#106;&#97;&#118;&#97;&#115;&#99;&#114;&#105;&#112;&#116;&#58;alert(1)">x</a>' },
  { name: "a vbscript", input: '<a href="vbscript:msgbox(1)">x</a>' },
  { name: "a data html", input: '<a href="data:text/html,<script>alert(1)</script>">x</a>' },
  { name: "a file", input: '<a href="file:///etc/passwd">x</a>' },

  // --- Markdown piégé ---
  { name: "md lien javascript", input: "[x](javascript:alert(1))" },
  { name: "md lien javascript majuscules", input: "[x](JaVaScRiPt:alert(1))" },
  { name: "md lien data html", input: "[x](data:text/html,<script>alert(1)</script>)" },
  { name: "md lien vbscript", input: "[x](vbscript:msgbox(1))" },
  { name: "md image onerror dans titre", input: '![x](https://a.test/i.png "t" onerror=alert(1) x="")' },
  { name: "md référence javascript", input: "[x][1]\n\n[1]: javascript:alert(1)" },

  // --- Contournements structurels (mXSS et mutations) ---
  { name: "svg style img", input: "<svg><style><img src=x onerror=alert(1)></style></svg>" },
  { name: "commentaire imbriqué", input: "<!-- <img src=x onerror=alert(1)> -->" },
  { name: "cdata piégé", input: "<svg><![CDATA[<script>alert(1)</script>]]></svg>" },
  { name: "style expression", input: "<style>*{x:expression(alert(1))}</style>" },
  { name: "table background", input: '<table background="javascript:alert(1)"></table>' },
];

/** Balises qui ne doivent JAMAIS apparaître dans le rendu, quel que soit le payload. */
export const FORBIDDEN_TAGS = [
  "script", "iframe", "object", "embed", "form", "meta", "base", "link",
  "style", "textarea", "select", "video", "audio",
  "marquee", "template", "noscript",
];

/**
 * Balises que le schéma de sanitize autorise APRÈS purge de leurs attributs
 * (vérifié dans hast-util-sanitize : details/summary sans aucun attribut,
 * input uniquement disabled + type=checkbox pour les listes de tâches GFM).
 * Elles sont inertes : aucun attribut, donc aucun gestionnaire possible.
 */
const INERT_TAG_PROFILES: Record<string, (el: Element) => boolean> = {
  /* Le sanitize ne garde que "open" (booléen visuel) ; tout gestionnaire
     on* est déjà intercepté par la règle générale ci-dessous. */
  details: (el) =>
    Array.from(el.attributes).every((a) => a.name === "open"),
  summary: (el) => el.attributes.length === 0,
  input: (el) =>
    el.attributes.length === 2
    && el.getAttribute("type") === "checkbox"
    && el.hasAttribute("disabled"),
};

/** Protocoles interdits en début d'attribut d'URL (espaces et casse ignorés). */
const FORBIDDEN_PROTOCOL = /^\s*(javascript|vbscript|data|file)\s*:/i;

/**
 * Fouille un conteneur rendu et renvoie la liste des violations trouvées :
 * balise interdite, attribut on*, ou URL à protocole dangereux.
 * Une liste vide = le pipeline a tenu.
 */
export function findViolations(container: HTMLElement): string[] {
  const violations: string[] = [];
  for (const el of Array.from(container.querySelectorAll("*"))) {
    const tag = el.tagName.toLowerCase();
    if (FORBIDDEN_TAGS.includes(tag)) {
      violations.push(`balise interdite <${tag}>`);
    }
    /* Les balises "inertes" ne le sont que si leur profil d'attributs est
       exactement celui, purgé, que le sanitize garantit. */
    const inertProfile = INERT_TAG_PROFILES[tag];
    if (inertProfile && !inertProfile(el)) {
      violations.push(`<${tag}> avec des attributs inattendus`);
    }
    for (const attr of Array.from(el.attributes)) {
      if (attr.name.toLowerCase().startsWith("on")) {
        violations.push(`attribut ${attr.name} sur <${tag}>`);
      }
      if (
        (attr.name === "href" || attr.name === "src" || attr.name === "xlink:href" || attr.name === "formaction")
        && FORBIDDEN_PROTOCOL.test(attr.value)
        // data:image dans un <img> est autorisé par le CSP et sans exécution
        && !(tag === "img" && /^data:image\//i.test(attr.value))
      ) {
        violations.push(`protocole dangereux ${attr.name}="${attr.value.slice(0, 40)}" sur <${tag}>`);
      }
    }
  }
  return violations;
}
