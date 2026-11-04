import json
import os

SAMPLE_NETWORK = {
    "Intermediates": [
        {
            "ID": "1-1E1",
            "TerrainLevel": 851.439,
            "InvertLevel": 850.335,
            "Depth": 1.104
        },
        {
            "ID": "1-1E2",
            "TerrainLevel": 850.984,
            "InvertLevel": 849.934,
            "Depth": 1.050
        },
        {
            "ID": "1-1E3",
            "TerrainLevel": 850.651,
            "InvertLevel": 849.533,
            "Depth": 1.118
        },
        {
            "ID": "1-1E4",
            "TerrainLevel": 850.246,
            "InvertLevel": 849.132,
            "Depth": 1.114
        },
        {
            "ID": "1-2E1",
            "TerrainLevel": 849.789,
            "InvertLevel": 848.396,
            "Depth": 1.393
        },
        {
            "ID": "1-2E2",
            "TerrainLevel": 848.955,
            "InvertLevel": 847.905,
            "Depth": 1.050
        }
    ],
    "Segments": [
        {
            "ID": "1-1",
            "Connection": {
                "kind": "DG",
                "InvertLevel": 849.112
            },
            "Pipe": {
                "Material": "PVC",
                "Length": 81,
                "Slope": 0.02005,
                "Diameter": 150
            },
            "Intermediates": ["1-1E1", "1-1E2", "1-1E3", "1-1E4"],
            "IntermediateDistance": 20
        },
        {
            "ID": "1-2",
            "Pipe": {
                "Material": "PVC",
                "Length": 60,
                "Slope": 0.02458,
                "Diameter": 150
            },
            "Intermediates": ["1-2E1", "1-2E2"],
            "IntermediateDistance": 20
        }
    ],
    "Primary": [
        {
            "ID": "SB1-001",
            "TerrainLevel": 851.862,
            "InvertLevel": 850.736,
            "Depth": 1.126,
            "UpstreamManholes": [],
            "DownstreamConnection": ["SB1-002", "1-1"],
            "psets": {
                "Topo": {
                    "RN": ""
                }
            }
        },
        {
            "ID": "SB1-002",
            "TerrainLevel": 850.238,
            "InvertLevel": 848.888,
            "Depth": 1.350,
            "UpstreamManholes": ["SB1-001"],
            "DownstreamConnection": ["SB1-003", "1-2"],
            "psets": {
                "Topo": {
                    "RN": ""
                }
            }
        },
        {
            "ID": "SB1-003",
            "TerrainLevel": 848.687,
            "InvertLevel": 847.413,
            "Depth": 1.274,
            "UpstreamManholes": ["SB1-002"],
            "psets": {
                "Topo": {
                    "RN": ""
                }
            }
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
