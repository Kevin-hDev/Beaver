import { useState, useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import { cleanupTauriListener } from "@/lib/tauri-listen";

interface GpuStatusEvent {
  accelerator: string;
  vram_used_mb: number;
  vram_total_mb: number;
  model_loaded: string | null;
}

export interface GpuStatus {
  accelerator: string;
  vramUsedMb: number;
  vramTotalMb: number;
  modelLoaded: string | null;
  hasModel: boolean;
  vramPercent: number;
}

const EMPTY: GpuStatus = {
  accelerator: "",
  vramUsedMb: 0,
  vramTotalMb: 0,
  modelLoaded: null,
  hasModel: false,
  vramPercent: 0,
};

export function useGpuStatus(): GpuStatus {
  const [status, setStatus] = useState<GpuStatus>(EMPTY);

  useEffect(() => {
    const unlisten = listen<GpuStatusEvent>("ollama-gpu-status", (e) => {
      const p = e.payload;
      const hasModel = Boolean(p.accelerator);
      const pct = p.vram_total_mb > 0
        ? Math.round((p.vram_used_mb / p.vram_total_mb) * 100)
        : 0;
      setStatus({
        accelerator: p.accelerator,
        vramUsedMb: p.vram_used_mb,
        vramTotalMb: p.vram_total_mb,
        modelLoaded: p.model_loaded,
        hasModel,
        vramPercent: Math.min(pct, 100),
      });
    });
    return () => { cleanupTauriListener(unlisten); };
  }, []);

  return status;
}
