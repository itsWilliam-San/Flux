import { useMemo } from "react";
import SearchComponent from "../Search";
import { Button } from "../ui/button";
import { cn } from "@/lib/utils";
import type {
    PluginCategory,
    PluginDescriptor,
    PluginOrigin,
} from "@/types/plugins";
import { Lock, Settings } from "lucide-react";
import { useTranslation } from "react-i18next";

const DEFAULT_CATEGORIES: { id: PluginCategory; labelKey: string; fallback: string }[] = [
    { id: "extraction", labelKey: "plugin_list.extraction", fallback: "Extraction" },
    { id: "exporter", labelKey: "plugin_list.exporter", fallback: "Exporter" },
];

const ORIGINS: { id: PluginOrigin; labelKey: string; fallback: string }[] = [
    { id: "local", labelKey: "plugin_list.local", fallback: "Local" },
    { id: "bridge", labelKey: "plugin_list.bridge", fallback: "Bridge" },
];

interface PluginListProps {
    plugins: PluginDescriptor[];
    selectedId: string | null;
    search: string;
    loading: boolean;
    hasStoredNetwork: boolean;
    showSettingsButton?: boolean;
    onSearchChange: (value: string) => void;
    onRefresh: () => void;
    onSelect: (id: string) => void;
    onOpenSettings?: () => void;
}

export default function PluginList({
    plugins,
    selectedId,
    search,
    loading,
    hasStoredNetwork,
    showSettingsButton = false,
    onSearchChange,
    onRefresh,
    onSelect,
    onOpenSettings,
}: PluginListProps) {
    const { t } = useTranslation();

    const categories = useMemo(() => {
        const catMap = new Map<string, string>();
        for (const cat of DEFAULT_CATEGORIES) {
            catMap.set(cat.id, cat.fallback);
        }
        for (const plugin of plugins) {
            const cat = plugin.manifest.category;
            if (!catMap.has(cat)) {
                const label = cat.charAt(0).toUpperCase() + cat.slice(1);
                catMap.set(cat, label);
            }
        }
        return Array.from(catMap.entries()).map(([id, fallback]) => ({
            id: id as PluginCategory,
            fallback,
        }));
    }, [plugins]);

    return (
        <aside className="flex h-full w-[280px] shrink-0 flex-col gap-3 overflow-hidden border-r border-border p-4 bg-muted/10">
            {showSettingsButton && onOpenSettings ? (
                <div className="flex items-center justify-between pb-1 border-b border-border">
                    <span className="text-xs font-semibold text-muted-foreground uppercase tracking-wider">
                        Flux
                    </span>
                    <button
                        type="button"
                        onClick={onOpenSettings}
                        className="rounded p-1 text-muted-foreground hover:bg-muted hover:text-foreground transition-colors cursor-pointer"
                        title={t("settings.title")}
                    >
                        <Settings className="size-4" />
                    </button>
                </div>
            ) : null}

            <SearchComponent value={search} onChange={onSearchChange} />

            <Button
                type="button"
                size="sm"
                onClick={onRefresh}
                disabled={loading}
                className="cursor-pointer"
            >
                {loading ? t("common.loading") : t("common.update")}
            </Button>

            <div className="min-h-0 flex-1 overflow-y-auto">
                {categories.map((category) => {
                    const catTitle =
                        category.id === "extraction"
                            ? t("plugin_list.extraction")
                            : category.id === "exporter"
                            ? t("plugin_list.exporter")
                            : category.fallback;

                    return (
                        <section key={category.id} className="mb-4">
                            <div className="flex items-center justify-between mb-1">
                                <h2 className="text-sm font-semibold tracking-wide text-foreground uppercase">
                                    {catTitle}
                                </h2>
                                {category.id === "exporter" && !hasStoredNetwork ? (
                                    <span
                                        className="flex items-center text-amber-500/80 text-[10px] shrink-0 font-normal"
                                        title={t("common.blocked_title")}
                                    >
                                        <Lock className="size-3 mr-0.5" />
                                        {t("common.blocked")}
                                    </span>
                                ) : null}
                            </div>

                            {ORIGINS.map((origin) => {
                                const items = plugins.filter(
                                    (plugin) =>
                                        plugin.manifest.category === category.id &&
                                        plugin.origin === origin.id,
                                );

                                const originLabel =
                                    origin.id === "local"
                                        ? t("plugin_list.local")
                                        : t("plugin_list.bridge");

                                return (
                                    <div key={origin.id} className="mt-2">
                                        <h3 className="text-xs font-medium text-muted-foreground">
                                            {originLabel}
                                        </h3>
                                        {items.length === 0 ? (
                                            <p className="py-1 pl-2 text-xs text-muted-foreground">
                                                {t("common.none")}
                                            </p>
                                        ) : (
                                            <ul className="mt-1 flex flex-col gap-0.5">
                                                {items.map((plugin) => {
                                                    const muted =
                                                        plugin.manifest.category ===
                                                        "exporter" &&
                                                        !hasStoredNetwork;
                                                    const selected =
                                                        selectedId ===
                                                        plugin.manifest.id;

                                                    return (
                                                        <li key={plugin.manifest.id}>
                                                            <button
                                                                type="button"
                                                                onClick={() =>
                                                                    onSelect(
                                                                        plugin.manifest
                                                                            .id,
                                                                    )
                                                                }
                                                                className={cn(
                                                                    "w-full rounded-md px-2 py-1.5 text-left text-sm transition-colors cursor-pointer",
                                                                    selected
                                                                        ? "bg-muted text-foreground font-medium"
                                                                        : "hover:bg-muted/60 text-muted-foreground hover:text-foreground",
                                                                    muted &&
                                                                        "opacity-60",
                                                                )}
                                                            >
                                                                <div className="flex items-center justify-between gap-1.5">
                                                                    <span className="truncate">
                                                                        {
                                                                            plugin
                                                                                .manifest
                                                                                .name
                                                                        }
                                                                    </span>
                                                                    {muted ? (
                                                                        <Lock className="size-3 text-amber-500/70 shrink-0" />
                                                                    ) : null}
                                                                </div>
                                                            </button>
                                                        </li>
                                                    );
                                                })}
                                            </ul>
                                        )}
                                    </div>
                                );
                            })}
                        </section>
                    );
                })}
            </div>
        </aside>
    );
}
