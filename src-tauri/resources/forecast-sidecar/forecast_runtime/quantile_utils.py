from .validation import forecast_quantile_index


def select_standard_quantiles(matrix, horizon, quantile_levels, drop_mean=False):
    if hasattr(matrix, "detach"):
        matrix = matrix.detach().cpu()
    if drop_mean and matrix.shape[-1] > 9:
        matrix = matrix[..., 1:]

    if len(matrix.shape) != 2:
        raise ValueError("prediction_failed")

    row_count, column_count = matrix.shape
    if row_count >= horizon and column_count >= 9:
        return [
            matrix[:horizon, forecast_quantile_index(level)]
            for level in quantile_levels
        ]
    if column_count >= horizon and row_count >= 9:
        return [
            matrix[forecast_quantile_index(level), :horizon]
            for level in quantile_levels
        ]
    raise ValueError("prediction_failed")
