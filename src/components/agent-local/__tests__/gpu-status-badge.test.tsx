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
  it("shows a percentage when total VRAM is known", () => {
    mockedUseGpuStatus.mockReturnValue({
      accelerator: "GPU",
      vramUsedMb: 4096,
      vramTotalMb: 8192,
      modelLoaded: null,
      hasModel: true,
      vramPercent: 50,
    });

    render(<GpuStatusBadge />);

    expect(screen.getByText("GPU 50%")).toBeTruthy();
  });

  it("shows used VRAM when total VRAM is unknown", () => {
    mockedUseGpuStatus.mockReturnValue({
      accelerator: "GPU",
      vramUsedMb: 5120,
      vramTotalMb: 0,
      modelLoaded: null,
      hasModel: true,
      vramPercent: 0,
    });

    render(<GpuStatusBadge />);

    expect(screen.getByText("GPU 5.0 GB")).toBeTruthy();
  });
});
