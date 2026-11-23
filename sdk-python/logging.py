import logging
from pathlib import Path

def setup_logging(log_dir_arg: str, plugin_id: str) -> Path:
    log_dir = Path(log_dir_arg) if log_dir_arg else Path("logs")
    log_dir.mkdir(parents=True, exist_ok=True)

    log_file = log_dir / f"{plugin_id}.log"

    logger = logging.getLogger(plugin_id)
    logger.setLevel(logging.INFO)
    logger.propagate = False

    if not logger.handlers:
        file_handler = logging.FileHandler(log_file, encoding="utf-8")
        formatter = logging.Formatter(fmt="%(asctime)s [%(levelname)s] %(name)s: %(message)s", datefmt="%Y-%m-%d %H:%M:%S")
        file_handler.setFormatter(formatter)
        logger.addHandler(file_handler)

    return log_file
