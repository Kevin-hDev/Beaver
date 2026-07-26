import sys
import tempfile
import types
import unittest
from pathlib import Path
from unittest.mock import patch

from forecast_runtime.tabpfn_adapter import TabPfnTsAdapter


class FakeSeries:
    dtype = types.SimpleNamespace(kind="f")

    def __init__(self, values):
        self.values = values

    def tolist(self):
        return self.values


class FakePredictions:
    def __init__(self, horizon, extra=False):
        length = horizon + int(extra)
        self.values = {
            "median": FakeSeries([50.0 + step for step in range(length)]),
            "q10": FakeSeries([10.0 + step for step in range(length)]),
            "q50": FakeSeries([40.0 + step for step in range(length)]),
            "q90": FakeSeries([90.0 + step for step in range(length)]),
        }
        self.columns = list(self.values)

    def __contains__(self, key):
        return key in self.values

    def __getitem__(self, key):
        return self.values[key]


class FakeFrame:
    def __init__(self, values):
        self.values = values


class FakePipeline:
    config = None
    calls = []
    extra = False

    def __init__(self, **config):
        FakePipeline.config = config

    def predict_df(self, context_df, future_df):
        FakePipeline.calls.append((context_df.values, future_df.values))
        return FakePredictions(len(future_df.values["timestamp"]), self.extra)


def fake_modules():
    pandas = types.SimpleNamespace(
        DataFrame=FakeFrame,
        date_range=lambda *args, **kwargs: ["generated"] * kwargs["periods"],
    )
    tabpfn = types.SimpleNamespace(
        TabPFNMode=types.SimpleNamespace(LOCAL="local"),
        TabPFNTSPipeline=FakePipeline,
    )
    return {"pandas": pandas, "tabpfn_time_series": tabpfn}


class TabPfnAdapterTests(unittest.TestCase):
    def setUp(self):
        FakePipeline.calls = []
        FakePipeline.config = None
        FakePipeline.extra = False
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.model_dir = Path(self.temp.name)
        self.checkpoint = self.model_dir.joinpath(
            "tabpfn-v3-regressor-v3_20260506_timeseries.ckpt"
        )
        self.checkpoint.write_bytes(b"fixture")

    def test_uses_local_checkpoint_and_real_dates_for_each_series(self):
        payload = {
            "history_rows": [
                {"date": "2026-01-01", "asset": "A", "value": 1.0},
                {"date": "2026-01-02", "asset": "A", "value": 2.0},
                {"date": "2026-01-01", "asset": "B", "value": 3.0},
                {"date": "2026-01-02", "asset": "B", "value": 4.0},
            ],
            "future_rows": [
                {"date": "2026-01-03", "asset": "B"},
                {"date": "2026-01-04", "asset": "B"},
                {"date": "2026-01-03", "asset": "A"},
                {"date": "2026-01-04", "asset": "A"},
            ],
            "date_column": "date",
            "target_column": "value",
            "series_column": "asset",
            "covariate_columns": [],
        }
        with patch.dict(sys.modules, fake_modules()):
            result = TabPfnTsAdapter(
                "tabpfn-ts", "tabpfn-ts-3", self.model_dir, "cpu"
            ).predict(payload, 2, [0.1, 0.5, 0.9])

        self.assertEqual(
            FakePipeline.config["tabpfn_model_config"]["model_path"],
            str(self.checkpoint),
        )
        self.assertEqual(
            FakePipeline.calls[0][0]["timestamp"],
            ["2026-01-01", "2026-01-02"],
        )
        self.assertEqual(
            FakePipeline.calls[0][1]["timestamp"],
            ["2026-01-03", "2026-01-04"],
        )
        self.assertEqual(
            [item["date"] for item in result["predictions"]],
            ["2026-01-03", "2026-01-04", "2026-01-03", "2026-01-04"],
        )
        self.assertTrue(
            all(item["value"] == item["q50"] for item in result["predictions"])
        )

    def test_rejects_wrong_output_length_and_covariates(self):
        adapter = TabPfnTsAdapter("tabpfn-ts", "model", self.model_dir, "cpu")
        with self.assertRaisesRegex(ValueError, "covariates_not_supported"):
            adapter.predict(
                {"values": [1.0], "covariate_columns": ["temp"]},
                2,
                [0.1, 0.5, 0.9],
            )

        FakePipeline.extra = True
        with patch.dict(sys.modules, fake_modules()):
            with self.assertRaisesRegex(ValueError, "prediction_failed"):
                adapter.predict({"values": [1.0, 2.0]}, 2, [0.1, 0.5, 0.9])


if __name__ == "__main__":
    unittest.main()
