export const MAX_WRITE_BYTES = 65_536;
export const MAX_PENDING_INPUT_BYTES = 256 * 1024;

const RAW_SLICE_BYTES = MAX_WRITE_BYTES - 4;
const encoder = new TextEncoder();

export function splitTerminalInput(data: string): string[] {
  const bytes = encoder.encode(data);
  const decoder = new TextDecoder();
  const chunks: string[] = [];

  for (let offset = 0; offset < bytes.length; offset += RAW_SLICE_BYTES) {
    const end = Math.min(offset + RAW_SLICE_BYTES, bytes.length);
    const chunk = decoder.decode(bytes.subarray(offset, end), { stream: end < bytes.length });
    if (chunk) chunks.push(chunk);
  }

  return chunks;
}

interface PendingChunk {
  data: string;
  bytes: number;
}

export class TerminalInputQueue {
  private readonly chunks: PendingChunk[] = [];
  private pendingBytes = 0;
  private draining = false;
  private closed = false;
  private idlePromise: Promise<void> | null = null;
  private resolveIdle: (() => void) | null = null;

  constructor(private readonly writer: (chunk: string) => Promise<void>) {}

  enqueue(data: string): boolean {
    if (this.closed) return false;
    const availableBytes = MAX_PENDING_INPUT_BYTES - this.pendingBytes;
    if (data.length > availableBytes) return false;
    const addedBytes = encoder.encode(data).length;
    if (addedBytes > availableBytes) return false;
    const chunks = splitTerminalInput(data).map((chunk) => ({
      data: chunk,
      bytes: encoder.encode(chunk).length,
    }));

    this.pendingBytes += addedBytes;
    this.chunks.push(...chunks);
    void this.drain();
    return true;
  }

  close(): void {
    if (this.closed) return;
    this.closed = true;
    this.discardQueuedChunks();
    this.settleIdleIfNeeded();
  }

  idle(): Promise<void> {
    if (!this.draining && this.chunks.length === 0) return Promise.resolve();
    if (!this.idlePromise) {
      this.idlePromise = new Promise<void>((resolve) => {
        this.resolveIdle = resolve;
      });
    }
    return this.idlePromise;
  }

  private async drain(): Promise<void> {
    if (this.draining) return;
    this.draining = true;
    try {
      while (!this.closed && this.chunks.length > 0) {
        const chunk = this.chunks.shift();
        if (!chunk) break;
        try {
          await this.writer(chunk.data);
          this.pendingBytes -= chunk.bytes;
        } catch {
          this.pendingBytes -= chunk.bytes;
          this.closed = true;
          this.discardQueuedChunks();
        }
      }
    } finally {
      this.draining = false;
      this.settleIdleIfNeeded();
    }
  }

  private discardQueuedChunks(): void {
    for (const chunk of this.chunks) this.pendingBytes -= chunk.bytes;
    this.chunks.length = 0;
  }

  private settleIdleIfNeeded(): void {
    if (this.draining || this.chunks.length > 0) return;
    this.resolveIdle?.();
    this.resolveIdle = null;
    this.idlePromise = null;
  }
}
