import { createContext, useContext, useState, useEffect, ReactNode } from "react";

export const enum Themes {
    Dark,
    Light,
    System,
}

interface ThemeContextType {
    theme: Themes;
    setTheme: (theme: Themes) => void;
}

const ThemeContext = createContext<ThemeContextType | undefined>(undefined);

export function applyTheme(theme: Themes) {
    if (typeof window === "undefined") return;
    const root = document.documentElement;
    const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");

    if (theme === Themes.Dark) {
        root.classList.add("dark");
    } else if (theme === Themes.Light) {
        root.classList.remove("dark");
    } else {
        if (mediaQuery.matches) {
            root.classList.add("dark");
        } else {
            root.classList.remove("dark");
        }
    }
}

export function getInitialTheme(): Themes {
    if (typeof window === "undefined") return Themes.System;
    const saved = localStorage.getItem("theme");
    return saved === "dark" ? Themes.Dark : saved === "light" ? Themes.Light : Themes.System;
}

// Immediately apply theme on module load to prevent light/dark flash
applyTheme(getInitialTheme());

export function ThemeProvider({ children }: { children: ReactNode }) {
    const [theme, setTheme] = useState<Themes>(getInitialTheme);

    useEffect(() => {
        const root = document.documentElement;
        const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");

        const applyDarkTheme = (isDark: boolean) => {
            if (isDark) {
                root.classList.add("dark");
            } else {
                root.classList.remove("dark");
            }
        };

        if (theme === Themes.Dark) {
            localStorage.setItem("theme", "dark");
            applyDarkTheme(true);
        } else if (theme === Themes.Light) {
            localStorage.setItem("theme", "light");
            applyDarkTheme(false);
        } else {
            localStorage.removeItem("theme");
            applyDarkTheme(mediaQuery.matches);

            const handleSystemChange = (e: MediaQueryListEvent) => {
                applyDarkTheme(e.matches);
            };

            mediaQuery.addEventListener("change", handleSystemChange);
            return () => {
                mediaQuery.removeEventListener("change", handleSystemChange);
            };
        }
    }, [theme]);

    return (
        <ThemeContext.Provider value={{ theme, setTheme }}>
            {children}
        </ThemeContext.Provider>
    );
}

export function useTheme() {
    const context = useContext(ThemeContext);
    if (!context) {
        throw new Error("useTheme must be used within a ThemeProvider");
    }
    return context;
}
