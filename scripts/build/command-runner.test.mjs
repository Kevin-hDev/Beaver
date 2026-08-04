import assert from "node:assert/strict";
import { mkdtemp, mkdir, rm, symlink } from "node:fs/promises";
import { join, sep } from "node:path";
import { test } from "node:test";
import { tmpdir } from "node:os";

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

function createInstrumentedChild() {
  const listeners = new Map();
  const added = [];
  const removed = [];
  return {
    added,
    removed,
    once(name, handler) {
      added.push({ name, handler });
      listeners.set(name, handler);
      return this;
    },
    off(name, handler) {
      removed.push({ name, handler });
      if (listeners.get(name) === handler) listeners.delete(name);
      return this;
    },
    emit(name, ...values) {
      listeners.get(name)?.(...values);
    },
    listenerCount() {
      return listeners.size;
    },
    kill() {},
  };
}

function assertListenersRemoved(child) {
  assert.deepEqual(child.added.map((entry) => entry.name), ["error", "exit"]);
  assert.deepEqual(child.removed.map((entry) => entry.name).sort(), ["error", "exit"]);
  assert.equal(child.listenerCount(), 0);
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

test("refuse un cwd absolu contenant un segment de traversée", () => {
  const cwdWithTraversal = `${process.cwd()}${sep}..`;
  assert.throws(() => validateCommandSpec(validSpec({ cwd: cwdWithTraversal })), GENERIC_ERROR);
});

test("refuse un cwd lien symbolique ou jonction", async (t) => {
  const temporaryRoot = await mkdtemp(join(tmpdir(), "beaver-runner-"));
  const target = join(temporaryRoot, "target");
  const linked = join(temporaryRoot, "linked");
  try {
    await mkdir(target);
    try {
      await symlink(target, linked, process.platform === "win32" ? "junction" : "dir");
    } catch (error) {
      if (error?.code === "EPERM" || error?.code === "EACCES") {
        t.skip("La création de lien n'est pas autorisée sur cet environnement.");
        return;
      }
      throw error;
    }
    assert.throws(() => validateCommandSpec(validSpec({ cwd: linked })), GENERIC_ERROR);
  } finally {
    await rm(temporaryRoot, { recursive: true, force: true });
  }
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
  const child = createInstrumentedChild();
  let killed = false;
  child.kill = () => {
    killed = true;
  };

  await assert.rejects(
    () => runCommand(validSpec({ timeoutMs: 1 }), () => child),
    GENERIC_ERROR,
  );
  assert.equal(killed, true);
  assertListenersRemoved(child);
});

test("nettoie les listeners après error puis exit", async () => {
  const child = createInstrumentedChild();
  const pending = runCommand(validSpec(), () => child);
  child.emit("error", new Error("private detail"));
  await assert.rejects(() => pending, GENERIC_ERROR);
  child.emit("exit", 0, null);
  assertListenersRemoved(child);
});

test("nettoie les listeners après exit puis error", async () => {
  const child = createInstrumentedChild();
  const pending = runCommand(validSpec(), () => child);
  child.emit("exit", 0, null);
  await pending;
  child.emit("error", new Error("private detail"));
  assertListenersRemoved(child);
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
