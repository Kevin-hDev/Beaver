import math
from collections import OrderedDict

from .config_utils import config_int
from .limits import MAX_PREDICTIONS, MAX_SERIES
from .validation import (
    forecast_quantile_index,
    quantile_key,
    read_numeric_value,
    read_series_value,
    read_string_value,
    validate_column_name,
    validate_row_dicts,
)


def build_joint_job(payload, horizon):
    rows = validate_row_dicts(payload.get("history_rows"), "invalid_history")
    future_rows = validate_row_dicts(payload.get("future_rows"), "invalid_future_rows")
    target_column = validate_column_name(
        payload.get("target_column"), "invalid_target_column"
    )
    date_column = validate_column_name(
        payload.get("date_column"), "invalid_date_column"
    )
    series_column = validate_column_name(
        payload.get("series_column"), "invalid_series_column"
    )
    if not rows:
        raise ValueError("invalid_history")

    grouped = OrderedDict()
    for row in rows:
        series_id = read_series_value(row, series_column)
        entry = grouped.setdefault(series_id, {"dates": [], "values": []})
        entry["dates"].append(read_string_value(row.get(date_column), "invalid_date"))
        entry["values"].append(
            read_numeric_value(row.get(target_column), "invalid_target")
        )

    if len(grouped) > MAX_SERIES or len(grouped) * horizon > MAX_PREDICTIONS:
        raise ValueError("prediction_budget_exceeded")
    _validate_alignment(grouped)
    _trim_history(grouped, payload)
    return {
        "series_ids": list(grouped),
        "values": [entry["values"] for entry in grouped.values()],
        "dates": _future_dates(
            future_rows, list(grouped), series_column, date_column, horizon
        ),
    }


def extract_quantile_grids(raw, quantile_levels, horizon, series_count):
    if hasattr(raw, "detach"):
        raw = raw.detach().cpu()
    if hasattr(raw, "tolist"):
        raw = raw.tolist()
    if not isinstance(raw, list) or len(raw) < 9:
        raise ValueError("prediction_failed")

    grids = []
    for level in quantile_levels:
        index = forecast_quantile_index(level)
        try:
            grid = raw[index][0]
        except (IndexError, TypeError):
            raise ValueError("prediction_failed") from None
        grids.append(_validated_grid(grid, series_count, horizon))
    median_index = quantile_levels.index(0.5)
    median = grids[median_index]
    _validate_interval_order(
        grids[quantile_levels.index(0.1)],
        median,
        grids[quantile_levels.index(0.9)],
    )
    return median, grids


def format_joint_predictions(job, median, quantiles, quantile_levels, horizon):
    predictions = []
    quantile_keys = [quantile_key(level) for level in quantile_levels]
    for series_index, series_id in enumerate(job["series_ids"]):
        dates = job["dates"][series_id]
        for step in range(horizon):
            value = median[series_index][step]
            item = {
                "date": dates[step],
                "value": value,
                "series_id": series_id,
                "q50": value,
            }
            for key, grid in zip(quantile_keys, quantiles, strict=True):
                item[key] = grid[series_index][step]
            predictions.append(item)
    return {"predictions": predictions}


def _validate_alignment(grouped):
    reference = None
    for entry in grouped.values():
        if not entry["values"]:
            raise ValueError("invalid_history")
        if reference is None:
            reference = entry["dates"]
        elif entry["dates"] != reference:
            raise ValueError("unaligned_series")


def _trim_history(grouped, payload):
    limit = config_int(payload, "context_length", 0, 0, 100000)
    if limit <= 0:
        return
    for entry in grouped.values():
        entry["dates"] = entry["dates"][-limit:]
        entry["values"] = entry["values"][-limit:]


def _future_dates(rows, series_ids, series_column, date_column, horizon):
    if not rows:
        defaults = [f"T+{step + 1}" for step in range(horizon)]
        return {series_id: defaults.copy() for series_id in series_ids}
    grouped = OrderedDict((series_id, []) for series_id in series_ids)
    for row in rows:
        series_id = read_series_value(row, series_column)
        if series_id not in grouped:
            raise ValueError("invalid_future_rows")
        grouped[series_id].append(
            read_string_value(row.get(date_column), "invalid_future_date")
        )
    if any(len(dates) != horizon for dates in grouped.values()):
        raise ValueError("invalid_future_rows")
    reference = next(iter(grouped.values()))
    if any(dates != reference for dates in grouped.values()):
        raise ValueError("unaligned_future_series")
    return grouped


def _validated_grid(grid, series_count, horizon):
    if not isinstance(grid, list) or len(grid) != series_count:
        raise ValueError("prediction_failed")
    result = []
    for series in grid:
        if not isinstance(series, list) or len(series) != horizon:
            raise ValueError("prediction_failed")
        values = [float(value) for value in series]
        if any(not math.isfinite(value) for value in values):
            raise ValueError("prediction_failed")
        result.append(values)
    return result


def _validate_interval_order(lower, median, upper):
    for lower_series, median_series, upper_series in zip(
        lower, median, upper, strict=True
    ):
        for low, middle, high in zip(
            lower_series, median_series, upper_series, strict=True
        ):
            if low > middle or middle > high:
                raise ValueError("prediction_failed")
