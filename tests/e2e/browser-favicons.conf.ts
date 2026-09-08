import { config as base } from "../../wdio.conf";
export const config: WebdriverIO.Config = {
  ...base,
  specs: ["./browser-favicons.spec.ts"],
  mochaOpts: { ui: "bdd", timeout: 600_000 },
};
