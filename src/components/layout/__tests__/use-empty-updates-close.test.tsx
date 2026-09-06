import { useCallback, useRef, useState } from "react";
import { cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, it } from "vitest";
import { useEmptyUpdatesClose } from "../use-empty-updates-close";

afterEach(cleanup);
function Surface({ count }: { count: number }) {
  const root = useRef<HTMLDivElement>(null);
  const [open, setOpen] = useState(false);
  const close = useCallback(() => setOpen(false), []);
  useEmptyUpdatesClose(count, open, close, root);
  return <div ref={root}>
    <div className="window-toolbar"><button>Navigation</button>
      {count > 0 && <button onClick={() => setOpen(true)}>Suivi</button>}
    </div>
    {open && <section aria-label="Installations"><button onClick={close}>Fermer</button></section>}
  </div>;
}
it("closes when the last result disappears, restores focus and does not reopen spontaneously", () => {
  const view = render(<Surface count={1} />);
  fireEvent.click(screen.getByText("Suivi"));
  screen.getByText("Fermer").focus();
  expect(screen.getByRole("region")).toBeVisible();
  view.rerender(<Surface count={0} />);
  expect(screen.queryByRole("region")).toBeNull();
  expect(screen.getByText("Navigation")).toHaveFocus();
  view.rerender(<Surface count={1} />);
  expect(screen.queryByRole("region")).toBeNull();
});
