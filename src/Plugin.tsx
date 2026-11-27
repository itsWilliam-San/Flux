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
  const [plugins, setPlugins] = useState<Record<string, PluginInfo>>({});
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [hasStoredNetwork, setHasStoredNetwork] = useState(false);

  const fetchPlugins = useCallback(async () => {
    setLoading(true);
    try {
      const result = await invoke<Record<string, PluginDescription>>(
        "update_plugins_list",
      );

      setPlugins((prev) => {
        const next: Record<string, PluginInfo> = {};

        for (const [id, description] of Object.entries(result)) {
          const existing = prev[id];
          next[id] = existing
            ? { ...existing, description }
            : { description, loaded: false };
        }

        return next;
      });
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

          setPlugins((prev) => {
            const current = prev[pluginId];
            if (!current) {
              return prev;
            }

            return {
              ...prev,
              [pluginId]: {
                ...current,
                loaded: pluginState === PluginState.Loaded,
              },
            };
          });
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
    for (const p of Object.values(plugins)) {
      for (const tag of p.description.manifest.tags) {
        if (tag.trim()) tagSet.add(tag.trim());
      }
    }
    return Array.from(tagSet).sort();
  }, [plugins]);

  // Filter plugins by both search query and active tag
  const filtered = useMemo(() => {
    const result: Record<string, PluginInfo> = {};

    for (const [key, value] of Object.entries(plugins)) {
      if (
        selectedTag &&
        !value.description.manifest.tags.includes(selectedTag)
      ) {
        continue;
      } else {
        if (matchesSearch(value.description, search)) {
          result[key] = value;
        }
      }
    }

    return result;
  }, [plugins, search, selectedTag]);

  // Keep selection valid when filtered results change
  useEffect(() => {
    setSelectedId((current) => {
      if (current && filtered[current]) {
        return current;
      }
      const firstKey = Object.keys(filtered)[0];
      return firstKey ?? null;
    });
  }, [filtered]);

  const selected: PluginInfo | null = selectedId
    ? (filtered[selectedId] ?? null)
    : null;

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
            plugins={Object.values(filtered)}
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
