import csv
import io
import openpyxl

def extract_station_rows(network):
    """Extracts station rows from the hydraulic network data."""
    rows = []
    if not network:
        return rows

    # Extract primaries (manholes)
    primaries = network.get("primaries") or network.get("Primary") or []
    for p in primaries:
        p_id = p.get("id") or p.get("ID")
        if isinstance(p_id, dict):
            p_id = p_id.get("0", "")
        t_level = p.get("terrain_level") if p.get("terrain_level") is not None else p.get("TerrainLevel", 0.0)
        i_level = p.get("invert_level") if p.get("invert_level") is not None else p.get("InvertLevel", 0.0)
        depth = p.get("depth") if p.get("depth") is not None else p.get("Depth", 0.0)
        rows.append([str(p_id), float(t_level), float(i_level), float(depth), "Primary"])

    # Extract intermediates
    intermediates = network.get("intermediates") or network.get("Intermediates") or []
    for inter in intermediates:
        inter_id = inter.get("id") or inter.get("ID")
        if isinstance(inter_id, dict):
            inter_id = inter_id.get("0", "")
        t_level = inter.get("terrain_level") if inter.get("terrain_level") is not None else inter.get("TerrainLevel", 0.0)
        i_level = inter.get("invert_level") if inter.get("invert_level") is not None else inter.get("InvertLevel", 0.0)
        depth = inter.get("depth") if inter.get("depth") is not None else inter.get("Depth", 0.0)
        rows.append([str(inter_id), float(t_level), float(i_level), float(depth), "Intermediate"])

    return rows

def export_network_to_csv(network, separator=";", include_headers=True):
    """Exports network station data to CSV formatted string."""
    rows = extract_station_rows(network)
    output = io.StringIO()
    writer = csv.writer(output, delimiter=separator)

    if include_headers:
        writer.writerow(["Station ID", "Terrain Level (m)", "Invert Level (m)", "Depth (m)", "Station Type"])

    for row in rows:
        writer.writerow(row)

    return output.getvalue(), len(rows)

def export_network_to_xlsx(network, sheet_name="Relatorio"):
    """Exports network station data to an openpyxl Workbook."""
    rows = extract_station_rows(network)
    wb = openpyxl.Workbook()
    ws = wb.active
    ws.title = sheet_name

    ws.append(["Station ID", "Terrain Level (m)", "Invert Level (m)", "Depth (m)", "Station Type"])
    for row in rows:
        ws.append(row)

    return wb, len(rows)

def run_detailed_export(request):
    params = request.get("params", {}) if isinstance(request, dict) and "params" in request else request
    action_params = params.get("parameters", {}) if isinstance(params, dict) else {}
    separator = action_params.get("separator", ";")
    include_headers = action_params.get("include_headers", True)

    network = params.get("network") or params.get("system")
    if not network:
        raise ValueError("No hydraulic network data found in request payload. Import network data first.")

    csv_content, row_count = export_network_to_csv(
        network,
        separator=separator,
        include_headers=include_headers
    )

    req_id = request.get("id", 1) if isinstance(request, dict) else 1

    return {
        "jsonrpc": "2.0",
        "result": {
            "status": "success",
            "message": f"Exportação CSV gerada com sucesso ({row_count} estações exportadas).",
            "action": "export_csv_detalhado",
            "format": "csv",
            "separator": separator,
            "rows_count": row_count,
            "content_preview": csv_content.strip()
        },
        "id": req_id
    }

def run_export_csv(request):
    params = request.get("params", {}) if isinstance(request, dict) and "params" in request else request
    action_params = params.get("parameters", {}) if isinstance(params, dict) else {}
    sheet_name = action_params.get("sheet_name", "Relatorio")

    network = params.get("network") or params.get("system")
    if not network:
        raise ValueError("No hydraulic network data found in request payload. Import network data first.")

    wb, row_count = export_network_to_xlsx(network, sheet_name=sheet_name)

    req_id = request.get("id", 1) if isinstance(request, dict) else 1

    return {
        "jsonrpc": "2.0",
        "result": {
            "status": "success",
            "action": "export_xlsv_detalhado",
            "format": "xlsx",
            "sheet_name": sheet_name,
            "rows_count": row_count,
            "openpyxl_version": openpyxl.__version__,
            "message": f"Workbook with sheet '{sheet_name}' prepared with {row_count} stations"
        },
        "id": req_id
    }
