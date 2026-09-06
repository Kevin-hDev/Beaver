import { describe, expect, it } from "vitest";
import { formatByteSize } from "./format-byte-size";

describe("shared byte size", () => {
  it("uses binary units consistently through large installations", () => {
    expect(formatByteSize(1023)).toBe("1,023 B");
    expect(formatByteSize(1024)).toBe("1 KiB");
    expect(formatByteSize(1024 ** 2)).toBe("1 MiB");
    expect(formatByteSize(3n * 1024n ** 3n)).toBe("3 GiB");
  });
  it("uses the locale for decimals and French unit names", () => {
    expect(formatByteSize(1.5 * 1024 ** 3, "fr")).toBe("1,5 Gio");
    expect(formatByteSize(1.5 * 1024 ** 3, "de")).toBe("1,5 GiB");
    expect(formatByteSize(0, "fr")).toBe("0 o");
  });
  it("does not display invalid or negative sizes", () => {
    expect(formatByteSize(Number.NaN)).toBe("0 B");
    expect(formatByteSize(Number.POSITIVE_INFINITY)).toBe("0 B");
    expect(formatByteSize(-1)).toBe("0 B");
  });
});
