import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { readFile } from "@tauri-apps/plugin-fs";
import i18n from "@/i18n";

const MAX_FILES = 15;
const MAX_SIZE = 20 * 1024 * 1024;
const IMAGE_EXTS = ["png", "jpg", "jpeg", "gif", "webp"];

export interface DroppedFile {
  name: string;
  path?: string;
  type: string;
  size: number;
  preview?: string;
  accessGrant?: string;
}

interface RegisteredAttachment {
  path: string;
  size: number;
  access_grant: string;
}

export function useFileDrop() {
  const [files, setFiles] = useState<DroppedFile[]>([]);
  const [dragging, setDragging] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const addFiles = useCallback(async (fileList: FileList | File[]) => {
    const newFiles = Array.from(fileList);
    if (files.length + newFiles.length > MAX_FILES) {
      setError(i18n.t("errors.maxFiles", { max: MAX_FILES }));
      return;
    }
    const oversized = newFiles.find((file) => file.size > MAX_SIZE);
    if (oversized) {
      setError(i18n.t("errors.fileTooLarge", { name: oversized.name }));
      return;
    }
    setError(null);
    const dropped: DroppedFile[] = await Promise.all(
      newFiles.map(async (f) => {
        const isImage = f.type.startsWith("image/");
        const preview = isImage ? await readAsDataUrl(f) : undefined;
        return { name: f.name, type: f.type, size: f.size, preview };
      }),
    );
    setFiles((prev) => [...prev, ...dropped]);
  }, [files.length]);

  const addByPaths = useCallback(async (paths: string[]) => {
    if (files.length + paths.length > MAX_FILES) {
      setError(i18n.t("errors.maxFiles", { max: MAX_FILES }));
      return;
    }
    setError(null);
    try {
      const registered = await invoke<RegisteredAttachment[]>("register_attachment_paths", {
        paths,
      });
      if (!isValidRegistration(registered)) throw new Error("invalid registration");
      const dropped: DroppedFile[] = [];
      for (const file of registered) {
        const name = file.path.split(/[\\/]/).pop() ?? file.path;
        const ext = name.split(".").pop()?.toLowerCase() ?? "";
        let preview: string | undefined;
        if (IMAGE_EXTS.includes(ext)) {
          const bytes = await readFile(file.path);
          let binary = "";
          for (let i = 0; i < bytes.length; i++) {
            binary += String.fromCharCode(bytes[i]);
          }
          const mimeMap: Record<string, string> = {
            png: "image/png", jpg: "image/jpeg", jpeg: "image/jpeg",
            gif: "image/gif", webp: "image/webp",
          };
          preview = `data:${mimeMap[ext] ?? "image/png"};base64,${btoa(binary)}`;
        }
        dropped.push({
          name,
          path: file.path,
          type: ext,
          size: file.size,
          preview,
          accessGrant: file.access_grant,
        });
      }
      setFiles((prev) => [...prev, ...dropped]);
    } catch {
      setError(i18n.t("errors.operationFailed"));
    }
  }, [files.length]);

  const removeFile = useCallback((index: number) => {
    setFiles((prev) => prev.filter((_, i) => i !== index));
  }, []);

  const clearFiles = useCallback(() => {
    setFiles([]);
    setError(null);
  }, []);

  return { files, dragging, error, addFiles, addByPaths, removeFile, clearFiles, setDragging };
}

function isValidRegistration(value: unknown): value is RegisteredAttachment[] {
  if (!Array.isArray(value) || value.length > MAX_FILES) return false;
  const items: unknown[] = value;
  return items.every((item) => {
    if (typeof item !== "object" || item === null) return false;
    const file = item as Record<string, unknown>;
    return typeof file.path === "string" && file.path.length > 0 && file.path.length <= 4096
      && typeof file.size === "number" && file.size >= 0 && file.size <= MAX_SIZE
      && typeof file.access_grant === "string" && file.access_grant.length === 67;
  });
}

function readAsDataUrl(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(reader.result as string);
    reader.onerror = reject;
    reader.readAsDataURL(file);
  });
}
