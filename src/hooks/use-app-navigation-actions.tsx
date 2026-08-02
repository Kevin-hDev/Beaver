import { createContext, useContext } from "react";

interface AppNavigationActions {
  openFileAccessSettings: () => void;
}

const Context = createContext<AppNavigationActions | null>(null);

interface ProviderProps extends AppNavigationActions {
  children?: React.ReactNode;
}

export function AppNavigationActionsProvider({
  children,
  openFileAccessSettings,
}: ProviderProps) {
  return (
    <Context.Provider value={{ openFileAccessSettings }}>
      {children}
    </Context.Provider>
  );
}

export function useAppNavigationActions(): AppNavigationActions {
  const actions = useContext(Context);
  if (!actions) {
    throw new Error("App navigation actions are unavailable");
  }
  return actions;
}
