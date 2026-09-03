import { Component, useEffect, useLayoutEffect, useRef, useState, type ReactNode } from "react";
import { useTranslation } from "react-i18next";
import { useStandardCatalog } from "./catalog-context";
import type { MountPermit } from "./mount-coordinator";
import type { StandardCatalogEntry } from "./types";

export function StandardContributionBoundary({
  entry,
  children,
}: {
  entry: StandardCatalogEntry;
  children: ReactNode;
}) {
  const catalog = useStandardCatalog();
  const resetKey = `${catalog.snapshot?.revision ?? 0}:${entry.extensionId}:${entry.contributionId}`;
  return (
    <RenderBoundary
      resetKey={resetKey}
      onFailure={() => catalog.reportMountFailure(entry)}
      onOpen={() => catalog.openExtension(entry.extensionId)}
    >
      <MountGate key={resetKey} entry={entry}>{children}</MountGate>
    </RenderBoundary>
  );
}

function MountGate({ entry, children }: { entry: StandardCatalogEntry; children: ReactNode }) {
  const { openExtension, prepareMount } = useStandardCatalog();
  const [permit, setPermit] = useState<MountPermit | null>(null);
  const [failed, setFailed] = useState(false);
  const permitRef = useRef<MountPermit | null>(null);
  const committed = useRef(false);

  useEffect(() => {
    let active = true;
    void prepareMount(entry).then((next) => {
      if (active) {
        permitRef.current = next;
        setPermit(next);
      }
      else next.cancel();
    }).catch(() => { if (active) setFailed(true); });
    return () => {
      active = false;
      if (!committed.current) permitRef.current?.cancel();
    };
  }, [entry, prepareMount]);

  useLayoutEffect(() => {
    if (!permit || committed.current) return;
    committed.current = true;
    void permit.commit().catch(() => setFailed(true));
  }, [permit]);

  if (failed) return <ContributionFallback onOpen={() => openExtension(entry.extensionId)} />;
  return permit ? children : null;
}

interface BoundaryProps {
  resetKey: string;
  onFailure: () => void;
  onOpen: () => void;
  children: ReactNode;
}

class RenderBoundary extends Component<BoundaryProps, { failed: boolean; resetKey: string }> {
  state = { failed: false, resetKey: this.props.resetKey };

  static getDerivedStateFromError() {
    return { failed: true };
  }

  static getDerivedStateFromProps(props: BoundaryProps, state: { failed: boolean; resetKey: string }) {
    return props.resetKey === state.resetKey ? null : { failed: false, resetKey: props.resetKey };
  }

  componentDidCatch() {
    this.props.onFailure();
  }

  render() {
    return this.state.failed
      ? <ContributionFallback onOpen={this.props.onOpen} />
      : this.props.children;
  }
}

function ContributionFallback({ onOpen }: { onOpen: () => void }) {
  const { t } = useTranslation();
  return (
    <div className="xui-error" role="alert">
      <span>{t("extensions.errors.view")}</span>
      <button type="button" className="btn btn-sm btn-secondary" onClick={onOpen}>
        {t("extensions.recovery.openDetail")}
      </button>
    </div>
  );
}
