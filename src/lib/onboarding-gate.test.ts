import { describe, expect, it } from "vitest";
import {
  hasCompletedOnboarding,
  onboardingCompletedPatch,
  ONBOARDING_COMPLETED_KEY,
  shouldReplayOnboarding,
} from "./onboarding-gate";

describe("onboarding gate", () => {
  it("detecte uniquement le booleen true", () => {
    expect(hasCompletedOnboarding({ [ONBOARDING_COMPLETED_KEY]: true })).toBe(true);
    expect(hasCompletedOnboarding({ [ONBOARDING_COMPLETED_KEY]: "true" })).toBe(false);
    expect(hasCompletedOnboarding(null)).toBe(false);
  });

  it("genere le patch de completion", () => {
    expect(onboardingCompletedPatch()).toEqual({ [ONBOARDING_COMPLETED_KEY]: true });
    expect(onboardingCompletedPatch(false)).toEqual({ [ONBOARDING_COMPLETED_KEY]: false });
  });

  it("rejoue l'onboarding uniquement en mode developpement", () => {
    expect(shouldReplayOnboarding("development")).toBe(true);
    expect(shouldReplayOnboarding("test")).toBe(false);
    expect(shouldReplayOnboarding("production")).toBe(false);
  });
});
