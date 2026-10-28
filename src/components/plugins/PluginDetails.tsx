import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "../ui/button";
import type { PluginDescriptor, PluginParameter } from "@/types/plugins";
import { Avatar, AvatarImage } from '@/components/ui/avatar';

interface PluginDetailsProps {
    plugin: PluginDescriptor | null;
    hasStoredNetwork: boolean;
}

const CATEGORY_LABEL: Record<string, string> = {
    extraction: "Extraction",
    exporter: "Exporter",
};

const ORIGIN_LABEL: Record<string, string> = {
    local: "Local",
    bridge: "Bridge",
};

export default function PluginDetails({
    plugin,
    hasStoredNetwork,
}: PluginDetailsProps) {
    const [paramValues, setParamValues] = useState<
        Record<string, Record<string, unknown>>
    >({});
    const [runningAction, setRunningAction] = useState<string | null>(null);
    const [actionResults, setActionResults] = useState<
        Record<string, { success: boolean; data?: unknown; error?: string }>
    >({});

    if (!plugin) {
        return (
            <section className="flex min-h-0 flex-1 items-center justify-center overflow-y-auto p-6 text-sm text-muted-foreground">
                No plugins connected. Connect a CAD plugin over TCP or install a
                local plugin.
            </section>
        );
    }

    const { manifest, origin, path } = plugin;
    const isLocal = origin === "local";
    const exporterLocked = manifest.category === "exporter" && !hasStoredNetwork;

    const handleParamChange = (
        exportId: string,
        paramName: string,
        val: unknown,
    ) => {
        setParamValues((prev) => ({
            ...prev,
            [exportId]: {
                ...(prev[exportId] ?? {}),
                [paramName]: val,
            },
        }));
    };

    const getParamValue = (
        exportId: string,
        param: PluginParameter,
    ): unknown => {
        const current = paramValues[exportId]?.[param.name];
        if (current !== undefined) {
            return current;
        }
        return param.default ?? (param.param_type === "boolean" ? false : "");
    };

    const handleExecuteAction = async (exportId: string) => {
        setRunningAction(exportId);
        setActionResults((prev) => {
            const next = { ...prev };
            delete next[exportId];
            return next;
        });

        try {
            const exportItem = manifest.exports.find((e) => e.id === exportId);
            const actionParams: Record<string, unknown> = {};

            if (exportItem?.parameters) {
                for (const p of exportItem.parameters) {
                    actionParams[p.name] = getParamValue(exportId, p);
                }
            }

            const response = await invoke<unknown>("execute_plugin_action", {
                pluginId: manifest.id,
                action: exportId,
                parameters: actionParams,
            });

            setActionResults((prev) => ({
                ...prev,
                [exportId]: {
                    success: true,
                    data: response,
                },
            }));
        } catch (err) {
            setActionResults((prev) => ({
                ...prev,
                [exportId]: {
                    success: false,
                    error: typeof err === "string" ? err : String(err),
                },
            }));
        } finally {
            setRunningAction(null);
        }
    };

    return (
        <section className="min-h-0 flex-1 overflow-y-auto p-6">
            <div className="flex items-start gap-4">
                {manifest.icon ? (
                    <Avatar className="size-12">
                        <AvatarImage src={manifest.icon.startsWith("data:") ? manifest.icon : `data:image/png;base64,${manifest.icon}`} />
                    </Avatar>
                ) : null}
                <div className="min-w-0">
                    <h1 className="text-xl font-semibold text-foreground">
                        {manifest.name}
                    </h1>
                    <p className="mt-1 text-sm text-muted-foreground">
                        {manifest.description}
                    </p>
                </div>
            </div>

            {exporterLocked ? (
                <p className="mt-4 rounded-md border border-border bg-muted/50 px-3 py-2 text-sm text-muted-foreground">
                    Notice: Export plugins usually operate on extracted network data.
                </p>
            ) : null}

            <dl className="mt-6 grid grid-cols-[auto_1fr] gap-x-4 gap-y-2 text-sm">
                <dt className="text-muted-foreground">Version</dt>
                <dd>{manifest.version}</dd>
                <dt className="text-muted-foreground">Author</dt>
                <dd>{manifest.author}</dd>
                <dt className="text-muted-foreground">Category</dt>
                <dd>{CATEGORY_LABEL[manifest.category] ?? manifest.category}</dd>
                {path ? (
                    <>
                        <dt className="text-muted-foreground">Path</dt>
                        <dd className="font-mono text-xs truncate" title={path}>
                            {path}
                        </dd>
                    </>
                ) : null}
                {manifest.repository ? (
                    <>
                        <dt className="text-muted-foreground">Repository</dt>
                        <dd className="truncate">{manifest.repository}</dd>
                    </>
                ) : null}
                {manifest.download_url ? (
                    <>
                        <dt className="text-muted-foreground">Download</dt>
                        <dd className="truncate">{manifest.download_url}</dd>
                    </>
                ) : null}
            </dl>

            <h2 className="mt-8 text-sm font-semibold uppercase tracking-wide text-foreground">
                Exports
            </h2>
            {manifest.exports.length === 0 ? (
                <p className="mt-2 text-sm text-muted-foreground">
                    This plugin does not declare any exports.
                </p>
            ) : (
                <ul className="mt-3 flex flex-col gap-4">
                    {manifest.exports.map((item) => {
                        const isExecuting = runningAction === item.id;
                        const result = actionResults[item.id];

                        return (
                            <li
                                key={item.id}
                                className="rounded-lg border border-border p-4 bg-card"
                            >
                                <div className="flex items-start justify-between gap-4">
                                    <div>
                                        <p className="font-medium text-foreground">
                                            {item.label}
                                        </p>
                                        <p className="mt-1 text-sm text-muted-foreground">
                                            {item.description}
                                        </p>
                                        <p className="mt-1 font-mono text-xs text-muted-foreground">
                                            {item.entrypoint}
                                        </p>
                                    </div>
                                    {isLocal ? (
                                        <Button
                                            type="button"
                                            size="sm"
                                            onClick={() =>
                                                handleExecuteAction(item.id)
                                            }
                                            disabled={isExecuting}
                                        >
                                            {isExecuting
                                                ? "Running…"
                                                : "Run Action"}
                                        </Button>
                                    ) : null}
                                </div>

                                {item.parameters &&
                                    item.parameters.length > 0 ? (
                                    <div className="mt-3 border-t border-border pt-3">
                                        <p className="text-xs font-semibold uppercase text-muted-foreground mb-2">
                                            Parameters
                                        </p>
                                        <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                                            {item.parameters.map((param) => {
                                                const val = getParamValue(
                                                    item.id,
                                                    param,
                                                );

                                                return (
                                                    <div
                                                        key={param.name}
                                                        className="flex flex-col gap-1 text-xs"
                                                    >
                                                        <label className="font-medium text-foreground">
                                                            {param.label}
                                                        </label>
                                                        {param.param_type ===
                                                            "boolean" ? (
                                                            <div className="flex items-center gap-2 mt-1">
                                                                <input
                                                                    type="checkbox"
                                                                    checked={
                                                                        Boolean(
                                                                            val,
                                                                        )
                                                                    }
                                                                    onChange={(
                                                                        e,
                                                                    ) =>
                                                                        handleParamChange(
                                                                            item.id,
                                                                            param.name,
                                                                            e
                                                                                .target
                                                                                .checked,
                                                                        )
                                                                    }
                                                                    className="rounded border-border text-primary focus:ring-primary size-4"
                                                                />
                                                                <span className="text-muted-foreground text-xs">
                                                                    {Boolean(
                                                                        val,
                                                                    )
                                                                        ? "Enabled"
                                                                        : "Disabled"}
                                                                </span>
                                                            </div>
                                                        ) : param.param_type ===
                                                            "select" &&
                                                            param.options ? (
                                                            <select
                                                                value={String(
                                                                    val ?? "",
                                                                )}
                                                                onChange={(
                                                                    e,
                                                                ) =>
                                                                    handleParamChange(
                                                                        item.id,
                                                                        param.name,
                                                                        e.target
                                                                            .value,
                                                                    )
                                                                }
                                                                className="rounded-md border border-input bg-background px-2 py-1 text-xs text-foreground focus:outline-none focus:ring-1 focus:ring-ring"
                                                            >
                                                                {param.options.map(
                                                                    (
                                                                        opt,
                                                                        idx,
                                                                    ) => (
                                                                        <option
                                                                            key={
                                                                                idx
                                                                            }
                                                                            value={String(
                                                                                opt,
                                                                            )}
                                                                        >
                                                                            {String(
                                                                                opt,
                                                                            )}
                                                                        </option>
                                                                    ),
                                                                )}
                                                            </select>
                                                        ) : (
                                                            <input
                                                                type="text"
                                                                value={String(
                                                                    val ?? "",
                                                                )}
                                                                onChange={(
                                                                    e,
                                                                ) =>
                                                                    handleParamChange(
                                                                        item.id,
                                                                        param.name,
                                                                        e.target
                                                                            .value,
                                                                    )
                                                                }
                                                                className="rounded-md border border-input bg-background px-2 py-1 text-xs text-foreground focus:outline-none focus:ring-1 focus:ring-ring"
                                                            />
                                                        )}
                                                    </div>
                                                );
                                            })}
                                        </div>
                                    </div>
                                ) : null}

                                {result ? (
                                    <div className="mt-3 border-t border-border pt-3">
                                        <p
                                            className={`text-xs font-semibold ${result.success ? "text-green-500" : "text-destructive"}`}
                                        >
                                            {result.success
                                                ? "Execution Output:"
                                                : "Execution Error:"}
                                        </p>
                                        <pre className="mt-1 max-h-48 overflow-y-auto rounded bg-muted p-2 font-mono text-xs text-foreground whitespace-pre-wrap">
                                            {result.success
                                                ? JSON.stringify(
                                                    result.data,
                                                    null,
                                                    2,
                                                )
                                                : result.error}
                                        </pre>
                                    </div>
                                ) : null}
                            </li>
                        );
                    })}
                </ul>
            )}
        </section>
    );
}
