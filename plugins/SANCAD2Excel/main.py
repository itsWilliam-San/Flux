import os
import sys
import openpyxl
import traceback
from typing import Optional
from pydantic import BaseModel, Field
from openpyxl.worksheet.worksheet import Worksheet

# --- MODELOS PYDANTIC ---
class FunParameters(BaseModel):
    expessura: float


class Pipe(BaseModel):
    Material: str
    Length: float = Field(gt=0.0, description="Comprimento deve ser maior que 0 metros")
    Slope: float
    Diameter: float = Field(gt=0.0, description="Diametro não pode ser um valor igual a 0")

class Primary(BaseModel):
    ID: str
    TerrainLevel: float
    InvertLevel: float
    Depth: float = Field(ge=0.0, description="Não é possivel ter profundidade negativa.")
    UpstreamManholes: Optional[list[str]]  = None
    DownstreamConnection: Optional[tuple[str, str]]  = None
    psets: Optional[dict] = None

class Intermediate(BaseModel):
    ID: str
    TerrainLevel: float
    InvertLevel: float
    Depth: float = Field(ge=0.0, description="Não é possivel ter profundidade negativa.")

class Connection(BaseModel):
    kind: str
    InvertLevel: float

class Segment(BaseModel):
    ID: str
    connection: Optional[Connection] = Field(default=None, alias="Connection")
    pipe: Pipe = Field(..., alias="Pipe")
    Intermediates: Optional[list[str]] = None
    IntermediateDistance: Optional[float] = None

class System(BaseModel):
    primary: list[Primary] = Field(..., alias="Primary")
    Intermediates: Optional[list[Intermediate]] = Field(default=None, alias="Intermediates")
    Segments: list[Segment]

class Parameters(BaseModel):
    network: System
    parameters: FunParameters

class Message(BaseModel):
    id: int
    jsonrpc: str
    method: str
    params: Parameters

# --- CONFIGURAÇÕES DE EXPORTAÇÃO ---
TEMPLATE_PATH: str = "./template.xlsx"
OUTPUT_DIR: str = "./output/"
FIRST_ROW: int = 15
LAST_ROW: int = 39

# --- 1. FUNÇÕES AUXILIARES DE VALIDAÇÃO E CONFIGURAÇÃO ---
def _load_and_validate_system(system_data: System) -> tuple[System, dict[str, Primary], dict[str, Intermediate]]:
    if not os.path.exists(TEMPLATE_PATH):
        raise FileNotFoundError(f"Arquivo de template não encontrado em: {TEMPLATE_PATH}")

    primary_dict: dict[str, Primary] = {p.ID: p for p in system_data.primary}
    inter_dict: dict[str, Intermediate] = {i.ID: i for i in (system_data.Intermediates or [])}
    return system_data, primary_dict, inter_dict


def _find_segment_nodes(segment, primaries: list[Primary], primary_dict: dict[str, Primary]) -> tuple[Primary, Primary]:
    """Localiza os nós de montante e jusante correspondentes ao segmento."""
    seg_id = segment.ID
    upstream_pv: Primary | None = None
    downstream_pv: Primary | None = None

    for p in primaries:
        downstream_conn = p.DownstreamConnection
        if downstream_conn and downstream_conn[1] == seg_id:
            upstream_pv = p
            downstream_id = downstream_conn[0]
            downstream_pv = primary_dict.get(downstream_id)
            break

    if not upstream_pv or not downstream_pv:
        raise ValueError(f"O segmento {seg_id} não possui um Montante e Jusante válidos.")

    return upstream_pv, downstream_pv


def _validate_segment_constraints(segment, inter_ids: list, inter_dist: float | None) -> None:
    """Valida as restrições de distância e limites de linhas do template para o segmento."""
    seg_id = segment.ID
    seg_total_length = segment.pipe.Length

    if inter_ids and inter_dist is None:
        raise ValueError(f"O segmento {seg_id} contém estacas intermediárias, porém não foi indicada a distância entre estacas.")

    if inter_dist is not None:
        covered_dist = len(inter_ids) * inter_dist
        if covered_dist > seg_total_length:
            raise ValueError(
                f"O Segmento {seg_id} com {len(inter_ids)} estacas e espaçamento de {inter_dist} "
                f"possui tamanho total de {covered_dist}, que excede o tamanho do segmento: {seg_total_length}"
            )

    total_rows = 2 + len(inter_ids)
    if total_rows > (LAST_ROW - FIRST_ROW):
        raise ValueError(
            f"Para representar o segmento {seg_id}, são necessárias {total_rows} linhas, "
            f"porém só existem {LAST_ROW - FIRST_ROW} linhas disponíveis no template."
        )


def _write_station_row(
    ws: Worksheet,
    row: int,
    *,
    rn: str,
    id_val: str,
    stake_name: str,
    distance: float,
    terrain_level: float,
    slope: float,
    invert_level: float,
    diameter: float,
    obs: str = ""
) -> None:
    """Escreve os dados e fórmulas padronizadas de uma linha (estaca ou PV) na planilha."""
    ws.cell(row=row, column=1, value=rn)
    ws.cell(row=row, column=2, value=id_val)
    ws.cell(row=row, column=3, value=stake_name)
    ws.cell(row=row, column=4, value=f"{distance:.2f}")
    ws.cell(row=row, column=5, value=f"{terrain_level:.3f}")
    ws.cell(row=row, column=6, value=f"{slope:.5f}")
    ws.cell(row=row, column=7, value=f"{invert_level:.3f}")
    ws.cell(row=row, column=8, value=f"=ROUNDUP((L{row}+1)/0.5, 0)*0.5")
    ws.cell(row=row, column=9, value=f"{diameter:.4f}")
    ws.cell(row=row, column=10, value=f"=G{row}+I{row}")
    ws.cell(row=row, column=11, value=f"=J{row}+H{row}")
    ws.cell(row=row, column=12, value=f"=E{row}-G{row}")
    ws.cell(row=row, column=13, value=f"=H{row}-L{row}")
    ws.cell(row=row, column=14, value=obs)


def _extract_rn(pv_data: Primary) -> str:
    """Extrai o valor de RN das propriedades (psets) de um PV se disponível."""
    if pv_data.psets and isinstance(pv_data.psets, dict):
        topo_data = pv_data.psets.get("Topo")
        if isinstance(topo_data, dict):
            return str(topo_data.get("RN", ""))
    return ""


# --- 3. PROCESSAMENTO DE CADA SEGMENTO ---
def _process_segment(
    segment,
    primaries: list[Primary],
    primary_dict: dict[str, Primary],
    inter_dict: dict[str, Intermediate],
    ws: Worksheet,
    expessure: float
) -> None:
    """Processa um segmento individual populando a planilha."""
    seg_id = segment.ID
    upstream_pv, downstream_pv = _find_segment_nodes(segment, primaries, primary_dict)

    inter_ids = segment.Intermediates or []
    inter_dist = segment.IntermediateDistance
    pipe = segment.pipe
    slope = pipe.Slope
    seg_total_length = pipe.Length

    _validate_segment_constraints(segment, inter_ids, inter_dist)

    ws.title = str(seg_id)

    current_row = FIRST_ROW
    cumulative_distance = 0.0

    # --- A. PV DE MONTANTE ---
    rn_up = _extract_rn(upstream_pv)
    connection = segment.connection
    kind = connection.kind if connection else ""

    _write_station_row(
        ws, current_row,
        rn=rn_up,
        id_val=upstream_pv.ID,
        stake_name="",
        distance=cumulative_distance,
        terrain_level=upstream_pv.TerrainLevel,
        slope=slope,
        invert_level=upstream_pv.InvertLevel,
        diameter=expessure,
    )
    current_row += 1
    # --- B. ESTACAS INTERMEDIÁRIAS ---
    for i_id in inter_ids:
        if current_row > LAST_ROW:
            break

        inter_data = inter_dict.get(i_id)
        if not inter_data:
            continue

        assert inter_dist is not None
        cumulative_distance += inter_dist

        _write_station_row(
            ws, current_row,
            rn="",
            id_val="",
            stake_name=i_id,
            distance=cumulative_distance,
            terrain_level=inter_data.TerrainLevel,
            slope=slope,
            invert_level=inter_data.InvertLevel,
            diameter=expessure
        )
        current_row += 1

    cumulative_distance = seg_total_length

    inv_conn = connection.InvertLevel if connection else None
    obs_conn = f"{kind}={downstream_pv.TerrainLevel - inv_conn:.2f}" if kind and inv_conn is not None else ""
    # --- C. PV DE JUSANTE ---
    if current_row <= LAST_ROW and downstream_pv:
        rn_down = _extract_rn(downstream_pv)

        _write_station_row(
            ws, current_row,
            rn=rn_down,
            id_val=downstream_pv.ID,
            stake_name="",
            distance=cumulative_distance,
            terrain_level=downstream_pv.TerrainLevel,
            slope=slope,
            invert_level=downstream_pv.InvertLevel,
            diameter=expessure,
            obs=obs_conn
        )


# --- 4. FUNÇÃO PRINCIPAL (ORQUESTRADORA) ---
def export_network_to_xlsx(network: System, parameters: Parameters) -> tuple[int, str]:
    """Orquestra o carregamento, processamento dos segmentos e salvamento de arquivos XLSX individuais por segmento."""
    try:
        system_data, primary_dict, inter_dict = _load_and_validate_system(network)

        os.makedirs(OUTPUT_DIR, exist_ok=True)
        total_segments_processed: int = 0
        last_saved_path = ""

        for segment in system_data.Segments:
            wb: openpyxl.Workbook = openpyxl.load_workbook(TEMPLATE_PATH)
            ws: Worksheet | None = wb.active

            if not ws:
                raise ValueError("No Active Worksheet found")

            _process_segment(
                segment=segment,
                primaries=system_data.primary,
                primary_dict=primary_dict,
                inter_dict=inter_dict,
                ws=ws,
                expessure=parameters.parameters.expessura
            )

            output_filename = f"{segment.ID}.xlsx"
            output_path = os.path.join(OUTPUT_DIR, output_filename)
            wb.save(output_path)
            last_saved_path = output_path
            total_segments_processed += 1

        return total_segments_processed, last_saved_path

    except Exception as e:
        print("[ERROR]", file=sys.stderr)
        print(traceback.format_exc(), file=sys.stderr)
        print(e, file=sys.stderr)
        sys.exit(1)

def run_export_xlsv(request: dict):
    message = Message(**request)

    params = message.params

    network: System = params.network

    result = export_network_to_xlsx(network, message.params)
    if not result:
        raise RuntimeError("Falha na geração dos arquivos Excel.")

    seg_count, saved_path = result
    req_id = request.get("id", 1) if isinstance(request, dict) else 1

    return {
        "jsonrpc": "2.0",
        "result": {
            "status": "success",
            "action": "export_xlsv",
            "format": "xlsx",
            "segments_count": seg_count,
            "openpyxl_version": openpyxl.__version__,
            "message": f"Arquivos gerados com sucesso ({seg_count} arquivos de segmentos criados) na pasta {OUTPUT_DIR}"
        },
        "id": req_id
    }
