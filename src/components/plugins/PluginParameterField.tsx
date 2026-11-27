import { Checkbox } from "@/components/ui/checkbox";
import { Field, FieldLabel } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from "@/components/ui/select";
import type { PluginParameter } from "@/types/plugins";

interface PluginParameterFieldProps {
    exportId: string;
    param: PluginParameter;
    value: unknown;
    onChange: (value: unknown) => void;
}

export default function PluginParameterField({
    exportId,
    param,
    value,
    onChange,
}: PluginParameterFieldProps) {
    const inputId = `${exportId}-${param.name}`;
    const isRequired = param.required ?? false;

    return (
        <Field className="flex flex-col gap-1 text-xs">
            <FieldLabel htmlFor={inputId}>
                {param.label}{" "}
                {isRequired && <span className="text-destructive">*</span>}
            </FieldLabel>

            {param.param_type === "boolean" ? (
                <div className="flex items-center gap-2 mt-1">
                    <Checkbox
                        id={inputId}
                        checked={Boolean(value)}
                        onCheckedChange={(checked) => onChange(Boolean(checked))}
                    />
                </div>
            ) : param.param_type === "select" && param.options ? (
                <Select
                    value={String(value ?? "")}
                    onValueChange={(newVal) => onChange(newVal)}
                >
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
                    id={inputId}
                    required={isRequired}
                    type="text"
                    value={String(value ?? "")}
                    onChange={(e) => onChange(e.target.value)}
                    className="rounded-md border border-input bg-background px-2 py-1 text-xs text-foreground focus:outline-none focus:ring-1 focus:ring-ring"
                />
            )}
        </Field>
    );
}
