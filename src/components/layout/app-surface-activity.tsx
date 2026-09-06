import { createContext, useContext, type ReactNode } from "react";

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
