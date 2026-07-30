import { SettingsDetailHeader } from "@/components/settings/shell/settings-detail-header";
import type { RegistryModel } from "@/types/agent";
import { ModelProfile } from "./model-profile";
import { ModelSearch } from "./model-search";
import { ModelVariantsList } from "./model-variants-list";

export interface OllamaSearchState {
  query: string;
  setQuery: (query: string) => void;
  results: RegistryModel[];
  setResults: (list: RegistryModel[]) => void;
  searching: boolean;
  setSearching: (searching: boolean) => void;
}

interface OllamaModelsViewProps {
  search: OllamaSearchState;
  family: string | null;
  variant: string | null;
  onSelectFamily: (family: string | null) => void;
  onSelectVariant: (variant: string | null) => void;
}

export function OllamaModelsView({
  search,
  family,
  variant,
  onSelectFamily,
  onSelectVariant,
}: OllamaModelsViewProps) {
  if (family && variant) {
    return (
      <>
        <SettingsDetailHeader
          title={variant}
          subtitle={family}
          onBack={() => onSelectVariant(null)}
        />
        <ModelProfile familyName={family} variantFullName={variant} />
      </>
    );
  }

  if (family) {
    return (
      <>
        <SettingsDetailHeader title={family} onBack={() => onSelectFamily(null)} />
        <ModelVariantsList familyName={family} onSelectVariant={onSelectVariant} />
      </>
    );
  }

  return <ModelSearch {...search} onSelectFamily={onSelectFamily} />;
}
