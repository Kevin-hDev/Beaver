export const ONBOARDING_COMPLETED_KEY = "onboarding_completed";

export function hasCompletedOnboarding(settings: Record<string, unknown> | null | undefined): boolean {
  return settings?.[ONBOARDING_COMPLETED_KEY] === true;
}

export function onboardingCompletedPatch(completed = true): Record<string, boolean> {
  return { [ONBOARDING_COMPLETED_KEY]: completed };
}

export function shouldReplayOnboarding(mode: string): boolean {
  return mode === "development";
}
