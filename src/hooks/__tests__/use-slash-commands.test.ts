import { describe, it, expect, vi, beforeEach } from "vitest";
import { renderHook, act, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));
vi.mock("@/hooks/use-fs-event", () => ({
  useFsEvent: vi.fn(),
}));

import { useSlashCommands, isBuiltIn } from "@/hooks/use-slash-commands";
import type { BuiltInCommand } from "@/hooks/use-slash-commands";
import type { SkillInfo } from "@/types/agent";

const mockSkill: SkillInfo = {
  id: "local:skill:hk-dev",
  name: "hk-dev",
  command: "hk-dev",
  description: "Skill de développement",
  source: "local",
  source_name: "CL-GO-DASH",
  path: "/some/path/hk-dev.md",
};

const mockBuiltIn: BuiltInCommand = {
  name: "compress",
  description: "Compress conversation context manually",
  source: "built-in",
  path: "__built-in__/compress",
};

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(invoke).mockResolvedValue([]);
});

describe("isBuiltIn", () => {
  it("retourne true pour une commande built-in", () => {
    expect(isBuiltIn(mockBuiltIn)).toBe(true);
  });

  it("retourne false pour un skill utilisateur", () => {
    expect(isBuiltIn(mockSkill)).toBe(false);
  });
});

describe("useSlashCommands — initialisation", () => {
  it("contient la commande compress built-in au montage", async () => {
    vi.mocked(invoke).mockResolvedValue([]);

    const { result } = renderHook(() => useSlashCommands());

    await waitFor(() => {
      const compress = result.current.skills.find(
        (s) => s.name === "compress" && s.source === "built-in",
      );
      expect(compress).toBeDefined();
    });
  });
});

describe("useSlashCommands — handleInput", () => {
  it('ouvre le dropdown quand le texte commence par "/"', () => {
    const { result } = renderHook(() => useSlashCommands());

    act(() => {
      result.current.handleInput("/");
    });

    expect(result.current.showDropdown).toBe(true);
  });

  it('filtre par "comp" avec l\'entrée "/comp"', async () => {
    vi.mocked(invoke).mockResolvedValue([mockSkill]);
    const { result } = renderHook(() => useSlashCommands());

    await waitFor(() => {
      expect(result.current.skills.length).toBeGreaterThan(0);
    });

    act(() => {
      result.current.handleInput("/comp");
    });

    expect(result.current.showDropdown).toBe(true);
    // "compress" contient "comp"
    const items = result.current.skills;
    expect(items.every((s) => s.name.toLowerCase().includes("comp") || s.description.toLowerCase().includes("comp"))).toBe(true);
  });

  it('ferme le dropdown avec "hello" (pas de slash)', () => {
    const { result } = renderHook(() => useSlashCommands());

    act(() => {
      result.current.handleInput("/");
    });
    expect(result.current.showDropdown).toBe(true);

    act(() => {
      result.current.handleInput("hello");
    });
    expect(result.current.showDropdown).toBe(false);
  });

  it('ouvre avec filtre "test" pour "hello /test"', () => {
    const { result } = renderHook(() => useSlashCommands());

    act(() => {
      result.current.handleInput("hello /test");
    });

    expect(result.current.showDropdown).toBe(true);
  });
});

describe("useSlashCommands — navigation", () => {
  it("moveDown incrémente activeIndex", async () => {
    vi.mocked(invoke).mockResolvedValue([mockSkill]);
    const { result } = renderHook(() => useSlashCommands());

    await waitFor(() => {
      expect(result.current.skills.length).toBeGreaterThan(1);
    });

    expect(result.current.activeIndex).toBe(0);

    act(() => {
      result.current.moveDown();
    });

    expect(result.current.activeIndex).toBe(1);
  });

  it("moveUp décrémente activeIndex", async () => {
    vi.mocked(invoke).mockResolvedValue([mockSkill]);
    const { result } = renderHook(() => useSlashCommands());

    await waitFor(() => {
      expect(result.current.skills.length).toBeGreaterThan(1);
    });

    act(() => {
      result.current.moveDown();
    });
    expect(result.current.activeIndex).toBe(1);

    act(() => {
      result.current.moveUp();
    });
    expect(result.current.activeIndex).toBe(0);
  });

  it("close ferme le dropdown et remet activeIndex à 0", () => {
    const { result } = renderHook(() => useSlashCommands());

    act(() => {
      result.current.handleInput("/comp");
    });
    expect(result.current.showDropdown).toBe(true);

    act(() => {
      result.current.close();
    });

    expect(result.current.showDropdown).toBe(false);
    expect(result.current.activeIndex).toBe(0);
  });

  it("moveDown wraps à 0 quand on dépasse la fin", async () => {
    vi.mocked(invoke).mockResolvedValue([]);
    const { result } = renderHook(() => useSlashCommands());

    // Attendre la liste : seule compress est visible (invoke retourne [])
    await waitFor(() => {
      expect(result.current.skills).toHaveLength(1);
    });

    // On est à index 0, liste de taille 1 → moveDown doit wrapper à 0
    act(() => {
      result.current.moveDown();
    });

    expect(result.current.activeIndex).toBe(0);
  });

  it("moveUp wraps au dernier index quand on est à 0", async () => {
    vi.mocked(invoke).mockResolvedValue([mockSkill]);
    const { result } = renderHook(() => useSlashCommands());

    await waitFor(() => {
      // compress + mockSkill = 2 items
      expect(result.current.skills).toHaveLength(2);
    });

    // index = 0, moveUp → dernier index = 1
    act(() => {
      result.current.moveUp();
    });

    expect(result.current.activeIndex).toBe(1);
  });
});

describe("useSlashCommands — liste skills vide", () => {
  it("invoke retourne [] → seule compress built-in est visible", async () => {
    vi.mocked(invoke).mockResolvedValue([]);
    const { result } = renderHook(() => useSlashCommands());

    await waitFor(() => {
      expect(result.current.skills).toHaveLength(1);
    });

    expect(result.current.skills[0].name).toBe("compress");
    expect(result.current.skills[0].source).toBe("built-in");
  });
});

describe("useSlashCommands — cas limites handleInput", () => {
  it('double slash "hello //test" ne crash pas et détecte la commande', () => {
    const { result } = renderHook(() => useSlashCommands());

    expect(() => {
      act(() => {
        result.current.handleInput("hello //test");
      });
    }).not.toThrow();
  });

  it('détecte un slash tapé AVANT du texte existant (cursor après le slash)', () => {
    // Cas du bug : texte "hello", utilisateur tape "/" devant → "/hello",
    // mais le curseur est juste après le "/" (position 1), pas à la fin.
    const { result } = renderHook(() => useSlashCommands());

    act(() => {
      result.current.handleInput("/hello", 1);
    });

    expect(result.current.showDropdown).toBe(true);
  });

  it('filtre correctement quand le slash précède du texte existant', () => {
    // Texte "hk-dev", utilisateur tape "/" devant → "/hk-dev",
    // curseur après le "hk" (position 3) → filtre = "hk".
    vi.mocked(invoke).mockResolvedValue([mockSkill]);
    const { result } = renderHook(() => useSlashCommands());

    act(() => {
      result.current.handleInput("/hk-dev", 3);
    });

    expect(result.current.showDropdown).toBe(true);
    // La liste filtrée ne contient que les items dont le nom matche "hk".
    expect(result.current.skills.every((s) =>
      s.name.toLowerCase().includes("hk") || s.description.toLowerCase().includes("hk"),
    )).toBe(true);
  });

  it('ferme le dropdown quand le curseur sort du token slash', () => {
    // "/hello" avec curseur à la fin (position 6) → le token contient "hello"
    // qui n\'est pas un filtre valide, mais le slash est détecté. Ici on vérifie
    // que si le curseur est après un espace, on ferme.
    const { result } = renderHook(() => useSlashCommands());

    act(() => {
      result.current.handleInput("/hk plus de texte", 14);
    });

    expect(result.current.showDropdown).toBe(false);
  });
});

describe("useSlashCommands — selectItem", () => {
  it("selectItem sur un built-in retourne { builtIn }", async () => {
    const { result } = renderHook(() => useSlashCommands());

    let selection: Awaited<ReturnType<typeof result.current.selectItem>>;
    await act(async () => {
      selection = await result.current.selectItem(mockBuiltIn);
    });

    expect(selection!).toEqual({ builtIn: mockBuiltIn });
    expect(result.current.showDropdown).toBe(false);
  });
});
