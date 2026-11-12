import sys
import json
import importlib.util
from pathlib import Path

def main():
    if len(sys.argv) < 3:
        sys.stderr.write("Usage: runner.py <plugin_dir> <entrypoint>\n")
        sys.exit(1)

    plugin_dir = Path(sys.argv[1]).resolve()
    entrypoint = sys.argv[2]

    if ":" not in entrypoint:
        sys.stderr.write(f"Invalid entrypoint format: {entrypoint}. Expected 'file.py:function_name'\n")
        sys.exit(1)

    file_name, func_name = entrypoint.split(":", 1)
    target_file = plugin_dir / file_name

    if not target_file.is_file():
        sys.stderr.write(f"Plugin entry file not found: {target_file}\n")
        sys.exit(1)

    if str(plugin_dir) not in sys.path:
        sys.path.insert(0, str(plugin_dir))

    # Read stdin JSON-RPC payload
    try:
        raw_in = sys.stdin.read()
        payload = json.loads(raw_in) if raw_in.strip() else {}
    except Exception as e:
        sys.stderr.write(f"Failed to parse stdin JSON payload: {e}\n")
        sys.exit(1)

    # Import module
    try:
        spec = importlib.util.spec_from_file_location("plugin_entry_module", str(target_file))
        if spec is None or spec.loader is None:
            sys.stderr.write(f"Could not load module specification from {target_file}\n")
            sys.exit(1)
        mod = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(mod)
    except Exception as e:
        sys.stderr.write(f"Error loading module {file_name}: {e}\n")
        sys.exit(1)

    func = getattr(mod, func_name, None)
    if func is None or not callable(func):
        sys.stderr.write(f"Function '{func_name}' not found or not callable in {file_name}\n")
        sys.exit(1)

    req_id = payload.get("id") if isinstance(payload, dict) else 1

    try:
        result = func(payload)
        # If the plugin returns an envelope that already is JSON-RPC 2.0, keep it.
        if isinstance(result, dict) and "jsonrpc" in result:
            response = result
        else:
            response = {
                "jsonrpc": "2.0",
                "result": result if result is not None else {"status": "success"},
                "id": req_id
            }
        sys.stdout.write(json.dumps(response))
    except Exception as e:
        error_response = {
            "jsonrpc": "2.0",
            "error": {
                "code": -32000,
                "message": str(e)
            },
            "id": req_id
        }
        sys.stdout.write(json.dumps(error_response))
        sys.stderr.write(f"Exception during execution of {func_name}: {e}\n")
        sys.exit(1)

if __name__ == "__main__":
    main()
