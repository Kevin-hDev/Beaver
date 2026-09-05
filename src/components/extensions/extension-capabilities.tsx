import { useId } from "react";
import { useTranslation } from "react-i18next";
import { SettingsCard } from "@/components/settings/settings-card";
import type { ExtensionRecord } from "@/types/extensions";
import "./extension-capabilities.css";

interface ExtensionCapabilitiesProps {
  extension: ExtensionRecord;
}

export function ExtensionCapabilities({ extension }: ExtensionCapabilitiesProps) {
  const { t } = useTranslation();
  const skills = extension.contributions?.skills ?? [];
  const resources = extension.contributions?.resources ?? [];
  const hasContent = skills.length > 0 || resources.length > 0;
  const titleId = useId();

  return (
    <section className="extcap-root" aria-labelledby={titleId}>
      <h3 id={titleId}>{t("extensions.detail.capabilities")}</h3>
      {hasContent ? (
        <SettingsCard className="extcap-card">
          {skills.length > 0 && (
            <CapabilityGroup title={t("extensions.detail.skills")}>
              {skills.map((skill) => (
                <CapabilityRow
                  key={skill.id}
                  name={skill.name}
                  description={skill.description}
                />
              ))}
            </CapabilityGroup>
          )}
          {resources.length > 0 && (
            <CapabilityGroup title={t("extensions.detail.resources")}>
              {resources.map((resource) => (
                <CapabilityRow
                  key={resource.id}
                  name={resource.name}
                  description={resource.description}
                  kind={t(`extensions.detail.resourceTypes.${resource.type}`)}
                />
              ))}
            </CapabilityGroup>
          )}
        </SettingsCard>
      ) : (
        <CapabilityState extension={extension} />
      )}
    </section>
  );
}

function CapabilityGroup({ title, children }: React.PropsWithChildren<{ title: string }>) {
  return (
    <div className="extcap-group">
      <h4>{title}</h4>
      <div className="extcap-list">{children}</div>
    </div>
  );
}

function CapabilityRow({ name, description, kind }: { name: string; description: string; kind?: string }) {
  return (
    <div className="extcap-row">
      <div className="extcap-row-heading">
        <span>{name}</span>
        {kind && <span className="extcap-kind">{kind}</span>}
      </div>
      <p>{description}</p>
    </div>
  );
}

function CapabilityState({ extension }: ExtensionCapabilitiesProps) {
  const { t } = useTranslation();
  if (extension.status === "loading") {
    return <p className="extcap-state" role="status">{t("extensions.detail.capabilitiesLoading")}</p>;
  }
  if (extension.status === "error" || extension.status === "incompatible") {
    return <p className="extcap-state extcap-error" role="alert">{t("extensions.detail.capabilitiesError")}</p>;
  }
  return <p className="extcap-state">{t("extensions.detail.capabilitiesEmpty")}</p>;
}
