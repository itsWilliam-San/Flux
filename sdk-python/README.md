Flux Python SDK for creating plugins.
---

A bunch of tools, functions, classes for easier development of plugins for Flux.

What is it
---

This SDK handles the JSON-RPC and the STDIN/STOUT communication as well as giving some basic functions and classes for working
with the [Network System](../docs/contracts/network/network-v1.md). A bunch of things should be out-of-box so the developers
may focus on the bussiness logics.

Installation
---

On your virtual environment or global Python use this command

```bash
pip install -e .
```

How to use it
---

```py
from flux_sdk import Plugin

class SayHiParams(BaseModel):
    name: str
    times: int = 1

plugin = Plugin(name="sample")

@plugin.method("ping")
def ping() -> str:
    return "pong"

@plugin.method("sum")
def sum(x: int, y: int) -> int:
    return x + y

@plugin.method("sayHi")
def sayHi(params: SayHiParams) -> str:
    retrun " ".join([f"Hi, {params.name}"] * params.times)

if __name__ == "__main__":
    plugin.logger.info("Sample Plugin Started")
    plugin.start()
```

> [!NOTE]
> Never uses the default `print` file (STDOUT) since this is used for communicate with Flux, if you need to register something for debugging
> or giving info for advance users, use the `logger` class inside the `Plugin` class.
> The default save location for your logs is on linux in the `$XDG_DATA_HOME/com.williamss.flux/logs` folder.
> For Windows it's the `%LOCALAPPDATA%/com.williamss.flux/logs` folder.


