import { Avatar, AvatarImage } from "@/components/ui/avatar";
import type { PluginDescription } from "@/types/plugins";
import { useTranslation } from "react-i18next";

interface PluginHeaderProps {
    plugin: PluginDescription;
    onSelectTag?: (tag: string) => void;
}

export default function PluginHeader({ plugin, onSelectTag }: PluginHeaderProps) {
    const { t } = useTranslation();
    const { manifest, path } = plugin;

    return (
        <div>
            {/* Main info row */}
            <div className="flex items-start gap-4">
                {manifest.icon ? (
                    <Avatar className="size-12 shrink-0">
                        <AvatarImage src={manifest.icon} alt={manifest.name} />
                    </Avatar>
                ) : null}
                <div className="min-w-0 flex-1">
                    <h1 className="text-xl font-semibold text-foreground">
                        {manifest.name}
                    </h1>
                    <p className="mt-1 text-sm text-muted-foreground">
                        {manifest.description}
                    </p>

                    {/* Tag Badges */}
                    {manifest.tags.length > 0 && (
                        <div className="mt-2.5 flex flex-wrap gap-1.5">
                            {manifest.tags.map((tag) => (
                                <button
                                    key={tag}
                                    type="button"
                                    onClick={() => onSelectTag?.(tag)}
                                    className="inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium bg-primary/10 text-primary hover:bg-primary/20 transition-colors cursor-pointer"
                                    title={t("plugin_detail.filter_by_tag", { tag })}
                                >
                                    #{tag}
                                </button>
                            ))}
                        </div>
                    )}
                </div>
            </div>

            {/* Metadata Description List */}
            <dl className="mt-6 grid grid-cols-[auto_1fr] gap-x-4 gap-y-2 text-sm border-t border-border/50 pt-4">
                <dt className="text-muted-foreground">{t("plugin_detail.version")}</dt>
                <dd className="font-medium text-foreground">{manifest.version}</dd>

                <dt className="text-muted-foreground">{t("plugin_detail.author")}</dt>
                <dd className="text-foreground">
                    {manifest.author.length > 0 ? manifest.author.join(", ") : t("common.none")}
                </dd>

                {path ? (
                    <>
                        <dt className="text-muted-foreground">{t("plugin_detail.path")}</dt>
                        <dd className="font-mono text-xs text-muted-foreground truncate" title={path}>
                            {path}
                        </dd>
                    </>
                ) : null}

                {manifest.repository ? (
                    <>
                        <dt className="text-muted-foreground">{t("plugin_detail.repository")}</dt>
                        <dd className="truncate">
                            <a
                                href={manifest.repository}
                                target="_blank"
                                rel="noreferrer"
                                className="underline underline-offset-2 hover:text-foreground text-xs"
                            >
                                {manifest.repository}
                            </a>
                        </dd>
                    </>
                ) : null}

                {manifest.download_url ? (
                    <>
                        <dt className="text-muted-foreground">{t("plugin_detail.download")}</dt>
                        <dd className="truncate text-xs text-muted-foreground">
                            {manifest.download_url}
                        </dd>
                    </>
                ) : null}
            </dl>
        </div>
    );
}
