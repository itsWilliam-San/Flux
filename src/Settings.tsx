import { useTranslation } from "react-i18next";
import { cn } from "@/lib/utils";
import { useState, useEffect } from "react";
import {
    Settings2,
    Languages,
    Sliders,
    Sun,
    Moon,
    Laptop,
    ArrowLeft,
    Check,
} from "lucide-react";
import { ButtonGroup } from "@/components/ui/button-group";
import { Button } from "@/components/ui/button";
import { Themes, useTheme } from "./useTheme";
import { invoke } from "@tauri-apps/api/core";

export const enum SettingsCategory {
    Appearance = "appearance",
    Language = "language",
    Behavior = "behavior",
}

interface SettingsWindowProps {
    onClose: () => void;
    onDecorationsChange?: (enabled: boolean) => void;
}

export default function SettingsWindow({
    onClose,
    onDecorationsChange,
}: SettingsWindowProps) {
    const { t, i18n } = useTranslation();
    const { theme, setTheme } = useTheme();

    const [activeCategory, setActiveCategory] = useState<SettingsCategory>(
        SettingsCategory.Appearance,
    );

    const [decorations, setDecorationsState] = useState<boolean>(() => {
        return localStorage.getItem("window_decorations") === "true";
    });

    const [closeAction, setCloseActionState] = useState<string>(() => {
        return localStorage.getItem("close_action") || "exit";
    });

    useEffect(() => {
        localStorage.setItem("window_decorations", String(decorations));
        invoke("set_window_decorations", { decorations }).catch((err) => {
            console.error("Failed to set window decorations:", err);
        });
        if (onDecorationsChange) {
            onDecorationsChange(decorations);
        }
    }, [decorations, onDecorationsChange]);

    const handleLanguageChange = (lang: string) => {
        i18n.changeLanguage(lang);
    };

    const handleCloseActionChange = (action: string) => {
        setCloseActionState(action);
        localStorage.setItem("close_action", action);
    };

    const categories = [
        {
            id: SettingsCategory.Appearance,
            label: t("settings.categories.appearance"),
            icon: Settings2,
        },
        {
            id: SettingsCategory.Language,
            label: t("settings.categories.language"),
            icon: Languages,
        },
        {
            id: SettingsCategory.Behavior,
            label: t("settings.categories.behavior"),
            icon: Sliders,
        },
    ];

    return (
        <div className="flex h-screen w-screen bg-background text-foreground overflow-hidden">
            {/* Sidebar */}
            <aside className="flex flex-col h-full w-[260px] shrink-0 border-r border-border p-4 bg-muted/20">
                <div className="mb-4 flex items-center justify-between">
                    <Button
                        type="button"
                        variant="ghost"
                        size="sm"
                        onClick={onClose}
                        className="gap-2 text-xs font-medium text-muted-foreground hover:text-foreground cursor-pointer"
                    >
                        <ArrowLeft className="size-4" />
                        {t("settings.back_to_plugins")}
                    </Button>
                </div>

                <div className="mb-2 px-2 text-xs font-semibold uppercase tracking-wider text-muted-foreground">
                    {t("settings.title")}
                </div>

                <nav className="flex flex-col gap-1">
                    {categories.map((cat) => {
                        const selected = activeCategory === cat.id;
                        const Icon = cat.icon;

                        return (
                            <button
                                key={cat.id}
                                type="button"
                                onClick={() => setActiveCategory(cat.id)}
                                className={cn(
                                    "flex items-center gap-3 w-full rounded-md px-3 py-2 text-sm font-medium transition-colors text-left cursor-pointer",
                                    selected
                                        ? "bg-primary text-primary-foreground"
                                        : "hover:bg-muted/60 text-muted-foreground hover:text-foreground",
                                )}
                            >
                                <Icon className="size-4 shrink-0" />
                                <span>{cat.label}</span>
                            </button>
                        );
                    })}
                </nav>
            </aside>

            {/* Content Area */}
            <main className="flex flex-1 flex-col p-8 overflow-y-auto max-w-3xl">
                {activeCategory === SettingsCategory.Appearance && (
                    <div className="flex flex-col gap-6">
                        <div>
                            <h1 className="text-xl font-bold tracking-tight">
                                {t("settings.appearance.title")}
                            </h1>
                            <p className="text-sm text-muted-foreground mt-1">
                                {t("settings.appearance.theme_desc")}
                            </p>
                        </div>

                        {/* Theme Mode Card */}
                        <div className="flex flex-col gap-3 rounded-lg border border-border p-4 bg-card">
                            <div className="flex flex-row items-center justify-between gap-4">
                                <div>
                                    <h2 className="text-sm font-semibold text-foreground">
                                        {t("settings.appearance.theme_title")}
                                    </h2>
                                    <p className="text-xs text-muted-foreground mt-0.5">
                                        {t("settings.appearance.theme_desc")}
                                    </p>
                                </div>

                                <ButtonGroup>
                                    <Button
                                        size="sm"
                                        variant={theme === Themes.Dark ? "default" : "outline"}
                                        onClick={() => setTheme(Themes.Dark)}
                                        className="gap-1.5 cursor-pointer"
                                    >
                                        <Moon className="size-3.5" />
                                        {t("settings.appearance.dark")}
                                    </Button>
                                    <Button
                                        size="sm"
                                        variant={theme === Themes.Light ? "default" : "outline"}
                                        onClick={() => setTheme(Themes.Light)}
                                        className="gap-1.5 cursor-pointer"
                                    >
                                        <Sun className="size-3.5" />
                                        {t("settings.appearance.light")}
                                    </Button>
                                    <Button
                                        size="sm"
                                        variant={theme === Themes.System ? "default" : "outline"}
                                        onClick={() => setTheme(Themes.System)}
                                        className="gap-1.5 cursor-pointer"
                                    >
                                        <Laptop className="size-3.5" />
                                        {t("settings.appearance.auto")}
                                    </Button>
                                </ButtonGroup>
                            </div>
                        </div>

                        {/* Window Decorations Card */}
                        <div className="flex flex-col gap-3 rounded-lg border border-border p-4 bg-card">
                            <div className="flex flex-row items-center justify-between gap-4">
                                <div>
                                    <h2 className="text-sm font-semibold text-foreground">
                                        {t("settings.appearance.decorations_title")}
                                    </h2>
                                    <p className="text-xs text-muted-foreground mt-0.5">
                                        {t("settings.appearance.decorations_desc")}
                                    </p>
                                </div>

                                <ButtonGroup>
                                    <Button
                                        size="sm"
                                        variant={decorations ? "default" : "outline"}
                                        onClick={() => setDecorationsState(true)}
                                        className="cursor-pointer"
                                    >
                                        {t("settings.appearance.enabled")}
                                    </Button>
                                    <Button
                                        size="sm"
                                        variant={!decorations ? "default" : "outline"}
                                        onClick={() => setDecorationsState(false)}
                                        className="cursor-pointer"
                                    >
                                        {t("settings.appearance.disabled")}
                                    </Button>
                                </ButtonGroup>
                            </div>
                        </div>
                    </div>
                )}

                {activeCategory === SettingsCategory.Language && (
                    <div className="flex flex-col gap-6">
                        <div>
                            <h1 className="text-xl font-bold tracking-tight">
                                {t("settings.language.title")}
                            </h1>
                            <p className="text-sm text-muted-foreground mt-1">
                                {t("settings.language.desc")}
                            </p>
                        </div>

                        <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                            <div
                                onClick={() => handleLanguageChange("pt-BR")}
                                className={cn(
                                    "flex items-center justify-between p-4 rounded-lg border transition-all cursor-pointer",
                                    i18n.language === "pt-BR"
                                        ? "border-primary bg-primary/10 shadow-xs"
                                        : "border-border bg-card hover:bg-muted/50",
                                )}
                            >
                                <div className="flex items-center gap-3">
                                    <span className="text-2xl">🇧🇷</span>
                                    <div>
                                        <p className="font-semibold text-sm">
                                            {t("settings.language.pt_br")}
                                        </p>
                                        <p className="text-xs text-muted-foreground">
                                            Português do Brasil
                                        </p>
                                    </div>
                                </div>
                                {i18n.language === "pt-BR" && (
                                    <Check className="size-4 text-primary" />
                                )}
                            </div>

                            <div
                                onClick={() => handleLanguageChange("en-US")}
                                className={cn(
                                    "flex items-center justify-between p-4 rounded-lg border transition-all cursor-pointer",
                                    i18n.language === "en-US"
                                        ? "border-primary bg-primary/10 shadow-xs"
                                        : "border-border bg-card hover:bg-muted/50",
                                )}
                            >
                                <div className="flex items-center gap-3">
                                    <span className="text-2xl">🇺🇸</span>
                                    <div>
                                        <p className="font-semibold text-sm">
                                            {t("settings.language.en_us")}
                                        </p>
                                        <p className="text-xs text-muted-foreground">
                                            English (United States)
                                        </p>
                                    </div>
                                </div>
                                {i18n.language === "en-US" && (
                                    <Check className="size-4 text-primary" />
                                )}
                            </div>
                        </div>
                    </div>
                )}

                {activeCategory === SettingsCategory.Behavior && (
                    <div className="flex flex-col gap-6">
                        <div>
                            <h1 className="text-xl font-bold tracking-tight">
                                {t("settings.behavior.title")}
                            </h1>
                            <p className="text-sm text-muted-foreground mt-1">
                                {t("settings.behavior.close_desc")}
                            </p>
                        </div>

                        <div className="flex flex-col gap-3 rounded-lg border border-border p-4 bg-card">
                            <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-4">
                                <div>
                                    <h2 className="text-sm font-semibold text-foreground">
                                        {t("settings.behavior.close_title")}
                                    </h2>
                                    <p className="text-xs text-muted-foreground mt-0.5">
                                        {t("settings.behavior.close_desc")}
                                    </p>
                                </div>

                                <ButtonGroup>
                                    <Button
                                        size="sm"
                                        variant={closeAction === "exit" ? "default" : "outline"}
                                        onClick={() => handleCloseActionChange("exit")}
                                        className="cursor-pointer"
                                    >
                                        {t("settings.behavior.close_exit")}
                                    </Button>
                                    <Button
                                        size="sm"
                                        variant={closeAction === "tray" ? "default" : "outline"}
                                        onClick={() => handleCloseActionChange("tray")}
                                        className="cursor-pointer"
                                    >
                                        {t("settings.behavior.close_tray")}
                                    </Button>
                                </ButtonGroup>
                            </div>
                        </div>
                    </div>
                )}
            </main>
        </div>
    );
}
