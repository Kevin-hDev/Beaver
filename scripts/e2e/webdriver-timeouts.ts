import {
  EXTENSION_HOST_SETUP_TIMEOUT_MS,
  WEBDRIVER_IMPLICIT_TIMEOUT_MS,
  WEBDRIVER_PAGE_LOAD_TIMEOUT_MS,
} from "./extension-setup-deadline";

type Timeouts = { script: number; implicit: number; pageLoad: number };

export async function applyWebdriverTimeouts(driver: {
  setTimeout: (timeouts: Timeouts) => Promise<unknown>;
  getTimeouts: () => Promise<Partial<Timeouts>>;
}): Promise<void> {
  const requested = {
    script: EXTENSION_HOST_SETUP_TIMEOUT_MS,
    implicit: WEBDRIVER_IMPLICIT_TIMEOUT_MS,
    pageLoad: WEBDRIVER_PAGE_LOAD_TIMEOUT_MS,
  };
  // The embedded driver ignores timeout capabilities at session creation.
  // Apply them to the live session and fail before mutations if not honored.
  await driver.setTimeout(requested);
  const actual = await driver.getTimeouts();
  if (Object.entries(requested).some(([key, value]) => actual[key as keyof Timeouts] !== value)) {
    throw new Error("WebDriver session timeouts were not applied");
  }
}
