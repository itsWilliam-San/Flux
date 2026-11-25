import inspect
import json
import sys
from collections.abc import Callable
from typing import Any, get_type_hints

from pydantic import BaseModel, TypeAdapter, ValidationError

from . import logging as local_logging
from . import rpc


def _coerce_value(value: Any, annotation: Any) -> Any:
    if annotation is Any or annotation is inspect.Parameter.empty:
        return value
    return TypeAdapter(annotation).validate_python(value)


class Plugin:
    def __init__(self, name: str) -> None:
        self.name = name

        log_dir: str = sys.argv[1] if len(sys.argv) > 1 else ""
        self.logger = local_logging.setup_logging(log_dir, plugin_id=name)

        self._methods: dict[str, Callable[..., Any]] = {}

    def method(
        self, rpc_name: str
    ) -> Callable[[Callable[..., Any]], Callable[..., Any]]:
        def decorator(func: Callable[..., Any]) -> Callable[..., Any]:
            self._methods[rpc_name] = func
            return func

        return decorator

    def _emit(self, payload: rpc.Response) -> None:
        line = payload.model_dump_json(exclude_none=True)
        sys.stdout.write(line + "\n")
        sys.stdout.flush()

    def _invoke_handler(
        self,
        func: Callable[..., Any],
        params: dict[str, Any] | list[Any] | None,
    ) -> Any:
        sig = inspect.signature(func)
        type_hints = get_type_hints(func)

        if isinstance(params, list):
            pos_params = [
                p
                for p in sig.parameters.values()
                if p.kind
                in (
                    inspect.Parameter.POSITIONAL_ONLY,
                    inspect.Parameter.POSITIONAL_OR_KEYWORD,
                )
            ]

            if len(pos_params) == 1:
                only_param = pos_params[0]
                annotation = type_hints.get(only_param.name)
                if isinstance(annotation, type) and issubclass(annotation, BaseModel):
                    field_names = list(annotation.model_fields.keys())
                    if len(params) > len(field_names):
                        raise TypeError(
                            f"Too many positional arguments for {annotation.__name__}: "
                            f"got {len(params)}, expected at most {len(field_names)}."
                        )
                    mapped = dict(zip(field_names, params))
                    return func(annotation.model_validate(mapped))

            coerced_args: list[Any] = []
            has_var_positional = any(
                p.kind == inspect.Parameter.VAR_POSITIONAL
                for p in sig.parameters.values()
            )
            for idx, arg in enumerate(params):
                if idx < len(pos_params):
                    param = pos_params[idx]
                    annotation = type_hints.get(param.name, param.annotation)
                    coerced_args.append(_coerce_value(arg, annotation))
                elif has_var_positional:
                    coerced_args.append(arg)
                else:
                    raise TypeError(
                        f"Too many positional arguments for '{func.__name__}': "
                        f"got {len(params)}, expected at most {len(pos_params)}."
                    )

            return func(*coerced_args)

        dict_params = params or {}

        positional_or_kw = [
            p
            for p in sig.parameters.values()
            if p.kind
            in (inspect.Parameter.POSITIONAL_OR_KEYWORD, inspect.Parameter.KEYWORD_ONLY)
        ]

        if len(positional_or_kw) == 1:
            param = positional_or_kw[0]
            annotation = type_hints.get(param.name)
            if isinstance(annotation, type) and issubclass(annotation, BaseModel):
                validated_model = annotation.model_validate(dict_params)
                return func(validated_model)

        return func(**dict_params)

    def _handle_line(self, raw_line: str) -> None:
        self.logger.debug(f"Got string: {raw_line}")
        line = raw_line.strip()
        if not line:
            return

        try:
            req = rpc.Request.model_validate_json(line)
        except json.JSONDecodeError as exc:
            self.logger.exception("Got an invalid JSON on stdin.")
            self._emit(rpc.make_error_response(rpc._PARSE_ERROR, data=str(exc)))
            return

        handler = self._methods.get(req.method)
        if handler is None:
            self.logger.warning("Method not found: %s", req.method)
            if req.id is not None:
                error = rpc.make_error_response(rpc.METHOD_NOT_FOUND, req.id)
                self._emit(error)
            return

        try:
            result = self._invoke_handler(handler, req.params)
        except ValidationError as exc:
            self.logger.warning("Invalid params for method '%s': %s", req.method, exc)
            if req.id is not None:
                error = rpc.make_error_response(
                    rpc.INVALID_PARAMS, req.id, data=str(exc)
                )
                self._emit(error)
            return
        except Exception as exc:
            self.logger.exception("Unhandled error in method '%s'.", req.method)
            if req.id is not None:
                error = rpc.make_error_response(
                    rpc.INTERNAL_ERROR, req.id, data=str(exc)
                )
                self._emit(error)
            return

        if req.id is not None:
            self._emit(rpc.Response(id=req.id, result=result))

    def start(self) -> None:
        """Run the blocking stdin read loop until standard input is closed.

        Logs unexpected loop exceptions before re-raising and records a final
        exit message when the loop terminates.
        """
        try:
            for line in sys.stdin:
                self._handle_line(line)
        except Exception:
            self.logger.exception("Unexpected error on plugin loop")
            raise
        finally:
            self.logger.info("Plugin exited")
