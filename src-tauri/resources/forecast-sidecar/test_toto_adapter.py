import sys
import types
import unittest
from contextlib import nullcontext
from unittest.mock import patch

from forecast_runtime.toto_adapter import TotoAdapter


class FakeTensor:
    def __init__(self, data):
        self.data = data
        self.shape = self._shape(data)

    def to(self, _device):
        return self

    @staticmethod
    def _shape(value):
        shape = []
        while isinstance(value, list):
            shape.append(len(value))
            value = value[0] if value else None
        return tuple(shape)


class FakeModel:
    config = types.SimpleNamespace(patch_size=2)

    def __init__(self, series_count=1, horizon=2, crossed=False, extra_step=False):
        self.series_count = series_count
        self.horizon = horizon
        self.crossed = crossed
        self.extra_step = extra_step
        self.calls = []

    def eval(self):
        return self

    def forecast(self, inputs, **options):
        self.calls.append((inputs, options))
        result = [
            [[
                [
                    float((level + 1) * 100 + series * 10 + step)
                    for step in range(self.horizon + int(self.extra_step))
                ]
                for series in range(self.series_count)
            ]]
            for level in range(9)
        ]
        if self.crossed:
            result[0][0][0][0] = 600.0
        return result


def fake_torch():
    return types.SimpleNamespace(
        tensor=lambda data, dtype=None: FakeTensor(data),
        float32="float32",
        bool="bool",
        long="long",
        no_grad=nullcontext,
        cuda=types.SimpleNamespace(is_available=lambda: False),
        backends=types.SimpleNamespace(),
    )


class TotoAdapterTests(unittest.TestCase):
    def adapter(self, model):
        adapter = TotoAdapter("toto-2", "fake", "/tmp/fake", device="cpu")
        adapter.model = model
        return adapter

    def test_rejects_covariates_explicitly(self):
        with self.assertRaisesRegex(ValueError, "covariates_not_supported"):
            self.adapter(FakeModel()).predict(
                {"values": [1.0, 2.0], "covariate_columns": ["weather"]},
                2,
                [0.1, 0.5, 0.9],
            )

    def test_multivariate_uses_one_joint_tensor_and_real_dates(self):
        model = FakeModel(series_count=2)
        payload = {
            "history_rows": [
                {"date": "2026-01-01", "asset": "B", "value": 10.0},
                {"date": "2026-01-01", "asset": "A", "value": 1.0},
                {"date": "2026-01-02", "asset": "B", "value": 11.0},
                {"date": "2026-01-02", "asset": "A", "value": 2.0},
            ],
            "future_rows": [
                {"date": "2026-01-03", "asset": "A"},
                {"date": "2026-01-03", "asset": "B"},
                {"date": "2026-01-04", "asset": "A"},
                {"date": "2026-01-04", "asset": "B"},
            ],
            "date_column": "date",
            "target_column": "value",
            "series_column": "asset",
            "covariate_columns": [],
        }
        with patch.dict(sys.modules, {"torch": fake_torch()}):
            result = self.adapter(model).predict(payload, 2, [0.1, 0.5, 0.9])

        self.assertEqual(len(model.calls), 1)
        inputs, _options = model.calls[0]
        self.assertEqual(inputs["target"].shape, (1, 2, 2))
        self.assertEqual(inputs["target"].data, [[[10.0, 11.0], [1.0, 2.0]]])
        self.assertEqual(inputs["series_ids"].shape, (1, 2))
        predictions = result["predictions"]
        self.assertEqual(
            [(item["series_id"], item["date"]) for item in predictions],
            [
                ("B", "2026-01-03"),
                ("B", "2026-01-04"),
                ("A", "2026-01-03"),
                ("A", "2026-01-04"),
            ],
        )
        self.assertTrue(
            all(item["value"] == item["q50"] for item in predictions)
        )
        self.assertTrue(all("q10" in item and "q90" in item for item in predictions))

    def test_rejects_unaligned_series(self):
        payload = {
            "history_rows": [
                {"date": "2026-01-01", "asset": "A", "value": 1.0},
                {"date": "2026-01-02", "asset": "B", "value": 2.0},
            ],
            "date_column": "date",
            "target_column": "value",
            "series_column": "asset",
        }
        with self.assertRaisesRegex(ValueError, "unaligned_series"):
            self.adapter(FakeModel(2)).predict(payload, 2, [0.1, 0.5, 0.9])

    def test_mono_series_contract_remains_available(self):
        model = FakeModel()
        with patch.dict(sys.modules, {"torch": fake_torch()}):
            result = self.adapter(model).predict(
                {"values": [1.0, 2.0], "covariate_columns": []},
                2,
                [0.1, 0.5, 0.9],
            )
        self.assertEqual(result["median"], [500.0, 501.0])
        self.assertEqual(result["q10"], [100.0, 101.0])
        self.assertEqual(result["q90"], [900.0, 901.0])

    def test_rejects_crossed_quantiles(self):
        with patch.dict(sys.modules, {"torch": fake_torch()}):
            with self.assertRaisesRegex(ValueError, "prediction_failed"):
                self.adapter(FakeModel(crossed=True)).predict(
                    {"values": [1.0, 2.0]}, 2, [0.1, 0.5, 0.9]
                )

    def test_rejects_output_longer_than_requested_horizon(self):
        with patch.dict(sys.modules, {"torch": fake_torch()}):
            with self.assertRaisesRegex(ValueError, "prediction_failed"):
                self.adapter(FakeModel(extra_step=True)).predict(
                    {"values": [1.0, 2.0]}, 2, [0.1, 0.5, 0.9]
                )


if __name__ == "__main__":
    unittest.main()
