import type { TFunction } from "i18next";

const PARAGRAPH_ERROR =
  /^unsupported_character (U\+[0-9A-F]{4,6}) paragraph ([1-9][0-9]{0,2})$/u;
const TITLE_ERROR = /^unsupported_character (U\+[0-9A-F]{4,6}) title$/u;

export function officeToolErrorMessage(
  error: string,
  t: TFunction,
): string | undefined {
  const paragraph = PARAGRAPH_ERROR.exec(error);
  if (paragraph) {
    return t("extensions.errors.office.unsupportedCharacterAtParagraph", {
      codePoint: paragraph[1],
      paragraph: paragraph[2],
    });
  }
  const title = TITLE_ERROR.exec(error);
  if (title) {
    return t("extensions.errors.office.unsupportedCharacterInTitle", {
      codePoint: title[1],
    });
  }
  if (error === "unsupported_character") {
    return t("extensions.errors.office.unsupportedCharacter");
  }
  if (error === "too_many_requests") {
    return t("extensions.errors.office.tooManyRequests");
  }
  return undefined;
}
