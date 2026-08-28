import { createContext, useContext, type ReactNode } from "react";
import { useUpdateChecker } from "@/hooks/use-update-checker";

type UpdateController = ReturnType<typeof useUpdateChecker>;
const UpdateContext = createContext<UpdateController | null>(null);

export function UpdateProvider({ children }: { children: ReactNode }) {
  const controller = useUpdateChecker();
  return <UpdateContext.Provider value={controller}>{children}</UpdateContext.Provider>;
}

export function useUpdates(): UpdateController {
  const controller = useContext(UpdateContext);
  if (!controller) throw new Error("UpdateProvider missing");
  return controller;
}
