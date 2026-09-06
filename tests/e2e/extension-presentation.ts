// Shared presentation setup for native extension acceptance tests.
export async function setExtensionPresentation(locale: string, theme: string): Promise<void> {
  await browser.execute((nextLocale, nextTheme) => {
    window.localStorage.setItem("clgo-language", nextLocale);
    window.localStorage.setItem("clgo-theme", nextTheme);
  }, locale, theme);
  await browser.refresh();
  await $('[data-e2e="app-root"]').waitForDisplayed();
  await browser.waitUntil(async () => (
    await browser.execute(() => document.documentElement.dataset.palette)
  ) === theme, { timeoutMsg: `Theme ${theme} was not applied` });
}
