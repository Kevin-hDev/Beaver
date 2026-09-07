import { createContext, useContext, type ReactNode } from "react";

// Hors d'une surface persistante, l'écran reste actif par défaut (réglages, heartbeat, etc.).
const AppSurfaceActivityContext = createContext(true);

export function AppSurfaceActivityProvider(props: {
  active: boolean;
  children: ReactNode;
}) {
  return (
    <AppSurfaceActivityContext.Provider value={props.active}>
      {props.children}
    </AppSurfaceActivityContext.Provider>
  );
}

export function useAppSurfaceActive(): boolean {
  return useContext(AppSurfaceActivityContext);
}
