import { useTranslation } from "react-i18next";
import type { MissingSessionDirectory } from "@/hooks/use-agent-missing-directory";
import "./missing-directory-prompt.css";

interface Props {
  directory: MissingSessionDirectory;
  resolving: boolean;
  onResolve: (action: "switch" | "create") => void;
}

export function MissingDirectoryPrompt({ directory, resolving, onResolve }: Props) {
  const { t } = useTranslation();
  return (
    <div className="mdp-root" role="alert">
      <span className="mdp-copy">
        <span className="mdp-title">{t("missingDirectory.title")}</span>
        <span className="mdp-path" title={directory.missing_path}>{directory.missing_path}</span>
        <span className="mdp-parent">
          {t("missingDirectory.parent", { path: directory.nearest_parent })}
        </span>
      </span>
      <span className="mdp-actions">
        <button type="button" disabled={resolving} onClick={() => onResolve("switch")}>
          {t("missingDirectory.switch")}
        </button>
        <button type="button" disabled={resolving} onClick={() => onResolve("create")}>
          {t("missingDirectory.create")}
        </button>
      </span>
    </div>
  );
}
