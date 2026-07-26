from .config_utils import config_bool, config_int, trim_history
from .config_utils import standard_quantile_levels
from .device_utils import move_model, move_tensor
from .toto_multivariate import (
    build_joint_job,
    extract_quantile_grids,
    format_joint_predictions,
)
from .validation import quantile_key, validate_values


class TotoAdapter:
    def __init__(self, _family_id, _model_name, model_dir, device="gpu"):
        self.model_dir = str(model_dir)
        self.device = device
        self.model = None

    def predict(self, payload, horizon, quantile_levels):
        quantile_levels = sorted(
            set(standard_quantile_levels(quantile_levels) + [0.1, 0.5, 0.9])
        )
        if payload.get("covariate_columns"):
            raise ValueError("covariates_not_supported")
        if payload.get("series_column"):
            job = build_joint_job(payload, horizon)
            median, quantiles = self._forecast_tensor(
                job["values"], horizon, quantile_levels, payload
            )
            return format_joint_predictions(
                job, median, quantiles, quantile_levels, horizon
            )
        values = trim_history(validate_values(payload.get("values")), payload)
        median, quantiles = self._forecast_tensor(
            [values], horizon, quantile_levels, payload
        )
        result = {"median": median[0]}
        for level, grid in zip(quantile_levels, quantiles, strict=True):
            result[quantile_key(level)] = grid[0]
        return result

    def _forecast_tensor(self, series_values, horizon, quantile_levels, payload):
        import torch

        model = move_model(self._load_model(), self.device).eval()
        patch_size = max(1, int(getattr(model.config, "patch_size", 32)))
        pad_count = (-len(series_values[0])) % patch_size
        padded = [
            ([values[0]] * pad_count) + values if pad_count else values
            for values in series_values
        ]
        masks = [
            ([False] * pad_count) + ([True] * len(values)) for values in series_values
        ]
        target = move_tensor(
            torch.tensor([padded], dtype=torch.float32), self.device
        )
        mask = move_tensor(torch.tensor([masks], dtype=torch.bool), self.device)
        series_ids = move_tensor(
            torch.tensor([[0] * len(series_values)], dtype=torch.long), self.device
        )
        decode_block_size = config_int(payload, "decode_block_size", 768, 1, 4096)
        has_missing = config_bool(payload, "has_missing_values", False) or bool(
            pad_count
        )
        with torch.no_grad():
            quantiles = model.forecast(
                {
                    "target": target,
                    "target_mask": mask,
                    "series_ids": series_ids,
                },
                horizon=horizon,
                decode_block_size=decode_block_size,
                has_missing_values=has_missing,
            )
        return extract_quantile_grids(
            quantiles, quantile_levels, horizon, len(series_values)
        )

    def _load_model(self):
        if self.model is None:
            from toto2 import Toto2Model

            self.model = Toto2Model.from_pretrained(self.model_dir)
        return self.model
