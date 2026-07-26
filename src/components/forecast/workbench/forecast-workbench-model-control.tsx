import { useForecastSelectionPolicy } from "../model-selection/use-forecast-selection-policy";
import { ForecastModelSelector } from "../widgets/forecast-model-selector";

export function ForecastWorkbenchModelControl() {
  const { policy, selectedModelId, selectModel, setMode, setCloudAllowed, ready } =
    useForecastSelectionPolicy();
  return (
    <ForecastModelSelector
      selectedModelId={selectedModelId}
      selectionMode={policy.mode}
      allowCloudInAuto={policy.allow_cloud_in_auto}
      selectionReady={ready}
      onSelectModel={selectModel}
      onModeChange={setMode}
      onCloudAllowedChange={setCloudAllowed}
      align="right"
    />
  );
}
