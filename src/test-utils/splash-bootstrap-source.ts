const SCRIPT_OPEN = "<script>";
const SCRIPT_CLOSE = "</script>";

export function extractSplashBootstrap(indexHtml: string): string {
  const contentStart = indexHtml.indexOf(SCRIPT_OPEN);
  const contentEnd = indexHtml.indexOf(SCRIPT_CLOSE, contentStart + SCRIPT_OPEN.length);
  if (contentStart < 0 || contentEnd < 0) {
    throw new Error("splash_bootstrap_missing");
  }
  return indexHtml.slice(contentStart + SCRIPT_OPEN.length, contentEnd);
}
