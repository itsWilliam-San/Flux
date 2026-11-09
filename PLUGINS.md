How to Install Plugins
---
> [!IMPORTANT]
> Flux is in the **Alpha stage**, so plugin installation steps may change. Please treat this document as the single source of truth.

1. Ensure Flux has been run at least once to initialize the application directories, or create a `plugins` folder manually in your system's App Data path:

   - **Linux**: `~/.local/share/com.williamss.flux/plugins`
   - **Windows**: `%APPDATA%\com.williamss.flux\plugins`
   - **macOS**: Not supported yet

2. Place each downloaded or custom plugin inside its own dedicated subdirectory within the `plugins` folder.
3. Launch Flux; your plugins will be detected and loaded automatically.

How to Create a Local Plugin
---
Every plugin requires a `manifest.json` file complying with the [Manifest Specification](docs/plugins/manifest-v1.md) to be recognized and listed by Flux.

Plugins execute using the Python runtime embedded within Flux.

A local plugin directory should follow this standard structure:

```text
MyPlugin/
├── manifest.json
└── main.py
```

How to Create a Bridge Plugin
---
Unlike local plugins, bridge plugins are not listed automatically. A bridge plugin must initiate a connection and transmit its manifest as an initial handshake. When a command is invoked, execution requests and parameters are routed through the local TCP socket according to the Manifest Specification.

The bridge communicates over `localhost:8080`.

> [!WARNING]
> Be very careful regarding the plugins you install. Currently, there are no sandbox restrictions limiting what a plugin can do on your system. Treat third-party plugins with the same level of caution and trust as any other executable software.
