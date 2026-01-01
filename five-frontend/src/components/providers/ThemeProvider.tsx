"use client";

import { useThemeStore } from "@/stores/theme-store";
import { useEffect, useState } from "react";

export function ThemeProvider({ children }: { children: React.ReactNode }) {
    const { theme } = useThemeStore();
    const [mounted, setMounted] = useState(false);

    useEffect(() => {
        setMounted(true);
    }, []);

    useEffect(() => {
        const root = window.document.documentElement;

        // Remove old theme class
        root.classList.remove("light", "dark");

        // Add new theme class
        root.classList.add(theme);

        // Also set style color-scheme for browser native UI controls
        root.style.colorScheme = theme;
    }, [theme]);

    // Prevent hydration mismatch by rendering nothing or a loader until mounted?
    // Or just render children. Converting theme requires client JS anyway.
    // For seamless ssr, we usually render children but the class might be wrong 
    // until hydration. To avoid flash, we could use a script, but for now 
    // a small flash or default dark is acceptable for this step.
    if (!mounted) {
        return <>{children}</>;
    }

    return <>{children}</>;
}
