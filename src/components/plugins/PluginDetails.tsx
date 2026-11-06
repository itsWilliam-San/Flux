import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "../ui/button";
import type { PluginDescriptor, PluginParameter } from "@/types/plugins";
import { Avatar, AvatarImage } from "@/components/ui/avatar";
import { AlertCircle, Lock } from "lucide-react";
import { useTranslation } from "react-i18next";
import ExecutionResult, { type ExecutionResultData } from "./ExecutionResult";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "../ui/select";
import { Field, FieldGroup, FieldLabel } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { Spinner } from "../ui/spinner";
import { Checkbox } from "@/components/ui/checkbox";

interface PluginDetailsProps {
    plugin: PluginDescriptor | null;
    hasStoredNetwork: boolean;
    onActionSuccess?: () => void;
}

export default function PluginDetails({
    plugin,
    hasStoredNetwork,
    onActionSuccess,
}: PluginDetailsProps) {
    const { t } = useTranslation();
    const [paramValues, setParamValues] = useState<
        Record<string, Record<string, unknown>>
    >({});
    const [runningAction, setRunningAction] = useState<string | null>(null);
    const [actionResults, setActionResults] = useState<
        Record<string, ExecutionResultData>
    >({});

    if (!plugin) {
        return (
            <section className="flex min-h-0 flex-1 items-center justify-center overflow-y-auto p-6 text-sm text-muted-foreground">
                {t("plugin_detail.no_plugin")}
            </section>
        );
    }

    const { manifest, origin, path } = plugin;
    const canExecute = origin === "local" || origin === "bridge";
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
        if (exporterLocked) {
            return;
        }

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

            if (onActionSuccess) {
                onActionSuccess();
            }
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
                        <AvatarImage
                            src={
                                manifest.icon.startsWith("data:")
                                    ? manifest.icon
                                    : `data:image/png;base64,${manifest.icon}`
                            }
                        />
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
                <div className="mt-4 flex items-start gap-3 rounded-md border border-amber-500/40 bg-amber-500/10 p-3 text-sm text-amber-500">
                    <AlertCircle className="size-5 shrink-0 mt-0.5" />
                    <div>
                        <p className="font-medium">
                            {t("plugin_detail.plugin_blocked_title")}
                        </p>
                        <p className="mt-0.5 text-xs text-muted-foreground">
                            {t("plugin_detail.plugin_blocked_description")}
                        </p>
                    </div>
                </div>
            ) : null}

            <dl className="mt-6 grid grid-cols-[auto_1fr] gap-x-4 gap-y-2 text-sm">
                <dt className="text-muted-foreground">
                    {t("plugin_detail.version")}
                </dt>
                <dd>{manifest.version}</dd>
                <dt className="text-muted-foreground">
                    {t("plugin_detail.author")}
                </dt>
                <dd>{manifest.author}</dd>
                <dt className="text-muted-foreground">
                    {t("plugin_detail.origin")}
                </dt>
                <dd>
                    {origin === "local"
                        ? t("plugin_list.local")
                        : t("plugin_list.bridge")}
                </dd>
                <dt className="text-muted-foreground">
                    {t("plugin_detail.category")}
                </dt>
                <dd>
                    {manifest.category === "exporter"
                        ? t("plugin_list.exporter")
                        : t("plugin_list.extraction")}
                </dd>
                {path ? (
                    <>
                        <dt className="text-muted-foreground">
                            {t("plugin_detail.path")}
                        </dt>
                        <dd className="font-mono text-xs truncate" title={path}>
                            {path}
                        </dd>
                    </>
                ) : null}
                {manifest.repository ? (
                    <>
                        <dt className="text-muted-foreground">
                            {t("plugin_detail.repository")}
                        </dt>
                        <dd className="truncate">{manifest.repository}</dd>
                    </>
                ) : null}
                {manifest.download_url ? (
                    <>
                        <dt className="text-muted-foreground">
                            {t("plugin_detail.download")}
                        </dt>
                        <dd className="truncate">{manifest.download_url}</dd>
                    </>
                ) : null}
            </dl>

            <h2 className="mt-8 text-sm font-semibold uppercase tracking-wide text-foreground">
                {t("plugin_detail.exports")}
            </h2>
            {manifest.exports.length === 0 ? (
                <p className="mt-2 text-sm text-muted-foreground">
                    {t("plugin_detail.no_exports")}
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
                                <form
                                    onSubmit={(e) => {
                                        e.preventDefault();
                                        handleExecuteAction(item.id);
                                    }}>
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
                                        {canExecute ? (
                                            <Button
                                                type="submit"
                                                size="sm"
                                                disabled={
                                                    isExecuting || exporterLocked
                                                }
                                                title={
                                                    exporterLocked
                                                        ? t("common.blocked_title")
                                                        : undefined
                                                }
                                                className="cursor-pointer"
                                            >
                                                {isExecuting ? (
                                                    <span className="flex items-center gap-1.5">
                                                        <Spinner data-icon="inline-start" />
                                                        {t("common.running")}
                                                    </span>
                                                ) : exporterLocked ? (
                                                    <span className="flex items-center gap-1.5">
                                                        <Lock className="size-3.5" />
                                                        {t("common.blocked")}
                                                    </span>
                                                ) : (
                                                    t("plugin_detail.run_action")
                                                )}
                                            </Button>
                                        ) : null}
                                    </div>


                                    {item.parameters &&
                                        item.parameters.length > 0 ? (
                                        <div className="mt-3 border-t border-border pt-3">
                                            <p className="text-xs font-semibold uppercase text-muted-foreground mb-2">
                                                {t("common.parameters")}
                                            </p>
                                            <FieldGroup className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                                                {item.parameters.map((param) => {
                                                    const val = getParamValue(
                                                        item.id,
                                                        param,
                                                    );
                                                    const isRequired = param.required ?? false;

                                                    return (
                                                        <Field key={param.name} className="fles flex-col gap-1 textxs">
                                                            <FieldLabel htmlFor={`${item.id}-${param.name}`}>
                                                                {param.label} {isRequired && <span className="text-destructive">*</span>}
                                                            </FieldLabel>
                                                            {
                                                                param.param_type ===
                                                                    "boolean" ? (
                                                                    <div className="flex items-center gap-2 mt-1">
                                                                        <Checkbox
                                                                            id={`${item.id}-${param.name}`}
                                                                            required={isRequired}
                                                                            checked={Boolean(
                                                                                val,
                                                                            )}
                                                                            onCheckedChange={(
                                                                                checked,
                                                                            ) =>
                                                                                handleParamChange(
                                                                                    item.id,
                                                                                    param.name,
                                                                                    Boolean(checked)
                                                                                )
                                                                            }
                                                                        />
                                                                    </div>
                                                                ) : param.param_type ===
                                                                    "select" &&
                                                                    param.options ? (
                                                                    <Select
                                                                        id={`${item.id}-${param.name}`}
                                                                        required={isRequired}
                                                                        value={String(val ?? "")}
                                                                        onValueChange={(newValue) => handleParamChange(item.id, param.name, newValue)}>
                                                                        <SelectTrigger className="w-full h-8 text-xs bg-background text-foreground border-input">
                                                                            <SelectValue />
                                                                        </SelectTrigger>
                                                                        <SelectContent className="bg-background text-popover-foreground border-border">
                                                                            {param.options.map((opt, idx) => (
                                                                                <SelectItem key={idx} value={String(opt)}>
                                                                                    {String(opt)}
                                                                                </SelectItem>
                                                                            ))}
                                                                        </SelectContent>
                                                                    </Select>
                                                                ) : (
                                                                    <Input
                                                                        id={`${item.id}-${param.name}`}
                                                                        required={isRequired}
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
                                                                )
                                                            }
                                                        </Field>
                                                    );
                                                })}
                                            </FieldGroup>
                                        </div>
                                    ) : null}

                                    {result ? (
                                        <ExecutionResult result={result} />
                                    ) : null}
                                </form>
                            </li>
                        );
                    })}
                </ul>
            )
            }
        </section>
    );
}
