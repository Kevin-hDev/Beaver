export type AppShortcutId =
  | "toggleTerminal" | "toggleSidebar" | "goBack" | "goForward"
  | "newSession" | "searchDialog" | "togglePreview"
  | "zoomIn" | "zoomOut" | "resetZoom"
  | "openSettings" | "searchConversation" | "focusComposer" | "selectSessionTab"
  | "changePermissions" | "sendMessage" | "newLine" | "stopResponse"
  | "submitEdit" | "cancelEdit";

export interface AppShortcutDefinition {
  id: AppShortcutId;
  i18n: `settings.shortcuts.${string}`;
  keys: readonly string[];
  codes: readonly string[];
  keyValues?: readonly string[];
  mod?: boolean;
  alt?: boolean;
  shift?: boolean;
  allowExtraShift?: boolean;
}

const DIGIT_CODES = [
  "Digit1", "Digit2", "Digit3", "Digit4", "Digit5", "Digit6", "Digit7", "Digit8", "Digit9",
  "Numpad1", "Numpad2", "Numpad3", "Numpad4", "Numpad5", "Numpad6", "Numpad7", "Numpad8", "Numpad9",
] as const;

// Autorité unique : l’écran des réglages et tous les gestionnaires lisent les
// mêmes touches, pour qu’un changement ne puisse plus diverger côté interface.
export const APP_SHORTCUTS: readonly AppShortcutDefinition[] = [
  { id: "toggleTerminal", i18n: "settings.shortcuts.toggleTerminal", keys: ["mod", "J"], codes: ["KeyJ"], mod: true },
  { id: "toggleSidebar", i18n: "settings.shortcuts.toggleSidebar", keys: ["mod", "B"], codes: ["KeyB"], mod: true },
  { id: "goBack", i18n: "settings.shortcuts.goBack", keys: ["mod", "◀"], codes: ["ArrowLeft"], mod: true },
  { id: "goForward", i18n: "settings.shortcuts.goForward", keys: ["mod", "▶"], codes: ["ArrowRight"], mod: true },
  { id: "newSession", i18n: "settings.shortcuts.newSession", keys: ["alt", "mod", "N"], codes: ["KeyN"], mod: true, alt: true },
  { id: "searchDialog", i18n: "settings.shortcuts.searchDialog", keys: ["mod", "G"], codes: ["KeyG"], mod: true },
  { id: "togglePreview", i18n: "settings.shortcuts.togglePreview", keys: ["alt", "mod", "B"], codes: ["KeyB"], mod: true, alt: true },
  { id: "openSettings", i18n: "settings.shortcuts.openSettings", keys: ["mod", ","], codes: ["Comma"], keyValues: [","], mod: true },
  { id: "searchConversation", i18n: "settings.shortcuts.searchConversation", keys: ["mod", "F"], codes: ["KeyF"], mod: true },
  { id: "focusComposer", i18n: "settings.shortcuts.focusComposer", keys: ["mod", "L"], codes: ["KeyL"], mod: true },
  { id: "selectSessionTab", i18n: "settings.shortcuts.selectSessionTab", keys: ["mod", "1–9"], codes: DIGIT_CODES, mod: true, allowExtraShift: true },
  { id: "changePermissions", i18n: "settings.shortcuts.changePermissions", keys: ["shift", "Tab"], codes: ["Tab"], shift: true },
  { id: "sendMessage", i18n: "settings.shortcuts.sendMessage", keys: ["Enter"], codes: ["Enter"] },
  { id: "newLine", i18n: "settings.shortcuts.newLine", keys: ["shift", "Enter"], codes: ["Enter"], shift: true },
  { id: "stopResponse", i18n: "settings.shortcuts.stopResponse", keys: ["Esc", "Esc"], codes: ["Escape"] },
  { id: "submitEdit", i18n: "settings.shortcuts.submitEdit", keys: ["mod", "Enter"], codes: ["Enter"], mod: true },
  { id: "cancelEdit", i18n: "settings.shortcuts.cancelEdit", keys: ["Esc"], codes: ["Escape"] },
  { id: "zoomIn", i18n: "settings.shortcuts.zoomIn", keys: ["mod", "+"], codes: ["Equal", "NumpadAdd"], keyValues: ["+", "="], mod: true, allowExtraShift: true },
  { id: "zoomOut", i18n: "settings.shortcuts.zoomOut", keys: ["mod", "-"], codes: ["Minus", "NumpadSubtract"], keyValues: ["-"], mod: true },
  { id: "resetZoom", i18n: "settings.shortcuts.resetZoom", keys: ["mod", "0"], codes: ["Digit0", "Numpad0"], keyValues: ["0"], mod: true },
];

type ShortcutEvent = Pick<KeyboardEvent,
  "altKey" | "code" | "ctrlKey" | "key" | "metaKey" | "shiftKey"
>;

export function matchesAppShortcut(event: ShortcutEvent, id: AppShortcutId): boolean {
  const shortcut = APP_SHORTCUTS.find((candidate) => candidate.id === id);
  if (!shortcut || !modifiersMatch(event, shortcut)) return false;
  if (shortcut.keyValues?.includes(event.key)) return true;

  const semanticOwner = APP_SHORTCUTS.find((candidate) => (
    modifiersMatch(event, candidate) && candidate.keyValues?.includes(event.key)
  ));
  if (semanticOwner) return false;
  return shortcut.codes.includes(event.code) || shortcut.codes.includes(event.key);
}

export function sessionTabIndexFromShortcut(event: ShortcutEvent): number | null {
  if (!matchesAppShortcut(event, "selectSessionTab")) return null;
  const match = /(?:Digit|Numpad)([1-9])/.exec(event.code);
  return match ? Number(match[1]) - 1 : null;
}

function modifiersMatch(event: ShortcutEvent, shortcut: AppShortcutDefinition): boolean {
  const hasMod = event.metaKey || event.ctrlKey;
  if (hasMod !== !!shortcut.mod || event.altKey !== !!shortcut.alt) return false;
  if (shortcut.shift) return event.shiftKey;
  return shortcut.allowExtraShift || !event.shiftKey;
}
