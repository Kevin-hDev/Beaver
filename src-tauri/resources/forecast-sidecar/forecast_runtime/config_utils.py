def config(payload):
    value = payload.get("model_config")
    return value if isinstance(value, dict) else {}


def config_bool(payload, key, default):
    value = config(payload).get(key)
    if isinstance(value, bool):
        return value
    return default


def config_int(payload, key, default, min_value=None, max_value=None):
    value = config(payload).get(key)
    if not isinstance(value, int):
        return default
    if min_value is not None and value < min_value:
        return default
    if max_value is not None and value > max_value:
        return default
    return value


def config_float(payload, key, default, min_value=None, max_value=None):
    value = config(payload).get(key)
    if not isinstance(value, (int, float)):
        return default
    value = float(value)
    if min_value is not None and value < min_value:
        return default
    if max_value is not None and value > max_value:
        return default
    return value


def trim_history(values, payload):
    limit = config_int(payload, "context_length", 0, 0, 100000)
    if limit <= 0 or len(values) <= limit:
        return values
    return values[-limit:]


def standard_quantile_levels(levels):
    normalized = []
    for level in levels:
        rounded = round_half_up(float(level), 1)
        rounded = max(0.1, min(0.9, rounded))
        if rounded not in normalized:
            normalized.append(rounded)
    return normalized or [0.1, 0.5, 0.9]


def round_half_up(value, digits):
    scale = 10**digits
    return int(value * scale + 0.5) / scale


def torch_dtype(payload, key="precision"):
    value = config(payload).get(key)
    if value in (None, "auto"):
        return None
    import torch

    return {
        "float32": torch.float32,
        "float16": torch.float16,
        "bfloat16": torch.bfloat16,
    }.get(value)
