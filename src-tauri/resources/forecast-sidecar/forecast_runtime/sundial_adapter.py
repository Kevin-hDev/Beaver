from .adapter_utils import forecast_payload_result, values_tensor
from .config_utils import config, config_int, torch_dtype
from .device_utils import move_model, move_tensor


class SundialAdapter:
    def __init__(self, _family_id, _model_name, model_dir, device="gpu"):
        self.model_dir = str(model_dir)
        self.device = device
        self.model = None

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
        import torch

        sample_count = config_int(payload, "num_samples", 64, 1, 512)
        context = move_tensor(values_tensor(values), self.device).view(1, -1)
        with torch.no_grad():
            model = move_model(self._load_model(payload), self.device).eval()
            samples = model.generate(
                context,
                max_new_tokens=horizon,
                num_samples=sample_count,
            )
        sample_values = samples.float()
        if sample_values.shape[1] == sample_count:
            sample_values = sample_values[0, :, :horizon]
            sample_dim = 0
        elif len(sample_values.shape) > 2 and sample_values.shape[2] == sample_count:
            sample_values = sample_values[0, :horizon, :]
            sample_dim = 1
        else:
            raise ValueError("prediction_failed")
        median = torch.quantile(sample_values, 0.5, dim=sample_dim)[:horizon]
        quantiles = [
            torch.quantile(sample_values, level, dim=sample_dim)[:horizon]
            for level in quantile_levels
        ]
        return median, quantiles

    def _load_model(self, payload):
        dtype_name = config(payload).get("dtype", "auto")
        if self.model is None or getattr(self, "dtype_name", None) != dtype_name:
            from transformers import AutoModelForCausalLM

            kwargs = {"trust_remote_code": True, "local_files_only": True}
            dtype = torch_dtype(payload, "dtype")
            if dtype is not None:
                kwargs["torch_dtype"] = dtype
            self.model = AutoModelForCausalLM.from_pretrained(
                self.model_dir, **kwargs
            )
            self.dtype_name = dtype_name
        return self.model
