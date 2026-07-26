import math
import os
import unittest
from pathlib import Path

from forecast_runtime.adapters import load_adapter


SMOKE_ENABLED = os.environ.get("FORECAST_SMOKE") == "1"


@unittest.skipUnless(SMOKE_ENABLED, "opt-in Forecast model smoke test")
class InstalledModelSmokeTests(unittest.TestCase):
    def test_installed_model_predicts_offline(self):
        family = required_env("FORECAST_SMOKE_FAMILY")
        model = required_env("FORECAST_SMOKE_MODEL")
        models_dir = Path(required_env("FORECAST_SMOKE_MODELS_DIR"))
        if not models_dir.is_dir():
            self.fail("invalid smoke model directory")

        os.environ["HF_HUB_OFFLINE"] = "1"
        os.environ["TRANSFORMERS_OFFLINE"] = "1"
        adapter = load_adapter(
            family,
            model,
            models_dir,
            os.environ.get("FORECAST_SMOKE_DEVICE", "cpu"),
        )
        values = [20.0 + math.sin(index / 6.0) for index in range(64)]
        result = adapter.predict(
            {"values": values, "model_config": {}},
            3,
            [0.1, 0.5, 0.9],
        )
        median = result.get("median")
        self.assertIsInstance(median, list)
        self.assertEqual(len(median), 3)
        self.assertTrue(all(math.isfinite(float(value)) for value in median))
        for key in ("q10", "q50", "q90"):
            self.assertEqual(len(result.get(key, [])), 3)


def required_env(name):
    value = os.environ.get(name)
    if not value:
        raise AssertionError(f"missing {name}")
    return value


if __name__ == "__main__":
    unittest.main()
