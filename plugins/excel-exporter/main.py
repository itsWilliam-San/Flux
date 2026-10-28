import csv
import io
import os
import openpyxl

def run_detailed_export(payload):
    params = payload.get("parameters", {})
    separator = params.get("separator", ";")
    include_headers = params.get("include_headers", True)

    output = io.StringIO()
    writer = csv.writer(output, delimiter=separator)

    if include_headers:
        writer.writerow(["Station ID", "Terrain Level (m)", "Invert Level (m)", "Depth (m)"])

    sample_rows = [
        ["PV-01", 100.50, 98.20, 2.30],
        ["PV-02", 99.80, 97.40, 2.40],
        ["PV-03", 98.60, 96.10, 2.50],
    ]
    for row in sample_rows:
        writer.writerow(row)

    csv_content = output.getvalue()

    return {
        "status": "success",
        "action": "export_csv_detalhado",
        "format": "csv",
        "separator": separator,
        "rows_count": len(sample_rows),
        "content_preview": csv_content.strip()
    }

def run_export_csv(payload):
    params = payload.get("parameters", {})
    sheet_name = params.get("sheet_name", "Relatorio")

    wb = openpyxl.Workbook()
    ws = wb.active
    ws.title = sheet_name

    ws.append(["Station ID", "Terrain Level (m)", "Invert Level (m)", "Depth (m)"])
    ws.append(["PV-01", 100.50, 98.20, 2.30])
    ws.append(["PV-02", 99.80, 97.40, 2.40])

    return {
        "status": "success",
        "action": "export_xlsv_detalhado",
        "format": "xlsx",
        "sheet_name": sheet_name,
        "openpyxl_version": openpyxl.__version__,
        "message": f"Workbook with sheet '{sheet_name}' prepared successfully"
    }
