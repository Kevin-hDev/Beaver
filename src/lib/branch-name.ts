type BranchNameValidationReason = "invalid_name" | "name_too_long";

export interface BranchNameValidation {
  valid: boolean;
  reason?: BranchNameValidationReason;
}

// KEEP IN SYNC with src-tauri/src/services/git/branch_create.rs::validate_branch_name.
export function validateBranchName(name: string): BranchNameValidation {
  if (Array.from(name).length > 100) {
    return { valid: false, reason: "name_too_long" };
  }
  if (
    name.length === 0
    || name.startsWith("-")
    || name.includes("..")
    || name.includes("\0")
    || name.includes("\\")
    || name.includes(":")
    || name.includes("~")
    || name.includes("^")
    || name.includes("?")
    || name.includes("*")
    || name.includes("[")
    || name.includes("@{")
    || name.includes("//")
    || name.startsWith("/")
    || name.endsWith("/")
    || name.endsWith(".lock")
    || Array.from(name).some((char) => {
      const code = char.codePointAt(0) ?? 0;
      return code <= 0x20 || code === 0x7f;
    })
  ) {
    return { valid: false, reason: "invalid_name" };
  }

  for (const segment of name.split("/")) {
    if (!segment || segment.startsWith(".") || segment.endsWith(".")) {
      return { valid: false, reason: "invalid_name" };
    }
  }

  return { valid: true };
}
