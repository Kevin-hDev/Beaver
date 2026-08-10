"use strict";

const ERROR_CODE = "image_inspection_disabled";
const types = Object.freeze([]);

function imageSize() {
  throw new Error(ERROR_CODE);
}

function disableTypes() {
  throw new Error(ERROR_CODE);
}

module.exports = imageSize;
module.exports.default = imageSize;
module.exports.imageSize = imageSize;
module.exports.disableTypes = disableTypes;
module.exports.types = types;
