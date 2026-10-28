import { useMemo } from "react";
import SearchComponent from "../Search";
import { Button } from "../ui/button";
import { cn } from "@/lib/utils";
import type {
    PluginCategory,
    PluginDescriptor,
    PluginOrigin,
} from "@/types/plugins";

const DEFAULT_CATEGORIES: { id: PluginCategory; label: string }[] = [
    { id: "extraction", label: "Extraction" },
    { id: "exporter", label: "Exporter" },
];

const ORIGINS: { id: PluginOrigin; label: string }[] = [
    { id: "local", label: "Local" },
    { id: "bridge", label: "Bridge" },
];

interface PluginListProps {
    plugins: PluginDescriptor[];
    selectedId: string | null;
    search: string;
    loading: boolean;
    hasStoredNetwork: boolean;
    onSearchChange: (value: string) => void;
    onRefresh: () => void;
    onSelect: (id: string) => void;
}

export default function PluginList({
    plugins,
    selectedId,
    search,
    loading,
    hasStoredNetwork,
    onSearchChange,
    onRefresh,
    onSelect,
}: PluginListProps) {
    const categories = useMemo(() => {
        const catMap = new Map<string, string>();
        for (const cat of DEFAULT_CATEGORIES) {
            catMap.set(cat.id, cat.label);
        }
        for (const plugin of plugins) {
            const cat = plugin.manifest.category;
            if (!catMap.has(cat)) {
                const label = cat.charAt(0).toUpperCase() + cat.slice(1);
                catMap.set(cat, label);
            }
        }
        return Array.from(catMap.entries()).map(([id, label]) => ({
            id: id as PluginCategory,
            label,
        }));
    }, [plugins]);

    return (
        <aside className="flex h-full w-[280px] shrink-0 flex-col gap-3 overflow-hidden border-r border-border p-4">
            <SearchComponent value={search} onChange={onSearchChange} />
            <Button
                type="button"
                size="sm"
                onClick={onRefresh}
                disabled={loading}
            >
                {loading ? "Updating…" : "Update"}
            </Button>
            <div className="min-h-0 flex-1 overflow-y-auto">
                {categories.map((category) => (
                    <section key={category.id} className="mb-4">
                        <h2 className="text-sm font-semibold tracking-wide text-foreground uppercase">
                            {category.label}
                        </h2>
                        {ORIGINS.map((origin) => {
                            const items = plugins.filter(
                                (plugin) =>
                                    plugin.manifest.category === category.id &&
                                    plugin.origin === origin.id,
                            );

                            return (
                                <div key={origin.id} className="mt-2">
                                    <h3 className="text-xs font-medium text-muted-foreground">
                                        {origin.label}
                                    </h3>
                                    {items.length === 0 ? (
                                        <p className="py-1 pl-2 text-xs text-muted-foreground">
                                            None
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
                                                                "w-full rounded-md px-2 py-1.5 text-left text-sm transition-colors",
                                                                selected
                                                                    ? "bg-muted text-foreground"
                                                                    : "hover:bg-muted/60",
                                                                muted &&
                                                                "opacity-60",
                                                            )}
                                                        >
                                                            <span className="block truncate font-medium">
                                                                {
                                                                    plugin
                                                                        .manifest
                                                                        .name
                                                                }
                                                            </span>
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
                ))}
            </div>
        </aside>
    );
}
