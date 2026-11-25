export interface PluginParameter {
    name: string;
    param_type: "string" | "boolean" | "select" | string;
    label: string;
    default?: string | boolean | number;
    options?: Array<string | number | boolean>;
    required?: boolean;
}

export interface PluginExport {
    id: string;
    label: string;
    description: string;
    entrypoint: string;
    parameters?: PluginParameter[];
    require_network_data?: boolean;
    update_network_data?: boolean;
}

export interface PluginManifest {
    name: string;
    description: string;
    version: string;
    /** Now an array — join with ", " for display */
    author: string[];
    /** Free-form tags for filtering and display */
    tags: string[];
    repository?: string;
    download_url?: string;
    /** asset:// URL or http(s):// URL */
    icon?: string;
    id: string;
    exports: PluginExport[];
}

/** What the backend's list_plugins command returns */
export interface PluginDescription {
    manifest: PluginManifest;
    /** Absolute path to the plugin directory */
    path: string;
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
