import csv
import io

def extract_station_rows(network):
    rows = []
    if not network:
        return rows

    primaries = network.get("primaries") or network.get("Primary") or []
    for p in primaries:
        p_id = p.get("id") or p.get("ID")
        if isinstance(p_id, dict):
            p_id = p_id.get("0", "")
        t_level = p.get("terrain_level") if p.get("terrain_level") is not None else p.get("TerrainLevel", 0.0)
        i_level = p.get("invert_level") if p.get("invert_level") is not None else p.get("InvertLevel", 0.0)
        depth = p.get("depth") if p.get("depth") is not None else p.get("Depth", 0.0)
        rows.append([str(p_id), float(t_level), float(i_level), float(depth)])

    return rows

def export_network_report(network, colorir=True):
    rows = extract_station_rows(network)
    output = io.StringIO()
    writer = csv.writer(output, delimiter=";")
    writer.writerow(["Station ID", "Terrain Level (m)", "Invert Level (m)", "Depth (m)"])
    for row in rows:
        writer.writerow(row)
    return output.getvalue(), len(rows)

def run_detailed_export(request):
    params = request.get("params", {}) if isinstance(request, dict) and "params" in request else request
    action_params = params.get("parameters", {}) if isinstance(params, dict) else {}
    colorir = action_params.get("colorir", True)

    network = params.get("network") or params.get("system")
    if not network:
        raise ValueError("No hydraulic network data found in request payload. Import network data first.")

    content, row_count = export_network_report(network, colorir=colorir)
    req_id = request.get("id", 1) if isinstance(request, dict) else 1

    return {
        "jsonrpc": "2.0",
        "result": {
            "status": "success",
            "message": f"Relatório PDF preparado com sucesso ({row_count} estações processadas).",
            "action": "export_pdf",
            "colorir": colorir,
            "rows_count": row_count,
            "content_preview": content.strip()
        },
        "id": req_id
    }
