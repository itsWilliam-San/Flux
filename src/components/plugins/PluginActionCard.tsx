import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Button } from "@/components/ui/button";
import { FieldGroup } from "@/components/ui/field";
import { Spinner } from "@/components/ui/spinner";
import type { PluginExport, PluginParameter } from "@/types/plugins";
import { AlertCircle, Lock } from "lucide-react";
import { useTranslation } from "react-i18next";
import ExecutionResult, { type ExecutionResultData } from "./ExecutionResult";
import PluginParameterField from "./PluginParameterField";

interface PluginActionCardProps {
    pluginId: string;
    exportItem: PluginExport;
    hasStoredNetwork: boolean;
    onActionSuccess?: () => void;
}

export default function PluginActionCard({
    pluginId,
    exportItem,
    hasStoredNetwork,
    onActionSuccess,
}: PluginActionCardProps) {
    const { t } = useTranslation();
    const [paramValues, setParamValues] = useState<Record<string, unknown>>({});
    const [isExecuting, setIsExecuting] = useState(false);
    const [result, setResult] = useState<ExecutionResultData | null>(null);

    const isLocked =
        (exportItem.require_network_data ?? false) && !hasStoredNetwork;

    const getParamValue = (param: PluginParameter): unknown => {
        const current = paramValues[param.name];
        if (current !== undefined) return current;
        return param.default ?? (param.param_type === "boolean" ? false : "");
    };

    const handleParamChange = (paramName: string, val: unknown) => {
        setParamValues((prev) => ({
            ...prev,
            [paramName]: val,
        }));
    };

    const handleExecute = async () => {
        if (isLocked || isExecuting) return;

        setIsExecuting(true);
        setResult(null);

        try {
            const actionParams: Record<string, unknown> = {};
            if (exportItem.parameters) {
                for (const p of exportItem.parameters) {
                    actionParams[p.name] = getParamValue(p);
                }
            }

            const response = await invoke<unknown>("execute_plugin_action", {
                pluginId,
                action: exportItem.id,
                parameters: actionParams,
            });

            setResult({
                success: true,
                data: response,
            });

            onActionSuccess?.();
        } catch (err) {
            setResult({
                success: false,
                error: typeof err === "string" ? err : String(err),
            });
        } finally {
            setIsExecuting(false);
        }
    };

    return (
        <li className="rounded-lg border border-border p-4 bg-card shadow-xs">
            <form
                onSubmit={(e) => {
                    e.preventDefault();
                    handleExecute();
                }}
            >
                {/* Header row: title/entrypoint + action button */}
                <div className="flex items-start justify-between gap-4">
                    <div className="min-w-0">
                        <p className="font-medium text-foreground text-base">
                            {exportItem.label}
                        </p>
                        <p className="mt-1 text-sm text-muted-foreground">
                            {exportItem.description}
                        </p>
                        <p className="mt-1 font-mono text-xs text-muted-foreground/80">
                            {exportItem.entrypoint}
                        </p>
                    </div>

                    <Button
                        type="submit"
                        size="sm"
                        disabled={isExecuting || isLocked}
                        title={isLocked ? t("common.blocked_title") : undefined}
                        className="cursor-pointer shrink-0"
                    >
                        {isExecuting ? (
                            <span className="flex items-center gap-1.5">
                                <Spinner data-icon="inline-start" />
                                {t("common.running")}
                            </span>
                        ) : isLocked ? (
                            <span className="flex items-center gap-1.5">
                                <Lock className="size-3.5" />
                                {t("common.blocked")}
                            </span>
                        ) : (
                            t("plugin_detail.run_action")
                        )}
                    </Button>
                </div>

                {/* Lock banner if network data is missing */}
                {isLocked ? (
                    <div className="mt-3 flex items-start gap-2.5 rounded-md border border-amber-500/40 bg-amber-500/10 p-2.5 text-xs text-amber-600 dark:text-amber-400">
                        <AlertCircle className="size-4 shrink-0 mt-0.5" />
                        <p>{t("plugin_detail.plugin_blocked_description")}</p>
                    </div>
                ) : null}

                {/* Parameters group */}
                {exportItem.parameters && exportItem.parameters.length > 0 ? (
                    <div className="mt-3 border-t border-border/60 pt-3">
                        <p className="text-xs font-semibold uppercase tracking-wider text-muted-foreground mb-2">
                            {t("common.parameters")}
                        </p>
                        <FieldGroup className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                            {exportItem.parameters.map((param) => (
                                <PluginParameterField
                                    key={param.name}
                                    exportId={exportItem.id}
                                    param={param}
                                    value={getParamValue(param)}
                                    onChange={(newVal) =>
                                        handleParamChange(param.name, newVal)
                                    }
                                />
                            ))}
                        </FieldGroup>
                    </div>
                ) : null}

                {/* Execution Result */}
                {result ? (
                    <div className="mt-4">
                        <ExecutionResult result={result} />
                    </div>
                ) : null}
            </form>
        </li>
    );
}
