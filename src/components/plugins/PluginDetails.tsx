import type { PluginInfo } from "@/types/plugins";
import { useTranslation } from "react-i18next";
import PluginActionCard from "./PluginActionCard";
import PluginHeader from "./PluginHeader";

interface PluginDetailsProps {
  plugin: PluginInfo | null;
  hasStoredNetwork: boolean;
  onActionSuccess?: () => void;
  onSelectTag?: (tag: string) => void;
}

export default function PluginDetails({
  plugin,
  hasStoredNetwork,
  onActionSuccess,
  onSelectTag,
}: PluginDetailsProps) {
  const { t } = useTranslation();

  if (!plugin) {
    return (
      <section className="flex min-h-0 flex-1 items-center justify-center overflow-y-auto p-6 text-sm text-muted-foreground">
        {t("plugin_detail.no_plugin")}
      </section>
    );
  }

  const { manifest } = plugin.description;

  return (
    <section className="min-h-0 flex-1 overflow-y-auto p-6 space-y-6">
      {/* Header and metadata */}
      <PluginHeader plugin={plugin} onSelectTag={onSelectTag} />

      {/* Exports section */}
      <div>
        <h2 className="text-sm font-semibold uppercase tracking-wide text-foreground">
          {t("plugin_detail.exports")}
        </h2>

        {manifest.exports.length === 0 ? (
          <p className="mt-2 text-sm text-muted-foreground">
            {t("plugin_detail.no_exports")}
          </p>
        ) : (
          <ul className="mt-3 flex flex-col gap-4">
            {manifest.exports.map((item) => (
              <PluginActionCard
                key={item.id}
                pluginId={manifest.id}
                exportItem={item}
                hasStoredNetwork={hasStoredNetwork}
                onActionSuccess={onActionSuccess}
              />
            ))}
          </ul>
        )}
      </div>
    </section>
  );
}
