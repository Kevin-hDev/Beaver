import { invoke } from "@tauri-apps/api/core";
import { UI_LIMITS } from "@/types/extension-ui-contract.generated";
import { sequenceExtensionUiLoad } from "../ui-load-sequencer";

export interface MountPermit {
  commit: () => Promise<void>;
  cancel: () => void;
}

interface MountJob {
  key: string;
  extensionId: string;
  attempts: number;
  ready: (permit: MountPermit) => void;
  failReady: (error: Error) => void;
  done: Promise<void>;
  finish: () => void;
  failDone: (error: Error) => void;
}

export function createMountCoordinator() {
  const completed = new Set<string>();
  const order: string[] = [];
  const jobs = new Map<string, MountJob>();
  const queue: MountJob[] = [];
  let active: MountJob | null = null;
  let blocked = false;

  async function prepare(key: string, extensionId: string, attempts: number): Promise<MountPermit> {
    if (blocked) throw generic();
    if (completed.has(key)) return noOpPermit();
    const current = jobs.get(key);
    if (current) return current.done.then(noOpPermit);
    if (jobs.size >= UI_LIMITS.maxGlobalStandardContributions) throw generic();
    let ready!: (permit: MountPermit) => void;
    let failReady!: (error: Error) => void;
    let finish!: () => void;
    let failDone!: (error: Error) => void;
    const readyPromise = new Promise<MountPermit>((resolve, reject) => {
      ready = resolve;
      failReady = reject;
    });
    const done = new Promise<void>((resolve, reject) => {
      finish = resolve;
      failDone = reject;
    });
    void done.catch(() => {});
    const job = { key, extensionId, attempts, ready, failReady, done, finish, failDone };
    jobs.set(key, job);
    queue.push(job);
    void drain();
    return readyPromise;
  }

  function drain(): void {
    if (active || queue.length === 0) return;
    active = queue.shift() ?? null;
    if (!active) return;
    const job = active;
    void sequenceExtensionUiLoad(async () => {
      const token = await invoke<number[]>("begin_extension_ui_load", {
        extensionId: job.extensionId,
        attempts: job.attempts,
      });
      await invoke("advance_extension_ui_load", {
        extensionId: job.extensionId,
        stage: "mount",
      });
      let settled = false;
      await new Promise<void>((resolve, reject) => {
        job.ready({
          commit: async () => {
            if (settled) return;
            settled = true;
            try {
              await invoke("acknowledge_extension_ui_load", {
                extensionId: job.extensionId,
                token,
              });
              remember(job.key);
              job.finish();
              resolve();
            } catch {
              reject(generic());
            }
          },
          cancel: () => {
            if (settled) return;
            settled = true;
            reject(generic());
          },
        });
      });
      jobs.delete(job.key);
      active = null;
      void drain();
    }).catch(() => {
      fail(job, generic());
    });
  }

  function fail(job: MountJob, error: Error) {
    blocked = true;
    job.failReady(error);
    job.failDone(error);
    jobs.delete(job.key);
    active = null;
    // Le journal reste volontairement présent : continuer masquerait un montage incomplet.
  }

  function remember(key: string) {
    if (completed.has(key)) return;
    completed.add(key);
    order.push(key);
    while (order.length > UI_LIMITS.maxGlobalStandardContributions) {
      const expired = order.shift();
      if (expired) completed.delete(expired);
    }
  }

  return { prepare };
}

function noOpPermit(): MountPermit {
  return { commit: async () => {}, cancel: () => {} };
}

function generic(): Error {
  return new Error("extension_ui_mount_failed");
}
