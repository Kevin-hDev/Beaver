import unittest

from forecast_runtime.timesfm_output import format_result
from test_timesfm_adapter import FakeMatrix, quantile_steps


class TimesFmOutputTests(unittest.TestCase):
    def test_rejects_crossed_quantiles(self):
        matrix = quantile_steps(1.0, 2)
        matrix[0][1] = 100.0
        jobs = [{"series_id": None, "dates": ["T+1", "T+2"]}]

        with self.assertRaisesRegex(ValueError, "prediction_failed"):
            format_result(
                jobs,
                [[10.0, 11.0]],
                [FakeMatrix(matrix)],
                [0.1, 0.5, 0.9],
                2,
            )


if __name__ == "__main__":
    unittest.main()
