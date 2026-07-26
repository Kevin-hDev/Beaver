import { describe, expect, it } from "vitest";
import { channelErrorKey } from "../channel-error";
import de from "@/i18n/de.json";
import en from "@/i18n/en.json";
import es from "@/i18n/es.json";
import fr from "@/i18n/fr.json";
import itJson from "@/i18n/it.json";
import ja from "@/i18n/ja.json";
import zh from "@/i18n/zh.json";

describe("channelErrorKey", () => {
  it("maps known generic codes to translations", () => {
    expect(channelErrorKey("invalidConfig")).toBe("channels.errors.invalidConfig");
    expect(channelErrorKey("unavailable")).toBe("channels.errors.unavailable");
  });

  it("never exposes an internal error as a translation key", () => {
    expect(channelErrorKey("/private/path: connection failed")).toBe("channels.errors.generic");
  });

  it("provides all generic errors in the seven languages", () => {
    const locales = [fr, en, es, de, itJson, zh, ja] as Array<{
      channels: { errors: { invalidConfig: string; unavailable: string; generic: string } };
    }>;
    for (const locale of locales) {
      expect(locale.channels.errors.invalidConfig).toBeTruthy();
      expect(locale.channels.errors.unavailable).toBeTruthy();
      expect(locale.channels.errors.generic).toBeTruthy();
    }
  });
});
