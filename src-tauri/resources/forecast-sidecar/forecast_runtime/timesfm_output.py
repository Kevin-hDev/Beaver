import math

from .adapter_utils import to_float_list
from .quantile_utils import select_standard_quantiles
from .validation import quantile_key


def format_result(jobs, raw_points, raw_quantiles, levels, horizon):
    try:
        point_rows = list(raw_points)
        quantile_matrices = list(raw_quantiles)
        if len(point_rows) != len(jobs) or len(quantile_matrices) != len(jobs):
            raise ValueError("prediction_failed")
        rows = []
        for job, point_row, matrix in zip(
            jobs, point_rows, quantile_matrices, strict=True
        ):
            model_points = to_float_list(point_row)
            selected = select_standard_quantiles(
                matrix, horizon, levels, drop_mean=True
            )
            quantiles = {
                quantile_key(level): to_float_list(values)
                for level, values in zip(levels, selected, strict=True)
            }
            points = quantiles[quantile_key(0.5)]
            values = model_points + [item for row in quantiles.values() for item in row]
            invalid_quantiles = any(
                len(row) != horizon for row in quantiles.values()
            )
            if len(model_points) != horizon or invalid_quantiles:
                raise ValueError("prediction_failed")
            if not all(math.isfinite(value) for value in values):
                raise ValueError("prediction_failed")
            ordered = list(quantiles.values())
            if any(
                lower[index] > upper[index]
                for lower, upper in zip(ordered, ordered[1:])
                for index in range(horizon)
            ):
                raise ValueError("prediction_failed")
            rows.append((job, points, quantiles))
    except (IndexError, TypeError, ValueError, OverflowError):
        raise ValueError("prediction_failed") from None

    if len(rows) == 1 and rows[0][0]["series_id"] is None:
        _, point_row, quantile_rows = rows[0]
        return {"median": point_row, **quantile_rows}
    return {"predictions": structured_predictions(rows, levels)}


def structured_predictions(rows, levels):
    predictions = []
    for job, point_row, quantile_rows in rows:
        for index, point in enumerate(point_row):
            item = {
                "date": job["dates"][index],
                "value": point,
                "series_id": job["series_id"],
            }
            for level in levels:
                key = quantile_key(level)
                item[key] = quantile_rows[key][index]
            predictions.append(item)
    return predictions
