import type { SkillInfo } from "@/types/agent";
import type { VoiceDeliverySnapshot } from "@/types/voice.generated";
import { insertAtSelection, type ComposerSelection } from "./composer-draft-insertion";

const MAX_DRAFTS = 64;
export const MAX_SKILLS_PER_DRAFT = 16;
export const WELCOME_COMPOSER_DRAFT_KEY = "welcome";
const MAX_LISTENERS = 64;

export interface ComposerDraftSkill { info: SkillInfo; content: string }
export interface ComposerDraftSnapshot {
  text: string;
  skills: ComposerDraftSkill[];
  selection: ComposerSelection | null;
}
interface DraftEntry extends ComposerDraftSnapshot {
  view: ComposerDraftSnapshot;
  pins: Set<string>;
  appliedDeliveryId: string | null;
  closed: boolean;
}

const EMPTY_DRAFT: ComposerDraftSnapshot = { text: "", skills: [], selection: null };
let drafts = new Map<string, DraftEntry>();
let nextListenerId = 1;
const listeners = new Map<number, () => void>();

function createEntry(closed = false): DraftEntry {
  return { ...EMPTY_DRAFT, view: EMPTY_DRAFT, pins: new Set(), appliedDeliveryId: null, closed };
}

function protectedEntry(entry: DraftEntry): boolean {
  return entry.pins.size > 0 || entry.appliedDeliveryId !== null;
}

function makeRoom(next: Map<string, DraftEntry>): boolean {
  while (next.size >= MAX_DRAFTS) {
    const candidate = [...next].find(([, entry]) => !protectedEntry(entry));
    if (!candidate) return false;
    next.delete(candidate[0]);
  }
  return true;
}

function publish(next: Map<string, DraftEntry>) {
  drafts = next;
  for (const listener of listeners.values()) listener();
}

function mutate(key: string, update: (entry: DraftEntry) => DraftEntry): boolean {
  const next = new Map(drafts);
  const existing = next.get(key);
  if (!existing && !makeRoom(next)) return false;
  next.delete(key);
  const updated = update(existing ?? createEntry());
  next.set(key, {
    ...updated,
    view: { text: updated.text, skills: updated.skills, selection: updated.selection },
  });
  publish(next);
  return true;
}

export function readComposerDraft(key: string): ComposerDraftSnapshot {
  const entry = drafts.get(key);
  return entry?.view ?? EMPTY_DRAFT;
}

export function updateComposerDraft(
  key: string,
  update: (current: ComposerDraftSnapshot) => ComposerDraftSnapshot,
): boolean {
  return mutate(key, (entry) => ({ ...entry, ...update(entry.view) }));
}

export function clearComposerDraft(key: string) {
  const entry = drafts.get(key);
  if (!entry) return;
  if (protectedEntry(entry) || entry.closed) {
    mutate(key, (current) => ({ ...current, ...EMPTY_DRAFT }));
    return;
  }
  const next = new Map(drafts);
  next.delete(key);
  publish(next);
}

export function openComposerDraft(key: string) {
  const entry = drafts.get(key);
  if (!entry || entry.closed) mutate(key, (current) => ({ ...current, closed: false }));
}

export function closeComposerDraft(key: string) {
  mutate(key, (entry) => ({ ...entry, closed: true }));
}

export function pinComposerDraft(key: string, operationId: string): boolean {
  return mutate(key, (entry) => ({ ...entry, pins: new Set(entry.pins).add(operationId) }));
}

export function unpinComposerDraft(key: string, operationId: string) {
  const entry = drafts.get(key);
  if (!entry?.pins.has(operationId)) return;
  mutate(key, (current) => {
    const pins = new Set(current.pins);
    pins.delete(operationId);
    return { ...current, pins };
  });
}

export function rememberComposerSelection(key: string, anchor: number, head: number) {
  mutate(key, (entry) => ({ ...entry, selection: { anchor, head }, closed: false }));
}

export function applyVoiceDelivery(delivery: VoiceDeliverySnapshot):
  "inserted" | "already-inserted" | "destination-closed" {
  const entry = drafts.get(delivery.draftKey);
  if (!entry || entry.closed) return "destination-closed";
  if (entry.appliedDeliveryId === delivery.id) return "already-inserted";
  const inserted = insertAtSelection(entry.text, delivery.text, entry.selection);
  mutate(delivery.draftKey, (current) => ({
    ...current,
    text: inserted.text,
    selection: { anchor: inserted.cursor, head: inserted.cursor },
    appliedDeliveryId: delivery.id,
  }));
  return "inserted";
}

export function acknowledgeVoiceDelivery(key: string, deliveryId: string) {
  const entry = drafts.get(key);
  if (entry?.appliedDeliveryId !== deliveryId) return;
  mutate(key, (current) => ({ ...current, appliedDeliveryId: null }));
}

export function subscribeComposerDrafts(listener: () => void): () => void {
  if (listeners.size >= MAX_LISTENERS) {
    console.error("[composer-draft] listener limit reached");
    return () => undefined;
  }
  const id = nextListenerId++;
  listeners.set(id, listener);
  return () => { listeners.delete(id); };
}

export function resetComposerDraftStoreForTests() {
  drafts = new Map();
  listeners.clear();
}
