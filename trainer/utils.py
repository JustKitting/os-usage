import json
import time
from pathlib import Path
from typing import Optional, Dict, Any


class MetricsLogger:
    """
    Lightweight JSON-lines metrics logger.

    Writes structured metrics for later analysis while also emitting
    the same payload to stdout for quick inspection.
    """

    def __init__(self, path: Optional[str] = None):
        self.path = Path(path) if path else None
        if self.path:
            self.path.parent.mkdir(parents=True, exist_ok=True)

    def log(self, event: str, payload: Dict[str, Any]):
        record = {
            "timestamp": time.time(),
            "event": event,
            **payload,
        }
        line = json.dumps(record, default=str)
        print(line, flush=True)

        if self.path:
            with self.path.open("a") as f:
                f.write(line + "\n")
