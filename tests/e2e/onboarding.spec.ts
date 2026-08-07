describe("first launch", () => {
  it("completes onboarding in the isolated desktop app", async () => {
    const welcome = await $('[data-e2e-step="welcome"]');
    await welcome.waitForDisplayed();
    await $('[data-e2e="onboarding-start"]').click();

    await $('[data-e2e-step="preferences"]').waitForDisplayed();
    await $('[data-e2e="preferences-continue"]').click();

    await $('[data-e2e-step="agent-import"]').waitForDisplayed();
    await $('[data-e2e="agent-import-continue"]').click();

    await $('[data-e2e-step="api"]').waitForDisplayed();
    await $('[data-e2e="api-skip"]').click();

    const ollamaSkip = await $('[data-e2e="ollama-skip"]');
    if (await ollamaSkip.isExisting()) await ollamaSkip.click();

    await $('[data-e2e="app-root"]').waitForDisplayed();
  });
});
