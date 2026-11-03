import json
import os

SAMPLE_NETWORK = {
    "Intermediates": [
        {
            "ID": "L-01",
            "TerrainLevel": 100.00,
            "InvertLevel": 50.00,
            "Depth": 50.00
        }
    ],
    "Segments": [
        {
            "ID": "SEG-ABC",
            "Connection": {
                "kind": "DropPipe",
                "InvertLevel": 10.00
            },
            "Pipe": {
                "Material": "PVC",
                "Length": 10.00,
                "Slope": 0.56,
                "Diameter": 0.05
            },
            "Intermediates": ["L-01"],
            "IntermediateDistance": 20.00
        }
    ],
    "Primary": [
        {
            "ID": "PV-01",
            "TerrainLevel": 100.00,
            "InvertLevel": 50.00,
            "Depth": 50.00,
            "UpstreamManholes": ["PV-02"],
            "DownstreamConnection": ["PV-03", "SEG-ABC"],
            "psets": {
                "Topo": {
                    "RN": "11"
                }
            }
        },
        {
            "ID": "PV-02",
            "TerrainLevel": 102.50,
            "InvertLevel": 52.10,
            "Depth": 50.40,
            "UpstreamManholes": [],
            "DownstreamConnection": ["PV-01", "SEG-ABC"],
            "psets": {}
        }
    ]
}

def import_network(request):
    params = request.get("params", {}) if isinstance(request, dict) and "params" in request else request
    action_params = params.get("parameters", {}) if isinstance(params, dict) else {}
    source = action_params.get("source", "sample_network")
    file_path = action_params.get("file_path", "")

    if source == "custom_file" and file_path and os.path.isfile(file_path):
        with open(file_path, "r", encoding="utf-8") as f:
            network_data = json.load(f)
    else:
        network_data = SAMPLE_NETWORK

    req_id = request.get("id", 1) if isinstance(request, dict) else 1

    return {
        "jsonrpc": "2.0",
        "result": {
            "status": "success",
            "message": "Hydraulic network data successfully imported into the system",
            "system": network_data
        },
        "id": req_id
    }
