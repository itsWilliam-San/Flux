# Plugin Manifest Specification

## Overview
The plugin manifest (`manifest.json`) describes a plugin's metadata, identity, and the actions (`exports`) it exposes to the main application. This specification is language-agnostic, allowing both Python scripts and C# modules to register capabilities, UI parameters, and entry points in a uniform way.

---

## Root Properties

- **`name`** (`string`): Human-readable name of the plugin.
- **`description`** (`string`): Brief summary of what the plugin does.
- **`version`** (`string`): Semantic versioning string (e.g., `"1.0.0"`).
- **`author`** (`string`): Name of the developer or organization.
- **`repository`** (`string`, optional): URL of the source code repository.
- **`download_url`** (`string`, optional): Direct download link for updates. Clients may also send `downloadUrl`; the host normalizes to `download_url`.
- **`icon`** (`string`, optional): A Base64 icon file.
- **`id`** (`string`): Unique reverse-domain identifier (e.g., `"com.mike.excel-exporter"`).
- **`category`** (`string`): Plugin role. Required. One of:
  - `"extraction"` — produces a sewer network `System` for the host to store.
  - `"exporter"` — consumes a stored `System` (disabled in the UI until data exists).
- **`exports`** (`array` of objects): List of actions/functions provided by the plugin.

Local vs Bridge (TCP / external CAD) is **not** part of the manifest. The host attaches `origin` when listing plugins.

---

## Exports Structure
Each object inside the `exports` array defines a callable feature or command exposed by the plugin.

- **`id`** (`string`): Unique identifier for the specific export action.
- **`label`** (`string`): User-friendly display name for UI buttons or menus.
- **`description`** (`string`): Explanation of what this specific action performs.
- **`entrypoint`** (`string`): Path and method to invoke the action (e.g., `"main.py:run_detailed_export"` or `"Sancad.Plugins.ExcelExporter:RunDetailedExport"`).
- **`parameters`** (`array` of objects, optional): Input fields required by this action to render dynamic UI forms.

---

## Parameter Types
Parameters allow the main application to automatically generate configuration forms and structure IPC payloads. Each parameter contains:

- **`name`** (`string`): Internal key name for the parameter.
- **`param_type`** (`string`): Data/UI type. Supported types: `"string"`, `"boolean"`, `"select"`. The host also accepts `paramType` and `type` as aliases.
- **`label`** (`string`): Label displayed next to the form field.
- **`default`**: Default value if none is provided by the user.
- **`options`** (`array`, optional): Only used when `param_type` is `"select"`. Lists the available choices.

---

## IPC Communication Standard
Communication between the backend and the plugin follows a unified format for both request and response payloads. The executing program receives a structure containing the action identifier and the corresponding parameters, and returns data using the exact same structural pattern.

- **`action`** (`string`): Path and method of the action (Is the same from `entrypoint`)
- **`parameters`** (`array` of objects, optional): The same array of objects from the `parameters` of the specified `entrypoint`.
