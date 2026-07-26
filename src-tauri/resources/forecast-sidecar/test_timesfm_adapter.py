import sys
import types
import unittest
from unittest.mock import patch

from forecast_runtime.timesfm_adapter import TimesFmAdapter


def quantile_steps(base, horizon):
    return [
        [base + step + offset for offset in range(10)]
        for step in range(horizon)
    ]


class FakeMatrix:
    def __init__(self, rows):
        self.rows = rows
        self.shape = (len(rows), len(rows[0]))

    def __getitem__(self, key):
        if not isinstance(key, tuple):
            return self.rows[key]
        row_selector, column_selector = key
        rows = self.rows if row_selector is Ellipsis else self.rows[row_selector]
        if isinstance(column_selector, slice):
            return FakeMatrix([row[column_selector] for row in rows])
        return [row[column_selector] for row in rows]


class FakeForecastConfig:
    def __init__(self, **values):
        self.values = values


class FakeModel:
    def __init__(self):
        self.compiles = []
        self.forecast_call = None
        self.xreg_call = None

    def compile(self, config):
        self.compiles.append(config.values)

    def forecast(self, **kwargs):
        self.forecast_call = kwargs
        horizon = kwargs["horizon"]
        count = len(kwargs["inputs"])
        points = [
            [10.0 + index + step for step in range(horizon)]
            for index in range(count)
        ]
        quantiles = [
            FakeMatrix(quantile_steps(1.0 + index, horizon))
            for index in range(count)
        ]
        return points, quantiles

    def forecast_with_covariates(self, **kwargs):
        self.xreg_call = kwargs
        count = len(kwargs["inputs"])
        values = next(iter(kwargs["dynamic_numerical_covariates"].values()))[0]
        horizon = len(values) - len(kwargs["inputs"][0])
        points = [
            [20.0 + index + step for step in range(horizon)]
            for index in range(count)
        ]
        quantiles = [
            FakeMatrix(quantile_steps(2.0 + index, horizon))
            for index in range(count)
        ]
        return points, quantiles


class FakeLoader:
    model = None
    path = None

    @classmethod
    def from_pretrained(cls, path):
        cls.path = path
        cls.model = FakeModel()
        return cls.model


def fake_timesfm():
    return types.SimpleNamespace(
        TimesFM_2p5_200M_torch=FakeLoader,
        ForecastConfig=FakeForecastConfig,
    )


class TimesFmAdapterTests(unittest.TestCase):
    def setUp(self):
        FakeLoader.model = None
        FakeLoader.path = None
        self.module_patch = patch.dict(sys.modules, {"timesfm": fake_timesfm()})
        self.module_patch.start()
        self.addCleanup(self.module_patch.stop)

    def test_uses_only_the_official_timesfm_2_5_api(self):
        adapter = TimesFmAdapter("timesfm-2-5", "model", "/models/timesfm")
        result = adapter.predict(
            {"values": [1.0, 2.0, 3.0], "model_config": {}},
            2,
            [0.1, 0.5, 0.9],
        )

        self.assertEqual(FakeLoader.path, "/models/timesfm")
        self.assertEqual(
            FakeLoader.model.forecast_call,
            {"horizon": 2, "inputs": [[1.0, 2.0, 3.0]]},
        )
        self.assertEqual(result["median"], [6.0, 7.0])
        self.assertEqual(result["median"], result["q50"])
        self.assertEqual(result["q10"], [2.0, 3.0])
        config = FakeLoader.model.compiles[0]
        self.assertEqual(config["max_context"], 32)
        self.assertEqual(config["max_horizon"], 2)
        self.assertFalse(config["return_backcast"])

    def test_batches_series_and_passes_complete_numeric_xreg(self):
        payload = {
            "history_rows": [
                {"store": "A", "date": "2026-01-01", "sales": 1, "promo": 0},
                {"store": "A", "date": "2026-01-02", "sales": 2, "promo": 1},
                {"store": "B", "date": "2026-01-01", "sales": 3, "promo": 0},
                {"store": "B", "date": "2026-01-02", "sales": 4, "promo": 0},
            ],
            "future_rows": [
                {"store": "B", "date": "2026-01-03", "promo": 0},
                {"store": "B", "date": "2026-01-04", "promo": 1},
                {"store": "A", "date": "2026-01-03", "promo": 1},
                {"store": "A", "date": "2026-01-04", "promo": 0},
            ],
            "date_column": "date",
            "target_column": "sales",
            "series_column": "store",
            "covariate_columns": ["promo"],
            "model_config": {},
        }
        result = TimesFmAdapter("family", "model", "/model").predict(
            payload, 2, [0.1, 0.5, 0.9]
        )

        call = FakeLoader.model.xreg_call
        self.assertEqual(call["inputs"], [[1.0, 2.0], [3.0, 4.0]])
        self.assertEqual(
            call["dynamic_numerical_covariates"],
            {"promo": [[0.0, 1.0, 1.0, 0.0], [0.0, 0.0, 0.0, 1.0]]},
        )
        self.assertEqual(call["xreg_mode"], "xreg + timesfm")
        self.assertTrue(FakeLoader.model.compiles[0]["return_backcast"])
        self.assertEqual(
            [item["date"] for item in result["predictions"]],
            ["2026-01-03", "2026-01-04", "2026-01-03", "2026-01-04"],
        )
        artificial = any(
            item["date"].startswith("T+") for item in result["predictions"]
        )
        self.assertFalse(artificial)

    def test_rejects_incomplete_or_non_numeric_xreg(self):
        base = {
            "history_rows": [{"date": "2026-01-01", "sales": 1, "promo": 0}],
            "future_rows": [{"date": "2026-01-02", "promo": "yes"}],
            "date_column": "date",
            "target_column": "sales",
            "series_column": None,
            "covariate_columns": ["promo"],
        }
        with self.assertRaisesRegex(ValueError, "invalid_covariates"):
            TimesFmAdapter("family", "model", "/model").predict(
                base, 1, [0.1, 0.5, 0.9]
            )
        self.assertIsNone(FakeLoader.model)

    def test_rejects_unsupported_quantiles_and_partial_future_dates(self):
        adapter = TimesFmAdapter("family", "model", "/model")
        with self.assertRaisesRegex(ValueError, "invalid_quantiles"):
            adapter.predict({"values": [1.0]}, 1, [0.05, 0.5, 0.95])
        payload = {
            "history_rows": [{"date": "2026-01-01", "sales": 1}],
            "future_rows": [{"date": "2026-01-02"}],
            "date_column": "date",
            "target_column": "sales",
        }
        with self.assertRaisesRegex(ValueError, "invalid_future_rows"):
            adapter.predict(payload, 2, [0.1, 0.5, 0.9])


if __name__ == "__main__":
    unittest.main()
