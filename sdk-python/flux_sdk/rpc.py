from typing import Any

from pydantic import BaseModel


class Request(BaseModel):
    method: str
    params: dict[str, Any] | list[Any] | None = None
    id: str | int | None = None


class Error(BaseModel):
    code: int
    message: str
    data: str | None = None


class Response(BaseModel):
    result: Any | None = None
    error: Error | None = None
    id: str | int | None = None


# pre-made errors
_PARSE_ERROR = Error(code=-32700, message="Parse error")
INVALID_REQUEST = Error(code=-32600, message="Invalid Request")
METHOD_NOT_FOUND = Error(code=-32601, message="Method not found")
INVALID_PARAMS = Error(code=-32602, message="Invalid params")
INTERNAL_ERROR = Error(code=-32603, message="Internal error")


def make_error_response(
    error: Error,
    req_id: str | int | None = None,
    data: str | None = None,
) -> Response:
    if data is not None:
        error = error.model_copy(update={data: data})
    return Response(
        error=error,
        id=req_id,
    )
