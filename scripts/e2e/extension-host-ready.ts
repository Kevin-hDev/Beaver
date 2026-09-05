import { EXTENSION_HOST_SETUP_TIMEOUT_MS } from "./extension-setup-deadline";

type HostStatus = { state: string; lastError?: string };

export async function waitForHostReady(
  readStatus: () => Promise<HostStatus>,
  waitUntil: (condition: () => Promise<boolean>, options: {
    timeout: number; timeoutMsg: string;
  }) => Promise<unknown>,
): Promise<void> {
  let latest: HostStatus = { state: "unknown" };
  await waitUntil(async () => {
    latest = await readStatus();
    // WDIO retries rejected predicates. Finish polling on terminal failure,
    // then propagate the error outside its retry loop.
    return latest.state === "running" || latest.state === "error";
  }, {
    timeout: EXTENSION_HOST_SETUP_TIMEOUT_MS,
    timeoutMsg: "Extension host did not become ready",
  });
  if (latest.state !== "running") {
    const code = /^extensions_[a-z_]{1,64}$/u.test(latest.lastError ?? "")
      ? latest.lastError : "host_error";
    throw new Error(`Extension host unavailable: ${code}`);
  }
}
