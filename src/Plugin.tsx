import { useCallback, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import PluginDetails from "./components/plugins/PluginDetails";
import PluginList from "./components/plugins/PluginList";
import type { PluginDescription } from "./types/plugins";
import { Loader2 } from "lucide-react";
import { useTranslation } from "react-i18next";

interface PluginsWindowProps {
    nativeDecorations?: boolean;
    onOpenSettings?: () => void;
}

function matchesSearch(plugin: PluginDescription, query: string): boolean {
    const q = query.trim().toLowerCase();
    if (!q) return true;

    const { manifest } = plugin;
    return (
        manifest.name.toLowerCase().includes(q) ||
        manifest.description.toLowerCase().includes(q) ||
        manifest.tags.some((tag) => tag.toLowerCase().includes(q))
    );
}

export default function PluginsWindow({
    nativeDecorations = false,
    onOpenSettings,
}: PluginsWindowProps) {
    const { t } = useTranslation();
    const [search, setSearch] = useState("");
    const [selectedTag, setSelectedTag] = useState<string | null>(null);
    const [loading, setLoading] = useState(true);
    const [plugins, setPlugins] = useState<PluginDescription[]>([]);
    const [selectedId, setSelectedId] = useState<string | null>(null);
    const [hasStoredNetwork, setHasStoredNetwork] = useState(false);

    const checkNetworkStatus = useCallback(async () => {
        try {
            const hasData = await invoke<boolean>("has_network_data");
            setHasStoredNetwork(hasData);
        } catch (error) {
            console.error("Error while checking network status:", error);
        }
    }, []);

    const fetchPlugins = useCallback(async () => {
        setLoading(true);
        try {
            const result = await invoke<PluginDescription[]>("list_plugins");
            setPlugins(result);
            await checkNetworkStatus();
        } catch (error) {
            console.error("Error while loading plugins:", error);
        } finally {
            setLoading(false);
        }
    }, [checkNetworkStatus]);

    useEffect(() => {
        fetchPlugins();
    }, [fetchPlugins]);

    // Listen for network data state changes from the backend
    useEffect(() => {
        let unlistenFn: UnlistenFn | undefined;
        listen<boolean>("network-data-changed", (event) => {
            setHasStoredNetwork(Boolean(event.payload));
        }).then((unlisten) => {
            unlistenFn = unlisten;
        });
        return () => {
            unlistenFn?.();
        };
    }, []);

    // Derive all unique tags from currently loaded plugins
    const availableTags = useMemo(() => {
        const tagSet = new Set<string>();
        for (const p of plugins) {
            for (const tag of p.manifest.tags) {
                if (tag.trim()) tagSet.add(tag.trim());
            }
        }
        return Array.from(tagSet).sort();
    }, [plugins]);

    // Filter plugins by both search query and active tag
    const filtered = useMemo(() => {
        return plugins.filter((plugin) => {
            if (selectedTag && !plugin.manifest.tags.includes(selectedTag)) {
                return false;
            }
            return matchesSearch(plugin, search);
        });
    }, [plugins, search, selectedTag]);

    // Keep selection valid when filtered results change
    useEffect(() => {
        setSelectedId((current) => {
            if (filtered.some((p) => p.manifest.id === current)) return current;
            return filtered[0]?.manifest.id ?? null;
        });
    }, [filtered]);

    const selected =
        filtered.find((p) => p.manifest.id === selectedId) ?? null;

    const handleTagSelect = useCallback((tag: string | null) => {
        setSelectedTag(tag);
    }, []);

    return (
        <div className="flex flex-1 min-h-0 h-full w-full flex-col bg-background text-foreground">
            {loading ? (
                <div className="flex min-h-0 flex-1 flex-col justify-center items-center gap-5">
                    <Loader2 className="size-10 animate-spin text-primary" />
                    <span className="text-sm text-muted-foreground">
                        {t("window.loading_plugins")}
                    </span>
                </div>
            ) : (
                <div className="flex min-h-0 flex-1 flex-row">
                    <PluginList
                        plugins={filtered}
                        availableTags={availableTags}
                        selectedTag={selectedTag}
                        selectedId={selectedId}
                        search={search}
                        loading={loading}
                        hasStoredNetwork={hasStoredNetwork}
                        onSearchChange={setSearch}
                        onTagSelect={handleTagSelect}
                        onRefresh={fetchPlugins}
                        onSelect={setSelectedId}
                        showSettingsButton={nativeDecorations}
                        onOpenSettings={onOpenSettings}
                    />
                    <PluginDetails
                        plugin={selected}
                        hasStoredNetwork={hasStoredNetwork}
                        onActionSuccess={checkNetworkStatus}
                        onSelectTag={handleTagSelect}
                    />
                </div>
            )}
        </div>
    );
}
