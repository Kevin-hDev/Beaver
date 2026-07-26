import { useMemo, useState } from "react";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { useLocalListNavigation, type LocalListNavItem } from "../use-local-list-navigation";

function LocalListHarness() {
  const [selected, setSelected] = useState("one");
  const [events, setEvents] = useState<string[]>([]);
  const items = useMemo<LocalListNavItem[]>(() => [
    { id: "group", onSelect: () => setEvents((current) => [...current, "toggle"]), onArrowRight: () => setEvents((current) => [...current, "open"]) },
    { id: "one", onSelect: () => setSelected("one") },
    { id: "two", onSelect: () => setSelected("two") },
  ], []);
  const nav = useLocalListNavigation({
    items,
    selectedId: selected,
    onEscape: () => setEvents((current) => [...current, "escape"]),
  });

  return (
    <div role="listbox" tabIndex={-1} onKeyDown={nav.listProps.onKeyDown}>
      <input data-testid="search" onKeyDown={nav.listProps.onKeyDown} />
      {items.map((item) => (
        <div
          key={item.id}
          role="option"
          ref={nav.getItemRef(item.id)}
          tabIndex={nav.isActive(item.id) ? 0 : -1}
          data-local-nav-active={nav.isActive(item.id) ? "true" : undefined}
          data-testid={item.id}
          aria-selected={nav.isActive(item.id)}
          onFocus={() => nav.activate(item.id)}
          onMouseEnter={() => nav.activate(item.id)}
          onKeyDown={nav.listProps.onKeyDown}
        >
          {item.id}
        </div>
      ))}
      <span data-testid="selected">{selected}</span>
      <span data-testid="events">{events.join(",")}</span>
    </div>
  );
}

describe("useLocalListNavigation", () => {
  afterEach(() => cleanup());

  beforeEach(() => {
    vi.spyOn(window, "requestAnimationFrame").mockImplementation((cb) => {
      window.setTimeout(() => cb(0), 0);
      return 0;
    });
  });

  it("navigue dans une liste locale depuis un champ de recherche", async () => {
    render(<LocalListHarness />);
    const search = screen.getByTestId("search");

    search.focus();
    fireEvent.keyDown(search, { key: "ArrowDown" });
    await waitFor(() => expect(document.activeElement).toBe(screen.getByTestId("two")));

    fireEvent.keyDown(screen.getByTestId("two"), { key: "Enter" });
    expect(screen.getByTestId("selected").textContent).toBe("two");
  });

  it("garde les touches gauche/droite et escape dans le scope local", async () => {
    render(<LocalListHarness />);
    const group = screen.getByTestId("group");

    group.focus();
    await waitFor(() => expect(group.getAttribute("data-local-nav-active")).toBe("true"));
    fireEvent.keyDown(group, { key: "ArrowRight" });
    fireEvent.keyDown(group, { key: "Enter" });
    fireEvent.keyDown(group, { key: "Escape" });

    expect(screen.getByTestId("events").textContent).toBe("open,toggle,escape");
  });
});
