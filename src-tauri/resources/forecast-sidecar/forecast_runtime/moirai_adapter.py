from .adapter_utils import forecast_payload_result
from .config_utils import config_int


class MoiraiAdapter:
    def __init__(self, _family_id, _model_name, model_dir, device="gpu"):
        self.model_dir = str(model_dir)
        self.device = device
        self.predictor = None
        self.predictor_key = None

    def predict(self, payload, horizon, quantile_levels):
        return forecast_payload_result(
            payload,
            horizon,
            quantile_levels,
            lambda values, length, levels: self._forecast_one(
                values, length, levels, payload
            ),
        )

    def _forecast_one(self, values, horizon, quantile_levels, payload):
        context_length = config_int(payload, "context_length", 0, 0, 100000)
        batch_size = config_int(payload, "batch_size", 1, 1, 1024)
        predictor = self._load_predictor(
            horizon, len(values), context_length, batch_size
        )
        dataset = self._dataset(values, "D")
        forecast = next(iter(predictor.predict(dataset)))
        median = forecast.quantile(0.5)[:horizon]
        quantiles = [forecast.quantile(level)[:horizon] for level in quantile_levels]
        return median, quantiles

    def _load_predictor(self, horizon, context_length, configured_context, batch_size):
        key = (horizon, configured_context, batch_size)
        if self.predictor is not None and self.predictor_key == key:
            return self.predictor
        from uni2ts.model.moirai2 import Moirai2Forecast, Moirai2Module

        model = Moirai2Forecast(
            module=Moirai2Module.from_pretrained(self.model_dir),
            prediction_length=horizon,
            context_length=min(
                configured_context or max(context_length, horizon), 1680
            ),
            target_dim=1,
            feat_dynamic_real_dim=0,
            past_feat_dynamic_real_dim=0,
        )
        self.predictor = model.create_predictor(batch_size=batch_size)
        self.predictor_key = key
        return self.predictor

    def _dataset(self, values, frequency):
        from gluonts.dataset.common import ListDataset

        return ListDataset([{"start": "2000-01-01", "target": values}], freq=frequency)
