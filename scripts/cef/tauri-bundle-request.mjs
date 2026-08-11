const ERROR_MESSAGE = "Windows bundle request is invalid";
const SUPPORTED_BUNDLES = new Set(["msi", "nsis"]);

export function resolveWindowsBundleRequest({ args, platform }) {
  if (
    !Array.isArray(args) ||
    args.length > 64 ||
    args.some(
      (argument) =>
        typeof argument !== "string" ||
        argument.length < 1 ||
        argument.length > 512 ||
        /[\0\r\n]/u.test(argument),
    )
  ) {
    throw new Error(ERROR_MESSAGE);
  }
  if (platform !== "win32" || args[0] !== "build") {
    return { args, bundleType: null };
  }

  const optionEnd = args.indexOf("--");
  const buildOptions = args.slice(1, optionEnd < 0 ? undefined : optionEnd);
  const noBundle = buildOptions.includes("--no-bundle");
  const bundles = requestedBundles(buildOptions);
  if (noBundle && bundles !== null) throw new Error(ERROR_MESSAGE);
  if (noBundle) return { args, bundleType: null };
  if (bundles === null) {
    const insertion = optionEnd < 0 ? args.length : optionEnd;
    return {
      args: [...args.slice(0, insertion), "--bundles=nsis", ...args.slice(insertion)],
      bundleType: "nsis",
    };
  }
  if (bundles.length !== 1 || !SUPPORTED_BUNDLES.has(bundles[0])) {
    throw new Error(ERROR_MESSAGE);
  }
  const trailing = optionEnd < 0 ? [] : args.slice(optionEnd);
  return {
    args: [args[0], ...normalizeBundleOption(buildOptions, bundles[0]), ...trailing],
    bundleType: bundles[0],
  };
}

function requestedBundles(options) {
  let result = null;
  for (let index = 0; index < options.length; index += 1) {
    const option = options[index];
    const inline = /^(?:--bundles|-b)=(.*)$/u.exec(option);
    if (inline) {
      if (result !== null || inline[1].length === 0) throw new Error(ERROR_MESSAGE);
      result = splitBundles([inline[1]]);
      continue;
    }
    if (option !== "--bundles" && option !== "-b") continue;
    if (result !== null) throw new Error(ERROR_MESSAGE);
    const values = [];
    while (index + 1 < options.length && !options[index + 1].startsWith("-")) {
      values.push(options[index + 1]);
      index += 1;
    }
    if (values.length === 0) throw new Error(ERROR_MESSAGE);
    result = splitBundles(values);
  }
  return result;
}

function splitBundles(values) {
  const bundles = values.flatMap((value) => value.split(","));
  if (bundles.some((value) => value.length === 0)) throw new Error(ERROR_MESSAGE);
  return bundles;
}

function normalizeBundleOption(options, bundleType) {
  const normalized = [];
  for (let index = 0; index < options.length; index += 1) {
    const option = options[index];
    if (/^(?:--bundles|-b)=/u.test(option)) {
      normalized.push(`--bundles=${bundleType}`);
      continue;
    }
    if (option !== "--bundles" && option !== "-b") {
      normalized.push(option);
      continue;
    }
    normalized.push(`--bundles=${bundleType}`);
    while (index + 1 < options.length && !options[index + 1].startsWith("-")) {
      index += 1;
    }
  }
  return normalized;
}
