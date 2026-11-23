from typing import Optional
from pydantic import BaseModel, Field

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
