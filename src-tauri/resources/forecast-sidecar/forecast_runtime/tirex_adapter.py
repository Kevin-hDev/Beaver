from .adapter_utils import forecast_payload_result, values_tensor
from .config_utils import config, standard_quantile_levels
from .device_utils import move_tensor, resolve_torch_device
from .quantile_utils import select_standard_quantiles


class TiRexAdapter:
    def __init__(self, _family_id, _model_name, model_dir, device="gpu"):
        self.model_dir = str(model_dir)
        self.device = device
        self.model = None

    def predict(self, payload, horizon, quantile_levels):
        quantile_levels = standard_quantile_levels(quantile_levels)
        return forecast_payload_result(
            payload,
            horizon,
            quantile_levels,
            lambda values, length, levels: self._forecast_one(
                values, length, levels, payload
            ),
        )

    def _forecast_one(self, values, horizon, quantile_levels, payload):
        import torch

        with torch.no_grad():
            quantiles, mean = self._load_model().forecast(
                context=move_tensor(values_tensor(values), self.device).view(1, -1),
                prediction_length=horizon,
            )
        if config(payload).get("output_type") == "mean":
            return mean[0][:horizon], None
        selected = select_standard_quantiles(
            quantiles[0], horizon, quantile_levels
        )
        return mean[0][:horizon], selected

    def _load_model(self):
        if self.model is None:
            from pathlib import Path
            from tirex import TiRexZero

            checkpoint = Path(self.model_dir).joinpath("model.ckpt")
            if not checkpoint.is_file():
                raise ValueError("model_checkpoint_missing")
            self.model = TiRexZero.from_pretrained(
                str(checkpoint),
                backend="torch",
                device=resolve_torch_device(self.device),
            )
        return self.model
