from collections import OrderedDict

from .config_utils import trim_history
from .limits import MAX_PREDICTIONS, MAX_SERIES
from .validation import (
    quantile_key,
    read_numeric_value,
    read_series_value,
    validate_values,
)


def values_tensor(values):
    import torch

    return torch.tensor(values, dtype=torch.float32)


def to_float_list(value):
    if hasattr(value, "detach"):
        value = value.detach().cpu()
    if hasattr(value, "numpy"):
        value = value.numpy()
    if hasattr(value, "tolist"):
        value = value.tolist()
    if isinstance(value, (int, float)):
        return [float(value)]
    return [float(item) for item in flatten(value)]


def flatten(value):
    if isinstance(value, (list, tuple)):
        for item in value:
            yield from flatten(item)
    else:
        yield value


def simple_result(median, quantile_levels, quantiles=None, horizon=None):
    median_values = to_float_list(median)
    if horizon is not None:
        median_values = median_values[:horizon]
    result = {"median": median_values}
    if quantiles is None:
        return result
    quantile_values = quantiles_to_lists(
        quantiles, len(quantile_levels), len(median_values)
    )
    for index, level in enumerate(quantile_levels):
        if index < len(quantile_values):
            result[quantile_key(level)] = quantile_values[index]
    return result


def forecast_payload_result(payload, horizon, quantile_levels, forecast_one):
    jobs = forecast_jobs(payload, horizon)
    if len(jobs) == 1 and jobs[0]["series_id"] is None:
        median, quantiles = forecast_one(jobs[0]["values"], horizon, quantile_levels)
        return simple_result(median, quantile_levels, quantiles, horizon)

    predictions = []
    for job in jobs:
        median, quantiles = forecast_one(job["values"], horizon, quantile_levels)
        median_values = to_float_list(median)[:horizon]
        quantile_values = quantiles_to_lists(
            quantiles, len(quantile_levels), len(median_values)
        )
        quantile_by_key = {
            quantile_key(level): values
            for level, values in zip(quantile_levels, quantile_values, strict=False)
        }
        lower_key = next((key for key in quantile_by_key if key < "q50"), None)
        upper_key = next(
            (key for key in reversed(quantile_by_key) if key > "q50"), None
        )
        for index, value in enumerate(median_values):
            date = (
                job["dates"][index]
                if index < len(job["dates"])
                else f"T+{index + 1}"
            )
            item = {
                "date": date,
                "value": value,
                "series_id": job["series_id"],
            }
            for key in quantile_by_key:
                quantile_value = quantile_at(quantile_by_key, key, index)
                if quantile_value is not None:
                    item[key] = quantile_value
            if "q10" not in item and lower_key in item:
                item["q10"] = item[lower_key]
            if "q90" not in item and upper_key in item:
                item["q90"] = item[upper_key]
            predictions.append(item)
    return {"predictions": predictions}


def forecast_jobs(payload, horizon):
    rows = payload.get("history_rows")
    target_column = payload.get("target_column")
    if not rows or not target_column:
        return [
            {
                "series_id": None,
                "values": trim_history(validate_values(payload.get("values")), payload),
                "dates": default_future_dates(horizon),
            }
        ]

    series_column = payload.get("series_column")
    grouped = OrderedDict()
    for row in rows:
        if not isinstance(row, dict):
            raise ValueError("invalid_history")
        series_id = row_series_id(row, series_column)
        value = read_numeric_value(row.get(target_column), "invalid_target")
        grouped.setdefault(series_id, []).append(value)

    if len(grouped) > MAX_SERIES or len(grouped) * horizon > MAX_PREDICTIONS:
        raise ValueError("prediction_budget_exceeded")
    future_dates = grouped_future_dates(payload, horizon)
    return [
        {
            "series_id": series_id,
            "values": trim_history(values, payload),
            "dates": future_dates.get(series_id, default_future_dates(horizon)),
        }
        for series_id, values in grouped.items()
    ]


def grouped_future_dates(payload, horizon):
    rows = payload.get("future_rows") or []
    date_column = payload.get("date_column")
    if not rows or not date_column:
        return {}

    series_column = payload.get("series_column")
    grouped = OrderedDict()
    for row in rows:
        if not isinstance(row, dict):
            continue
        series_id = row_series_id(row, series_column)
        date = row.get(date_column)
        if isinstance(date, str) and date.strip():
            grouped.setdefault(series_id, []).append(date.strip())

    for series_id, dates in grouped.items():
        if len(dates) < horizon:
            dates.extend(default_future_dates(horizon - len(dates), len(dates)))
        grouped[series_id] = dates[:horizon]
    return grouped


def default_future_dates(horizon, offset=0):
    return [f"T+{offset + index + 1}" for index in range(horizon)]


def row_series_id(row, series_column):
    if not series_column:
        return None
    return read_series_value(row, series_column)


def quantile_at(quantile_by_key, key, index):
    values = quantile_by_key.get(key)
    if values is None or index >= len(values):
        return None
    return values[index]


def quantiles_to_lists(quantiles, quantile_count, horizon):
    if hasattr(quantiles, "detach"):
        quantiles = quantiles.detach().cpu()
    if hasattr(quantiles, "numpy"):
        quantiles = quantiles.numpy()
    if hasattr(quantiles, "tolist"):
        quantiles = quantiles.tolist()
    squeezed = squeeze_singletons(quantiles)
    if not isinstance(squeezed, list):
        return []
    if looks_like_horizon_first(squeezed, horizon):
        return [
            [float(row[index]) for row in squeezed[:horizon] if index < len(row)]
            for index in range(quantile_count)
        ]
    return [to_float_list(row)[:horizon] for row in squeezed[:quantile_count]]


def squeeze_singletons(value):
    while isinstance(value, list) and len(value) == 1 and isinstance(value[0], list):
        value = value[0]
    return value


def looks_like_horizon_first(value, horizon):
    return len(value) >= horizon and isinstance(value[0], list) and len(value[0]) > 1
