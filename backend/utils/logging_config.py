import logging
import sys
import json
import os
from datetime import datetime


class JSONFormatter(logging.Formatter):
    """Custom JSON formatter for structured logging compatible with Loki."""

    def format(self, record):
        log_entry = {
            "timestamp": datetime.fromtimestamp(record.created).isoformat() + "Z",
            "level": record.levelname,
            "logger": record.name,
            "message": record.getMessage(),
            "module": record.module,
            "function": record.funcName,
            "line": record.lineno,
            "service": "qdrant-backend",
        }

        # Add exception info if present
        if record.exc_info:
            log_entry["exception"] = self.formatException(record.exc_info)

        # Add any extra fields from the log record
        excluded_keys = [
            "name",
            "msg",
            "args",
            "levelname",
            "levelno",
            "pathname",
            "filename",
            "module",
            "lineno",
            "funcName",
            "created",
            "msecs",
            "relativeCreated",
            "thread",
            "threadName",
            "processName",
            "process",
            "exc_info",
            "exc_text",
            "stack_info",
        ]
        for key, value in record.__dict__.items():
            if key not in excluded_keys:
                log_entry[key] = value

        return json.dumps(log_entry)


def configure_logger(name: str = "qdrant_search") -> logging.Logger:
    logger = logging.getLogger(name)
    if not logger.handlers:
        handler = logging.StreamHandler(sys.stdout)

        # Use JSON formatting if in Kubernetes environment, regular formatting otherwise
        if os.getenv("KUBERNETES_SERVICE_HOST"):
            formatter = JSONFormatter()
        else:
            formatter = logging.Formatter(
                "%(asctime)s - %(name)s - %(levelname)s - %(message)s"
            )

        handler.setFormatter(formatter)
        logger.addHandler(handler)
        logger.setLevel(logging.INFO)
    return logger


logger = configure_logger()
