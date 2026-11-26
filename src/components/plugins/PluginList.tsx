import { useEffect } from "react";
import SearchComponent from "../Search";
import { Button } from "../ui/button";
import { cn } from "@/lib/utils";
import { PluginInfo } from "@/types/plugins";
import { Lock, Settings, Tag, X } from "lucide-react";
import { useTranslation } from "react-i18next";
import { listen, UnlistenFn } from "@tauri-apps/api/event";

interface PluginListProps {
  plugins: PluginInfo[];
  availableTags: string[];
  selectedTag: string | null;
  selectedId: string | null;
  search: string;
  loading: boolean;
  hasStoredNetwork: boolean;
  showSettingsButton?: boolean;
  onSearchChange: (value: string) => void;
  onTagSelect: (tag: string | null) => void;
  onRefresh: () => void;
  onSelect: (id: string) => void;
  onOpenSettings?: () => void;
}

export default function PluginList({
  plugins,
  availableTags,
  selectedTag,
  selectedId,
  search,
  loading,
  hasStoredNetwork,
  showSettingsButton = false,
  onSearchChange,
  onTagSelect,
  onRefresh,
  onSelect,
  onOpenSettings,
}: PluginListProps) {
  const { t } = useTranslation();

  // Auto-refresh when the backend requests an update
  useEffect(() => {
    let unlistenFn: UnlistenFn | undefined;
    listen<boolean>("should-update-plugin-list", () => {
      onRefresh();
    }).then((unlisten) => {
      unlistenFn = unlisten;
    });
    return () => {
      unlistenFn?.();
    };
  }, [onRefresh]);

  return (
    <aside className="flex h-full w-72 shrink-0 flex-col gap-3 overflow-hidden border-r border-border p-4 bg-muted/10">
      {/* App title & settings */}
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

      {/* Search Input */}
      <SearchComponent value={search} onChange={onSearchChange} />

      {/* Tag Filter Chips */}
      {availableTags.length > 0 && (
        <div className="flex items-center gap-1 overflow-x-auto pb-1 no-scrollbar text-xs">
          <button
            type="button"
            onClick={() => onTagSelect(null)}
            className={cn(
              "rounded-full px-2 py-0.5 whitespace-nowrap transition-colors cursor-pointer text-[11px] font-medium border",
              selectedTag === null
                ? "bg-primary text-primary-foreground border-primary"
                : "bg-background text-muted-foreground border-border hover:bg-muted hover:text-foreground",
            )}
          >
            {t("plugin_list.all_tags")}
          </button>
          {availableTags.map((tag) => {
            const isSelected = selectedTag === tag;
            return (
              <button
                key={tag}
                type="button"
                onClick={() => onTagSelect(isSelected ? null : tag)}
                className={cn(
                  "rounded-full px-2 py-0.5 whitespace-nowrap transition-colors cursor-pointer text-[11px] font-medium border inline-flex items-center gap-1",
                  isSelected
                    ? "bg-primary text-primary-foreground border-primary"
                    : "bg-background text-muted-foreground border-border hover:bg-muted hover:text-foreground",
                )}
              >
                <Tag className="size-2.5 opacity-70" />
                <span>{tag}</span>
                {isSelected && <X className="size-2.5" />}
              </button>
            );
          })}
        </div>
      )}

      {/* Refresh / Update Button */}
      <Button
        type="button"
        size="sm"
        onClick={onRefresh}
        disabled={loading}
        className="cursor-pointer"
      >
        {loading ? t("common.loading") : t("common.update")}
      </Button>

      {/* Plugins List */}
      <div className="min-h-0 flex-1 overflow-y-auto">
        {plugins.length === 0 ? (
          <p className="py-4 text-center text-xs text-muted-foreground">
            {t("plugin_list.no_plugins")}
          </p>
        ) : (
          <ul className="flex flex-col gap-1">
            {plugins.map((plugin) => {
              const isSelected = selectedId === plugin.description.manifest.id;
              const allNeedNetwork =
                plugin.description.manifest.exports.length > 0 &&
                plugin.description.manifest.exports.every(
                  (e) => e.require_network_data === true,
                );
              const isBlocked = allNeedNetwork && !hasStoredNetwork;

              return (
                <li key={plugin.description.manifest.id}>
                  <button
                    type="button"
                    onClick={() => onSelect(plugin.description.manifest.id)}
                    className={cn(
                      "w-full rounded-md p-2 text-left text-sm transition-colors cursor-pointer border",
                      isSelected
                        ? "bg-muted text-foreground font-medium border-border shadow-xs"
                        : "border-transparent hover:bg-muted/60 text-muted-foreground hover:text-foreground",
                      isBlocked && "opacity-60",
                    )}
                  >
                    <div className="flex items-center justify-between gap-1.5">
                      <span className="truncate font-medium text-foreground">
                        {plugin.description.manifest.name}
                      </span>
                      {isBlocked ? (
                        <span title={t("common.blocked_title")}>
                          <Lock className="size-3 text-amber-500/80 shrink-0" />
                        </span>
                      ) : null}
                    </div>

                    {/* Tag pills inside plugin list item */}
                    {plugin.description.manifest.tags.length > 0 && (
                      <div className="mt-1.5 flex flex-wrap gap-1">
                        {plugin.description.manifest.tags.map((tag) => (
                          <span
                            key={tag}
                            onClick={(e) => {
                              e.stopPropagation();
                              onTagSelect(selectedTag === tag ? null : tag);
                            }}
                            className={cn(
                              "inline-block rounded px-1.5 py-0.2 text-[10px] leading-tight font-medium cursor-pointer transition-colors",
                              selectedTag === tag
                                ? "bg-primary text-primary-foreground"
                                : "bg-muted text-muted-foreground hover:bg-muted/80 hover:text-foreground",
                            )}
                          >
                            #{tag}
                          </span>
                        ))}
                      </div>
                    )}
                  </button>
                </li>
              );
            })}
          </ul>
        )}
      </div>
    </aside>
  );
}
