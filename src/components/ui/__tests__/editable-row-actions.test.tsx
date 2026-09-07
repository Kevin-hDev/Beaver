import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { useRef } from "react";
import { describe, expect, it, vi } from "vitest";
import { AppSurfaceActivityProvider } from "@/components/layout/app-surface-activity";
import {
  EditableRowActions,
  useEditableRowActions,
} from "@/components/ui/editable-row-actions";

function Harness({ onRename, onDelete, active = true }: {
  onRename: (name: string) => Promise<void>;
  onDelete: () => Promise<void>;
  active?: boolean;
}) {
  return (
    <AppSurfaceActivityProvider active={active}>
      <EditableRow onRename={onRename} onDelete={onDelete} />
    </AppSurfaceActivityProvider>
  );
}

function EditableRow({ onRename, onDelete }: {
  onRename: (name: string) => Promise<void>;
  onDelete: () => Promise<void>;
}) {
  const rootRef = useRef<HTMLDivElement>(null);
  const controller = useEditableRowActions({ rootRef, value: "Initial", onRename, onDelete });
  return (
    <div ref={rootRef}>
      {controller.editing && (
        <input
          aria-label="Nom"
          value={controller.draft}
          onChange={(event) => controller.setDraft(event.target.value)}
        />
      )}
      <EditableRowActions
        controller={controller}
        renameLabel="Renommer"
        deleteLabel="Supprimer"
        confirmLabel="Valider"
        cancelLabel="Annuler"
      />
    </div>
  );
}

describe("EditableRowActions", () => {
  it("valide le renommage avec Entrée mais jamais au blur", async () => {
    const rename = vi.fn(() => Promise.resolve());
    render(<Harness onRename={rename} onDelete={() => Promise.resolve()} />);
    fireEvent.click(screen.getByRole("button", { name: "Renommer" }));
    const input = screen.getByRole("textbox", { name: "Nom" });
    fireEvent.change(input, { target: { value: "Nouveau" } });
    fireEvent.blur(input);
    expect(rename).not.toHaveBeenCalled();
    fireEvent.keyDown(window, { key: "Enter" });
    await waitFor(() => expect(rename).toHaveBeenCalledWith("Nouveau"));
  });

  it("Échap et le clic extérieur annulent sans sauvegarder", () => {
    const rename = vi.fn(() => Promise.resolve());
    render(<><Harness onRename={rename} onDelete={() => Promise.resolve()} /><button>Dehors</button></>);
    fireEvent.click(screen.getByRole("button", { name: "Renommer" }));
    fireEvent.keyDown(window, { key: "Escape" });
    expect(screen.queryByRole("textbox")).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole("button", { name: "Renommer" }));
    fireEvent.mouseDown(screen.getByRole("button", { name: "Dehors" }));
    expect(screen.queryByRole("textbox")).not.toBeInTheDocument();
    expect(rename).not.toHaveBeenCalled();
  });

  it("demande une confirmation avant la suppression", async () => {
    const remove = vi.fn(() => Promise.resolve());
    render(<Harness onRename={() => Promise.resolve()} onDelete={remove} />);
    fireEvent.click(screen.getByRole("button", { name: "Supprimer" }));
    expect(remove).not.toHaveBeenCalled();
    fireEvent.click(screen.getByRole("button", { name: "Valider" }));
    await waitFor(() => expect(remove).toHaveBeenCalledOnce());
  });

  it("conserve la confirmation ouverte inactive et confirme une fois au retour", async () => {
    const remove = vi.fn(() => Promise.resolve());
    const view = render(<Harness onRename={() => Promise.resolve()} onDelete={remove} />);
    fireEvent.click(screen.getByRole("button", { name: "Supprimer" }));
    expect(screen.getByRole("dialog")).toBeInTheDocument();

    void act(() => view.rerender(
      <Harness active={false} onRename={() => Promise.resolve()} onDelete={remove} />,
    ));
    fireEvent.keyDown(window, { key: "Enter" });
    fireEvent.keyDown(window, { key: "Escape" });
    fireEvent.mouseDown(document.body);
    expect(remove).not.toHaveBeenCalled();
    expect(screen.getByRole("dialog")).toBeInTheDocument();

    void act(() => view.rerender(
      <Harness active onRename={() => Promise.resolve()} onDelete={remove} />,
    ));
    fireEvent.keyDown(window, { key: "Enter" });
    await waitFor(() => expect(remove).toHaveBeenCalledOnce());
  });
});
