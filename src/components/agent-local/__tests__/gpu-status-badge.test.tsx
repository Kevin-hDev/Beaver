import { cleanup, render, screen } from "@testing-library/react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { GpuStatusBadge } from "../gpu-status-badge";
import { useGpuStatus } from "@/hooks/use-gpu-status";

vi.mock("@/hooks/use-setting-value", () => ({
  useSettingValue: () => true,
}));

vi.mock("@/hooks/use-gpu-status", () => ({
  useGpuStatus: vi.fn(),
}));

const mockedUseGpuStatus = vi.mocked(useGpuStatus);

afterEach(() => {
  cleanup();
  vi.clearAllMocks();
});

describe("GpuStatusBadge", () => {
  it("identifies Apple unified memory as RAM", () => {
    mockedUseGpuStatus.mockReturnValue({
      accelerator: "RAM",
      vramUsedMb: 12288,
      vramTotalMb: 24576,
      modelLoaded: null,
      vramPercent: 50,
    });

    render(<GpuStatusBadge />);

    expect(screen.getByText("RAM 50%")).toBeTruthy();
  });

  it("shows a percentage when total VRAM is known", () => {
    mockedUseGpuStatus.mockReturnValue({
      accelerator: "VRAM",
      vramUsedMb: 4096,
      vramTotalMb: 8192,
      modelLoaded: null,
      vramPercent: 50,
    });

    render(<GpuStatusBadge />);

    expect(screen.getByText("VRAM 50%")).toBeTruthy();
  });

  it("shows used VRAM when total VRAM is unknown", () => {
    mockedUseGpuStatus.mockReturnValue({
      accelerator: "VRAM",
      vramUsedMb: 5120,
      vramTotalMb: 0,
      modelLoaded: null,
      vramPercent: 0,
    });

    render(<GpuStatusBadge />);

    expect(screen.getByText("VRAM 5.0 GB")).toBeTruthy();
  });
});
