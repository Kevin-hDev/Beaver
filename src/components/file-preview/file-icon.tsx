import fileDefault from "@/assets/helium-icons/file.svg?url";
import iconC from "@/assets/helium-icons/c.svg?url";
import iconCpp from "@/assets/helium-icons/cpp.svg?url";
import iconCss from "@/assets/helium-icons/css.svg?url";
import iconGo from "@/assets/helium-icons/go.svg?url";
import iconH from "@/assets/helium-icons/h.svg?url";
import iconHtml from "@/assets/helium-icons/html.svg?url";
import iconJava from "@/assets/helium-icons/java.svg?url";
import iconJs from "@/assets/helium-icons/javascript.svg?url";
import iconJson from "@/assets/helium-icons/json.svg?url";
import iconMd from "@/assets/helium-icons/markdown.svg?url";
import iconPy from "@/assets/helium-icons/python.svg?url";
import iconReact from "@/assets/helium-icons/react.svg?url";
import iconReactTs from "@/assets/helium-icons/react_ts.svg?url";
import iconRust from "@/assets/helium-icons/rust.svg?url";
import iconShell from "@/assets/helium-icons/console.svg?url";
import iconSql from "@/assets/helium-icons/database.svg?url";
import iconToml from "@/assets/helium-icons/settings.svg?url";
import iconTs from "@/assets/helium-icons/typescript.svg?url";
import iconYaml from "@/assets/helium-icons/yaml.svg?url";
import iconTable from "@/assets/helium-icons/table.svg?url";
import iconWord from "@/assets/helium-icons/word.svg?url";
import iconPdf from "@/assets/helium-icons/pdf.svg?url";
import iconImage from "@/assets/helium-icons/image.svg?url";
import iconSvg from "@/assets/helium-icons/svg.svg?url";
import iconXml from "@/assets/helium-icons/xml.svg?url";
import iconLock from "@/assets/helium-icons/lock.svg?url";
import iconLog from "@/assets/helium-icons/log.svg?url";
import iconZip from "@/assets/helium-icons/zip.svg?url";
import iconPs1 from "@/assets/helium-icons/powershell.svg?url";
import iconDocker from "@/assets/helium-icons/docker.svg?url";
import iconGit from "@/assets/helium-icons/git.svg?url";
import iconDoc from "@/assets/helium-icons/document.svg?url";
import iconEslint from "@/assets/helium-icons/eslint.svg?url";

const EXT_ICONS: Record<string, string> = {
  c: iconC,
  cpp: iconCpp,
  css: iconCss,
  go: iconGo,
  h: iconH,
  html: iconHtml,
  java: iconJava,
  js: iconJs,
  jsx: iconReact,
  json: iconJson,
  md: iconMd,
  py: iconPy,
  rs: iconRust,
  sh: iconShell,
  bash: iconShell,
  zsh: iconShell,
  sql: iconSql,
  toml: iconToml,
  ts: iconTs,
  tsx: iconReactTs,
  yaml: iconYaml,
  yml: iconYaml,
  xlsx: iconTable,
  xls: iconTable,
  xlsm: iconTable,
  csv: iconTable,
  ods: iconTable,
  tsv: iconTable,
  docx: iconWord,
  doc: iconWord,
  pdf: iconPdf,
  png: iconImage,
  jpg: iconImage,
  jpeg: iconImage,
  gif: iconImage,
  webp: iconImage,
  ico: iconImage,
  svg: iconSvg,
  xml: iconXml,
  lock: iconLock,
  log: iconLog,
  zip: iconZip,
  tar: iconZip,
  gz: iconZip,
  ps1: iconPs1,
  txt: iconDoc,
  gitignore: iconGit,
};

const NAME_ICONS: Record<string, string> = {
  Dockerfile: iconDocker,
  "docker-compose.yml": iconDocker,
  "docker-compose.yaml": iconDocker,
  ".eslintrc": iconEslint,
  ".eslintrc.js": iconEslint,
  ".eslintrc.json": iconEslint,
  "eslint.config.js": iconEslint,
  "eslint.config.ts": iconEslint,
  ".gitignore": iconGit,
  ".gitattributes": iconGit,
};

export function FileIcon({ name, size = 18 }: { name: string; size?: number | string }) {
  const ext = name.split(".").pop()?.toLowerCase() ?? "";
  const src = NAME_ICONS[name] ?? EXT_ICONS[ext] ?? fileDefault;
  return (
    <img
      className="fp-icon"
      src={src}
      alt=""
      style={{ width: size, height: size, flexShrink: 0 }}
    />
  );
}
