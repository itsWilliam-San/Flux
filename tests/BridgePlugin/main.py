import socket
import sys
import json
import time
from pathlib import Path
from typing import Any, Optional, Callable, Union
from pydantic import BaseModel

# 1. MODELOS PYDANTIC & ERROS JSON-RPC (Definidos primeiro)
class Parameter(BaseModel):
    name: str
    param_type: str
    label: str
    default: bool
    required: Optional[bool] = None

class Export(BaseModel):
    id: str
    label: str
    description: str
    entrypoint: str
    parameters: list[Parameter]

class Manifest(BaseModel):
    name: str
    description: str
    version: str
    author: str
    repository: Optional[str] = None
    download_url: Optional[str] = None
    icon: Optional[str] = None
    id: str
    category: str
    exports: list[Export]

class JsonError(BaseModel):
    code: int
    message: str
    data: Optional[str] = None

class ErrorResponse(BaseModel):
    jsonrpc: str = "2.0"
    error: JsonError
    id: Optional[str] = None

class Response(BaseModel):
    jsonrpc: str = "2.0"
    result: Any
    id: str

class Request(BaseModel):
    jsonrpc: str
    method: str
    params: Optional[Union[dict, list]] = None
    id: Optional[str] = None # If "None", it's a Notification

# RPC ERRORS CONSTANTS
PARSE_ERROR = JsonError(
    code=-32700,
    message="Parse error",
    data="Invalid JSON was received by the server. An error occurred on the server while parsing the JSON text."
)
INVALID_REQUEST = JsonError(
    code=-32600,
    message="Invalid Request",
    data="The JSON sent is not a valid Request object."
)
METHOD_NOT_FOUND = JsonError(
    code=-32601,
    message="Method not found",
    data="The method does not exist / is not available."
)
INVALID_PARAMS = JsonError(
    code=-32602,
    message="Invalid params",
    data="Invalid method parameter(s)."
)
INTERNAL_ERROR = JsonError(
    code=-32603,
    message="Internal error",
    data="Internal JSON-RPC error."
)

# 2. FUNÇÕES DE NEGÓCIO E REGISTRO DE MÉTODOS
def say_hi(name: str):
    return f"Hello {name}"

def say_bye(name: str):
    return f"Bye {name}"

REGISTERED_METHODS: dict[str, Callable] = {
    "say_hi": say_hi,
    "say_bye": say_bye
}


# 3. LÓGICA DE MANIFESTO E DESPACHO DE REQUISIÇÕES
def getManifestPath() -> Path:
    return Path(__file__).resolve().parent.joinpath("manifest.json")

def loadManifest(path: Path) -> Manifest:
    try:
        with open(path, 'r', encoding='utf-8') as file:
            data = json.load(file)
            return Manifest(**data)
    except FileNotFoundError:
        raise FileNotFoundError(f"[ERROR]: '{path}' not found.")
    except json.JSONDecodeError:
        raise ValueError(f"[ERROR]: '{path}' is not a valid JSON file.")
    except Exception as e:
        raise Exception(f"[ERROR]: {e}")

def dealWithRequest(request: bytes) -> Optional[bytes]:
    request_str = request.decode('utf-8')

    try:
        req: Request = Request.model_validate_json(request_str)

        if req.method not in REGISTERED_METHODS:
            err = ErrorResponse(error=METHOD_NOT_FOUND, id=req.id)
            return err.model_dump_json().encode('utf-8')

        func = REGISTERED_METHODS[req.method]
        params = req.params

        if params is None:
            result = func()
        elif isinstance(params, dict):
            result = func(**params)
        else:
            result = func(*params)

        if req.id is not None:
            res = Response(result=result, id=req.id)
            return res.model_dump_json().encode('utf-8')
        else:
            return None

    except json.JSONDecodeError as e:
        custom_invalid_params = INVALID_PARAMS.model_copy(update={"data": str(e)})

        err = ErrorResponse(error=custom_invalid_params, id=None)
        return err.model_dump_json().encode('utf-8')
    except Exception as e:
        err_response = ErrorResponse(
            error=JsonError(
                code=-32603,
                message="Internal server error",
                data=str(e)
            ),
            id=req.id # Should probably be fine since the JSONDecodeError is caught earlier
        )
        return err_response.model_dump_json().encode('utf-8')


# 4. CICLO DE CONEXÃO (START)
def start(manifest: Manifest):
    host = '127.0.0.1'
    port = 8080

    max_attempts = 3
    interval_secs = 5

    for attempt in range(max_attempts):
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as client_socket:
            try:
                print(f"Trying to connect to {host}:{port} (Attempt {attempt + 1}/{max_attempts})...")
                client_socket.connect((host, port))
                print(f"Connection done")

                manifest_json_bytes = manifest.model_dump_json().encode('utf-8')
                client_socket.sendall(manifest_json_bytes)
                print(f"Handshake done")

                while True:
                    response_bytes: bytes = client_socket.recv(1024)

                    if not response_bytes:
                        print("ConnectionClosed", file=sys.stderr)
                        break

                    reply_bytes = dealWithRequest(response_bytes)

                    if reply_bytes is not None:
                        # If "None", it's a Notification so no response back.
                        client_socket.sendall(reply_bytes)

            except (ConnectionRefusedError, socket.timeout, OSError) as e:
                print(f"[WARNING]: Failed to Connect. Reason: {e}")

                if attempt < max_attempts - 1:
                    print(f"A new attempt will run after {interval_secs}s.")
                    time.sleep(interval_secs)
                else:
                    print(f"No connection could be done. Exiting the script...", file=sys.stderr)
                    raise Exception("No connection could be established.")

            except Exception as e:
                raise Exception(f"[ERROR]: {e}")


if __name__ == "__main__":
    path = getManifestPath()
    manifest = loadManifest(path)
    start(manifest)
