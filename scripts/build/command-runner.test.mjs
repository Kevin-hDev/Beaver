import assert from "node:assert/strict";
import { test } from "node:test";

import { runCommand, validateCommandSpec } from "./command-runner.mjs";

const GENERIC_ERROR = /Build command failed/;

function createChild() {
  const listeners = new Map();
  return {
    once(name, handler) {
      listeners.set(name, handler);
      return this;
    },
    emit(name, ...values) {
      listeners.get(name)?.(...values);
    },
    kill() {},
  };
}

function validSpec(overrides = {}) {
  return {
    command: process.execPath,
    args: ["--version"],
    cwd: process.cwd(),
    ...overrides,
  };
}

test("transmet les arguments sans shell", async () => {
  const calls = [];
  const child = createChild();
  const pending = runCommand(validSpec(), (command, args, options) => {
    calls.push({ command, args, options });
    queueMicrotask(() => child.emit("exit", 0, null));
    return child;
  });

  await pending;
  assert.equal(calls[0].command, process.execPath);
  assert.deepEqual(calls[0].args, ["--version"]);
  assert.equal(calls[0].options.shell, false);
  assert.equal(calls[0].options.windowsHide, true);
});

test("refuse les commandes et arguments hors limites", async () => {
  const invalidSpecs = [
    validSpec({ command: 42 }),
    validSpec({ command: "x".repeat(513) }),
    validSpec({ command: "node\0" }),
    validSpec({ command: "node\r" }),
    validSpec({ command: "node\n" }),
    validSpec({ args: Array(65).fill("x") }),
    validSpec({ args: ["x".repeat(513)] }),
    validSpec({ args: ["line\nfeed"] }),
  ];

  for (const spec of invalidSpecs) {
    await assert.rejects(() => runCommand(spec), GENERIC_ERROR);
  }
});

test("refuse un cwd relatif et un PATH non borné", () => {
  assert.throws(() => validateCommandSpec(validSpec({ cwd: "." })), GENERIC_ERROR);
  assert.throws(
    () => validateCommandSpec(validSpec({ env: { PATH: "x".repeat(8193) } })),
    GENERIC_ERROR,
  );
});

test("accepte les valeurs exactes aux limites", () => {
  assert.doesNotThrow(() =>
    validateCommandSpec(
      validSpec({
        command: "x".repeat(512),
        args: Array(64).fill("x".repeat(512)),
        env: { PATH: "x".repeat(8192) },
      }),
    ),
  );
});

test("échoue fermée lors d'une erreur de lancement", async () => {
  const child = createChild();
  const pending = runCommand(validSpec(), () => child);
  queueMicrotask(() => child.emit("error", new Error("private detail")));
  await assert.rejects(() => pending, GENERIC_ERROR);
});

test("échoue fermée pour un code de sortie non nul", async () => {
  const child = createChild();
  const pending = runCommand(validSpec(), () => child);
  queueMicrotask(() => child.emit("exit", 1, null));
  await assert.rejects(() => pending, GENERIC_ERROR);
});

test("échoue fermée lorsqu'un signal termine le processus", async () => {
  const child = createChild();
  const pending = runCommand(validSpec(), () => child);
  queueMicrotask(() => child.emit("exit", null, "SIGTERM"));
  await assert.rejects(() => pending, GENERIC_ERROR);
});

test("tue le processus et échoue fermée au délai", async () => {
  const child = createChild();
  let killed = false;
  child.kill = () => {
    killed = true;
  };

  await assert.rejects(
    () => runCommand(validSpec({ timeoutMs: 1 }), () => child),
    GENERIC_ERROR,
  );
  assert.equal(killed, true);
});

test("ne modifie pas l'environnement fourni", async () => {
  const env = Object.freeze({ PATH: "safe-path", LANG: "fr_FR" });
  const child = createChild();
  const pending = runCommand(validSpec({ env }), (_command, _args, options) => {
    queueMicrotask(() => child.emit("exit", 0, null));
    assert.deepEqual({ ...options.env }, env);
    assert.notEqual(options.env, env);
    return child;
  });

  await pending;
  assert.deepEqual(env, { PATH: "safe-path", LANG: "fr_FR" });
});

test("utilise un instantané validé avant le lancement", async () => {
  const child = createChild();
  const spec = validSpec({ env: { PATH: "safe-path" } });
  let argsReads = 0;
  let envReads = 0;
  Object.defineProperties(spec, {
    args: {
      get() {
        argsReads += 1;
        return argsReads <= 3 ? ["--version"] : ["unsafe\nargument"];
      },
    },
    env: {
      get() {
        envReads += 1;
        return envReads === 1 ? { PATH: "safe-path" } : { PATH: "unsafe\npath" };
      },
    },
  });

  const pending = runCommand(spec, (_command, args, options) => {
    queueMicrotask(() => child.emit("exit", 0, null));
    assert.deepEqual(args, ["--version"]);
    assert.equal(options.env.PATH, "safe-path");
    return child;
  });

  await pending;
});
