export const IS_MAC = navigator.userAgent.includes("Mac");
export const IS_LINUX = navigator.userAgent.includes("Linux");
export const IS_WINDOWS = navigator.userAgent.includes("Windows");
export const MOD = IS_MAC ? "⌘" : "Ctrl+";
export const MOD_LABEL = IS_MAC ? "⌘" : "Ctrl";
export const MOD_KEY = IS_MAC ? "metaKey" : "ctrlKey";
export const ALT = IS_MAC ? "⌥" : "Alt+";
export const ALT_LABEL = IS_MAC ? "⌥" : "Alt";
