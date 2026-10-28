export type PluginCategory = "extraction" | "exporter" | (string & {});

export type PluginOrigin = "local" | "bridge";

export interface PluginParameter {
    name: string;
    param_type: "string" | "boolean" | "select" | string;
    label: string;
    default?: string | boolean | number;
    options?: Array<string | number | boolean>;
}

export interface PluginExport {
    id: string;
    label: string;
    description: string;
    entrypoint: string;
    parameters?: PluginParameter[];
}

export interface PluginManifest {
    name: string;
    description: string;
    version: string;
    author: string;
    repository?: string;
    download_url?: string;
    icon?: string;
    id: string;
    category: PluginCategory;
    exports: PluginExport[];
}

export interface PluginDescriptor {
    origin: PluginOrigin;
    manifest: PluginManifest;
    path?: string;
}

export interface IpcPayload {
    action: string;
    parameters?: PluginParameter[];
}

export interface ActionExecutionResult {
    status?: string;
    message?: string;
    output?: string;
    [key: string]: unknown;
}
