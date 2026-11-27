Bridge Plugin Mocker

Overview
---

This module serves as a Python mock plugin designed to test, validate, and simulate the inter-process communication (IPC) between the Flux and external Python runtimes. It implements a fully compliant JSON-RPC 2.0 server over TCP sockets, explicit error mapping, and reliable bidirectional message flows.

Purpose
---

The primary objective of this mocker is to provide a deterministic, isolated environment for testing the Flux core's plugin management pipeline. It allows you to:

- Validate the initial Handshake protocol via TCP, where the plugin automatically transmits its structured manifest upon connection.
- Test JSON-RPC 2.0 request parsing and dispatching, ensuring that incoming methods are correctly routed to local Python functions.
- Verify dynamic parameter unpacking, safely handling both positional arguments (list) and named arguments (dict) through Pydantic validation.
- Simulate standardized error propagation, returning strict JSON-RPC error codes (-32700 to -32603) without prematurely dropping the underlying TCP connection.

Key Features
---
- Explicit Data Contracts: Utilizes Pydantic models for strict payload validation of both manifests and RPC requests/responses.
- Resilient Transport Layer: Implements automatic connection retry logic and a continuous read loop over TCP streams.
- Standardized Error Handling: Differentiates between parse errors, invalid requests, unknown methods, invalid parameters, and internal server faults according to the JSON-RPC 2.0 specification.
