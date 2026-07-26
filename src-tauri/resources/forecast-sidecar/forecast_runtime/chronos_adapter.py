import json

import pandas as pd
import torch
from chronos import BaseChronosPipeline, Chronos2Pipeline

from .config_utils import config_int, trim_history
from .device_utils import move_tensor, transformers_device_map
from .validation import (
    quantile_key,
    read_numeric_value,
    read_series_value,
    read_string_value,
    validate_column_name,
    validate_column_names,
    validate_optional_column_name,
    validate_row_dicts,
    validate_values,
)


class ChronosAdapter:
    def __init__(self, _family_id, _model_name, model_dir, device="gpu"):
        self.device = device
        self.kind, self.pipeline = self._load_pipeline(model_dir)

    def predict(self, payload, horizon, quantile_levels):
        if self.kind == "chronos2" and "history_rows" in payload:
            return self._predict_df(payload, horizon, quantile_levels)
        values = trim_history(validate_values(payload.get("values")), payload)
        if self.kind == "chronos2":
            return self._predict_chronos2(values, horizon, quantile_levels)
        return self._predict_bolt(values, horizon, quantile_levels)

    def _predict_bolt(self, values, horizon, quantile_levels):
        context = move_tensor(torch.tensor(values, dtype=torch.float32), self.device)
        quantiles, median = self.pipeline.predict_quantiles(
            context, prediction_length=horizon, quantile_levels=quantile_levels
        )
        return self._simple_result(
            quantiles.squeeze(0), median.squeeze(0), quantile_levels
        )

    def _predict_chronos2(self, values, horizon, quantile_levels):
        context = move_tensor(
            torch.tensor(values, dtype=torch.float32).view(1, 1, -1), self.device
        )
        quantile_list, mean_list = self.pipeline.predict_quantiles(
            context, prediction_length=horizon, quantile_levels=quantile_levels
        )
        return self._simple_result(
            quantile_list[0].squeeze(0), mean_list[0].squeeze(0), quantile_levels
        )

    def _predict_df(self, payload, horizon, quantile_levels):
        history_rows = validate_row_dicts(
            payload.get("history_rows"), "invalid_history_rows"
        )
        future_rows = validate_row_dicts(
            payload.get("future_rows"), "invalid_future_rows"
        )
        date_column = validate_column_name(
            payload.get("date_column"), "invalid_date_column"
        )
        target_column = validate_column_name(
            payload.get("target_column"), "invalid_target_column"
        )
        series_column = validate_optional_column_name(payload.get("series_column"))
        covariate_columns = validate_column_names(payload.get("covariate_columns"))

        if not history_rows:
            raise ValueError("invalid_history_rows")
        if future_rows and series_column is None and len(future_rows) != horizon:
            raise ValueError("invalid_future_rows")

        history_df = pd.DataFrame(
            [
                build_history_record(
                    row,
                    date_column,
                    target_column,
                    series_column,
                    covariate_columns,
                )
                for row in history_rows
            ]
        )
        context_length = config_int(payload, "context_length", 0, 0, 100000)
        if context_length:
            history_df = history_df.groupby("item_id", group_keys=False).tail(
                context_length
            )
        future_df = None
        if future_rows:
            future_df = pd.DataFrame(
                [
                    build_future_record(
                        row, date_column, covariate_columns, series_column
                    )
                    for row in future_rows
                ]
            )

        predictions = self.pipeline.predict_df(
            history_df,
            future_df=future_df,
            id_column="item_id",
            timestamp_column="timestamp",
            target="target",
            prediction_length=horizon,
            quantile_levels=quantile_levels,
        ).sort_values(["item_id", "timestamp"])

        return {
            "predictions": [
                build_prediction_record(record, quantile_levels)
                for record in predictions.to_dict("records")
            ]
        }

    def _simple_result(self, quantiles, median, quantile_levels):
        result = {"median": median.tolist()}
        for index, level in enumerate(quantile_levels):
            result[quantile_key(level)] = quantiles[:, index].tolist()
        return result

    def _load_pipeline(self, model_dir):
        config_path = model_dir.joinpath("config.json")
        config = json.loads(config_path.read_text(encoding="utf-8"))
        architectures = config.get("architectures") or []
        if "Chronos2Model" in architectures:
            return "chronos2", Chronos2Pipeline.from_pretrained(
                str(model_dir), device_map=transformers_device_map(self.device)
            )
        return "chronos_bolt", BaseChronosPipeline.from_pretrained(
            str(model_dir), device_map=transformers_device_map(self.device)
        )


def build_history_record(
    row, date_column, target_column, series_column, covariate_columns
):
    record = {
        "item_id": read_series_value(row, series_column),
        "timestamp": read_string_value(row.get(date_column), "invalid_date"),
        "target": read_numeric_value(row.get(target_column), "invalid_target"),
    }
    for name in covariate_columns:
        record[name] = row.get(name)
    return record


def build_future_record(row, date_column, covariate_columns, series_column=None):
    record = {
        "item_id": read_series_value(row, series_column),
        "timestamp": read_string_value(row.get(date_column), "invalid_date"),
    }
    for name in covariate_columns:
        record[name] = row.get(name)
    return record


def build_prediction_record(record, quantile_levels):
    value = float(record["predictions"])
    item = {
        "series_id": record["item_id"],
        "date": str(record["timestamp"]),
        "value": value,
        "q50": float(record.get("0.5", value)),
    }
    keys = []
    for level in quantile_levels:
        raw_key = str(level)
        key = quantile_key(level)
        if raw_key in record:
            item[key] = float(record[raw_key])
            keys.append(key)
    lower_key = next((key for key in keys if key < "q50"), None)
    upper_key = next((key for key in reversed(keys) if key > "q50"), None)
    if "q10" not in item and lower_key:
        item["q10"] = item[lower_key]
    if "q90" not in item and upper_key:
        item["q90"] = item[upper_key]
    return item
