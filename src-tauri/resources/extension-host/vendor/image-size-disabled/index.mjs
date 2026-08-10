const ERROR_CODE = "image_inspection_disabled";

export const types = Object.freeze([]);

export function imageSize() {
  throw new Error(ERROR_CODE);
}

export function disableTypes() {
  throw new Error(ERROR_CODE);
}

export default imageSize;
