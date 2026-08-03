const MEBIBYTE_BYTES = 1024 * 1024;

export const DEPENDENCY_COPY_LIMITS = Object.freeze({
  maxEntries: 20_000,
  maxBytes: 128 * MEBIBYTE_BYTES,
  maxDepth: 32,
});

export const COMPLETE_RUNTIME_COPY_LIMITS = Object.freeze({
  maxEntries: DEPENDENCY_COPY_LIMITS.maxEntries + 4,
  maxBytes: 256 * MEBIBYTE_BYTES,
  maxDepth: DEPENDENCY_COPY_LIMITS.maxDepth + 1,
});
