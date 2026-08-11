import { completeOnboarding } from "./onboarding-flow";

describe("first launch", () => {
  it("completes onboarding in the isolated desktop app", async () => {
    await completeOnboarding();
  });
});
