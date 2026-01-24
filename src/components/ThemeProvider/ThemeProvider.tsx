import { createContext, useContext, useEffect, useSyncExternalStore } from "react";

type Theme = "dark" | "light" | "system";

type ThemeProviderProps = {
  children: React.ReactNode;
  defaultTheme?: Theme;
  storageKey?: string;
};

type ThemeProviderState = {
  theme: Theme;
  setTheme: (theme: Theme) => void;
  resolvedTheme: "dark" | "light";
};

const initialState: ThemeProviderState = {
  theme: "system",
  setTheme: () => null,
  resolvedTheme: "light",
};

const ThemeProviderContext = createContext<ThemeProviderState>(initialState);

function getSystemTheme(): "dark" | "light" {
  if (typeof window !== "undefined") {
    return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
  }
  return "light";
}

function subscribeToTheme(callback: () => void) {
  const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
  mediaQuery.addEventListener("change", callback);
  return () => mediaQuery.removeEventListener("change", callback);
}

function useStoredTheme(storageKey: string, defaultTheme: Theme): [Theme, (t: Theme) => void] {
  const theme = useSyncExternalStore(
    (callback) => {
      window.addEventListener("storage", callback);
      return () => window.removeEventListener("storage", callback);
    },
    () => (localStorage.getItem(storageKey) as Theme) || defaultTheme,
    () => defaultTheme
  );

  const setTheme = (newTheme: Theme) => {
    localStorage.setItem(storageKey, newTheme);
    window.dispatchEvent(new StorageEvent("storage", { key: storageKey }));
  };

  return [theme, setTheme];
}

export function ThemeProvider({
  children,
  defaultTheme = "system",
  storageKey = "db-sync-theme",
  ...props
}: ThemeProviderProps) {
  const [theme, setTheme] = useStoredTheme(storageKey, defaultTheme);

  const systemTheme = useSyncExternalStore(
    subscribeToTheme,
    getSystemTheme,
    (): "dark" | "light" => "light"
  );

  const resolvedTheme = theme === "system" ? systemTheme : theme;

  useEffect(() => {
    const root = window.document.documentElement;
    root.classList.remove("light", "dark");
    root.classList.add(resolvedTheme);
  }, [resolvedTheme]);

  const value = {
    theme,
    setTheme: (theme: Theme) => {
      localStorage.setItem(storageKey, theme);
      setTheme(theme);
    },
    resolvedTheme,
  };

  return (
    <ThemeProviderContext.Provider {...props} value={value}>
      {children}
    </ThemeProviderContext.Provider>
  );
}

export function useTheme() {
  const context = useContext(ThemeProviderContext);

  if (context === undefined) throw new Error("useTheme must be used within a ThemeProvider");

  return context;
}
