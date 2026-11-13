# Plugin Manifest Specification

## Overview
The plugin manifest (`manifest.json`) describes a plugin's metadata, identity, and the actions (`exports`) it exposes to the main application. This specification is language-agnostic, allowing both Python scripts and C# modules to register capabilities, UI parameters, and entry points in a uniform way.

---

## Root Properties

- **`name`** (`string`): Human-readable name of the plugin.
- **`description`** (`string`): Brief summary of what the plugin does.
- **`version`** (`string`): Semantic versioning string (e.g., `"1.0.0"`).
- **`author`** (`array` of `string`): Name(s) of the developer(s) or organization.
- **`repository`** (`string`, optional): URL of the source code repository.
- **`download_url`** (`string`, optional): Direct download link for updates. Clients may also send `downloadUrl`; the host normalizes to `download_url`.
- **`icon`** (`string`, optional): A Path to your icon, or a HTTP/HTTPS URL.
- **`id`** (`string`): Unique reverse-domain identifier (e.g., `"com.mike.excel-exporter"`).
- **`tags`** (`array` of `string`): What category your plugin is in. May be anything.
- **`exports`** (`array` of objects): List of actions/functions provided by the plugin.

Local vs Bridge (TCP / external CAD) is **not** part of the manifest. The host attaches `origin` when listing plugins.

---

## Exports Structure
Each object inside the `exports` array defines a callable feature or command exposed by the plugin.

- **`id`** (`string`): Unique identifier for the specific export action.
- **`label`** (`string`): User-friendly display name for UI buttons or menus.
- **`description`** (`string`): Explanation of what this specific action performs.
- **`entrypoint`** (`string`): method to invoke the action (e.g., `"run_detailed_export"`)
- **`parameters`** (`array` of objects, optional): Input fields required by this action to render dynamic UI forms.
- **`require_network_data`** (`boolean`, optional): If in order for this function to work, it needs the network
  data first. if empty, defaults to `false`.
- **`update_network_data`** (`boolean`, optional): If your function returns or changes the Network Data, mark this option as `true` and Flux will use the object inside the `result` field as the new Network Data. This works even if no other network data was imported before. Default to `false` if empty. If this options is `true`, **no other data** should be inside of it.

---

## Parameter Types
Parameters allow the main application to automatically generate configuration forms and structure IPC payloads. Each parameter contains:

- **`name`** (`string`): Internal key name for the parameter.
- **`param_type`** (`string`): Data/UI type. Supported types: `"string"`, `"boolean"`, `"select"`. The host also accepts `paramType` and `type` as aliases.
- **`label`** (`string`): Label displayed next to the form field.
- **`default`**: Default value if none is provided by the user.
- **`options`** (`array`, optional): Only used when `param_type` is `"select"`. Lists the available choices.
- **`required`** (`bool`, optional): If the parameter must be filled before sending it. Either using the `default` or the user selection. If not included, `false` will be the value.

---

## IPC Communication Standard
Communication between the backend and the plugin follows a unified format for both request and response payloads. The executing program receives a structure containing the action identifier and the corresponding parameters, and returns data using the exact same structural pattern.

- **`action`** (`string`): Path and method of the action (Is the same from `entrypoint`)
- **`parameters`** (`array` of objects, optional): The same array of objects from the `parameters` of the specified `entrypoint`.

## Exportation Plugins
Para plugins de exportação que precisam dos dados da rede, os valores ficaram dentro do `network` dentro do `params`.
A estrutura geral vai paracer o que está abaixo

```json
{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "say_hi",
    "params": {
        "network": { See the "network-v1.json" for more detail },
        "parameters": { "say_hi" parameters }
    }
}
```

## Local Plugins Specific
When searching for your script python, it'll always search for the main.py that should be placed in the directory that manifest.json.
