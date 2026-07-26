import unittest

from forecast_runtime.adapter_utils import forecast_jobs
from forecast_runtime.limits import MAX_HORIZON, MAX_SERIES
from forecast_runtime.validation import (
    quantile_key,
    validate_horizon,
    validate_quantiles,
)


class ValidationContractTests(unittest.TestCase):
    def test_horizon_accepts_global_boundary(self):
        self.assertEqual(validate_horizon(MAX_HORIZON), MAX_HORIZON)
        with self.assertRaises(ValueError):
            validate_horizon(MAX_HORIZON + 1)
        with self.assertRaises(ValueError):
            validate_horizon(1.5)

    def test_quantiles_must_be_sorted_unique_and_include_median(self):
        self.assertEqual(validate_quantiles([0.05, 0.5, 0.95]), [0.05, 0.5, 0.95])
        for values in ([0.5, 0.1], [0.1, 0.9], [0.1, 0.1, 0.5]):
            with self.assertRaises(ValueError):
                validate_quantiles(values)

    def test_quantile_key_preserves_half_percent_levels(self):
        self.assertEqual(quantile_key(0.005), "q0050")
        self.assertEqual(quantile_key(0.05), "q05")

    def test_series_times_horizon_budget_is_bounded(self):
        rows = [
            {"series": f"s-{index}", "target": 1.0}
            for index in range(MAX_SERIES + 1)
        ]
        payload = {
            "history_rows": rows,
            "target_column": "target",
            "series_column": "series",
        }
        with self.assertRaises(ValueError):
            forecast_jobs(payload, 1)


if __name__ == "__main__":
    unittest.main()
