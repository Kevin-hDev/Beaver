import os
from collections import OrderedDict

from .adapter_utils import forecast_jobs, row_series_id
from .config_utils import config_bool, config_int
from .timesfm_output import format_result
from .validation import read_numeric_value, validate_column_names

MAX_CONTEXT = 16_384


class TimesFmAdapter:
    def __init__(self, _family_id, _model_name, model_dir, device="gpu"):
        self.model_dir = str(model_dir)
        self.device = device
        self.model = None
        self.compiled_signature = None

    def predict(self, payload, horizon, quantile_levels):
        levels = validate_decile_levels(quantile_levels)
        jobs = forecast_jobs(payload, horizon)
        validate_future_dates(payload, jobs, horizon)
        covariates = strict_covariate_columns(payload)
        dynamic = build_dynamic_covariates(payload, jobs, horizon, covariates)
        inputs = [job["values"] for job in jobs]
        model = self._compiled_model(payload, inputs, horizon, bool(dynamic))

        if dynamic:
            points, quantiles = model.forecast_with_covariates(
                inputs=inputs,
                dynamic_numerical_covariates=dynamic,
                xreg_mode="xreg + timesfm",
            )
        else:
            points, quantiles = model.forecast(horizon=horizon, inputs=inputs)
        return format_result(jobs, points, quantiles, levels, horizon)

    def _compiled_model(self, payload, inputs, horizon, with_covariates):
        if self.device == "cpu":
            os.environ["CUDA_VISIBLE_DEVICES"] = ""
        import timesfm

        if self.model is None:
            self.model = timesfm.TimesFM_2p5_200M_torch.from_pretrained(
                self.model_dir
            )
        configured = config_int(payload, "context_length", 0, 0, MAX_CONTEXT)
        observed = min(MAX_CONTEXT, max(len(values) for values in inputs))
        context = max(32, configured or observed)
        positive = config_bool(payload, "non_negative_output", False)
        signature = (context, horizon, with_covariates, positive)
        if signature != self.compiled_signature:
            self.model.compile(
                timesfm.ForecastConfig(
                    max_context=context,
                    max_horizon=horizon,
                    normalize_inputs=True,
                    use_continuous_quantile_head=True,
                    force_flip_invariance=True,
                    infer_is_positive=positive,
                    fix_quantile_crossing=True,
                    return_backcast=with_covariates,
                )
            )
            self.compiled_signature = signature
        return self.model


def strict_covariate_columns(payload):
    raw = payload.get("covariate_columns")
    if raw in (None, []):
        return []
    columns = validate_column_names(raw)
    if not isinstance(raw, list) or len(columns) != len(raw):
        raise ValueError("invalid_covariates")
    if len(set(columns)) != len(columns):
        raise ValueError("invalid_covariates")
    return columns


def grouped_rows(payload, key):
    rows = payload.get(key)
    if not isinstance(rows, list) or not rows:
        raise ValueError("invalid_covariates")
    grouped = OrderedDict()
    for row in rows:
        if not isinstance(row, dict):
            raise ValueError("invalid_covariates")
        grouped.setdefault(row_series_id(row, payload.get("series_column")), []).append(
            row
        )
    return grouped


def validate_future_dates(payload, jobs, horizon):
    rows = payload.get("future_rows")
    if not rows:
        return
    grouped = grouped_rows(payload, "future_rows")
    expected_ids = [job["series_id"] for job in jobs]
    invalid_count = any(len(items) != horizon for items in grouped.values())
    if set(grouped) != set(expected_ids) or invalid_count:
        raise ValueError("invalid_future_rows")
    date_column = payload.get("date_column")
    if not isinstance(date_column, str) or not date_column.strip():
        raise ValueError("invalid_future_rows")
    for items in grouped.values():
        if any(
            not isinstance(row.get(date_column), str)
            or not row[date_column].strip()
            for row in items
        ):
            raise ValueError("invalid_future_rows")


def build_dynamic_covariates(payload, jobs, horizon, columns):
    if not columns:
        return None
    history = grouped_rows(payload, "history_rows")
    future = grouped_rows(payload, "future_rows")
    expected_ids = [job["series_id"] for job in jobs]
    if list(history) != expected_ids or set(future) != set(expected_ids):
        raise ValueError("invalid_covariates")
    result = {column: [] for column in columns}
    for job in jobs:
        series_id = job["series_id"]
        history_rows = history[series_id][-len(job["values"]):]
        future_rows = future[series_id]
        if len(future_rows) != horizon:
            raise ValueError("invalid_covariates")
        for column in columns:
            values = [
                read_numeric_value(row.get(column), "invalid_covariates")
                for row in history_rows + future_rows
            ]
            result[column].append(values)
    return result


def validate_decile_levels(levels):
    if not isinstance(levels, list) or not levels:
        raise ValueError("invalid_quantiles")
    normalized = []
    for level in levels:
        if isinstance(level, bool) or not isinstance(level, (int, float)):
            raise ValueError("invalid_quantiles")
        value = float(level)
        decile = round(value * 10)
        if decile < 1 or decile > 9 or abs(value * 10 - decile) > 1e-6:
            raise ValueError("invalid_quantiles")
        normalized.append(decile / 10)
    if normalized != sorted(set(normalized)) or 0.5 not in normalized:
        raise ValueError("invalid_quantiles")
    return normalized
