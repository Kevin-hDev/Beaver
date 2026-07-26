import math

from .adapter_utils import (
    forecast_jobs,
    row_series_id,
    to_float_list,
)
from .config_utils import config_bool
from .validation import quantile_key, read_string_value


class TabPfnTsAdapter:
    def __init__(self, _family_id, _model_name, model_dir, device="gpu"):
        self.device = device
        self.model_dir = model_dir
        self.pipeline = None

    def predict(self, payload, horizon, quantile_levels):
        if payload.get("covariate_columns"):
            raise ValueError("covariates_not_supported")
        rows = []
        for job in forecast_jobs(payload, horizon):
            median, quantiles = self._forecast_one(
                job,
                history_dates(payload, job),
                horizon,
                quantile_levels,
                payload,
            )
            rows.append((job, median, quantiles))
        if len(rows) == 1 and rows[0][0]["series_id"] is None:
            _, median, quantiles = rows[0]
            return simple_tabpfn_result(
                median, quantiles, quantile_levels, horizon
            )
        return structured_result(rows, quantile_levels, horizon)

    def _forecast_one(
        self, job, history_dates, horizon, quantile_levels, payload
    ):
        context_df, future_df = self._frames(job, history_dates, payload)
        pipeline = self._load_pipeline()
        predictions = pipeline.predict_df(
            context_df=context_df,
            future_df=future_df,
        )
        median = self._prediction_values(predictions, horizon)
        if not config_bool(payload, "probabilistic_output", True):
            return median, None
        quantiles = self._prediction_quantiles(
            predictions, horizon, quantile_levels
        )
        if quantiles and 0.5 in quantile_levels:
            median = quantiles[quantile_levels.index(0.5)]
        return median, quantiles

    def _load_pipeline(self):
        if self.pipeline is None:
            from tabpfn_time_series import TabPFNMode, TabPFNTSPipeline

            checkpoint = self.model_dir.joinpath(
                "tabpfn-v3-regressor-v3_20260506_timeseries.ckpt"
            )
            if not checkpoint.is_file():
                raise ValueError("model_checkpoint_missing")
            self.pipeline = TabPFNTSPipeline(
                tabpfn_mode=TabPFNMode.LOCAL,
                tabpfn_model_config={"model_path": str(checkpoint)},
            )
        return self.pipeline

    def _frames(self, job, history_dates, payload):
        import pandas as pd

        values = job["values"]
        series_id = job["series_id"] or "series-1"
        context_dates = history_dates or pd.date_range(
            "2000-01-01", periods=len(values), freq="D"
        )
        future_dates = job["dates"]
        if any(str(date).upper().startswith("T+") for date in future_dates):
            future_dates = pd.date_range(
                start=context_dates[-1],
                periods=len(future_dates) + 1,
                freq=payload.get("frequency") or "D",
            )[1:]
        context = pd.DataFrame(
            {
                "item_id": [series_id] * len(values),
                "timestamp": context_dates,
                "target": values,
            }
        )
        future = pd.DataFrame(
            {
                "item_id": [series_id] * len(future_dates),
                "timestamp": future_dates,
            }
        )
        return context, future

    def _prediction_values(self, predictions, horizon):
        for column in ("median", "mean", "prediction", "target"):
            if column in predictions and predictions[column].dtype.kind in "fiu":
                return exact_values(predictions[column].tolist(), horizon)
        candidates = [
            column
            for column in predictions.columns
            if column not in {"item_id", "timestamp"}
            and predictions[column].dtype.kind in "fiu"
        ]
        if not candidates:
            raise ValueError("prediction_failed")
        return exact_values(predictions[candidates[0]].tolist(), horizon)

    def _prediction_quantiles(self, predictions, horizon, quantile_levels):
        selected = []
        for level in quantile_levels:
            column = self._quantile_column(predictions, level)
            if column is None:
                return None
            selected.append(exact_values(predictions[column].tolist(), horizon))
        return selected

    def _quantile_column(self, predictions, level):
        pct = int(round(level * 100))
        candidates = {
            f"q{pct}",
            f"q{pct:02d}",
            f"p{pct}",
            f"{level}",
            f"{level:.1f}",
            f"quantile_{level}",
            f"quantile_{pct}",
        }
        for column in predictions.columns:
            if str(column).lower() in candidates:
                return column
        return None


def history_dates(payload, job):
    rows = payload.get("history_rows")
    date_column = payload.get("date_column")
    if not rows or not date_column:
        return None
    series_column = payload.get("series_column")
    dates = []
    for row in rows:
        if not isinstance(row, dict):
            raise ValueError("invalid_history")
        series_id = row_series_id(row, series_column)
        if series_id == job["series_id"]:
            dates.append(read_string_value(row.get(date_column), "invalid_date"))
    if len(dates) < len(job["values"]):
        raise ValueError("invalid_dates")
    return dates[-len(job["values"]):]


def exact_values(values, horizon):
    result = to_float_list(values)
    if len(result) != horizon or any(not math.isfinite(value) for value in result):
        raise ValueError("prediction_failed")
    return result


def structured_result(rows, levels, horizon):
    predictions = []
    for job, median, quantiles in rows:
        median = exact_values(median, horizon)
        quantile_rows = [exact_values(values, horizon) for values in quantiles]
        if len(quantile_rows) != len(levels):
            raise ValueError("prediction_failed")
        for index, value in enumerate(median):
            item = {
                "date": job["dates"][index],
                "value": value,
                "series_id": job["series_id"],
            }
            for level, values in zip(levels, quantile_rows, strict=True):
                if len(values) != horizon:
                    raise ValueError("prediction_failed")
                item[quantile_key(level)] = values[index]
            predictions.append(item)
    return {"predictions": predictions}


def simple_tabpfn_result(median, quantiles, levels, horizon):
    result = {"median": exact_values(median, horizon)}
    rows = [exact_values(values, horizon) for values in quantiles]
    if len(rows) != len(levels):
        raise ValueError("prediction_failed")
    for level, values in zip(levels, rows, strict=True):
        result[quantile_key(level)] = values
    return result
