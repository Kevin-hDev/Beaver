import { describe, it, expect, beforeEach, vi } from "vitest";
import { registerToast, showToast } from "./toast-emitter";

describe("toast-emitter", () => {
  beforeEach(() => {
    // Reset le handler à un no-op avant chaque test (l'état est global).
    registerToast(() => {});
  });

  it("showToast appelle le handler enregistré", () => {
    const fn = vi.fn();
    registerToast(fn);

    showToast("message de test", "error");

    expect(fn).toHaveBeenCalledOnce();
    expect(fn).toHaveBeenCalledWith("message de test", "error", undefined);
  });

  it("utilise 'error' comme type par défaut", () => {
    const fn = vi.fn();
    registerToast(fn);

    showToast("erreur");

    expect(fn).toHaveBeenCalledWith("erreur", "error", undefined);
  });

  it("transmet le type et la durée au handler", () => {
    const fn = vi.fn();
    registerToast(fn);

    showToast("succès", "success", 3000);

    expect(fn).toHaveBeenCalledWith("succès", "success", 3000);
  });

  it("supporte les types success, info, check", () => {
    const fn = vi.fn();
    registerToast(fn);

    showToast("ok1", "success");
    showToast("ok2", "info");
    showToast("ok3", "check");

    expect(fn).toHaveBeenNthCalledWith(1, "ok1", "success", undefined);
    expect(fn).toHaveBeenNthCalledWith(2, "ok2", "info", undefined);
    expect(fn).toHaveBeenNthCalledWith(3, "ok3", "check", undefined);
  });

  it("ne crash pas si aucun handler n'a été enregistré (default no-op)", () => {
    // Le handler par défaut est un no-op → showToast ne doit pas throw.
    expect(() => showToast("rien")).not.toThrow();
  });

  it("remplace le handler précédent à l'enregistrement", () => {
    const fn1 = vi.fn();
    const fn2 = vi.fn();
    registerToast(fn1);
    registerToast(fn2);

    showToast("message");

    expect(fn1).not.toHaveBeenCalled();
    expect(fn2).toHaveBeenCalledOnce();
  });
});
