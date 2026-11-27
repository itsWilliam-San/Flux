import { useCallback, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import PluginDetails from "./components/plugins/PluginDetails";
import PluginList from "./components/plugins/PluginList";
import type { PluginDescription, PluginInfo } from "./types/plugins";
import { PluginState } from "./types/plugins";
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
  const [plugins, setPlugins] = useState<PluginInfo[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [hasStoredNetwork, setHasStoredNetwork] = useState(false);

  const fetchPlugins = useCallback(async () => {
    setLoading(true);
    try {
      const result = await invoke<PluginDescription[]>("update_plugins_list");

      let mappedResult: PluginInfo[] = result.map((desc): PluginInfo => ({
        description: desc,
        loaded: false,
      }));

      setPlugins(mappedResult);
    } catch (error) {
      console.error("Error while loading plugins:", error);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    let unlistenFn: UnlistenFn | undefined;
    let active = true;

    async function init() {
      const unlisten = await listen<[string, PluginState]>(
        "plugin-state-changed",
        (event) => {
          const [pluginId, pluginState] = event.payload;
          console.log(
            "A plugin changed state to plugin changed state to %s",
            pluginState,
          );
          setPlugins((prev) =>
            prev.map((plugin) =>
              plugin.description.manifest.id === pluginId
                ? { ...plugin, loaded: pluginState === PluginState.Loaded }
                : plugin,
            ),
          );
        },
      );

      unlistenFn = unlisten;

      await fetchPlugins();

      if (!active) {
        unlisten();
        return;
      }

      invoke("start_plugins");
    }

    init();

    return () => {
      active = false;
      unlistenFn?.();
    };
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
      for (const tag of p.description.manifest.tags) {
        if (tag.trim()) tagSet.add(tag.trim());
      }
    }
    return Array.from(tagSet).sort();
  }, [plugins]);

  // Filter plugins by both search query and active tag
  const filtered = useMemo(() => {
    return plugins.filter((plugin) => {
      if (
        selectedTag &&
        !plugin.description.manifest.tags.includes(selectedTag)
      ) {
        return false;
      }
      return matchesSearch(plugin.description, search);
    });
  }, [plugins, search, selectedTag]);

  // Keep selection valid when filtered results change
  useEffect(() => {
    setSelectedId((current) => {
      if (filtered.some((p) => p.description.manifest.id === current))
        return current;
      return filtered[0]?.description.manifest.id ?? null;
    });
  }, [filtered]);

  const selected =
    filtered.find((p) => p.description.manifest.id === selectedId) ?? null;

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
            onSelectTag={handleTagSelect}
          />
        </div>
      )}
    </div>
  );
}
