import { useState } from "react";
import { useTranslation } from "react-i18next";
import { CheckCircle2, AlertCircle, ChevronDown, Copy, Check } from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

export interface ExecutionResultData {
    success: boolean;
    data?: unknown;
    error?: string;
}

interface ExecutionResultProps {
    result: ExecutionResultData;
}

function extractMessage(result: ExecutionResultData, t: (k: string) => string): string {
    if (result.success) {
        const data = result.data;
        if (typeof data === "string") {
            try {
                const parsed = JSON.parse(data);
                if (typeof parsed?.message === "string") return parsed.message;
                if (typeof parsed?.result?.message === "string") return parsed.result.message;
            } catch {
                return data;
            }
        } else if (typeof data === "object" && data !== null) {
            const obj = data as Record<string, unknown>;
            if (typeof obj.message === "string") return obj.message;
            if (typeof obj.result === "object" && obj.result !== null) {
                const resObj = obj.result as Record<string, unknown>;
                if (typeof resObj.message === "string") return resObj.message;
            }
            if (typeof obj.status === "string" && obj.status !== "success") {
                return `Status: ${obj.status}`;
            }
        }
        return t("plugin_detail.default_success_message");
    } else {
        const err = result.error;
        if (typeof err === "string") {
            try {
                const parsed = JSON.parse(err);
                if (typeof parsed?.message === "string") return parsed.message;
                if (typeof parsed?.error?.message === "string") return parsed.error.message;
            } catch {
                return err;
            }
        }
        if (typeof result.data === "object" && result.data !== null) {
            const obj = result.data as Record<string, unknown>;
            if (typeof obj.error === "object" && obj.error !== null) {
                const errObj = obj.error as Record<string, unknown>;
                if (typeof errObj.message === "string") return errObj.message;
            }
            if (typeof obj.message === "string") return obj.message;
        }
        return t("plugin_detail.default_error_message");
    }
}

function getRawPayload(result: ExecutionResultData): string {
    if (result.success) {
        return typeof result.data === "string"
            ? result.data
            : JSON.stringify(result.data, null, 2);
    }
    if (result.error) {
        return result.error;
    }
    return typeof result.data === "string"
        ? result.data
        : JSON.stringify(result.data, null, 2);
}

export default function ExecutionResult({ result }: ExecutionResultProps) {
    const { t } = useTranslation();
    const [isExpanded, setIsExpanded] = useState(false);
    const [copied, setCopied] = useState(false);

    const message = extractMessage(result, t);
    const rawPayload = getRawPayload(result);

    const handleCopy = async () => {
        try {
            await navigator.clipboard.writeText(rawPayload);
            setCopied(true);
            setTimeout(() => setCopied(false), 2000);
        } catch (err) {
            console.error("Failed to copy payload:", err);
        }
    };

    return (
        <div
            className={cn(
                "mt-3 rounded-lg border p-3.5 transition-colors",
                result.success
                    ? "border-green-500/30 bg-green-500/5 dark:bg-green-500/10"
                    : "border-destructive/30 bg-destructive/5 dark:bg-destructive/10",
            )}
        >
            {/* Summary Row */}
            <div className="flex items-start justify-between gap-3">
                <div className="flex items-start gap-2.5 min-w-0">
                    {result.success ? (
                        <CheckCircle2 className="size-4 shrink-0 text-green-600 dark:text-green-400 mt-0.5" />
                    ) : (
                        <AlertCircle className="size-4 shrink-0 text-destructive mt-0.5" />
                    )}
                    <div className="min-w-0">
                        <p
                            className={cn(
                                "text-xs font-semibold uppercase tracking-wider mb-0.5",
                                result.success
                                    ? "text-green-600 dark:text-green-400"
                                    : "text-destructive",
                            )}
                        >
                            {result.success
                                ? t("plugin_detail.execution_output")
                                : t("plugin_detail.execution_error")}
                        </p>
                        <p className="text-xs text-foreground font-medium leading-relaxed break-words">
                            {message}
                        </p>
                    </div>
                </div>

                <Button
                    type="button"
                    variant="ghost"
                    size="sm"
                    onClick={() => setIsExpanded(!isExpanded)}
                    className="h-7 px-2 text-xs text-muted-foreground hover:text-foreground shrink-0 gap-1 cursor-pointer"
                >
                    <span>
                        {isExpanded
                            ? t("plugin_detail.hide_details")
                            : t("plugin_detail.show_details")}
                    </span>
                    <ChevronDown
                        className={cn(
                            "size-3.5 transition-transform duration-200",
                            isExpanded && "rotate-180",
                        )}
                    />
                </Button>
            </div>

            {/* Expandable Technical Details Panel */}
            {isExpanded && (
                <div className="mt-3 border-t border-border/50 pt-2.5">
                    <div className="flex items-center justify-between gap-2 mb-1.5">
                        <span className="text-[11px] font-semibold uppercase tracking-wider text-muted-foreground">
                            {t("plugin_detail.technical_details")}
                        </span>
                        <Button
                            type="button"
                            variant="ghost"
                            size="sm"
                            onClick={handleCopy}
                            className="h-6 px-2 text-[11px] text-muted-foreground hover:text-foreground gap-1 cursor-pointer"
                        >
                            {copied ? (
                                <>
                                    <Check className="size-3 text-green-500" />
                                    <span className="text-green-600 dark:text-green-400">
                                        {t("plugin_detail.copied")}
                                    </span>
                                </>
                            ) : (
                                <>
                                    <Copy className="size-3" />
                                    <span>{t("plugin_detail.copy_json")}</span>
                                </>
                            )}
                        </Button>
                    </div>

                    <pre className="max-h-60 overflow-auto rounded bg-muted/80 p-2.5 font-mono text-[11px] text-foreground whitespace-pre-wrap select-all border border-border/40">
                        {rawPayload}
                    </pre>
                </div>
            )}
        </div>
    );
}
