import os
import sys
import openpyxl
import traceback
from typing import Optional
from pydantic import BaseModel, Field
from openpyxl.worksheet.worksheet import Worksheet

# --- MODELOS PYDANTIC ---
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
    Primary: list[Primary]
    Intermediates: Optional[list[Intermediate]] = None
    Segments: list[Segment]


# --- CONFIGURAÇÕES DE EXPORTAÇÃO ---
TEMPLATE_PATH: str = "./template.xlsx"
OUTPUT_DIR: str = "./output/"
FIRST_ROW: int = 15
LAST_ROW: int = 39

import os
import sys
import traceback
import openpyxl
from openpyxl.worksheet.worksheet import Worksheet

# --- 1. FUNÇÕES AUXILIARES DE VALIDAÇÃO E CONFIGURAÇÃO ---
def _load_and_validate_system(network: dict) -> tuple[System, dict[str, Primary], dict[str, Intermediate]]:
    """Valida os dados de entrada e carrega o modelo do sistema."""
    if not network:
        raise ValueError("Dados da rede hidráulica vazios.")

    if not os.path.exists(TEMPLATE_PATH):
        raise FileNotFoundError(f"Arquivo de template não encontrado em: {TEMPLATE_PATH}")

    print(network)
    system_data: System = System(**network)
    print(system_data)
    sys.exit(1)
    primary_dict: dict[str, Primary] = {p.ID: p for p in system_data.Primary}
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


# --- 2. FUNÇÃO AUXILIAR DE ESCRITA DE CÉLULAS (DRY) ---
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
    ws.cell(row=row, column=4, value=distance)
    ws.cell(row=row, column=5, value=terrain_level)
    ws.cell(row=row, column=6, value=slope)
    ws.cell(row=row, column=7, value=invert_level)
    ws.cell(row=row, column=8, value=f"=ROUNDUP((L{row}+1)/0.5, 0)*0.5")
    ws.cell(row=row, column=9, value=diameter)
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
    template_ws: Worksheet,
    wb: openpyxl.Workbook
) -> None:
    """Processa um segmento individual criando e populando sua respectiva aba na planilha."""
    seg_id = segment.ID
    upstream_pv, downstream_pv = _find_segment_nodes(segment, primaries, primary_dict)

    inter_ids = segment.Intermediate or []
    inter_dist = segment.IntermediateDistance
    pipe = segment.pipe
    slope = pipe.Slope
    diameter_val = pipe.Diameter
    seg_total_length = pipe.Length

    _validate_segment_constraints(segment, inter_ids, inter_dist)

    ws = wb.copy_worksheet(template_ws)
    ws.title = str(seg_id)

    current_row = FIRST_ROW
    cumulative_distance = 0.0

    # --- A. PV DE MONTANTE ---
    rn_up = _extract_rn(upstream_pv)
    connection = segment.connection
    kind = connection.kind if connection else ""
    inv_conn = connection.InvertLevel if connection else None
    obs_conn = f"{kind}={upstream_pv.TerrainLevel - inv_conn}" if kind and inv_conn is not None else ""

    _write_station_row(
        ws, current_row,
        rn=rn_up,
        id_val=upstream_pv.ID,
        stake_name="",
        distance=cumulative_distance,
        terrain_level=upstream_pv.TerrainLevel,
        slope=slope,
        invert_level=upstream_pv.InvertLevel,
        diameter=diameter_val,
        obs=obs_conn
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
            diameter=diameter_val
        )
        current_row += 1

    cumulative_distance = seg_total_length

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
            diameter=diameter_val
        )


# --- 4. FUNÇÃO PRINCIPAL (ORQUESTRADORA) ---
def export_network_to_xlsx(network: dict, output_filename: str = "relatorio_rede.xlsx") -> tuple[int, str]:
    """Orquestra o carregamento, processamento dos segmentos e salvamento do relatório em XLSX."""
    try:
        system_data, primary_dict, inter_dict = _load_and_validate_system(network)

        wb: openpyxl.Workbook = openpyxl.load_workbook(TEMPLATE_PATH)
        template_ws: Worksheet | None = wb.active

        if not template_ws:
            raise ValueError("No Active Worksheet found")

        total_segments_processed: int = 0

        for segment in system_data.Segments:
            _process_segment(
                segment=segment,
                primaries=system_data.Primary,
                primary_dict=primary_dict,
                inter_dict=inter_dict,
                template_ws=template_ws,
                wb=wb
            )
            total_segments_processed += 1

        # Remove o template original se foram criadas novas abas
        if len(wb.worksheets) > 1 and template_ws in wb.worksheets:
            wb.remove(template_ws)

        os.makedirs(OUTPUT_DIR, exist_ok=True)
        output_path = os.path.join(OUTPUT_DIR, output_filename)
        wb.save(output_path)

        return total_segments_processed, output_path

    except Exception as e:
        print("[ERROR]", file=sys.stderr)
        print(traceback.format_exc(), file=sys.stderr)
        print(e, file=sys.stderr)
        sys.exit(1)

def run_export_xlsv(request: dict):
    params: dict | None = request.get("params")

    if not params:
        raise ValueError("Params field not found")

    network: dict | None = params.get("network")

    if not network:
        raise ValueError("No hydraulic network data found in request payload. Import network data first.")

    result = export_network_to_xlsx(network)
    if not result:
        raise RuntimeError("Falha na geração do relatório Excel.")

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
            "message": f"Relatório gerado com sucesso ({seg_count} workspaces/segmentos criados) e salvo em {saved_path}"
        },
        "id": req_id
    }

"""
Explicando as posições dos valores.

Todos os valores podem ser posicionados apartir da coluna 14 até no máximo 39.

RN fica na linha A e somento nomes do PV possuem RN.
Nomes do PV deve começar na linha B.
nomes das estacas intermediarias deve começar na coluna C, ela tem que ficar entre as linhas horizontais das estacas do nomes do PV.
Valores da distância acumulada devem ficar na linha D.
Valores da cota do terreno estaca (m) que ficam na linha E
Declividade ficam na linha F
Cota Geratriz inferior interna do tubo na linha G
Comprimento da cruzeta fica na linha H e segue a formula "=ROUNDUP((L{Número da Linha atual}+1)/0.5,0)*0.5"
Diametro interno + Espessura do Tubo fica na linha I
Cota Geratriz superior externa do tubo fica na linha J e segue a formula "=G{número da linha atual}+I{número da linha atual}"
Cota borda superior da regua fica na linha K e segue a formula "=J{número da linha atual}+H{número da linha atual}"
Profundidade da vala fica na linha L e segue a formula "=E{número da linha atual}-G{número da linha atual}"
Altura da régua em relação a estaca fica na linha M e segue a formula "=H{número da linha atual} - L{número da linha atual}"
OBS de uma estaca (seja intermediaria ou de pv) ficam na linha N.

A nomeação das estacas intermediarias e dos PV pode ser possívelmente diferente das recebidas.

Como a nomeação dos PV e Estacas mudam de analista para analista, idealmente devemos colocar nos parâmetros do programa uma forma
de que o usuário possa customizar esses nomes. Pois mesmo na mesma empresa (Sanepar nesse caso), a forma de representar os pvs e estacas
pode mudar de analista para analista.

A primeira versão aqui não terá parametros para isso embora sera uma das primeiras atualizações futuras para dar suporte a isso.
As nomeações tem como base a que eu recebi da empresa GXH de um estilo especifico de analista, segue as regras abaixo.

Os valores da tabela que recebo da SANCAD começam com as iniciais SB seguido de um número, honestamente não sei o que significa.
Depois temos um tracho e um sequencia de três digitos que representa uma seção especifica. Essa seção não muda de acordo com o coletor.
Ou seja, se em uma mesma rede temos 4 coletores (poderiamos imaginar como galhos de um tronco) as seções de cada um desses coletores não
mudam de numeração conforme o coletor.

SB1-001 é um exemplo de nome de um PV.

Agora para números das Estacas isso acaba afetando, cada coletor muda o número inicial da estaca, e sua contagem muda de acordo com o coletor.
ENtão no coletor 001 uma estaca segue a númeração como 1-16E1. O primeiro digito é o número do coletor que ela está, os dois núemros representa
a seção que ele está. então digamos que temos três PV (cada seção é determinada por um trecho entre dois PVs) se o coletor é o 001, par ao primeiro
trecho (que no caso é separado em 20m) caso o pv tenha mais de 20 metros de distancia pore menos de 40, a primeira estaca seria 1-1E1 e acabaria por ali.
mas caso tivesse duas estacas intermediarias na mesma seção, nos teriamos uma outra estaca ao lado da 1-1E1 com a númeração 1-1E2.

Ou seja: O primeiro digito representa o coletor, o segundo digito representa o número da seção dentro do coletor (em ordem de fluxo), a letra E é
sigla para estaca, e o último número representa o número da estaca dentro daquela seção (em ordem de fluxo).

Agora que entendemos como é chegado o dado esperado e como é representado, vamos a como deve ser representado os valores.

O primeiro PV de um coletor é chamado de Terminal de Limpeza, e recebe uma nemração como "SB1-TL001". O segundo PV recebe nome como "SB1-PV002".
A numeração da estaca é apenas representada como um único digito, sendo ele o número da estaca naquela seção especifica.

Existe uma folha (ou workspace) para cada seção existente em uma rede coletora de esgoto.

Vale lembrar que nesse caso e template, o campo OBS contem sempre o tipo de conexão caso tenha, por exemplo se tem um Degrau de Queda ou seja,
se o segmeneto tem um campo "Connection", nós colocamos ele não em um Primary ou Estaca/Intermediates mas sim em OBS, onde vai ter o "kind" e
o valor de profunidade de topo para o GI mencionado em "InvertLevel" no mesmo campo.

# Como conseguir os dados.
1. Começemos pegando a lista dos Primaries
2. Extraimos de cada um seu "TerrainLevel", "InvertLevel", "Depth" e o que ele quiser salved dentro de "psets", como o "RN" dentro de "Topo" dentro do "psets"
    ou então "OBS" que também é um campo que no template é possivel ter caso tenha.
3. Agora para pegar as intermediarias nós peegamos o que tiver em "DownstreamConnection", lá teremos o ID do primary que ele se conecta e depois
    o ID Segmento que ele compoe.
4. Vamos ao Segmentos e pesquisemos o valor pelo seu ID, depois nós podemos pegar "Length" que será utilizado para calcular a Distância acumulada,
    o Length de lá representa o tamanho total do comprimento isso em conjunto com o campo "IntermediateDistance" dentro do Segments e a Quantidade
    de elementos presenta na lista em "Intermediates" também dentro do objeto Segmento nós conseguimos estrair as estacas, dentro de "Connection" caso
    exista vai conter duas informações, o "kind" que para esse programa não importa e o "InvertLevel" que representa a geratriz inferior do tubo
    na hora que se conecta ao tubo. quando não existe esse campo "Connection", é pq ele se conecta no fundo do PV, caso exista, o valor "InvertLevel" do
    "Connection" em conjunto com o nivel de topo do Primary mencionado em "DownstreamConnection" é possivel pegar o último valor que nos importa.
5. Pegamos as estacas em "Intermediates", pesquisando por eles como fizemos com Segmentos ou com o Primary, lá temos os valor que nos importa que é
    o "TerrainLevel" e "InvertLevel" e "Depth" para concluir os valores que nos importa.
6. Depois de ter todos esses dados nós inserimos nas devidas posições do template em conjunto com as formulas devidas.
7. Estaca intermediarias viam em seu devido lugar que é a estaca C, entre as Primary (afinal a gente também precisa colocar as informações dele).
"""
