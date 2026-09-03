export async function completeOnboarding(): Promise<void> {
  const welcome = $('[data-e2e-step="welcome"]');
  const appRoot = $('[data-e2e="app-root"]');
  await browser.waitUntil(async () => await welcome.isExisting() || await appRoot.isExisting(), {
    timeoutMsg: "Neither onboarding nor the application root became ready",
  });
  if (await appRoot.isExisting()) {
    await appRoot.waitForDisplayed();
    return;
  }
  await welcome.waitForDisplayed();
  await $('[data-e2e="onboarding-start"]').click();

  await $('[data-e2e-step="preferences"]').waitForDisplayed();
  await $('[data-e2e="preferences-continue"]').click();

  await $('[data-e2e-step="agent-import"]').waitForDisplayed();
  await $('[data-e2e="agent-import-continue"]').click();

  await $('[data-e2e-step="api"]').waitForDisplayed();
  await $('[data-e2e="api-skip"]').click();

  const ollamaSkip = $('[data-e2e="ollama-skip"]');
  if (await ollamaSkip.isExisting()) await ollamaSkip.click();

  await appRoot.waitForDisplayed();
}
