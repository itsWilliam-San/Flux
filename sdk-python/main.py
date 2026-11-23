from typing import Any, Callable
import logging

class Plugin:
    def __init__(self, name: str, logger: logging.Logger | None = None) -> None:
        self.name = name
        self.logger = logger or logging.getLogger(name)
        self._methods: dict[str, Callable[..., Any]] = {}

    def method(self, rpc_name: str) -> Callable[[Callable[..., Any]], Callable[..., Any]]:
        def decorator(func: Callable[..., Any]) -> Callable[..., Any]:
            self._methods[rpc_name] = func
            return func

        return decorator

    def dispatch(self, rpc_name: str, params: dict[str, Any]) -> Any:
        if rpc_name not in self._methods:
            raise ValueError(f"Método desconhecido: {rpc_name}")
        return self._methods[rpc_name](**params)


class Parameters(BaseModel):
    network: System
    parameters: FunParameters

class Message(BaseModel):
    id: int
    jsonrpc: str
    method: str
    params: Parameters
