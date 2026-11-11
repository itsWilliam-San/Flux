import socket
import sys
import json
import time
from pathlib import Path

import pydantic
from typing import Optional
from pydantic import BaseModel, Field

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

def loadManifest(path: Path) -> Manifest:
    try:
        with open(path, 'r', encoding='utf-8') as file:
            data = json.load(file)

            manifest = Manifest(**data)
            return manifest
    except FileNotFoundError:
        raise FileNotFoundError(f"[ERROR]: '{path}' not found.")
    except json.JSONDecodeError:
        raise ValueError(f"[ERROR]: '{path} is not a valid JSON file.")
    except Exception as e:
        raise Exception(f"[ERROR]: {e}")


def start(manifest: Manifest):
    host = '127.0.0.1'
    port = 8080

    max_attempts = 3
    interval_secs = 5

    for attempt in range(max_attempts):
        with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as client_socket:
            try:
                print(f"Trying to connect to {host}:{port}...")

                client_socket.connect((host, port))
                print(f"Connection done")

                manifest_json_bytes = manifest.model_dump_json().encode('utf-8')

                client_socket.sendall(manifest_json_bytes)
                print(f"Handhsake done")

                while True:
                    response = client_socket.recv(1024)

                    if not response:
                        print("Nothing received")
                        break
                    else:
                        print(response)

            except (ConnectionRefusedError, socket.timeout, OSError) as e:
                print(f"[WARNING]: Failed to Connect. Attempt: {attempt}. Reason: {e}")

                if attempt < max_attempts:
                    print(f"A new attempt will run after {interval_secs}s.")
                    time.sleep(interval_secs)
                else:
                    print(f"No connection could be done. Exiting the script...", file=sys.stderr)
                    raise Exception("No connecgtion could be estabilished.")

            except Exception as e:
                raise Exception(f"[ERROR]: {e}")

def getManifestPath() -> Path:
    return Path(__file__).resolve().parent.joinpath("manifest.json")

if __name__ == "__main__":
    path = getManifestPath()
    manifest = loadManifest(path)

    start(manifest)
