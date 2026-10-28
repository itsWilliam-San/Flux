import { useCallback, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import ModalTitleBar from "./components/ModalTitleBar";
import PluginDetails from "./components/plugins/PluginDetails";
import PluginList from "./components/plugins/PluginList";
import type { PluginDescriptor } from "./types/plugins";
import { Loader2 } from "lucide-react";

interface PluginsWindowProps {
    onClose: () => void;
}

const HAS_STORED_NETWORK = false;

function matchesSearch(plugin: PluginDescriptor, query: string): boolean {
    const q = query.trim().toLowerCase();
    if (!q) {
        return true;
    }

    const { manifest } = plugin;
    return [manifest.name, manifest.description].some((value) =>
        value.toLowerCase().includes(q),
    );
}

export default function PluginsWindow({ onClose }: PluginsWindowProps) {
    const [search, setSearch] = useState("");
    const [loading, setLoading] = useState(true);
    const [plugins, setPlugins] = useState<PluginDescriptor[]>([]);
    const [selectedId, setSelectedId] = useState<string | null>(null);

    const fetchPlugins = useCallback(async () => {
        setLoading(true);
        try {
            const result = await invoke<PluginDescriptor[]>("list_plugins");
            setPlugins(result);
        } catch (error) {
            console.error("Error while loading plugins:", error);
        } finally {
            setLoading(false);
        }
    }, []);

    useEffect(() => {
        fetchPlugins();
    }, [fetchPlugins]);

    const filtered = useMemo(
        () => plugins.filter((plugin) => matchesSearch(plugin, search)),
        [plugins, search],
    );

    useEffect(() => {
        setSelectedId((current) => {
            if (filtered.some((plugin) => plugin.manifest.id === current)) {
                return current;
            }
            return filtered[0]?.manifest.id ?? null;
        });
    }, [filtered]);

    const selected =
        filtered.find((plugin) => plugin.manifest.id === selectedId) ?? null;

    return (
        <div className="fixed inset-0 z-1000 flex h-screen w-screen flex-col bg-background dark:bg-background">
            <ModalTitleBar onClose={onClose} />
            {loading ? (
                <div className="flex min-h-0 flex-1 flex-col justify-center items-center gap-5">
                    <Loader2 className="h-15 w-15 animate-spin" />
                    <span>Searching for local and bridge Plugins...</span>
                </div>
            ) : (
                <div className="flex min-h-0 flex-1 flex-row">
                    <PluginList
                        plugins={filtered}
                        selectedId={selectedId}
                        search={search}
                        loading={loading}
                        hasStoredNetwork={HAS_STORED_NETWORK}
                        onSearchChange={setSearch}
                        onRefresh={fetchPlugins}
                        onSelect={setSelectedId}
                    />
                    <PluginDetails
                        plugin={selected}
                        hasStoredNetwork={HAS_STORED_NETWORK}
                    />
                </div>
            )
            }
        </div >
    );
}
