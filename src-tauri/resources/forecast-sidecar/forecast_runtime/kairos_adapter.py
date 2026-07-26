from .adapter_utils import (
    forecast_payload_result,
    values_tensor,
)
from .config_utils import config_bool, standard_quantile_levels
from .device_utils import move_model, move_tensor
from .validation import forecast_quantile_index


class KairosAdapter:
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

        context = move_tensor(values_tensor(values), self.device).view(1, -1)
        preserve_positivity = config_bool(payload, "preserve_positivity", True)
        flipped = config_bool(payload, "average_with_flipped_input", True)
        generation = config_bool(payload, "generation", True)
        with torch.no_grad():
            output = move_model(self._load_model(), self.device).eval()(
                past_target=context,
                prediction_length=horizon,
                generation=generation,
                preserve_positivity=preserve_positivity,
                average_with_flipped_input=flipped,
            )
        quantile_forecast = output["prediction_outputs"][0]
        median = quantile_forecast[forecast_quantile_index(0.5), :horizon]
        selected = [
            quantile_forecast[forecast_quantile_index(level), :horizon]
            for level in quantile_levels
        ]
        return median, selected

    def _load_model(self):
        if self.model is None:
            from tsfm.model.kairos import AutoModel

            self.model = AutoModel.from_pretrained(
                self.model_dir, local_files_only=True
            )
        return self.model
