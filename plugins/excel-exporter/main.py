import os
import openpyxl
import traceback

TEMPLATE_PATH = "./template.xlsx"
OUTPUT_DIR = "./output/"
FIRST_ROW = 15
LAST_ROW = 39


def export_network_to_xlsx(network, output_filename="relatorio_rede.xlsx"):
    try:
        if not network:
            raise ValueError("Dados da rede hidráulica vazios.")

        if not os.path.exists(TEMPLATE_PATH):
            raise FileNotFoundError(f"Arquivo de template não encontrado em: {TEMPLATE_PATH}")

        primaries = network.get("Primary") or []
        intermediates_list = network.get("Intermediates") or []
        segments_list = network.get("Segments") or []

        primary_dict = {p.get("ID"): p for p in primaries}
        inter_dict = {i.get("ID"): i for i in intermediates_list}

        # Carrega o workbook base do template
        wb = openpyxl.load_workbook(TEMPLATE_PATH)

        # Mantém uma referência ao template original
        template_ws = wb.active

        total_segments_processed = 0

        for seg_idx, segment in enumerate(segments_list):
            seg_id = segment.get("ID", f"Segmento_{seg_idx}")

            # 1. Encontra o PV de Montante (Upstream) e Jusante (Downstream)
            upstream_pv = None
            downstream_pv = None

            for p in primaries:
                downstream_conn = p.get("DownstreamConnection")
                if downstream_conn and len(downstream_conn) > 1 and downstream_conn[1] == seg_id:
                    upstream_pv = p
                    downstream_id = downstream_conn[0]
                    downstream_pv = primary_dict.get(downstream_id)
                    break

            if not upstream_pv or not downstream_pv:
                continue

            # 2. Cria nova aba copiada do template
            ws = wb.copy_worksheet(template_ws)
            ws.title = str(seg_id)

            current_row = FIRST_ROW
            cumulative_distance = 0.0

            pipe = segment.get("Pipe", {})
            slope = float(pipe.get("Slope", 0.0))
            raw_diam = pipe.get("Diameter", 0.0)
            diameter_val = float(raw_diam / 1000.0) if isinstance(raw_diam, (int, float)) else 0.15
            seg_total_length = float(pipe.get("Length", 0.0))
            connection = segment.get("Connection") or {}


            # --- A. PV DE MONTANTE ---
            pv_up_id = upstream_pv.get("ID", "")
            t_level_up = float(upstream_pv.get("TerrainLevel", 0.0))
            i_level_up = float(upstream_pv.get("InvertLevel", 0.0))
            rn_up = upstream_pv.get("psets", {}).get("Topo", {}).get("RN", "")

            kind = connection.get('kind', "")
            inv_conn = connection.get('InvertLevel', "")
            obs_conn = f"{kind}={t_level_up - float(inv_conn)}" if kind and inv_conn is not None else ""

            ws.cell(row=current_row, column=1, value=str(rn_up))
            ws.cell(row=current_row, column=2, value=str(pv_up_id))
            ws.cell(row=current_row, column=3, value="")
            ws.cell(row=current_row, column=4, value=float(cumulative_distance))
            ws.cell(row=current_row, column=5, value=t_level_up)
            ws.cell(row=current_row, column=6, value=slope)
            ws.cell(row=current_row, column=7, value=i_level_up)

            # Fórmulas ajustadas
            ws.cell(row=current_row, column=8, value=f"=ROUNDUP((L{current_row}+1)/0.5, 0)*0.5")
            ws.cell(row=current_row, column=9, value=diameter_val)
            ws.cell(row=current_row, column=10, value=f"=G{current_row}+I{current_row}")
            ws.cell(row=current_row, column=11, value=f"=J{current_row}+H{current_row}")
            ws.cell(row=current_row, column=12, value=f"=E{current_row}-G{current_row}")
            ws.cell(row=current_row, column=13, value=f"=H{current_row}-L{current_row}")
            ws.cell(row=current_row, column=14, value=obs_conn)

            current_row += 1

            # --- B. ESTACAS INTERMEDIÁRIAS ---
            inter_ids = segment.get("Intermediates", [])
            inter_dist = float(segment.get("IntermediateDistance", 20))

            for i_id in inter_ids:
                if current_row > LAST_ROW:
                    break

                inter_data = inter_dict.get(i_id)
                if not inter_data: # Proteção contra NoneType adicionada aqui
                    continue

                cumulative_distance += inter_dist
                int_t = float(inter_data.get("Terrainlevel", inter_data.get("TerrainLevel", 0.0)))
                int_i = float(inter_data.get("InvertLevel", 0.0))

                ws.cell(row=current_row, column=1, value="")
                ws.cell(row=current_row, column=2, value="")
                ws.cell(row=current_row, column=3, value=str(i_id))
                ws.cell(row=current_row, column=4, value=float(cumulative_distance))
                ws.cell(row=current_row, column=5, value=int_t)
                ws.cell(row=current_row, column=6, value=slope)
                ws.cell(row=current_row, column=7, value=int_i)
                ws.cell(row=current_row, column=8, value=f"=ROUNDUP((L{current_row}+1)/0.5, 0)*0.5")
                ws.cell(row=current_row, column=9, value=diameter_val)
                ws.cell(row=current_row, column=10, value=f"=G{current_row}+I{current_row}")
                ws.cell(row=current_row, column=11, value=f"=J{current_row}+H{current_row}")
                ws.cell(row=current_row, column=12, value=f"=E{current_row}-G{current_row}")
                ws.cell(row=current_row, column=13, value=f"=H{current_row}-L{current_row}")
                ws.cell(row=current_row, column=14, value="")

                current_row += 1

            covered_dist = len(inter_ids) * inter_dist
            if seg_total_length > covered_dist:
                cumulative_distance = seg_total_length
            else:
                cumulative_distance += inter_dist

            # --- C. PV DE JUSANTE ---
            if current_row <= LAST_ROW:
                pv_down_id = downstream_pv.get("ID", "")
                t_level_down = float(downstream_pv.get("TerrainLevel", 0.0))
                i_level_down = float(downstream_pv.get("InvertLevel", 0.0))
                rn_down = downstream_pv.get("psets", {}).get("Topo", {}).get("RN", "")

                ws.cell(row=current_row, column=1, value=str(rn_down))
                ws.cell(row=current_row, column=2, value=str(pv_down_id))
                ws.cell(row=current_row, column=3, value="")
                ws.cell(row=current_row, column=4, value=float(cumulative_distance))
                ws.cell(row=current_row, column=5, value=t_level_down)
                ws.cell(row=current_row, column=6, value=slope)
                ws.cell(row=current_row, column=7, value=i_level_down)
                ws.cell(row=current_row, column=8, value=f"=ROUNDUP((L{current_row}+1)/0.5, 0)*0.5")
                ws.cell(row=current_row, column=9, value=diameter_val)
                ws.cell(row=current_row, column=10, value=f"=G{current_row}+I{current_row}")
                ws.cell(row=current_row, column=11, value=f"=J{current_row}+H{current_row}")
                ws.cell(row=current_row, column=12, value=f"=E{current_row}-G{current_row}")
                ws.cell(row=current_row, column=13, value=f"=H{current_row}-L{current_row}")
                ws.cell(row=current_row, column=14, value="")

            total_segments_processed += 1

        # Remove o template original se foram criadas novas abas
        if len(wb.worksheets) > 1 and template_ws in wb.worksheets:
            wb.remove(template_ws)

        os.makedirs(OUTPUT_DIR, exist_ok=True)
        output_path = os.path.join(OUTPUT_DIR, output_filename)
        wb.save(output_path)

        return total_segments_processed, output_path
    except Exception as e:
        print("[ERROR]")
        traceback.print_exc()
        print(e)

def run_export_xlsv(request):
    params = request.get("params", {}) if isinstance(request, dict) and "params" in request else request

    network = params.get("network")
    if not network:
        raise ValueError("No hydraulic network data found in request payload. Import network data first.")

    seg_count, saved_path = export_network_to_xlsx(network)

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
